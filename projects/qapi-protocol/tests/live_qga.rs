//! Real current-QEMU guest-agent smoke; only a temporary private Unix socket
//! and files are used. No guest VM, root privileges or host configuration.
use qapi::qga::commands::{
    GuestExecCommand, GuestExecStatusCommand, GuestFileClose, GuestFileOpen,
    GuestFileReadCommand, GuestPing,
};
use qapi_protocol::qga::{FILE_READ_REPLY_BYTES, Qga};
use std::{process::Stdio, time::Duration};

#[tokio::test]
#[ignore = "requires qemu-ga; set QRAB_TEST_QGA and run with --ignored"]
async fn real_guest_agent_sync_ping_and_large_typed_file_read() {
    let binary =
        std::env::var_os("QRAB_TEST_QGA").unwrap_or_else(|| "qemu-ga".into());
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("qga.sock");
    let pidfile = dir.path().join("qga.pid");
    let config = dir.path().join("qga.conf");
    std::fs::write(&config, "[general]\n").unwrap();
    let payload = dir.path().join("payload");
    // A multi-megabyte typed reply, read under the per-call reply limit.
    std::fs::write(&payload, vec![b'x'; 2 * 1024 * 1024]).unwrap();
    let mut child = tokio::process::Command::new(binary)
        .arg("-c").arg(config).args(["-m", "unix-listen", "-p"]).arg(&socket)
        .arg("-t").arg(dir.path()).arg("-f").arg(pidfile)
        .arg("--allow-rpcs=guest-sync-delimited,guest-info,guest-ping,guest-file-open,guest-file-read,guest-file-close,guest-exec,guest-exec-status")
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::inherit())
        .kill_on_drop(true).spawn().expect("failed to launch test guest agent");
    tokio::time::timeout(Duration::from_secs(10), async {
        while !socket.exists() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "guest agent exited before listening"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("guest agent did not open its socket");
    let mut client = Qga::new(&socket).await.unwrap();
    eprintln!("Validated guest-agent version: {:?}", client.version());
    client.execute(&GuestPing::default()).await.unwrap();
    let mut open = GuestFileOpen::new(payload.to_str().unwrap().into());
    open.mode = Some("r".into());
    let handle = client.execute(&open).await.unwrap();
    let mut read = GuestFileReadCommand::new(handle);
    read.count = Some(2 * 1024 * 1024);
    let result = client
        .execute_with_reply_limit(&read, FILE_READ_REPLY_BYTES)
        .await
        .unwrap();
    assert_eq!(result.count, 2 * 1024 * 1024);
    assert!(result.buf_b64.len() > 2 * 1024 * 1024);
    // All bytes are x; check the exact base64 shape without another dependency.
    let expected = "eHh4".repeat((2 * 1024 * 1024) / 3) + "eHg=";
    assert_eq!(result.buf_b64, expected);
    client.execute(&GuestFileClose::new(handle)).await.unwrap();
    // Alternates use their actual scalar wire shape (capture-output: "none").
    let mut exec = GuestExecCommand::new("/usr/bin/test".into());
    exec.arg = Some(vec!["-e".into(), payload.to_str().unwrap().into()]);
    exec.capture_output = Some(qapi::qga::GuestExecCaptureOutput::Mode(
        qapi::qga::GuestExecCaptureOutputMode::None,
    ));
    let process = client.execute(&exec).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let status = client
                .execute(&GuestExecStatusCommand::new(process.pid))
                .await
                .unwrap();
            if status.exited {
                assert_eq!(status.exitcode, Some(0));
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    client.shutdown();
    child.start_kill().unwrap();
    tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .unwrap()
        .unwrap();
}
