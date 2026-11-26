mod support;
use qapi_protocol::{ClientOptions, Error, InvalidRequestKind, ProtocolViolation, Qmp};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::{io::AsyncWriteExt, net::UnixStream};

#[tokio::test]
async fn oversized_unterminated_frame_fails_without_waiting_for_newline() {
    let options = ClientOptions::default().with_max_message_bytes(512);
    let (client, _, mut reader, mut writer) = support::raw_qmp(options).await;
    let client = Arc::new(client);
    let other = client.clone();
    let call = tokio::spawn(async move { other.execute_raw::<Value>("ping", json!({})).await });
    support::read(&mut reader).await;
    writer.write_all(&vec![b'a'; 1024]).await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_secs(1), call)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    assert!(matches!(
        &*client.closed().await,
        Error::MessageTooLarge { limit: 512, .. }
    ));
}

#[tokio::test]
async fn default_limit_accepts_realistic_large_responses() {
    let (client, _, mut reader, mut writer) = support::raw_qmp(ClientOptions::default()).await;
    let call = tokio::spawn(async move {
        let result = client
            .execute_raw::<Value>("large", json!({}))
            .await
            .unwrap();
        assert_eq!(result.as_str().unwrap().len(), 2 * 1024 * 1024);
        client.shutdown().await;
    });
    let request = support::read(&mut reader).await;
    support::write(
        &mut writer,
        json!({"return": "x".repeat(2 * 1024 * 1024), "id": request["id"]}),
    )
    .await;
    call.await.unwrap();
}

#[tokio::test]
async fn malformed_envelopes_fail_the_connection_instead_of_becoming_success() {
    for malformed in [
        json!({"id": 1}),
        json!({"id": 1, "return": {}, "error": {"class":"X","desc":"X"}}),
        json!({"id": 1, "error": {"desc":"missing class"}}),
        json!([1]),
    ] {
        let (client, _, mut reader, mut writer) = support::raw_qmp(ClientOptions::default()).await;
        let client = Arc::new(client);
        let other = client.clone();
        let call =
            tokio::spawn(async move { other.execute(&qapi::commands::Quit::default()).await });
        support::read(&mut reader).await;
        support::write(&mut writer, malformed).await;
        assert!(
            tokio::time::timeout(Duration::from_secs(1), call)
                .await
                .unwrap()
                .unwrap()
                .is_err()
        );
        assert!(client.is_closed());
    }
}

#[tokio::test]
async fn truncated_json_at_eof_and_invalid_utf8_are_errors() {
    for bytes in [b"{\"return\":{},\"id\":1}".as_slice(), b"\xff\n".as_slice()] {
        let (client, _, mut reader, mut writer) = support::raw_qmp(ClientOptions::default()).await;
        let call =
            tokio::spawn(async move { client.execute_raw::<Value>("ping", json!({})).await });
        support::read(&mut reader).await;
        writer.write_all(bytes).await.unwrap();
        drop(writer);
        assert!(call.await.unwrap().is_err());
    }
}

#[tokio::test]
async fn fragmented_read_survives_outbound_activity() {
    let (client, mut events, mut reader, mut writer) =
        support::raw_qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    writer.write_all(b"{\"event\":\"ST").await.unwrap();
    tokio::task::yield_now().await;
    let other = client.clone();
    let call = tokio::spawn(async move { other.execute_raw::<Value>("ping", json!({})).await });
    let request = support::read(&mut reader).await;
    writer
        .write_all(b"OP\",\"timestamp\":{\"seconds\":-1,\"microseconds\":-1}}\r\n")
        .await
        .unwrap();
    support::write(&mut writer, json!({"return": null, "id": request["id"]})).await;
    assert!(call.await.unwrap().unwrap().is_null());
    let event = events.recv().await.unwrap();
    assert_eq!(event.event, "STOP");
    assert_eq!(event.timestamp.microseconds, -1);
    client.shutdown().await;
}

#[tokio::test]
async fn invalid_raw_requests_fail_before_writing_and_do_not_poison_the_session() {
    let (client, _, mut peer) = support::qmp(ClientOptions::default()).await;
    let limit = ClientOptions::default().max_message_bytes;
    for oob in [false, true] {
        for (name, args) in [
            ("", json!({})),
            ("ping", json!(null)),
            ("ping", json!([])),
            ("ping", json!("text")),
            ("ping", json!({"data": "x".repeat(limit)})),
        ] {
            let result = if oob {
                client.execute_raw_oob::<Value>(name, args).await
            } else {
                client.execute_raw::<Value>(name, args).await
            };
            match result {
                Err(Error::InvalidRequest(InvalidRequestKind::EmptyCommandName)) => {
                    assert!(name.is_empty())
                }
                Err(Error::InvalidRequest(InvalidRequestKind::ArgumentsNotObject {
                    command,
                    ..
                })) => assert_eq!(command, "ping"),
                Err(Error::InvalidRequest(InvalidRequestKind::RequestTooLarge {
                    limit: refused,
                    ..
                })) => assert_eq!(refused, limit),
                other => panic!("{name} (oob: {oob}): {other:?}"),
            }
            assert!(!client.is_closed());
            assert_eq!(client.queue_depth(), 0);
        }
    }
    // Nothing reached the peer: the next command it reads is the next call.
    let (result, ()) = tokio::join!(client.execute_raw::<Value>("ping", json!({})), async {
        assert_eq!(peer.recv_command().await.unwrap().0, "ping");
        peer.send_return(json!({})).await.unwrap();
    });
    assert_eq!(result.unwrap(), json!({}));
    client.shutdown().await;
}

#[tokio::test]
async fn handshake_requires_real_capability_ack_and_preserves_early_events() {
    for response in [
        json!({}),
        json!({"return":{},"id":99}),
        json!({"return":null,"id":0}),
    ] {
        let (client, peer) = UnixStream::pair().unwrap();
        let (read, mut write) = peer.into_split();
        let mut read = tokio::io::BufReader::new(read);
        let server = tokio::spawn(async move {
            support::write(&mut write, support::greeting()).await;
            support::read(&mut read).await;
            support::write(&mut write, response).await;
        });
        assert!(Qmp::from_stream(client).await.is_err());
        server.await.unwrap();
    }
    let (client, peer) = UnixStream::pair().unwrap();
    let server = tokio::spawn(async move {
        let mut peer = qapi_protocol::server::QmpSession::from_stream(peer);
        peer.send_greeting(11, 1, 3).await.unwrap();
        peer.send_event("STOP", json!({})).await.unwrap();
        peer.negotiate().await.unwrap();
        peer
    });
    let (client, mut events) = Qmp::from_stream(client).await.unwrap();
    let _peer = server.await.unwrap();
    assert_eq!(events.recv().await.unwrap().event, "STOP");
    assert_eq!(client.greeting().version.package, "");
    client.shutdown().await;
}

#[tokio::test]
async fn unknown_error_classes_are_preserved_and_valid_errors_allow_next_command() {
    let (client, _, mut peer) = support::qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    let other = client.clone();
    let call = tokio::spawn(async move { other.execute_raw::<Value>("ping", json!({})).await });
    peer.recv_command().await.unwrap();
    peer.send_error("FutureError", "exact description")
        .await
        .unwrap();
    match call.await.unwrap().unwrap_err() {
        Error::Remote(error) => {
            assert_eq!(error.class, "FutureError");
            assert_eq!(error.desc, "exact description");
            assert!(error.known_class().is_none());
        }
        other => panic!("{other:?}"),
    }
    assert!(!client.is_closed());
    client.shutdown().await;
}

#[tokio::test]
async fn large_upload_does_not_block_reading_an_earlier_large_reply() {
    use tokio::io::AsyncBufReadExt;
    let (client, _, mut reader, mut writer) = support::raw_qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    let other = client.clone();
    let download =
        tokio::spawn(async move { other.execute_raw::<Value>("download", json!({})).await });
    let first = support::read(&mut reader).await;
    let other = client.clone();
    let upload = tokio::spawn(async move {
        other
            .execute_raw::<Value>("upload", json!({"data":"y".repeat(4*1024*1024)}))
            .await
    });
    // The second command has begun writing; the socket cannot buffer it all.
    assert!(!reader.fill_buf().await.unwrap().is_empty());
    tokio::time::timeout(Duration::from_secs(2), async {
        support::write(
            &mut writer,
            json!({"return":"x".repeat(2*1024*1024),"id":first["id"]}),
        )
        .await;
        let second = support::read(&mut reader).await;
        assert_eq!(
            second["arguments"]["data"].as_str().unwrap().len(),
            4 * 1024 * 1024
        );
        support::write(&mut writer, json!({"return":{},"id":second["id"]})).await;
        assert_eq!(
            download.await.unwrap().unwrap().as_str().unwrap().len(),
            2 * 1024 * 1024
        );
        upload.await.unwrap().unwrap();
    })
    .await
    .expect("full-duplex read/write deadlocked under backpressure");
    client.shutdown().await;
}

#[tokio::test]
async fn pretty_printed_json_and_escaped_delimiters_are_framed_as_one_message() {
    let (client, _, mut reader, mut writer) = support::raw_qmp(ClientOptions::default()).await;
    let call = tokio::spawn(async move { client.execute_raw::<Value>("pretty", json!({})).await });
    let request = support::read(&mut reader).await;
    let payload = json!([{"text":"quoted \"}]}\" and newline\n inside a string"}]);
    let mut bytes =
        serde_json::to_vec_pretty(&json!({"return":payload,"id":request["id"]})).unwrap();
    bytes.extend_from_slice(b"\r\n");
    // Split in the middle of an escaped string to exercise persistent lexer state.
    for chunk in bytes.chunks(7) {
        writer.write_all(chunk).await.unwrap();
        tokio::task::yield_now().await;
    }
    assert_eq!(call.await.unwrap().unwrap(), payload);
}

#[tokio::test]
async fn shutdown_cancels_a_blocked_write_and_wakes_its_caller() {
    use tokio::io::AsyncBufReadExt;
    let (client, mut events, mut reader, _writer) =
        support::raw_qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    let other = client.clone();
    let call = tokio::spawn(async move {
        other
            .execute_raw::<Value>("upload", json!({"data":"x".repeat(4*1024*1024)}))
            .await
    });
    assert!(!reader.fill_buf().await.unwrap().is_empty());
    tokio::time::timeout(Duration::from_secs(1), client.shutdown())
        .await
        .unwrap();
    assert!(matches!(call.await.unwrap(), Err(Error::Disconnected(_))));
    assert_eq!(client.queue_depth(), 0);
    assert!(events.recv().await.is_none());
}

#[tokio::test]
async fn unknown_ids_of_any_json_type_are_dropped_without_harming_routing() {
    let (client, _, mut reader, mut writer) = support::raw_qmp(ClientOptions::default()).await;
    let call = tokio::spawn(async move { client.execute_raw::<Value>("ping", json!({})).await });
    let request = support::read(&mut reader).await;
    for id in [
        json!(null),
        json!("stale"),
        json!({"old":true}),
        json!([1]),
        json!(999),
    ] {
        support::write(&mut writer, json!({"return":"wrong","id":id})).await;
    }
    support::write(&mut writer, json!({"return":"correct","id":request["id"]})).await;
    assert_eq!(call.await.unwrap().unwrap(), json!("correct"));
}

#[tokio::test]
async fn idless_server_error_closes_connection_and_preserves_the_cause() {
    let (client, _, mut reader, mut writer) = support::raw_qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    let other = client.clone();
    let call = tokio::spawn(async move { other.execute_raw::<Value>("ping", json!({})).await });
    support::read(&mut reader).await;
    support::write(
        &mut writer,
        json!({"error":{"class":"GenericError","desc":"parser rejected input"}}),
    )
    .await;
    match call.await.unwrap().unwrap_err() {
        Error::Disconnected(cause) => match &*cause {
            Error::Remote(error) => assert_eq!(error.desc, "parser rejected input"),
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    }
    client.shutdown().await;
}

/// A reply of exactly `bytes` JSON bytes, and the string it returns.
fn reply_of(bytes: usize, id: &serde_json::Value) -> (Vec<u8>, String) {
    let empty = serde_json::to_vec(&json!({"return": "", "id": id})).unwrap();
    let text = "x".repeat(bytes - empty.len());
    let reply = serde_json::to_vec(&json!({"return": text, "id": id})).unwrap();
    (reply, text)
}

#[tokio::test]
async fn the_limit_counts_json_bytes_without_either_terminator() {
    const LIMIT: usize = 256;
    for (bytes, terminator, fits) in [
        (LIMIT, "\r\n", true),
        (LIMIT, "\n", true),
        (LIMIT + 1, "\r\n", false),
        (LIMIT + 1, "\n", false),
    ] {
        let options = ClientOptions::default().with_max_message_bytes(LIMIT);
        let (client, _, mut reader, mut writer) = support::raw_qmp(options).await;
        let client = Arc::new(client);
        let other = client.clone();
        let call = tokio::spawn(async move { other.execute_raw::<Value>("ping", json!({})).await });
        let request = support::read(&mut reader).await;
        let (mut reply, text) = reply_of(bytes, &request["id"]);
        assert_eq!(reply.len(), bytes);
        reply.extend_from_slice(terminator.as_bytes());
        writer.write_all(&reply).await.unwrap();
        let result = call.await.unwrap();
        if fits {
            assert_eq!(result.unwrap(), json!(text));
        } else {
            assert!(matches!(
                &*client.closed().await,
                Error::MessageTooLarge { limit: LIMIT, .. }
            ));
        }
    }
}

// Serde reads a struct from an array of its members too: this one would be
// a greeting, after five members that are not.
#[tokio::test]
async fn an_array_is_not_a_greeting() {
    let (client, peer) = UnixStream::pair().unwrap();
    let (_read, mut write) = peer.into_split();
    let greeting = support::greeting()["QMP"].clone();
    support::write(&mut write, json!([null, null, null, null, null, greeting])).await;
    assert!(matches!(
        Qmp::from_stream(client).await,
        Err(Error::Protocol(ProtocolViolation::NotAnObject(None)))
    ));
}

#[tokio::test]
async fn capabilities_enable_oob_only_when_the_greeting_offers_it() {
    for offered in [true, false] {
        let (client, peer) = UnixStream::pair().unwrap();
        let (read, mut write) = peer.into_split();
        let mut read = tokio::io::BufReader::new(read);
        let server = tokio::spawn(async move {
            let mut greeting = support::greeting();
            if !offered {
                greeting["QMP"]["capabilities"] = json!([]);
            }
            support::write(&mut write, greeting).await;
            let capabilities = support::read(&mut read).await;
            support::write(&mut write, json!({"return": {}, "id": capabilities["id"]})).await;
            (capabilities, read, write)
        });
        let (client, _) = Qmp::from_stream(client).await.unwrap();
        let (capabilities, _read, _write) = server.await.unwrap();
        assert_eq!(capabilities["execute"], "qmp_capabilities");
        if offered {
            assert_eq!(capabilities["arguments"], json!({"enable": ["oob"]}));
        } else {
            assert!(capabilities.get("arguments").is_none(), "{capabilities}");
            assert!(matches!(
                client
                    .execute_raw_oob::<Value>("query-yank", json!({}))
                    .await,
                Err(Error::InvalidRequest(InvalidRequestKind::OobNotNegotiated))
            ));
        }
    }
}

#[tokio::test]
async fn a_reply_before_its_request_is_written_closes_the_connection() {
    use tokio::io::AsyncBufReadExt;
    let (client, _, mut reader, mut writer) = support::raw_qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    let other = client.clone();
    let upload = tokio::spawn(async move {
        other
            .execute_raw::<Value>("upload", json!({"data":"x".repeat(4*1024*1024)}))
            .await
    });
    // The upload has begun writing, as request 1, and cannot finish.
    assert!(!reader.fill_buf().await.unwrap().is_empty());
    support::write(&mut writer, json!({"return": {}, "id": 1})).await;
    match upload.await.unwrap() {
        Err(Error::Disconnected(cause)) => assert!(matches!(
            &*cause,
            Error::Protocol(ProtocolViolation::EarlyResponse)
        )),
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn event_data_must_be_an_object() {
    for data in [json!([1]), json!("text"), json!(null)] {
        let (client, mut events, _reader, mut writer) =
            support::raw_qmp(ClientOptions::default()).await;
        support::write(
            &mut writer,
            json!({"event": "STOP", "data": data, "timestamp": {"seconds": 0, "microseconds": 0}}),
        )
        .await;
        let reason = tokio::time::timeout(Duration::from_secs(1), client.closed())
            .await
            .unwrap_or_else(|_| panic!("an event with data {data} was accepted"));
        assert!(
            matches!(
                &*reason,
                Error::Protocol(ProtocolViolation::EventDataNotObject)
            ),
            "{data}"
        );
        assert!(events.recv().await.is_none());
    }
}

#[tokio::test]
async fn a_unit_reply_must_be_an_empty_object() {
    let (client, _, mut peer) = support::qmp(ClientOptions::default()).await;
    let stop = qapi::commands::Stop::default();
    let (result, ()) = tokio::join!(client.execute(&stop), async {
        peer.recv_command().await.unwrap();
        peer.send_return(json!({"unexpected": true})).await.unwrap();
    });
    assert!(matches!(result, Err(Error::Decode(_))), "{result:?}");
    assert!(!client.is_closed());
}

#[tokio::test]
async fn an_event_capacity_beyond_the_channel_limit_is_refused() {
    let options = ClientOptions::default().with_event_capacity(usize::MAX);
    let (client, _peer) = UnixStream::pair().unwrap();
    assert!(matches!(
        Qmp::from_stream_with_options(client, options).await,
        Err(Error::InvalidRequest(InvalidRequestKind::Options {
            option: "event_capacity",
            ..
        }))
    ));
}
