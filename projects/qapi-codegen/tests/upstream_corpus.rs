//! The parser against QEMU's own: its schema test fixtures, valid and invalid,
//! documentation attached to definitions as QEMU's parser attaches it, in the
//! fixtures and in the production schemas, and the diagnostics of invalid
//! input.
use qapi_codegen::versions::{PINNED_VERSIONS, parse_pinned_versions, parse_version_tag};
use qapi_codegen::{
    DocumentationError, ParseErrorKind, ReadError, parse_schema_with_version,
    read_schema_with_version,
};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The grammar of the newest pin, which vendor/qemu must be checked out at:
/// the QEMU parser and fixtures used here come from its working tree.
fn baseline() -> (u32, u32, u32) {
    let pins = parse_pinned_versions(PINNED_VERSIONS).unwrap();
    let pin = pins.last().unwrap();
    let head = Command::new("git")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .arg("-C")
        .arg(qemu_root())
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&head.stdout).trim(),
        pin.commit,
        "vendor/qemu is not checked out at {}",
        pin.tag
    );
    parse_version_tag(&pin.tag).unwrap()
}

fn qemu_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/qemu")
}

/// The grammar of the current release, for inputs that are not QEMU's files.
const CURRENT: (u32, u32, u32) = (11, 1, 3);

#[test]
fn freeform_documentation_cannot_replace_definition_documentation() {
    let section = "##\n# = Section\n#\n# Body.\n##\n";
    for ending in ["\n", "\r\n", "\r"] {
        for definition in [
            "{'struct':'Example','data':{}}",
            "{'enum':'Example','data':['one']}",
            "{'alternate':'Example','data':{'one':'str'}}",
            "{'union':'Example','data':{'one':'str'}}",
            "{'command':'example'}",
            "{'event':'EXAMPLE'}",
        ] {
            let input = format!("{section}{definition}").replace('\n', ending);
            assert!(
                parse_schema_with_version(&input, CURRENT).is_err(),
                "{input:?}"
            );
        }
        let input =
            format!("{section}##\n# @Example:\n# Summary.\n##\n{{'struct':'Example','data':{{}}}}")
                .replace('\n', ending);
        parse_schema_with_version(&input, CURRENT).unwrap();
        let input = format!("{section}{{'pragma':{{}}}}\n{{'struct':'Example','data':{{}}}}")
            .replace('\n', ending);
        parse_schema_with_version(&input, CURRENT).unwrap();
    }
    for version in [(2, 7, 0), (2, 8, 0)] {
        parse_schema_with_version(
            &format!("{section}{{'struct':'Example','data':{{}}}}"),
            version,
        )
        .unwrap();
    }
}

#[test]
fn pre_formal_docs_remain_comments_and_keep_recognized_sections() {
    let input = "# ordinary\n##\n# = Section\n##\n## @Example\n# Description.\n##\n{'struct':'Example','data':{### inline comment\n'value':'str'}}\n### trailing comment";
    let schema = parse_schema_with_version(input, (2, 8, 0)).unwrap();
    assert!(
        schema
            .iter()
            .any(|token| matches!(token, qapi_codegen::QapiSchemaToken::Doc(_)))
    );
    assert!(schema.iter().any(
        |token| matches!(token, qapi_codegen::QapiSchemaToken::Struct(value) if value.doc.is_some())
    ));
    assert!(parse_schema_with_version(input, (2, 9, 0)).is_err());
}

#[test]
fn malformed_schema_diagnostics_locate_later_definitions() {
    let prefix = "# Unicode comment: ☃\n{'struct':'Good','data':{}}\n\n    ";
    for ending in ["\n", "\r\n", "\r"] {
        let prefix = prefix.replace('\n', ending);
        // Where QEMU's parser stops: at the unexpected token, or at the
        // expression that it cannot tell the meaning of.
        for (invalid, column) in [
            ("{'struct':'Bad','data':{'value':}}", 37),
            ("{'struct':'Bad','data':{'value':'str',}}", 43),
            ("{'pragma':{'doc-required':'str'}}", 5),
            ("unexpected", 5),
        ] {
            let error = parse_schema_with_version(&format!("{prefix}{invalid}"), CURRENT)
                .unwrap_err()
                .to_string();
            assert!(
                error.contains(&format!("line 4, column {column}")),
                "{error}"
            );
            assert!(error.contains(&invalid[column - 5..]), "{error}");
        }
    }
    let error = parse_schema_with_version(
        &format!("{prefix}{{'struct':'Bad','data':{{'value':'str','value':'int'}}}}"),
        CURRENT,
    )
    .unwrap_err()
    .to_string();
    // At the value of the duplicate key, as QEMU reports it.
    assert!(error.contains("line 4, column 51"), "{error}");
    assert!(error.contains("duplicate key"), "{error}");
}

#[test]
fn duplicate_doc_sections_report_the_original_source_location() {
    let input = "##\n# @example:\n# Since: 1.0\n#\n# Since: 2.0\n##\n{'command':'example'}\n";
    for ending in ["\n", "\r\n", "\r"] {
        let error = parse_schema_with_version(&input.replace('\n', ending), CURRENT).unwrap_err();
        assert_eq!((error.line, error.column), (5, 1), "{error}");
        assert_eq!(
            error.kind,
            ParseErrorKind::Documentation(Box::new(DocumentationError::DuplicateSection {
                section: "Since".into()
            }))
        );
    }
}

fn assert_documentation_coverage(entry: &str) {
    use qapi_codegen::QapiSchemaToken;
    use std::collections::BTreeSet;
    let version = baseline();
    let repo = qemu_root();
    let schema = repo.join(entry);
    let output = Command::new("python3")
        .arg("-c")
        .arg(
            r#"
import sys
sys.path.insert(0, sys.argv[1] + '/scripts')
from qapi.parser import QAPISchemaParser
for expr in QAPISchemaParser(sys.argv[2]).exprs:
    if expr.doc is not None:
        print(expr.doc.symbol)
        for section in expr.doc.all_sections:
            if section.kind.name in ('MEMBER', 'FEATURE'):
                print(expr.doc.symbol + '\t' + section.kind.name + '\t' + section.name + '\t' + ' '.join(section.text.split()))
"#,
        )
        .arg(&repo)
        .arg(&schema)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected: BTreeSet<_> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    let mut actual = BTreeSet::new();
    for (_, content) in read_schema_with_version(&schema, Some(version)).unwrap() {
        for token in parse_schema_with_version(&content, version).unwrap() {
            let doc = match token {
                QapiSchemaToken::Enum(value) => value.doc,
                QapiSchemaToken::Struct(value) => value.doc,
                QapiSchemaToken::Union(value) => value.doc,
                QapiSchemaToken::Alternate(value) => value.doc,
                QapiSchemaToken::Command(value) => value.doc,
                QapiSchemaToken::Event(value) => value.doc,
                _ => continue,
            };
            if let Some(doc) = doc {
                actual.insert(doc.name.to_string());
                for (kind, fields) in [("MEMBER", &doc.fields), ("FEATURE", &doc.features)] {
                    for (name, lines) in fields {
                        let text = lines
                            .join(" ")
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ");
                        actual.insert(format!("{}\t{kind}\t{name}\t{text}", doc.name));
                    }
                }
                if doc.name == "cmd" {
                    assert_eq!(doc.returns, ["@Object"]);
                    assert_eq!(doc.errors, ["some"]);
                    assert!(doc.body.iter().any(|line| line.starts_with("Returns:")));
                    assert!(
                        doc.body
                            .iter()
                            .any(|line| line.contains("admonition:: Notes"))
                    );
                }
            }
        }
    }
    assert_eq!(actual, expected);
}

#[test]
fn upstream_documentation_stays_attached_to_definitions() {
    assert_documentation_coverage("tests/qapi-schema/doc-good.json");
}

#[test]
fn production_documentation_matches_qemu() {
    for entry in ["qapi/qapi-schema.json", "qga/qapi-schema.json"] {
        assert_documentation_coverage(entry);
    }
}

#[test]
fn parse_errors_report_a_location_and_a_bounded_excerpt() {
    let input = format!(
        "# ☃\n{{ 'struct': 'Example', 'data': {{\n  'name': 'str',\n  'name': 'int' }} }}\n{}",
        "☃".repeat(1000)
    );
    let message = parse_schema_with_version(&input, CURRENT)
        .unwrap_err()
        .to_string();
    assert!(message.contains("line 4, column 11"), "{message}");
    assert!(
        message.chars().count() < 240,
        "error excerpt is not bounded"
    );
}

#[test]
fn include_errors_name_the_directive_qemu_names() {
    let version = baseline();
    let root = qemu_root().join("tests/qapi-schema");
    for (fixture, file, line, looped) in [
        ("include-cycle.json", "include-cycle-c.json", 1, true),
        (
            "include-self-cycle.json",
            "include-self-cycle.json",
            1,
            true,
        ),
        ("include-no-file.json", "include-no-file.json", 1, false),
    ] {
        match read_schema_with_version(&root.join(fixture), Some(version)) {
            Err(ReadError::Include {
                from,
                line: at,
                column,
                kind,
            }) => {
                assert_eq!(from.file_name().unwrap(), file, "{fixture}");
                assert_eq!((at, column), (line, 1), "{fixture}");
                assert_eq!(
                    matches!(kind, qapi_codegen::IncludeError::Loop { .. }),
                    looped,
                    "{fixture}: {kind}"
                );
            }
            other => panic!("{fixture}: {other:?}"),
        }
    }
    // A name with an escape, and a target that is no file, are located at
    // their directive too.
    let dir = std::env::temp_dir().join(format!("qapi-include-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("adir")).unwrap();
    let main = dir.join("main.json");
    std::fs::write(
        &main,
        "{ 'enum': 'E', 'data': [] }\n\n  { 'include': 'mis\\\\sing.json' }\n{ 'include': 'adir' }\n",
    )
    .unwrap();
    let error = read_schema_with_version(&main, Some(version)).unwrap_err();
    assert!(
        matches!(&error, ReadError::Include { line: 3, column: 3, kind: qapi_codegen::IncludeError::Unreadable { name, .. }, .. } if name == "mis\\sing.json"),
        "{error:?}"
    );
    std::fs::write(
        &main,
        "{ 'enum': 'E', 'data': [] }\n{ 'include': 'adir' }\n",
    )
    .unwrap();
    let error = read_schema_with_version(&main, Some(version)).unwrap_err();
    assert!(
        matches!(
            &error,
            ReadError::Include {
                line: 2,
                column: 1,
                kind: qapi_codegen::IncludeError::Unreadable { .. },
                ..
            }
        ),
        "{error:?}"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn upstream_positive_schema_fixtures_parse() {
    let version = baseline();
    let root = qemu_root().join("tests/qapi-schema");
    let mut fixtures = Vec::new();
    for entry in std::fs::read_dir(&root).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|extension| extension == "err")
            && std::fs::read_to_string(&path).unwrap().trim().is_empty()
            && path.with_extension("json").exists()
        {
            fixtures.push(path.with_extension("json"));
        }
    }
    fixtures.sort();
    assert!(
        !fixtures.is_empty(),
        "QEMU positive fixture corpus is missing"
    );
    let mut failures = Vec::new();
    for path in fixtures {
        if let Err(error) = read_schema_with_version(&path, Some(version)) {
            failures.push(format!("{}: {error:#}", path.display()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn invalid_definition_documentation_cannot_fall_back_to_plain_comments() {
    let version = baseline();
    let root = qemu_root().join("tests/qapi-schema");
    for fixture in [
        "doc-before-include.json",
        "doc-before-pragma.json",
        "doc-missing-expr.json",
        "doc-missing-colon.json",
        "doc-invalid-end.json",
        "doc-empty-symbol.json",
    ] {
        // QEMU's expected error, `<file>:<line>[:<column>]: <message>`.
        let expected = std::fs::read_to_string(root.join(fixture).with_extension("err")).unwrap();
        let line: usize = expected.split(':').nth(1).unwrap().parse().unwrap();
        // A documentation fault, at the line QEMU names.
        match read_schema_with_version(&root.join(fixture), Some(version)) {
            Err(ReadError::Parse { source, .. })
                if matches!(source.kind, ParseErrorKind::Documentation(_)) =>
            {
                assert_eq!(source.line, line, "{fixture}: {source} (QEMU: {expected})");
            }
            other => panic!("{fixture}: {other:?} (QEMU: {expected})"),
        }
    }
}
