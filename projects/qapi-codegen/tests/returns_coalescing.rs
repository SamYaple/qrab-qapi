//! Compile a command whose newer return type its older one is on the wire
//! from a release, as `query-target`'s `QemuTargetInfo` is `TargetInfo`
//! from 3.0: the enum of both, whose newer variant starts at that release.
mod common;
use common::{generate, git, run_fixture, write_module};
use std::path::Path;

#[test]
fn a_coalesced_return_type_is_the_returns_type_of_a_build_from_its_release() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("schema");
    std::fs::create_dir_all(repo.join("qapi")).unwrap();
    git(&repo, &["init", "-q"]);
    let arch = "{'enum':'Arch','data':['x86','arm']}";
    for (tag, schema) in [
        (
            "v5.0.0",
            "{'struct':'TargetInfo','data':{'arch':'str'}}
             {'command':'query-target','returns':'TargetInfo'}",
        ),
        (
            "v6.0.0",
            "{'struct':'TargetInfo','data':{'arch':'Arch'}}
             {'command':'query-target','returns':'TargetInfo'}",
        ),
        (
            "v10.1.0",
            "{'struct':'QemuTargetInfo','data':{'arch':'Arch'}}
             {'command':'query-target','returns':'QemuTargetInfo'}",
        ),
    ] {
        std::fs::write(
            repo.join("qapi/qapi-schema.json"),
            format!("{arch}\n{schema}"),
        )
        .unwrap();
        git(&repo, &["add", "."]);
        git(
            &repo,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "-qm",
                tag,
            ],
        );
        git(&repo, &["tag", tag]);
    }
    let output = generate(&repo, &["v5.0.0", "v6.0.0", "v10.1.0"]).unwrap();
    assert!(
        output
            .commands
            .contains(r#"#[qapi(returns(added = "6.0.0", value = "QemuTargetInfo"))]"#),
        "{}",
        output.commands
    );
    let fixture = temp.path().join("fixture");
    write_module(&fixture.join("src/generated"), output);
    let prelude = r#"
use qapi::{QapiCommand, with_version};
mod generated;
pub use generated::*;
use generated::commands::QueryTarget;
type Returns = <QueryTarget as QapiCommand>::Returns;
fn decode(version: (u32, u32, u32), arch: &str) -> Returns {
    with_version(version, || serde_json::from_value(serde_json::json!({ "arch": arch }))).unwrap()
}
"#;
    // The full history keeps the enum, whose newer variant starts where the
    // types agree.
    run_fixture(
        &workspace,
        &fixture,
        &format!(
            "{prelude}{}",
            r#"
fn main() {
    use generated::commands::QueryTargetReturns;
    assert!(matches!(decode((5, 0, 0), "x86"), QueryTargetReturns::V5_0_0(TargetInfo { .. })));
    match decode((9, 2, 0), "arm") {
        QueryTargetReturns::V6_0_0(QemuTargetInfo { arch: Arch::Arm }) => {}
        other => panic!("{other:?}"),
    }
}
"#
        ),
    );
}
