//! The commands a peer lists while the client connects, QMP's
//! `query-commands` and a guest agent's `guest-info`, refuse a command the
//! peer cannot run before anything is sent.
mod support;

use qapi::{
    commands::{QueryBlock, QueryStatus, QueryYank},
    qga::commands::{GuestFsfreezeStatusCommand, GuestGetTime, GuestPing},
};
use qapi_protocol::{
    ClientOptions, Error, MissingCommand, ProtocolViolation, Qmp, QmpEvents,
    Required, Unavailability,
    qga::Qga,
    server::{QgaSession, QmpSession},
};
use serde_json::{Value, json};
use tokio::net::UnixStream;

/// A command of a release after the peer's 11.1.3.
#[derive(serde::Serialize)]
struct FutureCommand {}

impl qapi::QapiCommand for FutureCommand {
    type Returns = ();
    const NAME: &'static str = "x-future";
    const AVAILABLE_SINCE: (u32, u32, u32) = (12, 0, 0);
}

/// A client of a mock QEMU 11.1.3 that answers `query-commands` with
/// `offered`.
async fn offering(offered: &[&str]) -> (Qmp, QmpEvents, QmpSession) {
    let (client, server) = UnixStream::pair().unwrap();
    let mut peer = QmpSession::from_stream(server);
    peer.offer_commands(offered.iter().copied());
    let (client, ()) = tokio::join!(Qmp::from_stream(client), async {
        peer.send_greeting(11, 1, 3).await.unwrap();
        peer.negotiate().await.unwrap();
    });
    let (client, events) = client.unwrap();
    (client, events, peer)
}

fn unavailable(
    result: qapi_protocol::Result<impl std::fmt::Debug>,
) -> (String, Unavailability) {
    match result {
        Err(Error::Unavailable {
            name, version: (11, 1, 3), reason, ..
        }) => (name, reason),
        other => panic!("expected Unavailable, got {other:?}"),
    }
}

#[tokio::test]
async fn a_qmp_session_asks_for_the_commands_once() {
    let (client, _events, mut peer) =
        offering(&["query-status", "query-block"]).await;
    assert_eq!(peer.probes(), 1);
    let commands = client.commands().expect("the mock listed its commands");
    assert!(commands.offers("query-status") && commands.offers("query-block"));
    assert!(!commands.offers("query-cpus-fast"));
    assert_eq!(
        commands.names().collect::<Vec<_>>(),
        ["query-block", "query-status"]
    );
    for _ in 0..3 {
        let status_query = QueryStatus::default();
        let (status, ()) = tokio::join!(client.execute(&status_query), async {
            assert_eq!(peer.recv_command().await.unwrap().0, "query-status");
            peer.send_return(json!({"running": true, "status": "running"}))
                .await
                .unwrap();
        });
        assert!(status.unwrap().running);
    }
    assert_eq!(peer.probes(), 1, "calls never ask again");
}

#[tokio::test]
async fn a_command_the_peer_does_not_list_fails_before_anything_is_sent() {
    let (client, _events, mut peer) = offering(&["query-status"]).await;
    let refused = client.execute(&QueryBlock::default()).await;
    let error = refused.as_ref().map(drop).unwrap_err().to_string();
    assert_eq!(
        unavailable(refused),
        ("query-block".to_owned(), Unavailability::NotOffered)
    );
    assert_eq!(
        error,
        "query-block is not available in QEMU 11.1.3: the peer does not offer it"
    );
    assert_eq!(
        unavailable(
            client.execute_raw::<Value>("query-block", json!({})).await
        ),
        ("query-block".to_owned(), Unavailability::NotOffered)
    );
    assert_eq!(
        unavailable(client.execute_oob(&QueryYank::default()).await),
        ("query-yank".to_owned(), Unavailability::NotOffered)
    );
    // The next command the peer reads is the one it lists: the refused
    // calls sent nothing, and the connection stays usable.
    let status_query = QueryStatus::default();
    let (status, ()) = tokio::join!(client.execute(&status_query), async {
        assert_eq!(peer.recv_command().await.unwrap().0, "query-status");
        peer.send_return(json!({"running": false, "status": "paused"}))
            .await
            .unwrap();
    });
    assert!(!status.unwrap().running);
    assert!(!client.is_closed());
}

#[tokio::test]
async fn the_release_decides_before_the_list() {
    // A peer may list a command that a distribution backported; a typed
    // command that its release lacks stays refused for the release.
    let (client, _events, _peer) =
        offering(&["query-status", "x-future"]).await;
    assert_eq!(
        unavailable(client.execute(&FutureCommand {}).await),
        ("x-future".to_owned(), Unavailability::Release)
    );
    assert!(client.supports::<QueryStatus>());
    assert!(!client.supports::<QueryBlock>());
    assert!(!client.supports::<FutureCommand>());
}

#[tokio::test]
async fn require_lists_every_command_the_peer_cannot_run() {
    let (client, _events, _peer) = offering(&["query-status"]).await;
    client.require(&[Required::of::<QueryStatus>()]).unwrap();
    let error = client
        .require(&[
            Required::of::<QueryBlock>(),
            Required::of::<QueryStatus>(),
            Required::of::<FutureCommand>(),
        ])
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "QEMU 11.1.3 cannot run query-block (the peer does not offer it), \
         x-future (that release does not have it)"
    );
    let Error::MissingCommands { version, missing, .. } = error else {
        panic!("expected MissingCommands, got {error:?}");
    };
    assert_eq!(version, (11, 1, 3));
    let missing: Vec<(&str, Unavailability)> = missing
        .iter()
        .map(|MissingCommand { name, reason, .. }| (name.as_str(), *reason))
        .collect();
    assert_eq!(
        missing,
        [
            ("query-block", Unavailability::NotOffered),
            ("x-future", Unavailability::Release)
        ]
    );
}

#[tokio::test]
async fn a_refused_query_keeps_only_the_release_checks() {
    // The mock answers query-commands with CommandNotFound by default.
    let (client, _events, mut peer) =
        support::qmp(ClientOptions::default()).await;
    assert_eq!(peer.probes(), 1);
    assert!(client.commands().is_none());
    assert!(client.supports::<QueryBlock>());
    assert!(!client.supports::<FutureCommand>());
    let block_query = QueryBlock::default();
    let (blocks, ()) = tokio::join!(client.execute(&block_query), async {
        assert_eq!(peer.recv_command().await.unwrap().0, "query-block");
        peer.send_return(json!([])).await.unwrap();
    });
    assert!(blocks.unwrap().is_empty());
}

#[tokio::test]
async fn the_query_can_be_turned_off() {
    let options = ClientOptions::default().with_probe_commands(false);
    let (client, server) = UnixStream::pair().unwrap();
    let mut peer = QmpSession::from_stream(server);
    peer.expect_probe(false);
    let (client, ()) =
        tokio::join!(Qmp::from_stream_with_options(client, options), async {
            peer.send_greeting(11, 1, 3).await.unwrap();
            peer.negotiate().await.unwrap();
        });
    let (client, _events) = client.unwrap();
    assert_eq!(peer.probes(), 0);
    assert!(client.commands().is_none());
    let status_query = QueryStatus::default();
    let (status, ()) = tokio::join!(client.execute(&status_query), async {
        assert_eq!(peer.recv_command().await.unwrap().0, "query-status");
        peer.send_return(json!({"running": true, "status": "running"}))
            .await
            .unwrap();
    });
    assert!(status.unwrap().running);
}

#[tokio::test]
async fn a_mock_keeps_the_first_command_of_a_client_that_does_not_ask() {
    // A mock that waits for the query while serving a client that turned it
    // off reads the client's first command during negotiate, and returns it
    // on the next read.
    let (client, server) = UnixStream::pair().unwrap();
    let mock = tokio::spawn(async move {
        let mut peer = QmpSession::from_stream(server);
        peer.send_greeting(11, 1, 3).await.unwrap();
        peer.negotiate().await.unwrap();
        assert_eq!(peer.recv_command().await.unwrap().0, "query-status");
        peer.send_return(json!({"running": true, "status": "running"}))
            .await
            .unwrap();
        peer.probes()
    });
    let options = ClientOptions::default().with_probe_commands(false);
    let (client, _events) =
        Qmp::from_stream_with_options(client, options).await.unwrap();
    assert!(client.execute(&QueryStatus::default()).await.unwrap().running);
    assert_eq!(mock.await.unwrap(), 0);
}

#[tokio::test]
async fn a_query_reply_with_another_id_breaks_the_handshake() {
    let (client, peer) = UnixStream::pair().unwrap();
    let (read, mut write) = peer.into_split();
    let mut read = tokio::io::BufReader::new(read);
    let server = tokio::spawn(async move {
        support::write(&mut write, support::greeting()).await;
        let caps = support::read(&mut read).await;
        support::write(&mut write, json!({"return": {}, "id": caps["id"]}))
            .await;
        let query = support::read(&mut read).await;
        assert_eq!(query["execute"], "query-commands");
        support::write(&mut write, json!({"return": [], "id": 7})).await;
        (read, write)
    });
    assert!(matches!(
        Qmp::from_stream(client).await,
        Err(Error::Protocol(ProtocolViolation::CommandsReplyId))
    ));
    let _ = server.await.unwrap();
}

#[tokio::test]
async fn events_before_the_query_reply_are_kept() {
    let (client, peer) = UnixStream::pair().unwrap();
    let (read, mut write) = peer.into_split();
    let mut read = tokio::io::BufReader::new(read);
    let server = tokio::spawn(async move {
        support::write(&mut write, support::greeting()).await;
        let caps = support::read(&mut read).await;
        support::write(&mut write, json!({"return": {}, "id": caps["id"]}))
            .await;
        let query = support::read(&mut read).await;
        support::write(
            &mut write,
            json!({"event": "STOP", "data": {}, "timestamp": {"seconds": 1, "microseconds": 2}}),
        )
        .await;
        support::write(
            &mut write,
            json!({"return": [{"name": "query-status"}], "id": query["id"]}),
        )
        .await;
        (read, write)
    });
    let (client, mut events) = Qmp::from_stream(client).await.unwrap();
    let _peer = server.await.unwrap();
    assert_eq!(events.recv().await.unwrap().event, "STOP");
    assert!(client.commands().unwrap().offers("query-status"));
}

/// A guest agent client whose agent lists `commands` in `guest-info`.
async fn agent_listing(
    options: ClientOptions,
    commands: Value,
) -> (Qga, QgaSession) {
    let (client, server) = UnixStream::pair().unwrap();
    let mut peer = QgaSession::from_stream(server);
    let (client, ()) =
        tokio::join!(Qga::from_stream_with_options(client, options), async {
            peer.handle_sync().await.unwrap();
            assert_eq!(peer.recv_command().await.unwrap().0, "guest-info");
            peer.send_return(
                json!({"version": "11.1.3", "supported_commands": commands}),
            )
            .await
            .unwrap();
        });
    (client.unwrap(), peer)
}

fn agent_commands() -> Value {
    json!([
        {"name": "guest-info", "enabled": true, "success-response": true},
        {"name": "guest-ping", "enabled": true, "success-response": true},
        {"name": "guest-fsfreeze-status", "enabled": false, "success-response": true},
    ])
}

#[tokio::test]
async fn a_guest_agent_refuses_what_guest_info_lists_as_disabled_or_omits() {
    let (mut client, mut peer) =
        agent_listing(ClientOptions::default(), agent_commands()).await;
    let commands = client.commands().expect("the agent listed its commands");
    assert!(commands.offers("guest-ping"));
    assert!(commands.is_disabled("guest-fsfreeze-status"));
    assert!(!commands.offers("guest-fsfreeze-status"));
    assert_eq!(
        unavailable(
            client.execute(&GuestFsfreezeStatusCommand::default()).await
        ),
        ("guest-fsfreeze-status".to_owned(), Unavailability::Disabled)
    );
    let disabled =
        client.execute_raw::<Value>("guest-fsfreeze-status", json!({})).await;
    assert_eq!(
        disabled.as_ref().map(drop).unwrap_err().to_string(),
        "guest-fsfreeze-status is not available in QEMU 11.1.3: the guest agent has it disabled"
    );
    assert_eq!(
        unavailable(client.execute(&GuestGetTime::default()).await),
        ("guest-get-time".to_owned(), Unavailability::NotOffered)
    );
    assert!(client.supports::<GuestPing>());
    assert!(!client.supports::<GuestFsfreezeStatusCommand>());
    let Error::MissingCommands { missing, .. } = client
        .require(&[
            Required::of::<GuestPing>(),
            Required::of::<GuestFsfreezeStatusCommand>(),
            Required::of::<GuestGetTime>(),
        ])
        .unwrap_err()
    else {
        panic!("expected MissingCommands");
    };
    assert_eq!(
        missing.iter().map(|command| command.reason).collect::<Vec<_>>(),
        [Unavailability::Disabled, Unavailability::NotOffered]
    );
    // The refused calls sent nothing and kept the session aligned.
    let ping_query = GuestPing::default();
    let (ping, ()) = tokio::join!(client.execute(&ping_query), async {
        assert_eq!(peer.recv_command().await.unwrap().0, "guest-ping");
        peer.send_return(json!({})).await.unwrap();
    });
    ping.unwrap();
    assert!(!client.is_closed());
}

#[tokio::test]
async fn a_guest_info_reply_without_guest_info_is_no_command_list() {
    for commands in [
        json!([]),
        json!([{"name": "guest-ping", "enabled": true, "success-response": true}]),
    ] {
        let (client, _peer) =
            agent_listing(ClientOptions::default(), commands).await;
        assert!(client.commands().is_none());
        assert!(client.supports::<GuestFsfreezeStatusCommand>());
    }
}

#[tokio::test]
async fn a_guest_agent_client_that_turns_the_list_off_sends_anything_the_release_has()
 {
    let options = ClientOptions::default().with_probe_commands(false);
    let (mut client, mut peer) = agent_listing(options, agent_commands()).await;
    assert!(client.commands().is_none());
    let freeze_query = GuestFsfreezeStatusCommand::default();
    let (status, ()) = tokio::join!(client.execute(&freeze_query), async {
        assert_eq!(
            peer.recv_command().await.unwrap().0,
            "guest-fsfreeze-status"
        );
        peer.send_return(json!("thawed")).await.unwrap();
    });
    status.unwrap();
}
