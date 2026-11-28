//! Real QMP smoke test. It needs no KVM, guest image or systemd.
use qapi::{QapiCommand, commands::*};
use qapi_protocol::{ClientOptions, Qmp, Required, Unavailability};
use std::{
    collections::BTreeSet,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

#[derive(Clone, Copy, Debug)]
enum MonitorFrontend {
    MonitorOption,
    MonitorObject,
}

#[tokio::test]
#[ignore = "requires a QEMU binary; set QRAB_TEST_QEMU and run with --ignored"]
async fn real_qmp_handshake_schema_and_typed_commands() {
    let binary = std::env::var_os("QRAB_TEST_QEMU")
        .unwrap_or_else(|| "qemu-system-x86_64".into());
    let objects = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::process::Command::new(&binary)
            .args(["-object", "help"])
            .kill_on_drop(true)
            .output(),
    )
    .await
    .expect("QEMU object discovery timed out")
    .expect("failed to list QEMU object types");
    assert!(
        objects.status.success(),
        "QEMU object discovery failed: {}",
        String::from_utf8_lossy(&objects.stderr)
    );
    let mut frontends = vec![MonitorFrontend::MonitorOption];
    if [objects.stdout, objects.stderr].iter().any(|output| {
        String::from_utf8_lossy(output)
            .lines()
            .any(|line| line.trim() == "monitor-qmp")
    }) {
        frontends.push(MonitorFrontend::MonitorObject);
    }
    for frontend in frontends {
        for pretty in [false, true] {
            smoke(&binary, frontend, pretty).await;
        }
    }
}

async fn smoke(binary: &OsStr, frontend: MonitorFrontend, pretty: bool) {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("qmp.sock");
    let mut command = tokio::process::Command::new(binary);
    command
        .args([
            "-machine",
            "none",
            "-accel",
            "tcg",
            "-nodefaults",
            "-display",
            "none",
            "-S",
            "-chardev",
        ])
        .arg(format!(
            "socket,id=qmp,path={},server=on,wait=off",
            socket.display()
        ));
    let pretty = if pretty { "on" } else { "off" };
    match frontend {
        MonitorFrontend::MonitorOption => command
            .arg("-mon")
            .arg(format!("chardev=qmp,mode=control,pretty={pretty}")),
        MonitorFrontend::MonitorObject => command
            .arg("-object")
            .arg(format!("monitor-qmp,id=monitor,chardev=qmp,pretty={pretty}")),
    };
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("failed to launch test QEMU");
    tokio::time::timeout(Duration::from_secs(10), async {
        while !socket.exists() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "QEMU exited before opening QMP"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("QEMU did not open QMP socket");
    let (qmp, _) = Qmp::new(&socket).await.unwrap();
    let version = qmp.execute(&QueryVersion::default()).await.unwrap();
    let triple =
        version.qemu.expect("modern query-version must include VersionTriple");
    assert_eq!(
        qmp.version(),
        (triple.major as u32, triple.minor as u32, triple.micro as u32)
    );
    eprintln!(
        "Validated QMP peer version: {:?}, frontend={frontend:?}, pretty={pretty}",
        qmp.version()
    );
    let status = qmp.execute(&QueryStatus::default()).await.unwrap();
    assert!(!status.running);
    let commands = qmp.execute(&QueryCommands::default()).await.unwrap();
    // The list the client asked for while connecting is the same one.
    let listed: BTreeSet<_> =
        qmp.commands().expect("QEMU answers query-commands").names().collect();
    assert_eq!(
        listed,
        commands.iter().map(|c| c.name.as_str()).collect::<BTreeSet<_>>()
    );
    let known: BTreeSet<_> =
        qapi::metadata::COMMANDS.iter().map(|c| c.wire_name()).collect();
    let missing: Vec<_> = commands
        .iter()
        .filter(|c| !known.contains(c.name.as_str()))
        .map(|c| &c.name)
        .collect();
    assert!(
        missing.is_empty(),
        "Live QEMU commands missing from generated metadata: {missing:?}"
    );
    let schema = qmp.execute(&QueryQmpSchema::default()).await.unwrap();
    assert!(schema.iter().any(|s| s.name == QueryStatus::NAME));
    // Real pipelining and OOB execution against the current QEMU monitor.
    let query_name = QueryName::default();
    let query_status = QueryStatus::default();
    let query_version = QueryVersion::default();
    let query_yank = QueryYank::default();
    let replies = tokio::join!(
        qmp.execute(&query_version),
        qmp.execute(&query_status),
        qmp.execute(&query_name),
        qmp.execute_oob(&query_yank)
    );
    replies.0.unwrap();
    replies.1.unwrap();
    replies.2.unwrap();
    replies.3.unwrap();
    // A name that QEMU did not list fails before anything is sent.
    assert!(matches!(
        qmp.execute_raw::<serde_json::Value>(
            "qrab-no-such-command",
            serde_json::json!({})
        )
        .await,
        Err(qapi_protocol::Error::Unavailable {
            reason: Unavailability::NotOffered,
            ..
        })
    ));
    qmp.execute(&QueryStatus::default()).await.unwrap();
    qmp.execute(&Quit::default()).await.unwrap();
    qmp.shutdown().await;
    let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(status.success());
}

/// Launch QEMU without a machine, its QMP monitor on a socket in `dir`.
async fn launch(
    binary: &OsStr,
    dir: &Path,
    extra: &[&str],
) -> (tokio::process::Child, PathBuf) {
    let socket = dir.join("qmp.sock");
    let mut child = tokio::process::Command::new(binary)
        .args([
            "-machine",
            "none",
            "-accel",
            "tcg",
            "-nodefaults",
            "-display",
            "none",
        ])
        .args(extra)
        .arg("-chardev")
        .arg(format!(
            "socket,id=qmp,path={},server=on,wait=off",
            socket.display()
        ))
        .args(["-mon", "chardev=qmp,mode=control"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("failed to launch test QEMU");
    tokio::time::timeout(Duration::from_secs(10), async {
        while !socket.exists() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "QEMU exited before opening QMP"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("QEMU did not open QMP socket");
    (child, socket)
}

#[tokio::test]
#[ignore = "requires a QEMU binary; set QRAB_TEST_QEMU and run with --ignored"]
async fn real_qmp_lists_its_commands_before_the_machine_exists() {
    let binary = std::env::var_os("QRAB_TEST_QEMU")
        .unwrap_or_else(|| "qemu-system-x86_64".into());
    let dir = tempfile::tempdir().unwrap();
    let (mut child, socket) =
        launch(&binary, dir.path(), &["--preconfig"]).await;
    // query-commands runs in the preconfig state, before machine creation.
    let (qmp, _) = Qmp::new(&socket).await.unwrap();
    let commands =
        qmp.commands().expect("QEMU answers query-commands in preconfig");
    for name in ["query-status", "x-exit-preconfig", "quit"] {
        assert!(commands.offers(name), "{name} is listed");
    }
    assert!(qmp.supports::<QueryStatus>());
    qmp.require(&[Required::of::<QueryStatus>(), Required::of::<Quit>()])
        .unwrap();
    qmp.shutdown().await;
    // Without the list, the same name reaches QEMU, which refuses it.
    let options = ClientOptions::default().with_probe_commands(false);
    let (qmp, _) = Qmp::connect_with_options(&socket, options).await.unwrap();
    assert!(qmp.commands().is_none());
    assert!(matches!(
        qmp.execute_raw::<serde_json::Value>(
            "qrab-no-such-command",
            serde_json::json!({})
        )
        .await,
        Err(qapi_protocol::Error::Remote(_))
    ));
    qmp.execute(&Quit::default()).await.unwrap();
    qmp.shutdown().await;
    let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(status.success());
}
