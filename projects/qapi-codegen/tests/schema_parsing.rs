//! The pinned releases against the QEMU tree: the pins themselves, release
//! discovery and extraction, and every pinned schema parsed with its grammar.

use std::path::{Path, PathBuf};
use std::process::Command;

use qapi_codegen::versions::{
    PINNED_VERSIONS, PinnedVersion, TreeError, discover_qga_version_tags, discover_version_tags,
    extract_schema_from_tag, find_qga_schema_entry, find_schema_entry, parse_pinned_versions,
    parse_version_tag, parse_version_tag_full, verify_pinned_versions,
};
use qapi_codegen::{QapiSchemaToken, parse_schema_with_version, read_schema_with_version};

fn qemu_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/qemu")
}

fn pins() -> Vec<PinnedVersion> {
    parse_pinned_versions(PINNED_VERSIONS).unwrap()
}

/// Run git in `dir`, whatever repository the environment names.
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_COMMON_DIR")
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

/// The grammar of the newest pin, which vendor/qemu must be checked out at:
/// the working-tree test below reads its files.
fn baseline() -> (u32, u32, u32) {
    let pin = pins().pop().unwrap();
    assert_eq!(
        git(&qemu_root(), &["rev-parse", "HEAD"]),
        pin.commit,
        "vendor/qemu is not checked out at {}",
        pin.tag
    );
    parse_version_tag(&pin.tag).unwrap()
}

/// What a schema defines.
#[derive(Debug, Default)]
struct Summary {
    definitions: usize,
    events: usize,
    alternates: usize,
    pragmas: usize,
    conditional: usize,
    simple_unions: usize,
}

/// Read a schema and its includes and parse them with `version`'s grammar.
fn parse_tree(entry: &Path, version: (u32, u32, u32)) -> Result<Summary, String> {
    let sources =
        read_schema_with_version(entry, Some(version)).map_err(|error| error.to_string())?;
    let mut summary = Summary::default();
    for (path, content) in &sources {
        let schema = parse_schema_with_version(content, version)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        for token in schema.iter() {
            match token {
                QapiSchemaToken::Struct(value) => {
                    summary.conditional += usize::from(value.r#if.is_some())
                }
                QapiSchemaToken::Command(value) => {
                    summary.conditional += usize::from(value.r#if.is_some())
                }
                QapiSchemaToken::Union(value) => {
                    summary.simple_unions += usize::from(value.discriminator.is_none())
                }
                QapiSchemaToken::Event(_) => summary.events += 1,
                QapiSchemaToken::Alternate(_) => summary.alternates += 1,
                QapiSchemaToken::Pragma(_) => {
                    summary.pragmas += 1;
                    continue;
                }
                QapiSchemaToken::Enum(_) => {}
                _ => continue,
            }
            summary.definitions += 1;
        }
    }
    Ok(summary)
}

/// What one pinned release's QMP schema defines.
fn release(tag: &str) -> Summary {
    let version = parse_version_tag(tag).unwrap();
    let schemas = extract_schema_from_tag(&qemu_root(), tag).unwrap();
    let entry = find_schema_entry(schemas.path()).unwrap();
    parse_tree(&entry, version).unwrap()
}

// ── The checked-out baseline ──

#[test]
fn the_checked_out_baseline_parses() {
    let version = baseline();
    let root = qemu_root();
    let qmp = find_schema_entry(&root).unwrap();
    assert!(qmp.ends_with("qapi/qapi-schema.json"));
    let qga = find_qga_schema_entry(&root).unwrap();
    assert!(qga.ends_with("qga/qapi-schema.json"));
    let files = read_schema_with_version(&qmp, Some(version)).unwrap().len();
    assert!(
        files > 10,
        "expected more than 10 QMP schema files, got {files}"
    );
    let count = parse_tree(&qmp, version).unwrap().definitions;
    assert!(
        count > 500,
        "expected more than 500 QMP definitions, got {count}"
    );
    let count = parse_tree(&qga, version).unwrap().definitions;
    assert!(
        count > 50,
        "expected more than 50 QGA definitions, got {count}"
    );
}

// ── The pins ──

#[test]
fn the_tree_has_exactly_the_pinned_releases_at_their_commits() {
    let repo = qemu_root();
    let pins = pins();
    verify_pinned_versions(&repo, &pins).unwrap();
    // Newer tags in the tree are releases the pins do not cover yet.
    let newest = parse_version_tag_full(&pins.last().unwrap().tag).unwrap();
    let pinned_era = |tags: Vec<String>| {
        tags.into_iter()
            .filter(|tag| parse_version_tag_full(tag).unwrap() <= newest)
            .collect::<Vec<_>>()
    };
    // Discovery orders the four-component tags of old releases numerically,
    // as the lockfile does.
    let all: Vec<_> = pins.iter().map(|pin| pin.tag.clone()).collect();
    assert_eq!(pinned_era(discover_qga_version_tags(&repo).unwrap()), all);
    let qmp: Vec<_> = pins
        .iter()
        .filter(|pin| parse_version_tag(&pin.tag).unwrap() >= (1, 0, 0))
        .map(|pin| pin.tag.clone())
        .collect();
    assert_eq!(qmp.first().map(String::as_str), Some("v1.0"));
    assert_eq!(pinned_era(discover_version_tags(&repo).unwrap()), qmp);
}

#[test]
fn every_pinned_release_parses_with_its_grammar() {
    let repo = qemu_root();
    let pins = pins();
    verify_pinned_versions(&repo, &pins).unwrap();
    let (mut qmp, mut qga) = (0, 0);
    let mut failures = Vec::new();
    for pin in &pins {
        let version = parse_version_tag(&pin.tag).unwrap();
        let schemas = extract_schema_from_tag(&repo, &pin.tag).unwrap();
        for (protocol, entry, parsed) in [
            ("QMP", find_schema_entry(schemas.path()), &mut qmp),
            ("QGA", find_qga_schema_entry(schemas.path()), &mut qga),
        ] {
            let Some(entry) = entry else { continue };
            match parse_tree(&entry, version) {
                Ok(_) => *parsed += 1,
                Err(error) => failures.push(format!("{} {protocol}: {error}", pin.tag)),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    // The QMP schema starts with 1.0, the guest agent's with 0.15.
    let qmp_era = pins
        .iter()
        .filter(|pin| parse_version_tag(&pin.tag).unwrap() >= (1, 0, 0))
        .count();
    assert_eq!(qmp, qmp_era);
    assert_eq!(qga, pins.len());
}

#[test]
fn pins_are_checked_in_the_named_tree_only() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("qemu");
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    let tree = git(&repo, &["mktree"]);
    let commit = git(
        &repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit-tree",
            &tree,
            "-m",
            "fixture",
        ],
    );
    git(&repo, &["update-ref", "refs/tags/v11.1.3", &commit]);
    let pin = |tag: &str, commit: &str| PinnedVersion {
        tag: tag.into(),
        commit: commit.into(),
    };
    verify_pinned_versions(&repo, &[pin("v11.1.3", &commit)]).unwrap();
    assert!(matches!(
        verify_pinned_versions(&repo, &[pin("v11.1.3", &"0".repeat(40))]),
        Err(TreeError::MovedRelease { tag, found, .. }) if tag == "v11.1.3" && found == commit
    ));
    assert!(matches!(
        verify_pinned_versions(&repo, &[pin("v11.2.0", &commit)]),
        Err(TreeError::MissingRelease { tag }) if tag == "v11.2.0"
    ));
    // An empty directory in a tree, such as an uninitialized submodule, is
    // not that tree.
    let empty = repo.join("empty");
    std::fs::create_dir(&empty).unwrap();
    assert!(matches!(
        verify_pinned_versions(&empty, &[pin("v11.1.3", &commit)]),
        Err(TreeError::Failed {
            command: "rev-parse",
            ..
        })
    ));
    assert!(matches!(
        verify_pinned_versions(&dir.path().join("missing"), &[pin("v11.1.3", &commit)]),
        Err(TreeError::Repository { .. })
    ));
    // Only release tags are extracted, and a release without schemas is empty.
    let schemas = extract_schema_from_tag(&repo, "v11.1.3").unwrap();
    assert_eq!(std::fs::read_dir(schemas.path()).unwrap().count(), 0);
    for name in ["HEAD", "--output=x", "v11.2.0"] {
        assert!(
            matches!(
                extract_schema_from_tag(&repo, name),
                Err(TreeError::MissingRelease { .. })
            ),
            "{name}"
        );
    }
}

// ── Grammar eras ──

#[test]
fn release_1_1_schema_uses_the_type_keyword() {
    let count = release("v1.1.0").definitions;
    assert!(count > 30, "expected more than 30 definitions, got {count}");
}

#[test]
fn release_2_1_schema_has_events_across_included_files() {
    let version = parse_version_tag("v2.1.0").unwrap();
    let schemas = extract_schema_from_tag(&qemu_root(), "v2.1.0").unwrap();
    let entry = find_schema_entry(schemas.path()).unwrap();
    assert!(
        read_schema_with_version(&entry, Some(version))
            .unwrap()
            .len()
            > 1
    );
    assert!(parse_tree(&entry, version).unwrap().events > 0);
}

#[test]
fn release_2_5_schema_has_alternates() {
    assert!(release("v2.5.0").alternates > 0);
}

#[test]
fn release_4_0_schema_has_pragmas_and_conditions() {
    let summary = release("v4.0.0");
    assert!(
        summary.pragmas > 0 && summary.conditional > 0,
        "{summary:?}"
    );
}

#[test]
fn release_10_2_schema_has_only_flat_unions() {
    let summary = release("v10.2.0");
    assert_eq!(summary.simple_unions, 0);
    assert!(summary.definitions > 500, "{summary:?}");
}
