mod support;
use qapi_protocol::{ClientOptions, Error, QMP_QUEUE_CAPACITY, Qmp, TimeoutPhase};
use serde_json::json;
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
const SHORT: Duration = Duration::from_millis(20);

#[tokio::test]
async fn pipelined_replies_are_routed_out_of_order() {
    let (client, _, mut peer) = support::qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    let mut tasks = Vec::new();
    for index in 0..8 {
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            let response = client
                .execute_raw(json!({"execute": "echo", "arguments": {"index": index}}))
                .await
                .unwrap();
            assert_eq!(response, json!(index));
        }));
    }
    let mut requests = Vec::new();
    for _ in 0..8 {
        requests.push(peer.recv_command_full().await.unwrap());
    }
    for request in requests.into_iter().rev() {
        peer.send_return_with_id(request.arguments["index"].clone(), request.id)
            .await
            .unwrap();
    }
    for task in tasks {
        task.await.unwrap();
    }
    assert_eq!(client.queue_depth(), 0);
    client.shutdown().await;
}

#[tokio::test]
async fn cancelled_sent_commands_keep_remote_queue_slots() {
    let (client, _, mut peer) = support::qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    let mut tasks = Vec::new();
    for _ in 0..QMP_QUEUE_CAPACITY {
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            client.execute_raw(json!({"execute": "slow"})).await
        }));
    }
    let mut requests = Vec::new();
    for _ in 0..8 {
        requests.push(peer.recv_command_full().await.unwrap());
    }
    for task in tasks {
        task.abort();
        let _ = task.await;
    }
    let other = client.clone();
    let ninth = tokio::spawn(async move { other.execute_raw(json!({"execute": "ninth"})).await });
    assert!(
        tokio::time::timeout(Duration::from_millis(50), peer.recv_command_full())
            .await
            .is_err()
    );
    assert_eq!(client.queue_depth(), 8);
    let first = requests.remove(0);
    peer.send_return_with_id(json!({}), first.id).await.unwrap();
    let request = peer.recv_command_full().await.unwrap();
    assert_eq!(request.name, "ninth");
    for request in requests {
        peer.send_return_with_id(json!({}), request.id)
            .await
            .unwrap();
    }
    peer.send_return_with_id(json!({}), request.id)
        .await
        .unwrap();
    ninth.await.unwrap().unwrap();
    assert_eq!(client.queue_depth(), 0);
    client.shutdown().await;
}

#[tokio::test]
async fn oob_bypasses_saturated_normal_admission_and_shutdown_wakes_everyone() {
    let (client, _, mut peer) = support::qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    let mut tasks = Vec::new();
    for _ in 0..9 {
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            client.execute_raw(json!({"execute": "slow"})).await
        }));
    }
    for _ in 0..8 {
        peer.recv_command_full().await.unwrap();
    }
    let other = client.clone();
    let oob = tokio::spawn(async move {
        other
            .execute_raw_oob(json!({"exec-oob": "query-yank"}))
            .await
    });
    let request = peer.recv_command_full().await.unwrap();
    assert!(request.oob);
    peer.send_return_with_id(json!([]), request.id)
        .await
        .unwrap();
    assert_eq!(oob.await.unwrap().unwrap(), json!([]));
    // This is real client shutdown, not killing the mock server as a workaround.
    client.shutdown().await;
    for task in tasks {
        assert!(matches!(task.await.unwrap(), Err(Error::Disconnected(_))));
    }
    assert_eq!(client.queue_depth(), 0);
    assert!(peer.recv_command().await.is_err());
}

#[tokio::test]
async fn command_deadline_includes_waiting_for_admission() {
    let (client, _, mut peer) = support::qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            client.execute_raw(json!({"execute": "slow"})).await
        }));
    }
    for _ in 0..8 {
        peer.recv_command_full().await.unwrap();
    }
    let result = client
        .execute_raw_with_timeout(json!({"execute": "never-send"}), Duration::from_millis(20))
        .await;
    assert!(matches!(
        result,
        Err(Error::Timeout {
            phase: TimeoutPhase::BeforeDispatch
        })
    ));
    assert!(
        !client.is_closed(),
        "timing out before dispatch must not close the session"
    );
    client.shutdown().await;
    for task in tasks {
        assert!(task.await.unwrap().is_err());
    }
}

#[tokio::test]
async fn timing_out_after_dispatch_says_so_and_closes_the_session() {
    let (client, _, mut peer) = support::qmp(ClientOptions::default()).await;
    let (result, _) = tokio::join!(
        client.execute_raw_with_timeout(json!({"execute": "slow"}), Duration::from_millis(20)),
        peer.recv_command()
    );
    assert!(matches!(
        result,
        Err(Error::Timeout {
            phase: TimeoutPhase::AfterDispatch
        })
    ));
    assert!(matches!(
        &*client.closed().await,
        Error::Timeout {
            phase: TimeoutPhase::AfterDispatch
        }
    ));
}

#[qapi::qapi(command, returns = "()", allow_oob)]
struct SlowOob {}

#[tokio::test]
async fn every_call_kind_takes_a_deadline_of_its_own() {
    type Call = fn(Arc<Qmp>) -> BoxFuture<qapi_protocol::Result<()>>;
    let calls: [(&str, Call); 5] = [
        ("execute", |qmp| {
            Box::pin(async move { qmp.execute_with_timeout(&SlowOob {}, SHORT).await })
        }),
        ("execute_oob", |qmp| {
            Box::pin(async move { qmp.execute_oob_with_timeout(&SlowOob {}, SHORT).await })
        }),
        ("execute_raw", |qmp| {
            Box::pin(async move {
                qmp.execute_raw_with_timeout(json!({"execute": "slow"}), SHORT)
                    .await
                    .map(drop)
            })
        }),
        ("execute_raw_oob", |qmp| {
            Box::pin(async move {
                qmp.execute_raw_oob_with_timeout(json!({"exec-oob": "slow"}), SHORT)
                    .await
                    .map(drop)
            })
        }),
        ("execute_raw_with_fds", |qmp| {
            Box::pin(async move {
                let fd = std::fs::File::open("/dev/null").unwrap().into();
                qmp.execute_raw_with_fds_and_timeout(json!({"execute": "getfd"}), vec![fd], SHORT)
                    .await
                    .map(drop)
            })
        }),
    ];
    // The connection's own deadline would outlast the test.
    let options = ClientOptions::default().with_command_timeout(Duration::from_secs(3600));
    for (name, call) in calls {
        let (client, _, mut peer) = support::qmp(options.clone()).await;
        let calling = async { tokio::join!(call(Arc::new(client)), peer.recv_command()).0 };
        let result = tokio::time::timeout(Duration::from_secs(5), calling)
            .await
            .unwrap_or_else(|_| panic!("{name} kept the connection's deadline"));
        assert!(
            matches!(
                result,
                Err(Error::Timeout {
                    phase: TimeoutPhase::AfterDispatch
                })
            ),
            "{name}: {result:?}"
        );
    }
}

// The call's deadline and the driver's expiry of the same request elapse
// together, and on a multi-thread runtime either can be noticed first.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_expired_call_reports_its_own_timeout_on_any_worker() {
    for _ in 0..100 {
        let (client, _, mut peer) = support::qmp(ClientOptions::default()).await;
        let client = Arc::new(client);
        let caller = client.clone();
        let call = tokio::spawn(async move {
            caller
                .execute_raw_with_timeout(json!({"execute": "slow"}), Duration::from_millis(5))
                .await
        });
        peer.recv_command().await.unwrap();
        let result = call.await.unwrap();
        assert!(
            matches!(
                result,
                Err(Error::Timeout {
                    phase: TimeoutPhase::AfterDispatch
                })
            ),
            "{result:?}"
        );
    }
}

#[tokio::test]
async fn cancelled_unanswered_request_still_has_a_background_deadline() {
    let options = ClientOptions::default().with_command_timeout(Duration::from_secs(2));
    let (client, _, mut peer) = support::qmp(options).await;
    let client = Arc::new(client);
    let other = client.clone();
    let task = tokio::spawn(async move { other.execute_raw(json!({"execute": "slow"})).await });
    peer.recv_command().await.unwrap();
    task.abort();
    let _ = task.await;
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(3)).await;
    let reason = client.closed().await;
    assert!(matches!(
        &*reason,
        Error::Timeout {
            phase: TimeoutPhase::AfterDispatch
        }
    ));
    assert_eq!(client.queue_depth(), 0);
    assert!(
        client
            .execute_raw(json!({"execute": "query-status"}))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn event_overflow_and_dropped_receivers_do_not_break_commands() {
    let options = ClientOptions::default().with_event_capacity(2);
    let (client, events, mut peer) = support::qmp(options).await;
    let client = Arc::new(client);
    for _ in 0..10 {
        peer.send_event("STOP", json!({})).await.unwrap();
    }
    let other = client.clone();
    let call = tokio::spawn(async move { other.execute_raw(json!({"execute": "ping"})).await });
    peer.recv_command().await.unwrap();
    peer.send_return(json!({})).await.unwrap();
    call.await.unwrap().unwrap();
    assert_eq!(client.dropped_events(), 8);
    drop(events);
    peer.send_event("STOP", json!({})).await.unwrap();
    let other = client.clone();
    let call = tokio::spawn(async move { other.execute_raw(json!({"execute": "ping"})).await });
    peer.recv_command().await.unwrap();
    peer.send_return(json!({})).await.unwrap();
    call.await.unwrap().unwrap();
    client.shutdown().await;
}

#[tokio::test]
async fn oversized_events_are_not_retained_in_the_event_queue() {
    let options = ClientOptions::default().with_max_event_bytes(256);
    let (client, mut events, mut peer) = support::qmp(options).await;
    let client = Arc::new(client);
    peer.send_event("LARGE", json!({"data":"x".repeat(1024)}))
        .await
        .unwrap();
    let other = client.clone();
    let call = tokio::spawn(async move { other.execute_raw(json!({"execute":"ping"})).await });
    peer.recv_command().await.unwrap();
    peer.send_return(json!({})).await.unwrap();
    call.await.unwrap().unwrap();
    assert_eq!(client.dropped_events(), 1);
    assert!(events.try_recv().is_err());
    client.shutdown().await;
}

#[tokio::test]
async fn a_request_cancelled_while_queued_is_never_sent() {
    use tokio::io::AsyncBufReadExt;
    let (client, _, mut reader, mut writer) = support::raw_qmp(ClientOptions::default()).await;
    let client = Arc::new(client);
    let other = client.clone();
    let upload = tokio::spawn(async move {
        other
            .execute_raw(json!({"execute":"upload","arguments":{"data":"x".repeat(4*1024*1024)}}))
            .await
    });
    // The driver is writing the upload, which the socket cannot hold, so
    // the next request waits in its queue.
    assert!(!reader.fill_buf().await.unwrap().is_empty());
    let other = client.clone();
    let cancelled =
        tokio::spawn(async move { other.execute_raw(json!({"execute":"cancelled"})).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    cancelled.abort();
    let _ = cancelled.await;
    let other = client.clone();
    let after = tokio::spawn(async move { other.execute_raw(json!({"execute":"after"})).await });
    let first = support::read(&mut reader).await;
    assert_eq!(first["execute"], "upload");
    support::write(&mut writer, json!({"return":{},"id":first["id"]})).await;
    let next = support::read(&mut reader).await;
    assert_eq!(next["execute"], "after");
    support::write(&mut writer, json!({"return":{},"id":next["id"]})).await;
    upload.await.unwrap().unwrap();
    after.await.unwrap().unwrap();
    assert_eq!(client.queue_depth(), 0);
}

// Once a dispatched request has expired, the connection is done: a request
// that reaches the driver before it notices the expiry is not sent either.
#[tokio::test]
async fn no_request_is_sent_after_a_dispatched_one_expired() {
    for _ in 0..16 {
        let (client, _, mut peer) = support::qmp(ClientOptions::default()).await;
        let client = Arc::new(client);
        let other = client.clone();
        let expiring = tokio::spawn(async move {
            other
                .execute_raw_with_timeout(json!({"execute": "slow"}), Duration::from_secs(1))
                .await
        });
        peer.recv_command().await.unwrap();
        tokio::time::pause();
        let other = client.clone();
        let late = tokio::spawn(async move { other.execute_raw(json!({"execute": "late"})).await });
        tokio::time::advance(Duration::from_secs(2)).await;
        assert!(matches!(
            expiring.await.unwrap(),
            Err(Error::Timeout {
                phase: TimeoutPhase::AfterDispatch
            })
        ));
        assert!(late.await.unwrap().is_err());
        match peer.recv_command().await {
            Err(_) => {}
            Ok((name, _)) => panic!("{name} was sent after the session expired"),
        }
        tokio::time::resume();
    }
}
