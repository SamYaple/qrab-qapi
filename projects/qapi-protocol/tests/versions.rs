//! Version gating belongs to each protocol connection.
//! That includes QGA peers whose version differs from the host's QEMU version.

// The tests use items and members of older releases, which the newest
// release no longer has and which are therefore deprecated.
#![allow(deprecated)]

mod support;
use qapi_protocol::{
    ClientOptions, Error, Qmp,
    qga::Qga,
    server::{QgaSession, QmpSession},
};
use tokio::net::UnixListener;

#[qapi::qapi(command, name = "test-version", added = "1.0.0", returns = "bool")]
struct TestVersion {
    #[qapi(added = "11.0.0")]
    new_field: Option<bool>,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multiple_qmp_versions_serialize_independently() {
    let dir = tempfile::tempdir().unwrap();
    let mut clients = Vec::new();
    let mut servers = Vec::new();
    for major in [10, 11] {
        let path = dir.path().join(format!("qmp-{major}"));
        let listener = UnixListener::bind(&path).unwrap();
        servers.push(tokio::spawn(async move {
            let mut session = QmpSession::accept(&listener).await.unwrap();
            session.send_greeting(major, 1, 0).await.unwrap();
            session.negotiate().await.unwrap();
            for _ in 0..3 {
                let (name, args) = session.recv_command().await.unwrap();
                assert_eq!(name, "test-version");
                assert_eq!(args.get("new_field").is_some(), major >= 11);
                session.send_return(serde_json::json!(true)).await.unwrap();
            }
        }));
        let (client, _) = Qmp::new(&path).await.unwrap();
        assert_eq!(client.version(), (major, 1, 0));
        clients.push(client);
    }
    let tasks: Vec<_> = clients
        .into_iter()
        .map(|client| {
            tokio::spawn(async move {
                let newer = client.version().0 >= 11;
                for _ in 0..3 {
                    // The field that only the newer peer has is refused
                    // for the older one, and sent to the newer.
                    let refused = client
                        .execute(&TestVersion {
                            new_field: Some(true),
                        })
                        .await;
                    if newer {
                        assert!(refused.unwrap());
                    } else {
                        assert!(
                            matches!(refused, Err(Error::Unavailable { ref name, version: (10, 1, 0), .. }) if name == "TestVersion.new_field"),
                            "{refused:?}"
                        );
                        assert!(
                            client
                                .execute(&TestVersion { new_field: None })
                                .await
                                .unwrap()
                        );
                    }
                    tokio::task::yield_now().await;
                }
                client.shutdown().await;
            })
        })
        .collect();
    for task in tasks {
        task.await.unwrap();
    }
    for server in servers {
        server.await.unwrap();
    }
}

#[tokio::test]
async fn qga_uses_guest_version_instead_of_host_default() {
    // This deliberately disagrees with both agents below.
    let _ = qapi::set_qemu_version((1, 0, 0));
    let dir = tempfile::tempdir().unwrap();
    let mut clients = Vec::new();
    let mut servers = Vec::new();
    for major in [10, 11] {
        let path = dir.path().join(format!("qga-{major}"));
        let listener = UnixListener::bind(&path).unwrap();
        servers.push(tokio::spawn(async move {
            let mut session = QgaSession::accept(&listener).await.unwrap();
            session.handle_sync().await.unwrap();
            assert_eq!(session.recv_command().await.unwrap().0, "guest-info");
            session.send_return(serde_json::json!({ "version": format!("{major}.1.0"), "supported_commands": [] })).await.unwrap();
            let (name, args) = session.recv_command().await.unwrap();
            assert_eq!(name, "test-version");
            assert_eq!(args.get("new_field").is_some(), major >= 11);
            session.send_return(serde_json::json!(true)).await.unwrap();
        }));
        let client = Qga::new(&path).await.unwrap();
        assert_eq!(client.version(), (major, 1, 0));
        clients.push(client);
    }
    for mut client in clients {
        let new_field = (client.version().0 >= 11).then_some(true);
        assert!(client.execute(&TestVersion { new_field }).await.unwrap());
    }
    for server in servers {
        server.await.unwrap();
    }
}

#[tokio::test]
async fn typed_guest_file_read_decodes_response_not_request() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("qga");
    let listener = UnixListener::bind(&path).unwrap();
    let server = tokio::spawn(async move {
        let mut session = QgaSession::accept(&listener).await.unwrap();
        session.handle_sync().await.unwrap();
        assert_eq!(session.recv_command().await.unwrap().0, "guest-info");
        session
            .send_return(serde_json::json!({ "version": "11.1.3", "supported_commands": [] }))
            .await
            .unwrap();
        let (name, args) = session.recv_command().await.unwrap();
        assert_eq!(name, "guest-file-read");
        assert_eq!(args, serde_json::json!({"handle": 7, "count": 3}));
        session
            .send_return(
                serde_json::json!({"count": 3, "buf-b64": "YWJj", "eof": true}),
            )
            .await
            .unwrap();
    });
    let mut client = Qga::new(&path).await.unwrap();
    let mut read = qapi::qga::commands::GuestFileReadCommand::new(7);
    read.count = Some(3);
    let reply: qapi::qga::GuestFileRead = client.execute(&read).await.unwrap();
    assert_eq!(reply.count, 3);
    assert_eq!(reply.buf_b64, "YWJj");
    assert!(reply.eof);
    server.await.unwrap();
}

#[qapi::qapi(
    command,
    name = "test-new-command",
    added = "11.0.0",
    returns = "bool"
)]
struct TestNewCommand {}

#[qapi::qapi(
    command,
    name = "test-old-command",
    added = "1.0.0",
    removed = "10.0.0",
    returns = "bool"
)]
struct TestOldCommand {}

#[qapi::qapi(added = "1.0.0")]
enum TestMode {
    #[qapi(name = "old")]
    Old,
    #[qapi(name = "new")]
    #[qapi(added = "11.0.0")]
    New,
}

#[qapi::qapi(command, name = "test-mode", added = "1.0.0", returns = "bool")]
struct TestModeCommand {
    mode: TestMode,
}

#[qapi::qapi(added = "11.0.0")]
struct TestNewType {
    value: bool,
}

#[qapi::qapi(added = "1.0.0")]
struct TestShared {
    #[qapi(name = "value", added = "11.0.0")]
    value: Option<String>,
    #[qapi(name = "value", removed = "11.0.0")]
    value_legacy: Option<i64>,
}

#[qapi::qapi(command, name = "test-nested", added = "1.0.0", returns = "bool")]
struct TestNestedCommand {
    new_type: Option<TestNewType>,
    shared: Option<TestShared>,
}

#[tokio::test]
async fn commands_types_variants_and_fields_the_peer_lacks_are_unavailable() {
    let (client, _, mut peer) =
        support::qmp_version(ClientOptions::default(), (10, 1, 0)).await;
    match client.execute(&TestNewCommand::default()).await {
        Err(Error::Unavailable { name, version, .. }) => {
            assert_eq!(name, "test-new-command");
            assert_eq!(version, (10, 1, 0));
        }
        other => panic!("{other:?}"),
    }
    match client.execute(&TestOldCommand::default()).await {
        Err(Error::Unavailable { name, version, .. }) => {
            assert_eq!(name, "test-old-command");
            assert_eq!(version, (10, 1, 0));
        }
        other => panic!("{other:?}"),
    }
    match client.execute(&TestModeCommand { mode: TestMode::New }).await {
        Err(Error::Unavailable { name, version, .. }) => {
            assert_eq!(name, "TestMode::New");
            assert_eq!(version, (10, 1, 0));
        }
        other => panic!("{other:?}"),
    }
    let nested = [
        (
            TestNestedCommand {
                new_type: Some(TestNewType { value: true }),
                shared: None,
            },
            "TestNewType",
        ),
        (
            TestNestedCommand {
                new_type: None,
                shared: Some(TestShared {
                    value: Some("new".into()),
                    value_legacy: None,
                }),
            },
            "TestShared.value",
        ),
    ];
    for (command, expected) in nested {
        match client.execute(&command).await {
            Err(Error::Unavailable { name, version, .. }) => {
                assert_eq!(name, expected);
                assert_eq!(version, (10, 1, 0));
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(!client.is_closed());
    // Nothing was sent: the next request the peer sees is the next call.
    let (result, ()) = tokio::join!(
        client.execute(&TestModeCommand { mode: TestMode::Old }),
        async {
            let (name, args) = peer.recv_command().await.unwrap();
            assert_eq!(name, "test-mode");
            assert_eq!(args, serde_json::json!({"mode": "old"}));
            peer.send_return(serde_json::json!(true)).await.unwrap();
        }
    );
    assert!(result.unwrap());
    client.shutdown().await;
}

#[tokio::test]
async fn replies_decode_into_the_return_type_of_the_peers_release() {
    use qapi::commands::{QueryTarget, QueryTargetReturns};
    // `QemuTargetInfo`, which QEMU returns from 10.1, is `TargetInfo` on
    // the wire from 3.0, so it is the return type from then.
    for (version, reply) in [
        ((2, 12, 0), serde_json::json!({"arch": "x86_64"})),
        ((9, 2, 0), serde_json::json!({"arch": "x86_64"})),
        ((10, 1, 0), serde_json::json!({"arch": "aarch64"})),
    ] {
        let (client, _, mut peer) =
            support::qmp_version(ClientOptions::default(), version).await;
        let query_target = QueryTarget::default();
        let (result, ()) = tokio::join!(client.execute(&query_target), async {
            assert_eq!(peer.recv_command().await.unwrap().0, "query-target");
            peer.send_return(reply).await.unwrap();
        });
        match result.unwrap() {
            QueryTargetReturns::V1_2_0(info) if version < (3, 0, 0) => {
                assert_eq!(info.arch_legacy2.as_deref(), Some("x86_64"));
            }
            QueryTargetReturns::V3_0_0(qapi::QemuTargetInfo {
                arch, ..
            }) if version >= (3, 0, 0) => {
                let expected = if version < (10, 1, 0) {
                    qapi::SysEmuTarget::X8664
                } else {
                    qapi::SysEmuTarget::Aarch64
                };
                assert_eq!(arch.to_string(), expected.to_string());
            }
            other => panic!("{version:?}: {other:?}"),
        }
        client.shutdown().await;
    }
}

#[qapi::qapi(
    command,
    name = "test-address",
    added = "1.3.0",
    returns = "qapi::SocketAddress",
    allow_oob
)]
struct TestAddress {}

#[tokio::test]
async fn qmp_reply_layout_uses_each_peer_version_in_band_and_oob() {
    for version in [(2, 9, 0), (11, 1, 3)] {
        let (client, server) = tokio::net::UnixStream::pair().unwrap();
        let mut peer = QmpSession::from_stream(server);
        let (client, ()) = tokio::join!(Qmp::from_stream(client), async {
            peer.send_greeting(version.0, version.1, version.2).await.unwrap();
            peer.negotiate().await.unwrap();
        });
        let (client, _) = client.unwrap();
        let reply = if version < (2, 10, 0) {
            serde_json::json!({"type":"inet","data":{"host":"127.0.0.1","port":"10809"}})
        } else {
            serde_json::json!({"type":"inet","host":"127.0.0.1","port":"10809"})
        };
        let server_reply = reply.clone();
        let server = tokio::spawn(async move {
            for _ in 0..if version < (2, 12, 0) { 1 } else { 2 } {
                assert_eq!(
                    peer.recv_command().await.unwrap().0,
                    "test-address"
                );
                peer.send_return(server_reply.clone()).await.unwrap();
            }
        });
        let value = client.execute(&TestAddress::default()).await.unwrap();
        assert_eq!(qapi::to_value(version, &value).unwrap(), reply);
        if version >= (2, 12, 0) {
            let value =
                client.execute_oob(&TestAddress::default()).await.unwrap();
            assert_eq!(qapi::to_value(version, &value).unwrap(), reply);
        }
        client.shutdown().await;
        server.await.unwrap();
    }
}

#[tokio::test]
async fn qga_reply_layout_uses_guest_version() {
    let (client, server) = tokio::net::UnixStream::pair().unwrap();
    let mut peer = QgaSession::from_stream(server);
    let reply = serde_json::json!({"type":"inet","data":{"host":"127.0.0.1","port":"10809"}});
    let server_reply = reply.clone();
    let server = tokio::spawn(async move {
        peer.handle_sync().await.unwrap();
        assert_eq!(peer.recv_command().await.unwrap().0, "guest-info");
        peer.send_return(
            serde_json::json!({"version":"2.9.0","supported_commands":[]}),
        )
        .await
        .unwrap();
        assert_eq!(peer.recv_command().await.unwrap().0, "test-address");
        peer.send_return(server_reply).await.unwrap();
    });
    let mut client = Qga::from_stream(client).await.unwrap();
    let value = client.execute(&TestAddress::default()).await.unwrap();
    assert_eq!(qapi::to_value((2, 9, 0), &value).unwrap(), reply);
    server.await.unwrap();
}

#[tokio::test]
async fn guest_agent_four_component_versions_use_their_first_three_components()
{
    for version in ["2.4.0.1", "2.8.1.1", "3.1.1.1-package"] {
        let (client, server) = tokio::net::UnixStream::pair().unwrap();
        let mut peer = QgaSession::from_stream(server);
        let server = tokio::spawn(async move {
            peer.handle_sync().await.unwrap();
            assert_eq!(peer.recv_command().await.unwrap().0, "guest-info");
            peer.send_return(
                serde_json::json!({"version":version,"supported_commands":[]}),
            )
            .await
            .unwrap();
        });
        let client = Qga::from_stream(client).await.unwrap();
        let expected = match version {
            "2.4.0.1" => (2, 4, 0),
            "2.8.1.1" => (2, 8, 1),
            _ => (3, 1, 1),
        };
        assert_eq!(client.version(), expected);
        server.await.unwrap();
    }
}

#[tokio::test]
async fn malformed_four_component_guest_versions_do_not_become_a_different_peer_version()
 {
    for version in ["2.8.1.x", "2.8.1.1.1", "2.8.1rc0.1", "2.8.1.4294967296"] {
        let (client, server) = tokio::net::UnixStream::pair().unwrap();
        let mut peer = QgaSession::from_stream(server);
        let server = tokio::spawn(async move {
            peer.handle_sync().await.unwrap();
            assert_eq!(peer.recv_command().await.unwrap().0, "guest-info");
            peer.send_return(
                serde_json::json!({"version":version,"supported_commands":[]}),
            )
            .await
            .unwrap();
        });
        match Qga::from_stream(client).await {
            Err(Error::AgentVersion(error)) => {
                assert_eq!(error.input(), version)
            }
            other => panic!("{version}: {other:?}"),
        }
        server.await.unwrap();
    }
}

#[tokio::test]
async fn guest_agent_replies_decode_into_the_return_type_of_its_release() {
    use qapi::qga::commands::{GuestFstrim, GuestFstrimReturns};
    for (version, reply) in [
        ("2.3.0", serde_json::json!({})),
        (
            "2.4.0",
            serde_json::json!({"paths": [{"path": "/", "trimmed": 4096}]}),
        ),
    ] {
        let (mut client, mut peer) =
            support::qga_version(ClientOptions::default(), version).await;
        let guest_fstrim = GuestFstrim::default();
        let (result, ()) = tokio::join!(client.execute(&guest_fstrim), async {
            assert_eq!(peer.recv_command().await.unwrap().0, "guest-fstrim");
            peer.send_return(reply).await.unwrap();
        });
        match result.unwrap() {
            GuestFstrimReturns::V1_2_0(()) if version == "2.3.0" => {}
            GuestFstrimReturns::V2_4_0(response) if version == "2.4.0" => {
                assert_eq!(response.paths[0].trimmed, Some(4096));
            }
            other => panic!("{version}: {other:?}"),
        }
    }
}
