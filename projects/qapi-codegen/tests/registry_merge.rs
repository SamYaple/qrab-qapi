//! The registry merged from representative pinned releases: boxed commands,
//! version ranges, events, unions, recursion, return types and build
//! conditions, and the errors that name the release that failed.

use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

use qapi_codegen::qapi_ir::{Attribute, Metadata, rustify_name};
use qapi_codegen::unified::{
    RegistryError, UnifiedRegistry, UnifiedStruct, UnifiedType, WarningKind, build_qga_registry,
    build_qmp_registry,
};
use qapi_codegen::versions::{
    PINNED_VERSIONS, extract_schema_from_tag, find_schema_entry, parse_pinned_versions,
    parse_version_tag,
};
use qapi_codegen::{
    IncludeError, QapiSchemaToken, ReadError, parse_schema_with_version, read_schema_with_version,
};

fn qemu_root() -> &'static Path {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    Box::leak(workspace_root.join("vendor/qemu").into_boxed_path())
}

/// The newest pinned release.
fn newest_pin() -> String {
    parse_pinned_versions(PINNED_VERSIONS)
        .unwrap()
        .pop()
        .unwrap()
        .tag
}

/// Representative version tags covering all parser eras, and the newest pin.
fn representative_tags() -> Vec<String> {
    let mut tags: Vec<String> = [
        "v1.0", "v1.1.0", "v1.7.0", "v2.0.0", "v2.1.0", "v2.5.0", "v2.8.0", "v2.12.0", "v3.0.0",
        "v5.0.0", "v7.0.0", "v9.0.0", "v10.2.0", "v11.0.0",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    tags.push(newest_pin());
    tags
}

/// The QMP registry of the representative tags, built once.
fn qmp() -> &'static UnifiedRegistry {
    static REGISTRY: OnceLock<UnifiedRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let tags = representative_tags();
        let tags: Vec<_> = tags.iter().map(String::as_str).collect();
        build_qmp_registry(qemu_root(), &tags).expect("build_qmp_registry failed")
    })
}

/// The QGA registry of a few pinned releases, built once.
fn qga() -> &'static UnifiedRegistry {
    static REGISTRY: OnceLock<UnifiedRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        build_qga_registry(qemu_root(), &["v1.4.0", "v2.4.0", "v2.5.0", "v11.1.3"])
            .expect("build_qga_registry failed")
    })
}

/// Look up a type by its QAPI name: registry keys are Rust names.
fn lookup<'r>(registry: &'r UnifiedRegistry, raw_name: &str) -> Option<&'r UnifiedType> {
    registry.types.get(&rustify_name(raw_name))
}

/// The struct that a QAPI name stands for, which must exist.
fn structure<'r>(registry: &'r UnifiedRegistry, raw_name: &str) -> &'r UnifiedStruct {
    match lookup(registry, raw_name) {
        Some(UnifiedType::Struct(value)) => value,
        Some(UnifiedType::Enum(_)) => panic!("{raw_name} is an enum, not a struct"),
        None => panic!("{raw_name} is missing from the registry"),
    }
}

fn attributes(value: &UnifiedType) -> &[Attribute] {
    match value {
        UnifiedType::Struct(value) => &value.ir.meta.attributes,
        UnifiedType::Enum(value) => &value.ir.meta.attributes,
    }
}

fn has_unit(attributes: &[Attribute], unit: &str) -> bool {
    attributes
        .iter()
        .any(|attr| matches!(attr, Attribute::Unit(name) if name == unit))
}

fn item<'a>(meta: &'a Metadata, key: &str) -> Option<&'a str> {
    meta.attributes.iter().find_map(|attr| match attr {
        Attribute::Item { name, value } if name == key => Some(value.as_str()),
        _ => None,
    })
}

/// A git repository with one tagged commit per release, each with its files.
fn repository(releases: &[(&str, &[(&str, &str)])]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
            ])
            .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
            .args(args)
            .current_dir(dir.path())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    git(&["init", "-q"]);
    for (tag, files) in releases {
        std::fs::write(dir.path().join("VERSION"), tag).unwrap();
        for (path, text) in *files {
            let path = dir.path().join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        git(&["add", "-A"]);
        git(&["commit", "-qm", tag]);
        git(&["tag", tag]);
    }
    dir
}

#[test]
fn registry_errors_name_the_release() {
    let error = build_qmp_registry(Path::new("."), &["v1.x"]).unwrap_err();
    assert!(
        matches!(&error, RegistryError::Tag { tag } if tag == "v1.x"),
        "{error:?}"
    );
    let error = build_qmp_registry(qemu_root(), &["v99.0.0"]).unwrap_err();
    assert!(
        matches!(&error, RegistryError::Extract { tag, .. } if tag == "v99.0.0"),
        "{error:?}"
    );
    let repo = repository(&[
        ("v0.15.0", &[]),
        ("v1.0", &[]),
        (
            "v2.1.0",
            &[("qapi-schema.json", "{ 'include': 'missing.json' }")],
        ),
        (
            "v2.2.0",
            &[("qapi-schema.json", "{ 'enum': 'int', 'data': [] }")],
        ),
    ]);
    // Before 1.0 a release has no QMP schema and is left out.
    let registry = build_qmp_registry(repo.path(), &["v0.15.0"]).unwrap();
    assert!(registry.types.is_empty());
    let error = build_qmp_registry(repo.path(), &["v0.15.0", "v1.0"]).unwrap_err();
    assert!(
        matches!(&error, RegistryError::MissingEntry { tag, protocol: "QMP" } if tag == "v1.0"),
        "{error:?}"
    );
    let error = build_qga_registry(repo.path(), &["v1.0"]).unwrap_err();
    assert!(
        matches!(&error, RegistryError::MissingEntry { tag, protocol: "QGA" } if tag == "v1.0"),
        "{error:?}"
    );
    let error = build_qmp_registry(repo.path(), &["v2.1.0"]).unwrap_err();
    assert!(
        matches!(&error, RegistryError::Read { tag, .. } if tag == "v2.1.0"),
        "{error:?}"
    );
    let error = build_qmp_registry(repo.path(), &["v2.2.0"]).unwrap_err();
    assert!(
        matches!(&error, RegistryError::Schema { tag, .. } if tag == "v2.2.0"),
        "{error:?}"
    );
    assert_eq!(error.to_string(), "Failed to process v2.2.0");
}

#[test]
fn tolerated_schema_quirks_are_returned_as_warnings() {
    let registry = build_qmp_registry(qemu_root(), &["v1.7.0", "v2.4.0"]).unwrap();
    let warning = |tag: &str, kind: &WarningKind| {
        registry
            .warnings
            .iter()
            .any(|warning| warning.tag == tag && &warning.kind == kind)
    };
    let driver = registry
        .warnings
        .iter()
        .find_map(|warning| match &warning.kind {
            WarningKind::NonEnumDiscriminator {
                union,
                discriminator,
                discriminator_type,
            } if warning.tag == "v1.7.0" && union == "BlockdevOptions" => {
                Some((discriminator.as_str(), discriminator_type.as_str()))
            }
            _ => None,
        });
    assert_eq!(driver, Some(("driver", "str")), "{:?}", registry.warnings);
    assert!(
        warning(
            "v2.4.0",
            &WarningKind::MislabeledDocumentation {
                definition: "struct 'RockerSwitch'".into(),
                documented: "Rocker".into(),
            }
        ),
        "{:?}",
        registry.warnings
    );
}

// ── Boxed commands ──

/// When a command uses `'data': 'TypeRef', 'boxed': true` and the command name
/// rustifies to a different name than the data type (e.g., netdev_add -> NetdevAdd
/// vs Netdev -> Netdev), the command struct must survive as its own entry in the
/// registry, not be merged onto the data type and lost.
#[test]
fn test_boxed_command_different_name_not_lost() {
    let boxed_commands = [
        // Transitioned from inline to boxed
        ("netdev_add", "Netdev"),
        ("blockdev-add", "BlockdevOptions"),
        ("object-add", "ObjectOptions"),
        ("set_password", "SetPasswordOptions"),
        ("expire_password", "ExpirePasswordOptions"),
        ("nbd-server-add", "NbdServerAddOptions"),
        ("block_set_io_throttle", "BlockIOThrottle"),
        // Always boxed (never had inline data)
        ("block-export-add", "BlockExportOptions"),
        ("query-stats", "StatsFilter"),
        ("announce-self", "AnnounceParameters"),
        ("set-numa-node", "NumaOptions"),
        ("block-job-change", "BlockJobChangeOptions"),
    ];
    for (cmd_name, data_ref) in boxed_commands {
        let command = structure(qmp(), cmd_name);
        assert!(
            has_unit(&command.ir.meta.attributes, "command"),
            "{cmd_name} is a command"
        );
        assert_eq!(command.versions.until, None, "{cmd_name} is current");
        assert_ne!(rustify_name(cmd_name), rustify_name(data_ref));
        let data = lookup(qmp(), data_ref).unwrap_or_else(|| panic!("{data_ref} is missing"));
        assert!(
            !has_unit(attributes(data), "command"),
            "{data_ref}, the data of {cmd_name}, is no command"
        );
    }
}

/// drive-mirror -> DriveMirror and DriveMirror -> DriveMirror share one entry.
#[test]
fn test_boxed_command_same_name_still_merges() {
    for (cmd_name, data_ref) in [
        ("drive-mirror", "DriveMirror"),
        ("blockdev-snapshot-sync", "BlockdevSnapshotSync"),
        ("migrate-set-parameters", "MigrateSetParameters"),
    ] {
        assert_eq!(rustify_name(cmd_name), rustify_name(data_ref));
        let shared = structure(qmp(), data_ref);
        assert!(
            has_unit(&shared.ir.meta.attributes, "command"),
            "{data_ref} is the command {cmd_name} too"
        );
        assert_eq!(item(&shared.ir.meta, "name"), Some(cmd_name));
    }
}

/// Commands with allow-preconfig keep it with boxed data of another name.
#[test]
fn test_boxed_command_preserves_allow_preconfig() {
    for cmd_name in [
        "netdev_add",
        "block-export-add",
        "set-numa-node",
        "blockdev-add",
        "object-add",
    ] {
        let command = structure(qmp(), cmd_name);
        assert!(
            has_unit(&command.ir.meta.attributes, "allow_preconfig"),
            "{cmd_name} allows preconfig"
        );
    }
}

/// Boxed commands keep their return types, `()` included.
#[test]
fn test_boxed_command_preserves_returns() {
    assert_eq!(
        structure(qmp(), "query-stats").returns(),
        Some("Vec<StatsResult>")
    );
    for cmd_name in ["netdev_add", "announce-self", "block-export-add"] {
        assert_eq!(
            structure(qmp(), cmd_name).returns(),
            Some("()"),
            "{cmd_name}"
        );
    }
}

// ── Version ranges ──

/// Commands that went from inline data to boxed are one lifetime from the
/// first representative release that has them.
#[test]
fn test_transitioned_commands_version_ranges() {
    for (cmd_name, since) in [
        ("netdev_add", (1, 7, 0)),
        ("blockdev-add", (1, 7, 0)),
        ("object-add", (2, 0, 0)),
        ("set_password", (1, 1, 0)),
        ("expire_password", (1, 1, 0)),
        ("nbd-server-add", (1, 7, 0)),
        ("block_set_io_throttle", (1, 1, 0)),
    ] {
        let command = structure(qmp(), cmd_name);
        assert_eq!(command.versions.since, since, "{cmd_name}");
        assert_eq!(command.versions.until, None, "{cmd_name}");
    }
}

/// Commands boxed from their introduction start in the first representative
/// release that has them.
#[test]
fn test_always_boxed_commands_version_ranges() {
    for (cmd_name, since) in [
        ("block-export-add", (7, 0, 0)),
        ("query-stats", (9, 0, 0)),
        ("announce-self", (5, 0, 0)),
        ("set-numa-node", (3, 0, 0)),
        ("block-job-change", (9, 0, 0)),
    ] {
        let command = structure(qmp(), cmd_name);
        assert_eq!(command.versions.since, since, "{cmd_name}");
        assert_eq!(command.versions.until, None, "{cmd_name}");
    }
}

// ── Field evolution ──

/// netdev_add had inline {id, type, props} before 5.0, then boxed Netdev:
/// Netdev's fields are inlined into the command, and the old inline fields
/// end where the boxed data starts.
#[test]
fn test_field_evolution_inline_to_boxed() {
    let command = structure(qmp(), "netdev_add");
    assert!(!command.ir.fields.iter().any(|f| f.name == "data"));
    // Only the flat-union branch and the legacy gen:false argument map flatten.
    let bad_flatten = command.ir.fields.iter().any(|f| {
        has_unit(&f.meta.attributes, "flatten")
            && !has_unit(&f.meta.attributes, "union")
            && !f.r#type.starts_with("serde_json::Map<")
    });
    assert!(!bad_flatten);
    assert!(command.ir.fields.iter().any(|f| f.name == "type"));
    assert!(
        command
            .field_versions
            .values()
            .any(|range| range.until == Some((5, 0, 0)))
    );
}

/// Data types referenced by boxed commands get no command attributes.
#[test]
fn test_data_types_no_command_leakage() {
    for type_name in [
        "Netdev",
        "BlockdevOptions",
        "ObjectOptions",
        "SetPasswordOptions",
        "ExpirePasswordOptions",
        "NbdServerAddOptions",
        "BlockIOThrottle",
        "BlockExportOptions",
        "StatsFilter",
        "AnnounceParameters",
        "NumaOptions",
        "BlockJobChangeOptions",
    ] {
        let value = lookup(qmp(), type_name).unwrap_or_else(|| panic!("{type_name} is missing"));
        let attrs = attributes(value);
        assert!(!has_unit(attrs, "command"), "{type_name}");
        assert!(
            !attrs
                .iter()
                .any(|a| matches!(a, Attribute::Item { name, .. } if name == "returns")),
            "{type_name}"
        );
    }
}

// ── Enums ──

#[test]
fn test_enum_version_tracking() {
    let enums: Vec<_> = qmp()
        .types
        .values()
        .filter_map(|value| match value {
            UnifiedType::Enum(value) => Some(&value.versions),
            UnifiedType::Struct(_) => None,
        })
        .collect();
    let added_later = enums.iter().filter(|range| range.since > (1, 0, 0)).count();
    assert!(added_later > 50, "{added_later} enums added after 1.0");
    assert!(enums.iter().any(|range| range.until.is_some()));
}

#[test]
fn test_deprecated_feature_tracking() {
    let deprecated = qmp()
        .types
        .values()
        .filter(|value| {
            let range = match value {
                UnifiedType::Enum(value) => &value.versions,
                UnifiedType::Struct(value) => &value.versions,
            };
            range.deprecated.is_some()
        })
        .count();
    assert!(deprecated > 0);
}

// ── Completeness ──

/// The definitions of the newest pin, parsed with its own grammar.
fn newest_definitions() -> Vec<QapiSchemaToken<'static>> {
    let tag = newest_pin();
    let version = parse_version_tag(&tag).unwrap();
    let tree = extract_schema_from_tag(qemu_root(), &tag).unwrap();
    let entry = find_schema_entry(tree.path()).unwrap();
    let sources = read_schema_with_version(&entry, Some(version)).unwrap();
    sources
        .into_iter()
        .flat_map(|(_, content)| {
            let content: &'static str = Box::leak(content.into_boxed_str());
            parse_schema_with_version(content, version).unwrap().0
        })
        .collect()
}

/// Every command of the newest pin is a command of the registry, with its
/// exact wire name.
#[test]
fn test_all_latest_commands_in_registry() {
    let mut missing = Vec::new();
    for token in newest_definitions() {
        let QapiSchemaToken::Command(command) = token else {
            continue;
        };
        let found = qmp().types.values().any(|value| {
            let UnifiedType::Struct(value) = value else {
                return false;
            };
            let wire = item(&value.ir.meta, "name").unwrap_or(&value.ir.name);
            has_unit(&value.ir.meta.attributes, "command") && wire == command.name.as_ref()
        });
        if !found {
            missing.push(command.name.to_string());
        }
    }
    assert!(missing.is_empty(), "Commands missing: {missing:?}");
}

#[test]
fn test_registry_command_count() {
    let commands = qmp()
        .types
        .values()
        .filter(|value| {
            matches!(value, UnifiedType::Struct(_)) && has_unit(attributes(value), "command")
        })
        .count();
    assert!(commands > 150, "{commands} commands");
}

// ── Events ──

/// Every event of the newest pin is in the registry, under its Event name.
#[test]
fn test_all_latest_events_in_registry() {
    let mut missing = Vec::new();
    for token in newest_definitions() {
        if let QapiSchemaToken::Event(event) = token
            && !qmp()
                .types
                .contains_key(&rustify_name(&format!("{}Event", event.name)))
        {
            missing.push(event.name.to_string());
        }
    }
    assert!(missing.is_empty(), "Events missing: {missing:?}");
}

#[test]
fn test_events_have_event_attribute() {
    for event_name in [
        "SHUTDOWNEvent",
        "RESETEvent",
        "STOPEvent",
        "RESUMEEvent",
        "DEVICE_DELETEDEvent",
    ] {
        let event = structure(qmp(), event_name);
        assert!(has_unit(&event.ir.meta.attributes, "event"), "{event_name}");
    }
}

#[test]
fn test_removed_events_have_until() {
    let events: Vec<_> = qmp()
        .types
        .values()
        .filter_map(|value| match value {
            UnifiedType::Struct(value) if has_unit(&value.ir.meta.attributes, "event") => {
                Some(value)
            }
            _ => None,
        })
        .collect();
    assert!(events.len() > 40, "{} events", events.len());
    assert!(events.iter().any(|event| event.versions.until.is_some()));
}

// ── Unions, alternates and recursion ──

#[test]
fn test_unions_have_discriminators() {
    let unions: Vec<_> = qmp()
        .types
        .values()
        .filter_map(|value| match value {
            UnifiedType::Struct(value) if value.ir.fields.iter().any(|f| f.name == "u") => {
                Some(value)
            }
            _ => None,
        })
        .collect();
    assert!(unions.len() > 10, "{} unions", unions.len());
    for union in unions {
        assert!(
            union
                .ir
                .fields
                .iter()
                .any(|f| has_unit(&f.meta.attributes, "discriminator")),
            "{} has a discriminator",
            union.ir.name
        );
    }
}

#[test]
fn test_alternates_are_enums() {
    for alt_name in ["BlockdevRef", "StrOrNull"] {
        match lookup(qmp(), alt_name) {
            Some(UnifiedType::Enum(value)) => assert!(value.ir.variants.len() >= 2),
            other => panic!("{alt_name} should be an enum: {other:?}"),
        }
    }
}

/// BlockdevOptions holds its branches by value, a branch holds a BlockdevRef
/// and BlockdevRef, an alternate, holds a BlockdevOptions: the inline fields
/// of that cycle are boxed, arrays are not.
#[test]
fn recursive_types_are_boxed_through_unions_and_alternates() {
    let boxed = |owner: &str, field: &str| {
        let value = structure(qmp(), owner);
        let field = value
            .ir
            .fields
            .iter()
            .find(|f| f.name == field)
            .unwrap_or_else(|| panic!("{owner}.{field} is missing"));
        has_unit(&field.meta.attributes, "boxed")
    };
    assert!(boxed("BlockdevOptions", "u"));
    assert!(boxed("BlockdevOptionsGenericCowFormat", "file"));
    assert!(!boxed("BlockdevOptions", "driver"));
    assert!(!boxed("BlockdevOptionsQuorum", "children"));
}

// ── Return types ──

/// A command's return type is the newest release's, and the earlier ones
/// stay in its history.
#[test]
fn command_returns_follow_the_newest_release() {
    for (registry, command, history) in [
        (
            qmp(),
            "device-list-properties",
            &[
                ((1, 7, 0), "Vec<DevicePropertyInfo>"),
                ((2, 12, 0), "Vec<ObjectPropertyInfo>"),
            ][..],
        ),
        (
            qmp(),
            "query-target",
            // `TargetInfo` is `QemuTargetInfo` on the wire from 3.0, so
            // that returns from then.
            &[((1, 7, 0), "TargetInfo"), ((3, 0, 0), "QemuTargetInfo")],
        ),
        (
            qga(),
            "guest-fstrim",
            &[
                ((1, 4, 0), "()"),
                ((2, 4, 0), "GuestFilesystemTrimResponse"),
            ],
        ),
    ] {
        let value = structure(registry, command);
        let expected: Vec<_> = history.iter().map(|(v, r)| (*v, r.to_string())).collect();
        assert_eq!(value.returns_history, expected, "{command}");
        assert_eq!(
            value.returns(),
            history.last().map(|(_, r)| *r),
            "{command}"
        );
    }
}

/// A command with the Rust name of a type that is not its argument type,
/// such as its response, gets a `Command` type of its own.
#[test]
fn commands_named_like_another_type_get_command_types() {
    for (registry, command, other) in [
        (qga(), "guest-exec", "GuestExec"),
        (qga(), "guest-file-read", "GuestFileRead"),
        (qmp(), "blockdev-snapshot", "BlockdevSnapshot"),
        (qmp(), "block-dirty-bitmap-merge", "BlockDirtyBitmapMerge"),
    ] {
        let request = structure(registry, &format!("{other}Command"));
        assert!(
            has_unit(&request.ir.meta.attributes, "command"),
            "{command}"
        );
        assert_eq!(item(&request.ir.meta, "name"), Some(command));
        let value = structure(registry, other);
        assert!(!has_unit(&value.ir.meta.attributes, "command"), "{other}");
    }
}

// ── Conditions, names and documentation ──

#[test]
fn test_condition_attributes_preserved() {
    let conditional = qmp()
        .types
        .values()
        .filter(|value| {
            attributes(value)
                .iter()
                .any(|a| matches!(a, Attribute::Item { name, .. } if name == "condition"))
        })
        .count();
    assert!(conditional > 10, "{conditional} conditional types");
}

#[test]
fn test_field_version_tracking() {
    let mut added_later = 0;
    let mut removed = 0;
    for value in qmp().types.values() {
        if let UnifiedType::Struct(value) = value {
            for range in value.field_versions.values() {
                added_later += usize::from(range.since > value.versions.since);
                removed += usize::from(range.until.is_some());
            }
        }
    }
    assert!(added_later > 50, "{added_later} fields added later");
    assert!(removed > 10, "{removed} fields removed");
}

#[test]
fn test_name_attributes_preserved() {
    let named = qmp()
        .types
        .values()
        .filter(|value| {
            attributes(value)
                .iter()
                .any(|a| matches!(a, Attribute::Item { name, .. } if name == "name"))
        })
        .count();
    assert!(named > 50, "{named} types with a wire name");
}

/// The registry keeps the newest schema documentation of a type and its
/// fields, without the blank lines around it.
#[test]
fn test_doc_preservation_in_ir() {
    let status = structure(qmp(), "StatusInfo");
    let doc = status.ir.meta.schema_doc.as_deref().unwrap();
    assert!(!doc.is_empty() && !doc.starts_with('\n') && !doc.ends_with('\n'));
    let running = status
        .ir
        .fields
        .iter()
        .find(|f| f.name == "running")
        .unwrap();
    let doc = running.meta.schema_doc.as_deref().unwrap();
    assert!(doc.contains("VCPUs"), "{doc}");
    assert!(!doc.starts_with('\n') && !doc.ends_with('\n'));
}

#[test]
fn registry_errors_name_the_schema_file_in_its_release() {
    let repo = repository(&[
        (
            "v9.0.0",
            &[
                ("qapi/qapi-schema.json", "{ 'include': 'one.json' }"),
                ("qapi/one.json", "{ 'enum': }"),
            ],
        ),
        (
            "v9.1.0",
            &[(
                "qapi/one.json",
                "{ 'command': 'ping' }\n{ 'command': 'pong', 'data': 'Missing' }",
            )],
        ),
        ("v9.2.0", &[("qapi/one.json", "{ 'include': 'two.json' }")]),
        ("v10.0.0", &[("qapi/one.json", "{ 'include': 'one.json' }")]),
    ]);
    let error = build_qmp_registry(repo.path(), &["v9.0.0"]).unwrap_err();
    let RegistryError::Read { source, .. } = &error else {
        panic!("{error:?}")
    };
    assert!(
        matches!(source, ReadError::Parse { path, .. } if path == Path::new("v9.0.0:qapi/one.json")),
        "{source:?}"
    );
    assert_eq!(
        source.to_string(),
        "Failed to parse schema file v9.0.0:qapi/one.json with the grammar of QEMU 9.0.0"
    );
    let error = build_qmp_registry(repo.path(), &["v9.1.0"]).unwrap_err();
    let RegistryError::Schema { source, .. } = &error else {
        panic!("{error:?}")
    };
    assert_eq!(source.file(), Some(Path::new("v9.1.0:qapi/one.json")));
    let error = build_qmp_registry(repo.path(), &["v9.2.0"]).unwrap_err();
    let RegistryError::Read { source, .. } = &error else {
        panic!("{error:?}")
    };
    assert!(
        matches!(source, ReadError::Include { from, kind: IncludeError::Unreadable { .. }, .. }
            if from == Path::new("v9.2.0:qapi/one.json")),
        "{source:?}"
    );
    assert_eq!(
        source.to_string(),
        "v9.2.0:qapi/one.json:1:1: can't read include file 'two.json'"
    );
    let error = build_qmp_registry(repo.path(), &["v10.0.0"]).unwrap_err();
    assert!(
        matches!(&error, RegistryError::Read {
            source: ReadError::Include { from, kind: IncludeError::Loop { path }, .. }, ..
        } if from == Path::new("v10.0.0:qapi/one.json") && path == from),
        "{error:?}"
    );
    // A file that is not UTF-8, which the helper cannot write.
    std::fs::write(repo.path().join("qapi/one.json"), b"\xff").unwrap();
    for args in [
        &["add", "-A"][..],
        &["commit", "-qm", "v10.1.0"],
        &["tag", "v10.1.0"],
    ] {
        let status = Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
            ])
            .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
            .args(args)
            .current_dir(repo.path())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    }
    let error = build_qmp_registry(repo.path(), &["v10.1.0"]).unwrap_err();
    assert!(
        matches!(&error, RegistryError::Read { source: ReadError::Read { path, .. }, .. }
            if path == Path::new("v10.1.0:qapi/one.json")),
        "{error:?}"
    );
}

#[test]
fn a_release_is_checked_in_the_order_of_its_includes() {
    // Until 2.11 the QMP schema is at the top of the tree, so its label
    // has no directory.
    let main = "{ 'struct': 'a b', 'data': {} }\n{ 'include': 'qapi/one.json' }\n";
    let repo = repository(&[
        (
            "v2.9.0",
            &[
                ("qapi-schema.json", main),
                ("qapi/one.json", "{ 'include': 'two.json' }\n"),
                ("qapi/two.json", "{ 'struct': 'c d', 'data': {} }\n"),
            ],
        ),
        (
            "v2.10.0",
            &[("qapi-schema.json", "{ 'include': 'qapi/one.json' }\n")],
        ),
    ]);
    for (tag, file) in [
        ("v2.9.0", "v2.9.0:qapi-schema.json"),
        ("v2.10.0", "v2.10.0:qapi/two.json"),
    ] {
        let error = build_qmp_registry(repo.path(), &[tag]).unwrap_err();
        let RegistryError::Schema { source, .. } = &error else {
            panic!("{error:?}")
        };
        assert_eq!(
            (source.file(), source.line()),
            (Some(Path::new(file)), Some(1)),
            "{source}"
        );
    }
}
