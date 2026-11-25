//! The `qapi-codegen` binary on fixture QEMU trees: its exit status, the
//! release pins it reads, what it writes and how its errors name their
//! inputs.

use std::path::Path;
use std::process::{Command, Output};

/// Run git in `dir`, whatever repository the environment names.
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_COMMON_DIR")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
        ])
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
        .args(["-c", "core.hooksPath=/dev/null"])
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}

/// A directory with a QEMU tree `qemu` that has a tagged commit for each
/// release, with its QMP and QGA schemas, and the pin of each release.
fn workspace(releases: &[(&str, &str, &str)]) -> (tempfile::TempDir, Vec<String>) {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("qemu");
    std::fs::create_dir_all(repo.join("qapi")).unwrap();
    std::fs::create_dir_all(repo.join("qga")).unwrap();
    git(&repo, &["init", "-q"]);
    let mut pins = Vec::new();
    for (tag, qmp, qga) in releases {
        std::fs::write(repo.join("qapi/qapi-schema.json"), qmp).unwrap();
        std::fs::write(repo.join("qga/qapi-schema.json"), qga).unwrap();
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-qm", tag]);
        git(&repo, &["tag", tag]);
        pins.push(format!("{tag} {}", git(&repo, &["rev-parse", "HEAD"])));
    }
    (dir, pins)
}

/// Run the binary in `dir`, with a lockfile `pins.lock` of `pins`.
fn codegen(dir: &Path, pins: &[&str], args: &[&str]) -> Output {
    std::fs::write(dir.join("pins.lock"), pins.join("\n") + "\n").unwrap();
    Command::new(env!("CARGO_BIN_EXE_qapi-codegen"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

const MODULES: [&str; 4] = ["commands.rs", "events.rs", "metadata.rs", "mod.rs"];

/// The names of the files in `dir`, sorted.
fn files(dir: &Path) -> Vec<String> {
    let mut names: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn help_and_usage_errors_have_their_own_exit_status() {
    let dir = tempfile::tempdir().unwrap();
    let help = codegen(dir.path(), &[], &["--help"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .starts_with("Usage: qapi-codegen ")
    );
    assert!(help.stderr.is_empty());
    for (args, message) in [
        (&[][..], "--output-dir is required"),
        (
            &["--output-dir", "out", "--bogus"],
            "Unknown option --bogus",
        ),
    ] {
        let output = codegen(dir.path(), &[], args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
        let stderr = stderr(&output);
        assert!(
            stderr.starts_with(&format!("error: {message}\n\nUsage: qapi-codegen ")),
            "{stderr}"
        );
    }
    assert_eq!(files(dir.path()), ["pins.lock"]);
}

#[test]
fn the_versions_file_selects_the_releases_to_generate() {
    let mislabeled = "##\n# @Wrong:\n#\n# Since: 2.8\n##\n{ 'struct': 'Right', 'data': {} }\n";
    let (dir, pins) = workspace(&[
        ("v2.8.0", mislabeled, "{ 'command': 'guest-old' }"),
        (
            "v9.0.0",
            "{ 'command': 'x-ping' }",
            "{ 'command': 'guest-ping' }",
        ),
    ]);
    let pins: Vec<_> = pins.iter().map(String::as_str).collect();
    let output = codegen(
        dir.path(),
        &pins,
        &[
            "--versions-file",
            "pins.lock",
            "--output-dir",
            "out",
            "qemu",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let stderr = stderr(&output);
    let mut lines = stderr.lines();
    assert!(
        lines.next().unwrap().starts_with("warning: v2.8.0: "),
        "{stderr}"
    );
    assert!(
        lines.next().unwrap().starts_with("Generated out in "),
        "{stderr}"
    );
    assert_eq!(lines.next(), None);
    assert_eq!(files(&dir.path().join("out")), MODULES);
    let commands = std::fs::read_to_string(dir.path().join("out/commands.rs")).unwrap();
    assert!(commands.contains("pub struct XPing"), "{commands}");

    // Only the pinned releases are read: v2.8.0 is not, here.
    let output = codegen(
        dir.path(),
        &pins[1..],
        &[
            "--qga",
            "--versions-file=pins.lock",
            "--output-dir=qga",
            "qemu",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", self::stderr(&output));
    assert!(self::stderr(&output).starts_with("Generated qga in "));
    let commands = std::fs::read_to_string(dir.path().join("qga/commands.rs")).unwrap();
    assert!(commands.contains("pub struct GuestPing"), "{commands}");
    assert!(!commands.contains("GuestOld"), "{commands}");

    // Without the file, the embedded pins apply, which this tree lacks.
    let output = codegen(dir.path(), &[], &["--output-dir", "embedded", "qemu"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = self::stderr(&output);
    assert!(
        stderr.starts_with(
            "error: Failed to check qemu against the release pins of the embedded qemu-versions.lock\n  caused by: Missing QEMU release "
        ),
        "{stderr}"
    );
    assert!(!dir.path().join("embedded").exists());
}

#[test]
fn a_failure_names_its_input_and_writes_nothing() {
    let (dir, pins) = workspace(&[
        (
            "v9.0.0",
            "{ 'command': 'ping' }",
            "{ 'command': 'guest-ping' }",
        ),
        (
            "v9.1.0",
            "{ 'command': 'ping' }\n{ 'command': 'ping' }",
            "{ 'command': 'guest-ping' }",
        ),
    ]);
    let out = dir.path().join("out");
    std::fs::create_dir(&out).unwrap();
    std::fs::write(out.join("mod.rs"), "old").unwrap();
    let zeros = format!("v9.0.0 {}", "0".repeat(40));
    let commit = pins[0].split_once(' ').unwrap().1;
    for (pins, message) in [
        (
            vec!["# The pins", "v9.0.0 nothex"],
            "error: Invalid release lockfile pins.lock\n  caused by: Invalid commit pin for v9.0.0 on line 2\n".to_owned(),
        ),
        (
            vec![zeros.as_str()],
            format!(
                "error: Failed to check qemu against the release pins of pins.lock\n  caused by: QEMU release v9.0.0 moved: expected {}, found {commit}\n",
                "0".repeat(40)
            ),
        ),
        (
            vec![&pins[0], &pins[1]],
            "error: Failed to generate the QMP modules from qemu\n  caused by: Failed to process v9.1.0\n  caused by: Invalid QAPI command 'ping' in v9.1.0:qapi/qapi-schema.json:2:1: ".to_owned(),
        ),
        (
            vec!["v0.15.0 0000000000000000000000000000000000000000"],
            "error: Refusing to write empty QMP modules: pins.lock pins no release from 1.0, the first with a QMP schema\n".to_owned(),
        ),
    ] {
        let output = codegen(
            dir.path(),
            &pins,
            &["qemu", "--output-dir", "out", "--versions-file", "pins.lock"],
        );
        assert_eq!(output.status.code(), Some(1), "{pins:?}");
        let stderr = stderr(&output);
        assert!(stderr.starts_with(&message), "{pins:?}: {stderr}");
        assert_eq!(files(&out), ["mod.rs"]);
        assert_eq!(std::fs::read_to_string(out.join("mod.rs")).unwrap(), "old");
    }
    let output = codegen(
        dir.path(),
        &[],
        &["--versions-file", "missing.lock", "--output-dir", "x"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output)
            .starts_with("error: Failed to read the release lockfile missing.lock\n  caused by: "),
        "{}",
        stderr(&output)
    );
    assert!(!dir.path().join("x").exists());
}

#[test]
fn empty_guest_agent_modules_are_refused_too() {
    let (dir, pins) = workspace(&[("v0.15.0", "", "{ 'command': 'guest-ping' }")]);
    let output = codegen(
        dir.path(),
        &["v0.14.0 0000000000000000000000000000000000000000"],
        &[
            "qemu",
            "--qga",
            "--output-dir",
            "out",
            "--versions-file",
            "pins.lock",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stderr(&output),
        "error: Refusing to write empty QGA modules: pins.lock pins no release from 0.15, the first with a QGA schema\n"
    );
    let output = codegen(
        dir.path(),
        &[&pins[0]],
        &[
            "qemu",
            "--qga",
            "--output-dir",
            "out",
            "--versions-file",
            "pins.lock",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(files(&dir.path().join("out")), MODULES);
}

#[test]
fn a_write_failure_names_the_output() {
    let (dir, pins) = workspace(&[("v9.0.0", "{ 'command': 'ping' }", "")]);
    std::fs::write(dir.path().join("out"), "a file").unwrap();
    let output = codegen(
        dir.path(),
        &[&pins[0]],
        &[
            "qemu",
            "--output-dir",
            "out",
            "--versions-file",
            "pins.lock",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = stderr(&output);
    assert!(
        stderr.starts_with("error: Failed to write out\n  caused by: "),
        "{stderr}"
    );
}

#[test]
fn schema_docs_include_the_schema_documentation() {
    let schema = "##\n# @x-ping:\n#\n# Checks that the monitor answers.\n#\n# Since: 9.0\n##\n{ 'command': 'x-ping' }\n";
    let (dir, pins) = workspace(&[("v9.0.0", schema, "")]);
    let pins: Vec<_> = pins.iter().map(String::as_str).collect();
    for (out, documented) in [("docs", true), ("reference", false)] {
        let mut args = vec!["--versions-file", "pins.lock", "--output-dir", out, "qemu"];
        if documented {
            args.push("--schema-docs");
        }
        let output = codegen(dir.path(), &pins, &args);
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
        let commands = std::fs::read_to_string(dir.path().join(out).join("commands.rs")).unwrap();
        assert_eq!(
            commands.contains("Checks that the monitor answers."),
            documented,
            "{commands}"
        );
        assert_eq!(
            commands.contains("see the QEMU QMP Reference Manual"),
            !documented,
            "{commands}"
        );
    }
}

/// The error example of docs/codegen.md, which this run must print.
fn documented_error() -> String {
    let docs = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/codegen.md"
    ))
    .unwrap();
    let (_, example) = docs.split_once("```text\nerror: ").unwrap();
    let (example, _) = example.split_once("```").unwrap();
    format!("error: {example}")
}

#[test]
fn an_error_prints_as_the_documentation_shows() {
    let (dir, pins) = workspace(&[(
        "v11.1.3",
        "{ 'command': 'ping' }",
        "{ 'command': 'broken', 'returns': 'Missing' }",
    )]);
    let output = codegen(
        dir.path(),
        &[&pins[0]],
        &[
            "--qga",
            "--versions-file",
            "pins.lock",
            "--output-dir",
            "out",
            "qemu",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stderr(&output), documented_error());
}

#[test]
fn a_parse_error_names_its_fault_in_words() {
    let (dir, pins) = workspace(&[(
        "v11.1.3",
        "{ 'command': 'ping' }",
        "{ 'command': 'ping' }\n{ 'command': 'x' 'y' }\n",
    )]);
    let output = codegen(
        dir.path(),
        &[&pins[0]],
        &[
            "--qga",
            "--versions-file",
            "pins.lock",
            "--output-dir",
            "out",
            "qemu",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stderr(&output),
        concat!(
            "error: Failed to generate the QGA modules from qemu\n",
            "  caused by: Failed to read the schema of v11.1.3\n",
            "  caused by: Failed to parse schema file v11.1.3:qga/qapi-schema.json with the grammar of QEMU 11.1.3\n",
            "  caused by: QAPI parse error at line 2, column 18: unexpected character: \"'y' }\\n\"\n",
        )
    );
}
