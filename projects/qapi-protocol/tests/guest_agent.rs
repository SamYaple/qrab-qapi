mod support;
use qapi_protocol::{
    ClientOptions, Error, InvalidRequestKind, ProtocolViolation, TimeoutPhase, qga::Qga,
};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};

#[tokio::test]
async fn timed_out_response_closes_session_and_cannot_be_reused() {
    let options = ClientOptions::default().with_command_timeout(Duration::from_millis(20));
    let (mut client, mut peer) = support::qga(options).await;
    let guest_ping = qapi::qga::commands::GuestPing::default();
    let (result, _) = tokio::join!(client.execute(&guest_ping), peer.recv_command());
    assert!(matches!(
        result,
        Err(Error::Timeout {
            phase: TimeoutPhase::AfterDispatch,
            ..
        })
    ));
    assert!(client.is_closed());
    assert!(matches!(
        client
            .execute(&qapi::qga::commands::GuestPing::default())
            .await,
        Err(Error::Closed)
    ));
    assert!(peer.recv_command().await.is_err());
}

#[tokio::test]
async fn cancelling_a_partially_read_response_closes_session() {
    let (client, peer) = UnixStream::pair().unwrap();
    let server = tokio::spawn(async move {
        let (read, mut write) = peer.into_split();
        let mut read = BufReader::new(read);
        assert_eq!(read.read_u8().await.unwrap(), 0xff);
        let sync = support::read(&mut read).await;
        write.write_all(&[0xff]).await.unwrap();
        support::write(&mut write, json!({"return":sync["arguments"]["id"]})).await;
        support::read(&mut read).await;
        support::write(
            &mut write,
            json!({"return":{"version":"11.1.3","supported_commands":[]}}),
        )
        .await;
        support::read(&mut read).await;
        write.write_all(b"{\"return\":").await.unwrap();
        let mut byte = [0u8];
        assert_eq!(read.read(&mut byte).await.unwrap(), 0);
    });
    let mut client = Qga::from_stream(client).await.unwrap();
    assert!(
        tokio::time::timeout(
            Duration::from_millis(30),
            client.execute(&qapi::qga::commands::GuestPing::default())
        )
        .await
        .is_err()
    );
    assert!(client.is_closed());
    server.await.unwrap();
}

#[tokio::test]
async fn sync_ignores_stale_replies_and_resets_at_each_sentinel() {
    let (client, peer) = UnixStream::pair().unwrap();
    let server = tokio::spawn(async move {
        let (read, mut write) = peer.into_split();
        let mut read = BufReader::new(read);
        assert_eq!(read.read_u8().await.unwrap(), 0xff);
        let sync = support::read(&mut read).await;
        let nonce = sync["arguments"]["id"].as_i64().unwrap();
        // Stale replies of an earlier client, without a sentinel.
        support::write(&mut write, json!({"return":nonce})).await;
        support::write(
            &mut write,
            json!({"error":{"class":"CommandNotFound","desc":"The command guest-exec has not been found"}}),
        )
        .await;
        write
            .write_all(b"stale without newline\xffbroken json\n\xff{\"return\":\"wrong type\"}\n")
            .await
            .unwrap();
        write.write_all(&[0xff]).await.unwrap();
        support::write(&mut write, json!({"return":nonce.wrapping_add(1)})).await;
        write
            .write_all(b"\xff{\"return\":garbage\xff")
            .await
            .unwrap();
        support::write(&mut write, json!({"return":nonce})).await;
        assert_eq!(support::read(&mut read).await["execute"], "guest-info");
        support::write(
            &mut write,
            json!({"return":{"version":"11.1.3","supported_commands":[]}}),
        )
        .await;
        // Keep both socket halves alive until the client explicitly closes.
        let mut byte = [0];
        assert_eq!(read.read(&mut byte).await.unwrap(), 0);
    });
    let mut client = Qga::from_stream(client).await.unwrap();
    assert_eq!(client.version(), (11, 1, 3));
    client.shutdown();
    server.await.unwrap();
}

/// Before qemu-ga 1.1 (QEMU 1.0's error carries `data`), and disabled.
#[tokio::test]
async fn an_agent_that_cannot_sync_fails_the_handshake_with_its_error() {
    for refusal in [
        json!({"error":{"class":"CommandNotFound","desc":"The command guest-sync-delimited has not been found","data":{"name":"guest-sync-delimited"}}}),
        json!({"error":{"class":"CommandNotFound","desc":"Command guest-sync-delimited has been disabled"}}),
    ] {
        let (client, peer) = UnixStream::pair().unwrap();
        let (read, mut write) = peer.into_split();
        let mut read = BufReader::new(read);
        let expected = refusal["error"]["desc"].clone();
        let server = tokio::spawn(async move {
            assert_eq!(read.read_u8().await.unwrap(), 0xff);
            support::read(&mut read).await;
            support::write(&mut write, refusal).await;
            (read, write)
        });
        let result = tokio::time::timeout(Duration::from_secs(1), Qga::from_stream(client))
            .await
            .expect("the refusal is not a handshake timeout");
        match result {
            Err(Error::SyncUnsupported(error)) => {
                assert_eq!(error.class, "CommandNotFound");
                assert_eq!(error.desc, expected);
            }
            other => panic!("{other:?}"),
        }
        server.await.unwrap();
    }
}

#[tokio::test]
async fn error_responses_preserve_unknown_classes_and_allow_reuse() {
    let (mut client, mut peer) = support::qga(ClientOptions::default()).await;
    let guest_ping = qapi::qga::commands::GuestPing::default();
    let (result, ()) = tokio::join!(client.execute(&guest_ping), async {
        peer.recv_command().await.unwrap();
        peer.send_error("NewAgentError", "exact").await.unwrap();
    });
    match result.unwrap_err() {
        Error::Remote(e) => assert_eq!(e.class, "NewAgentError"),
        e => panic!("{e:?}"),
    }
    assert!(!client.is_closed());
    let guest_ping = qapi::qga::commands::GuestPing::default();
    let (result, ()) = tokio::join!(client.execute(&guest_ping), async {
        peer.recv_command().await.unwrap();
        peer.send_return(json!({})).await.unwrap();
    });
    result.unwrap();
}

#[tokio::test]
async fn malformed_reply_is_not_success_and_invalid_arguments_are_not_sent() {
    let (mut client, mut peer) = support::qga(ClientOptions::default()).await;
    assert!(matches!(
        client.execute_raw::<Value>("guest-ping", json!([])).await,
        Err(Error::InvalidRequest(InvalidRequestKind::ArgumentsNotObject { ref command, .. }))
            if command == "guest-ping"
    ));
    assert!(!client.is_closed());
    // Empty object cannot be silently converted into Option<Required> = None.
    #[derive(serde::Deserialize)]
    struct Required {
        _required: String,
    }
    let (result, ()) = tokio::join!(
        client.execute_raw::<Option<Required>>("test", json!({})),
        async {
            peer.recv_command().await.unwrap();
            peer.send_return(json!({})).await.unwrap();
        }
    );
    assert!(matches!(result, Err(Error::Decode(_))));
    assert!(
        !client.is_closed(),
        "a typed decoding failure does not desynchronize the wire"
    );
}

#[tokio::test]
async fn response_less_shutdown_is_an_explicit_delivery_only_operation() {
    let (mut client, mut peer) = support::qga(ClientOptions::default()).await;
    let command = qapi::qga::commands::GuestShutdown::default();
    assert!(matches!(
        client.execute(&command).await,
        Err(Error::InvalidRequest(InvalidRequestKind::NoSuccessResponse { ref command, .. }))
            if command == "guest-shutdown"
    ));
    client.send_no_response(&command).await.unwrap();
    assert!(client.is_closed());
    assert_eq!(peer.recv_command().await.unwrap().0, "guest-shutdown");
    assert!(peer.recv_command().await.is_err());
}

#[tokio::test]
async fn write_backpressure_is_inside_the_command_deadline() {
    let options = ClientOptions::default().with_command_timeout(Duration::from_millis(30));
    let (mut client, _peer) = support::qga(options).await;
    let result = client
        .execute_raw::<Value>("big", json!({"data":"x".repeat(4*1024*1024)}))
        .await;
    assert!(matches!(
        result,
        Err(Error::Timeout {
            phase: TimeoutPhase::AfterDispatch,
            ..
        })
    ));
    assert!(client.is_closed());
}

/// A reply of `[0,0,0,...]` is the worst case for a JSON value tree, about
/// sixteen times its size in memory. The guest is not trusted.
fn hostile_reply(bytes: usize) -> Value {
    json!(vec![0; bytes / 2])
}

#[tokio::test]
async fn a_reply_over_the_limit_is_refused_at_the_limit_and_closes_the_session() {
    let options = ClientOptions::default().with_max_message_bytes(4096);
    let (mut client, mut peer) = support::qga(options).await;
    let guest_ping = qapi::qga::commands::GuestPing::default();
    let (result, ()) = tokio::join!(client.execute(&guest_ping), async {
        peer.recv_command().await.unwrap();
        peer.send_return(hostile_reply(64 * 1024)).await.unwrap();
    });
    assert!(matches!(
        result,
        Err(Error::MessageTooLarge { limit: 4096, .. })
    ));
    assert!(client.is_closed());
}

#[tokio::test]
async fn a_large_reply_of_the_wrong_shape_is_a_decode_error_and_keeps_the_session() {
    let (mut client, mut peer) = support::qga(ClientOptions::default()).await;
    let read = qapi::qga::commands::GuestFileReadCommand::new(1);
    let (result, ()) = tokio::join!(client.execute(&read), async {
        peer.recv_command().await.unwrap();
        peer.send_return(hostile_reply(1024 * 1024)).await.unwrap();
    });
    assert!(matches!(result, Err(Error::Decode(_))));
    assert!(!client.is_closed());
}

#[tokio::test]
async fn a_per_call_reply_limit_admits_a_large_file_read_only_for_that_call() {
    let options = ClientOptions::default().with_max_message_bytes(4096);
    let (mut client, mut peer) = support::qga(options).await;
    let mut read = qapi::qga::commands::GuestFileReadCommand::new(1);
    read.count = Some(12 * 1024);
    let data = "eHh4".repeat(4 * 1024);
    let (result, ()) = tokio::join!(client.execute_with_reply_limit(&read, 64 * 1024), async {
        peer.recv_command().await.unwrap();
        peer.send_return(json!({"count": 12 * 1024, "buf-b64": data, "eof": false}))
            .await
            .unwrap();
    });
    assert_eq!(result.unwrap().buf_b64.len(), 16 * 1024);
    let (result, ()) = tokio::join!(client.execute(&read), async {
        peer.recv_command().await.unwrap();
        peer.send_return(json!({"count": 12 * 1024, "buf-b64": data, "eof": false}))
            .await
            .unwrap();
    });
    assert!(matches!(
        result,
        Err(Error::MessageTooLarge { limit: 4096, .. })
    ));
    assert!(qapi_protocol::qga::FILE_READ_REPLY_BYTES > ClientOptions::default().max_message_bytes);
}

/// Requests have the message limit too: docs/protocol.md gives the largest
/// `guest-file-write` chunk that fits the default limit.
#[tokio::test]
async fn a_file_write_chunk_must_fit_the_request_limit() {
    let write = |bytes: usize| {
        let mut write = qapi::qga::commands::GuestFileWriteCommand::new(
            "A".repeat(bytes.div_ceil(3) * 4),
            i64::MIN,
        );
        write.count = Some(bytes as i64);
        write
    };
    let (mut client, mut peer) = support::qga(ClientOptions::default()).await;
    let chunk = 6 * 1024 * 1024 - 1024;
    let largest = write(chunk);
    let (result, ()) = tokio::join!(client.execute(&largest), async {
        let (name, arguments) = peer.recv_command().await.unwrap();
        assert_eq!(name, "guest-file-write");
        peer.send_return(json!({"count": arguments["count"], "eof": false}))
            .await
            .unwrap();
    });
    assert_eq!(result.unwrap().count, chunk as i64);
    assert!(matches!(
        client.execute(&write(6 * 1024 * 1024)).await,
        Err(Error::InvalidRequest(InvalidRequestKind::RequestTooLarge { limit, .. }))
            if limit == ClientOptions::default().max_message_bytes
    ));
    assert!(!client.is_closed(), "nothing was sent");
}

/// A client synchronized with a raw peer, which answers by hand.
async fn raw_qga(options: ClientOptions) -> (Qga, support::RawReader, support::RawWriter) {
    let (client, peer) = UnixStream::pair().unwrap();
    let (read, mut write) = peer.into_split();
    let mut read = BufReader::new(read);
    let (client, ()) = tokio::join!(Qga::from_stream_with_options(client, options), async {
        assert_eq!(read.read_u8().await.unwrap(), 0xff);
        let sync = support::read(&mut read).await;
        write.write_all(&[0xff]).await.unwrap();
        support::write(&mut write, json!({"return":sync["arguments"]["id"]})).await;
        support::read(&mut read).await;
        support::write(
            &mut write,
            json!({"return":{"version":"11.1.3","supported_commands":[]}}),
        )
        .await;
    });
    (client.unwrap(), read, write)
}

#[tokio::test]
async fn a_reply_that_is_not_a_valid_response_closes_the_session() {
    for (reply, malformed) in [
        (b"not json\n".as_slice(), true),
        (b"[1]\n".as_slice(), false),
        (b"{\"neither\":1}\n".as_slice(), false),
        (
            b"{\"return\":{},\"error\":{\"class\":\"X\",\"desc\":\"\"}}\n".as_slice(),
            false,
        ),
    ] {
        let (mut client, mut read, mut write) = raw_qga(ClientOptions::default()).await;
        let guest_ping = qapi::qga::commands::GuestPing::default();
        let (result, ()) = tokio::join!(client.execute(&guest_ping), async {
            support::read(&mut read).await;
            write.write_all(reply).await.unwrap();
        });
        match result {
            Err(Error::Malformed(_)) if malformed => {}
            Err(Error::Protocol(_)) if !malformed => {}
            other => panic!("{:?}: {other:?}", String::from_utf8_lossy(reply)),
        }
        assert!(client.is_closed());
    }
}

/// Serde reads a struct from an array of its members too, which would make
/// `[nonce]` a sync reply and `[info]` the guest-info reply.
#[tokio::test]
async fn arrays_are_not_replies_during_the_handshake() {
    let (client, peer) = UnixStream::pair().unwrap();
    let server = tokio::spawn(async move {
        let (read, mut write) = peer.into_split();
        let mut read = BufReader::new(read);
        assert_eq!(read.read_u8().await.unwrap(), 0xff);
        let sync = support::read(&mut read).await;
        let nonce = &sync["arguments"]["id"];
        write.write_all(&[0xff]).await.unwrap();
        support::write(&mut write, json!([nonce])).await;
        write.write_all(&[0xff]).await.unwrap();
        support::write(&mut write, json!({"return": nonce})).await;
        assert_eq!(support::read(&mut read).await["execute"], "guest-info");
        support::write(
            &mut write,
            json!([{"version":"11.1.3","supported_commands":[]}]),
        )
        .await;
        (read, write)
    });
    assert!(matches!(
        Qga::from_stream(client).await,
        Err(Error::Protocol(ProtocolViolation::NotAnObject(None)))
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn a_timeout_before_any_byte_is_written_keeps_the_session() {
    use std::io::Write;
    let (client, server) = UnixStream::pair().unwrap();
    // A second handle on the client's socket fills what it can send.
    let client = client.into_std().unwrap();
    let mut filler = client.try_clone().unwrap();
    let client = UnixStream::from_std(client).unwrap();
    let mut peer = qapi_protocol::server::QgaSession::from_stream(server);
    let options = ClientOptions::default().with_command_timeout(Duration::from_millis(300));
    let (client, ()) = tokio::join!(Qga::from_stream_with_options(client, options), async {
        peer.handle_sync().await.unwrap();
        peer.recv_command().await.unwrap();
        peer.send_return(json!({"version":"11.1.3","supported_commands":[]}))
            .await
            .unwrap();
    });
    let mut client = client.unwrap();
    // Spaces, which the peer reads as the next request's leading whitespace.
    loop {
        match filler.write(&[b' '; 4096]) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(error) => panic!("{error}"),
        }
    }
    assert!(matches!(
        client
            .execute(&qapi::qga::commands::GuestPing::default())
            .await,
        Err(Error::Timeout {
            phase: TimeoutPhase::BeforeDispatch,
            ..
        })
    ));
    assert!(!client.is_closed());
    let guest_ping = qapi::qga::commands::GuestPing::default();
    let (result, ()) = tokio::join!(client.execute(&guest_ping), async {
        assert_eq!(peer.recv_command().await.unwrap().0, "guest-ping");
        peer.send_return(json!({})).await.unwrap();
    });
    result.unwrap();
}
