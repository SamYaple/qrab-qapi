use std::{path::Path, process::Command};

pub fn git(repo: &Path, args: &[&str]) {
    let result = Command::new("git")
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "tag.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

/// The QMP modules of the releases `tags` of the QEMU tree `repo`.
pub fn generate(
    repo: &Path,
    tags: &[&str],
) -> Result<qapi_codegen::unified::GeneratedOutput, qapi_codegen::unified::GenerateError> {
    use qapi_codegen::unified::{Docs, Protocol, Releases};
    qapi_codegen::unified::generate(repo, Releases::Tags(tags), Protocol::Qmp, Docs::Reference)
}

pub fn write_module(path: &Path, output: qapi_codegen::unified::GeneratedOutput) {
    output.write_to(path).unwrap();
}

/// The dependency line `name = ...` of a manifest, so that a fixture asks for
/// what the workspace asks for.
fn dependency(manifest: &Path, name: &str) -> String {
    let text = std::fs::read_to_string(manifest).unwrap();
    let prefix = format!("{name} = ");
    text.lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("{} declares no {name}", manifest.display()))
        .to_owned()
}

/// Build and run a fixture crate against the workspace's qrab-qapi. It
/// resolves its dependencies from a copy of the workspace's `Cargo.lock`,
/// offline, and builds under this test's target directory. Its root
/// declares, before `main`, the `metadata_types` module that the generated
/// metadata re-exports, a copy of qrab-qapi's.
pub fn run_fixture(workspace: &Path, fixture: &Path, main: &str) {
    let manifest = format!(
        r#"[package]
name = "qapi-generated-fixture"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
qapi = {{ package = "qrab-qapi", path = {:?} }}
qapi-macros = {{ package = "qrab-qapi-macros", path = {:?} }}
{}
{}
"#,
        workspace.join("projects/qapi"),
        workspace.join("projects/qapi-macros"),
        dependency(&workspace.join("Cargo.toml"), "serde"),
        dependency(&workspace.join("Cargo.toml"), "serde_json"),
    );
    let fixture_manifest = fixture.join("Cargo.toml");
    std::fs::write(&fixture_manifest, manifest).unwrap();
    std::fs::write(
        fixture.join("src/main.rs"),
        format!("mod metadata_types;\n{main}"),
    )
    .unwrap();
    std::fs::copy(
        workspace.join("projects/qapi/src/metadata_types.rs"),
        fixture.join("src/metadata_types.rs"),
    )
    .unwrap();
    std::fs::copy(workspace.join("Cargo.lock"), fixture.join("Cargo.lock")).unwrap();
    let target = Path::new(env!("CARGO_TARGET_TMPDIR")).join("qapi-codegen-fixtures");
    let cargo = |args: &[&str]| {
        let mut command = Command::new(env!("CARGO"));
        command
            .args(args)
            .arg("--manifest-path")
            .arg(&fixture_manifest);
        command
    };
    // Lock the fixture package itself; every other package keeps the
    // workspace's locked version, and the run then refuses any change.
    let update = cargo(&["update", "--offline", "--quiet", "--workspace"])
        .output()
        .unwrap();
    assert!(
        update.status.success(),
        "Locking the fixture failed:\n{}",
        String::from_utf8_lossy(&update.stderr)
    );
    let mut command = cargo(&["run", "--locked", "--offline", "--quiet"]);
    command.arg("--target-dir").arg(target);
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "Generated fixture failed:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
