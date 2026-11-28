//! Run-time warnings go to standard error once per process, so a test of
//! them runs its body again in a child process of its own and reads what
//! that child printed.

/// Whether this process is the child that runs a test's body.
pub fn is_child() -> bool {
    std::env::var_os("QAPI_MACROS_TEST_CHILD").is_some()
}

/// Run the test `name` of this binary alone in a child process and return
/// its standard error.
pub fn stderr_of(name: &str) -> String {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([name, "--exact", "--nocapture", "--test-threads=1"])
        .env("QAPI_MACROS_TEST_CHILD", "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{name} failed in the child: {stderr}");
    stderr
}
