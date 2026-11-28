//! Descriptors ride SCM_RIGHTS with the frame of the command that takes them
//! (`getfd`), the way QEMU's QMP socket chardev expects.
use qapi_protocol::Qmp;
use rustix::net::{
    RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags, recvmsg,
};
use serde_json::{Value, json};
use std::{
    io::{IoSliceMut, Read, Write},
    mem::MaybeUninit,
    os::fd::OwnedFd,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};

const GREETING: &str = r#"{"QMP":{"version":{"qemu":{"major":11,"minor":1,"micro":3},"package":""},"capabilities":[]}}"#;

/// Read one line, collecting any descriptors that arrive with its bytes.
async fn line_with_fds(stream: &UnixStream) -> (Value, Vec<OwnedFd>) {
    let mut bytes = Vec::new();
    let mut fds = Vec::new();
    loop {
        let mut chunk = [0u8; 1];
        let mut space =
            [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(4))];
        let n = stream
            .async_io(tokio::io::Interest::READABLE, || {
                let mut control = RecvAncillaryBuffer::new(&mut space);
                let mut iov = [IoSliceMut::new(&mut chunk)];
                let msg = recvmsg(
                    stream,
                    &mut iov,
                    &mut control,
                    RecvFlags::empty(),
                )?;
                for message in control.drain() {
                    if let RecvAncillaryMessage::ScmRights(received) = message {
                        fds.extend(received);
                    }
                }
                Ok(msg.bytes)
            })
            .await
            .unwrap();
        assert_eq!(n, 1, "peer closed mid-line");
        if chunk[0] == b'\n' {
            return (serde_json::from_slice(&bytes).unwrap(), fds);
        }
        bytes.push(chunk[0]);
    }
}

async fn send(stream: &mut UnixStream, value: Value) {
    stream.write_all(format!("{value}\n").as_bytes()).await.unwrap();
}

/// Answer the capability negotiation and then the client's
/// `query-commands`, as a peer without that command, so that the client
/// checks calls against the release alone.
async fn negotiate(stream: &mut UnixStream) {
    let (capabilities, fds) = line_with_fds(stream).await;
    assert_eq!(capabilities["execute"], "qmp_capabilities");
    assert!(fds.is_empty());
    send(stream, json!({"return":{},"id":capabilities["id"]})).await;
    let (query, fds) = line_with_fds(stream).await;
    assert_eq!(query["execute"], "query-commands");
    assert!(fds.is_empty());
    send(
        stream,
        json!({"error":{"class":"CommandNotFound","desc":"The command query-commands has not been found"},"id":query["id"]}),
    )
    .await;
}

#[tokio::test]
async fn getfd_carries_its_descriptor_and_plain_commands_carry_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("qmp.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        stream.write_all(format!("{GREETING}\n").as_bytes()).await.unwrap();
        negotiate(&mut stream).await;

        let (getfd, fds) = line_with_fds(&stream).await;
        assert_eq!(getfd["execute"], "getfd");
        assert_eq!(getfd["arguments"]["fdname"], "nic-lan-tap0");
        assert_eq!(fds.len(), 1, "exactly the getfd descriptor");
        // The received descriptor is the client's pipe: read through it,
        // then close it before answering.
        let mut read = std::fs::File::from(fds.into_iter().next().unwrap());
        let mut text = [0u8; 22];
        read.read_exact(&mut text).unwrap();
        assert_eq!(&text, b"through the descriptor");
        drop(read);
        send(&mut stream, json!({"return":{},"id":getfd["id"]})).await;

        let (plain, fds) = line_with_fds(&stream).await;
        assert_eq!(plain["execute"], "query-status");
        assert!(fds.is_empty(), "a plain command carries no descriptors");
        send(
            &mut stream,
            json!({"return":{"status":"running","running":true},"id":plain["id"]}),
        )
        .await;

        // Keep the stream open until the client is done.
        let mut rest = String::new();
        let _ = BufReader::new(stream).read_line(&mut rest).await;
    });

    let (qmp, _events) = Qmp::new(&path).await.unwrap();
    let (read, write) = rustix::pipe::pipe().unwrap();
    let mut write = std::fs::File::from(write);
    write.write_all(b"through the descriptor").unwrap();
    qmp.getfd("nic-lan-tap0", read).await.unwrap();
    // The server closed its copy before answering; the client closed its own
    // once sent, so no reader is left.
    let error = write.write_all(b"x").unwrap_err();
    assert_eq!(
        error.raw_os_error(),
        Some(rustix::io::Errno::PIPE.raw_os_error()),
        "the client kept the descriptor"
    );
    let status =
        qmp.execute_raw::<Value>("query-status", json!({})).await.unwrap();
    assert_eq!(status["status"], "running");
    drop(qmp);
    server.await.unwrap();
}

#[tokio::test]
async fn more_descriptors_than_one_message_carries_are_refused_before_sending()
{
    let (client, server) = UnixStream::pair().unwrap();
    let mut server = server;
    let (qmp, ()) = tokio::join!(Qmp::from_stream(client), async {
        server.write_all(format!("{GREETING}\n").as_bytes()).await.unwrap();
        negotiate(&mut server).await;
    });
    let (qmp, _events) = qmp.unwrap();
    let fds: Vec<OwnedFd> = (0..254)
        .map(|_| std::fs::File::open("/dev/null").unwrap().into())
        .collect();
    assert!(matches!(
        qmp.execute_raw_with_fds::<Value>("getfd", json!({"fdname":"x"}), fds)
            .await,
        Err(qapi_protocol::Error::InvalidRequest(
            qapi_protocol::InvalidRequestKind::TooManyFds {
                count: 254,
                max: 253,
                ..
            }
        ))
    ));
    assert!(!qmp.is_closed(), "the refusal does not end the connection");
    assert_eq!(qmp.queue_depth(), 0);
}

#[tokio::test]
async fn descriptors_of_a_refused_request_are_closed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("qmp.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        stream.write_all(format!("{GREETING}\n").as_bytes()).await.unwrap();
        negotiate(&mut stream).await;
        let mut rest = String::new();
        let _ = BufReader::new(stream).read_line(&mut rest).await;
    });
    let (qmp, _events) = Qmp::new(&path).await.unwrap();
    let (read, write) = rustix::pipe::pipe().unwrap();
    // Invalid arguments are refused before any I/O, and the descriptor goes with it.
    assert!(matches!(
        qmp.execute_raw_with_fds::<Value>("getfd", json!([]), vec![read]).await,
        Err(qapi_protocol::Error::InvalidRequest(
            qapi_protocol::InvalidRequestKind::ArgumentsNotObject { .. }
        ))
    ));
    // With the only reader closed, writing to the pipe fails with EPIPE.
    let mut write = std::fs::File::from(write);
    let error = write.write_all(b"x").unwrap_err();
    assert_eq!(
        error.raw_os_error(),
        Some(rustix::io::Errno::PIPE.raw_os_error())
    );
    drop(qmp);
    server.await.unwrap();
}

#[tokio::test]
async fn descriptors_go_once_with_a_frame_written_in_parts() {
    let (client, server) = UnixStream::pair().unwrap();
    let mut server = server;
    let (qmp, ()) = tokio::join!(Qmp::from_stream(client), async {
        server.write_all(format!("{GREETING}\n").as_bytes()).await.unwrap();
        negotiate(&mut server).await;
    });
    let (qmp, _events) = qmp.unwrap();
    let fd: OwnedFd = std::fs::File::open("/dev/null").unwrap().into();
    // More than the socket buffers, so the frame takes several writes.
    let padding = "x".repeat(512 * 1024);
    let args = json!({"opaque": padding});
    let call = qmp.execute_raw_with_fds::<Value>("add-fd", args, vec![fd]);
    let (result, ()) = tokio::join!(call, async {
        let (add, fds) = line_with_fds(&server).await;
        assert_eq!(
            add["arguments"]["opaque"].as_str().unwrap().len(),
            512 * 1024
        );
        assert_eq!(
            fds.len(),
            1,
            "the descriptor went with the first part only"
        );
        send(
            &mut server,
            json!({"return":{"fdset-id":0,"fd":3},"id":add["id"]}),
        )
        .await;
    });
    assert_eq!(result.unwrap()["fdset-id"], 0);
}
