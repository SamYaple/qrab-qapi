//! Sanity tests for the generated `qapi::metadata` module.
//!
//! Asserts the shape contract that tools rendering the schema rely on: every
//! command/struct/enum is present, flat unions are flagged, alternates and
//! branch enums are distinguishable, and the type-ref pool resolves
//! correctly.

// The tests use items and members of older releases, which the newest
// release no longer has and which are therefore deprecated.
#![allow(deprecated)]

use qapi::metadata::{COMMANDS, ENUMS, EnumKind, STRUCTS};

#[test]
fn tables_are_sorted_and_find_every_entry_by_name() {
    let names: Vec<_> = STRUCTS.keys().collect();
    assert!(
        names.is_sorted_by(|a, b| a < b),
        "STRUCTS names are not strictly sorted"
    );
    for (name, info) in STRUCTS.iter() {
        assert_eq!(info.name(), name);
        assert!(std::ptr::eq(STRUCTS.get(name).unwrap(), info), "{name}");
    }
    let names: Vec<_> = ENUMS.keys().collect();
    assert!(
        names.is_sorted_by(|a, b| a < b),
        "ENUMS names are not strictly sorted"
    );
    for (name, info) in ENUMS.iter() {
        assert_eq!(info.name(), name);
        assert!(std::ptr::eq(ENUMS.get(name).unwrap(), info), "{name}");
    }
    assert!(STRUCTS.get("NoSuchType").is_none());
    assert!(!ENUMS.contains_key(""));
    assert_eq!(STRUCTS.len(), STRUCTS.values().len());
}

#[test]
fn historical_discriminator_fields_do_not_hide_current_unions() {
    for (name, wire_name) in [
        ("SocketAddress", "type"),
        ("NumaOptions", "type"),
        ("NetLegacyOptions", "type"),
        ("BlockdevOptions", "driver"),
        ("ChardevBackend", "type"),
        ("CpuInfoFast", "target"),
        ("ImageInfoSpecific", "type"),
        ("InputEvent", "type"),
    ] {
        let info = STRUCTS.get(name).unwrap();
        assert!(info.is_union(), "{name}");
        assert!(info.discriminator_field().is_some(), "{name}");
        let branch = ENUMS.get(info.branch_enum().unwrap()).unwrap();
        assert_eq!(branch.kind(), EnumKind::UnionBranch, "{name}");
        let discriminator = info
            .fields()
            .iter()
            .find(|field| Some(field.name()) == info.discriminator_field())
            .unwrap();
        assert_eq!(discriminator.wire_name(), wire_name);
        // The newest representation lasts as long as its union.
        assert_eq!(discriminator.until(), info.until(), "{name}");
    }
}

#[test]
fn metadata_references_resolve_including_enum_payloads() {
    use qapi::metadata::{TYPE_REFS, TypeKind, TypeRef};
    // A reference into the pool, as an index.
    let index = |ty: &TypeRef| {
        TYPE_REFS
            .iter()
            .position(|entry| std::ptr::eq(entry, ty))
            .unwrap_or_else(|| panic!("{ty:?} is not an entry of TYPE_REFS"))
    };
    for (at, ty) in TYPE_REFS.iter().enumerate() {
        match ty.kind() {
            TypeKind::Struct => {
                assert!(STRUCTS.contains_key(ty.name()), "{}", ty.name())
            }
            TypeKind::Enum => {
                assert!(ENUMS.contains_key(ty.name()), "{}", ty.name())
            }
            TypeKind::List => assert!(index(ty.element().unwrap()) < at),
            TypeKind::Primitive => {}
            other => panic!("{other:?}"),
        }
    }
    for metadata in STRUCTS.values() {
        for field in metadata.fields() {
            index(field.type_ref());
        }
    }
    for metadata in ENUMS.values() {
        for variant in metadata.variants() {
            if metadata.kind() == EnumKind::Unit {
                assert!(variant.payload_type().is_none());
            }
            if let Some(ty) = variant.payload_type() {
                index(ty);
                if let Some(name) = variant.payload_struct() {
                    assert_eq!(ty.kind(), TypeKind::Struct);
                    assert_eq!(ty.name(), name);
                }
            } else {
                assert!(variant.payload_struct().is_none());
            }
        }
    }
    for command in COMMANDS {
        command.returns().map(index);
    }
}

#[test]
fn commands_table_has_full_qapi_surface() {
    // QEMU 10.2 carries roughly 280 QMP commands. A floor of 250 prevents
    // accidental schema-export regressions without needing to bump the
    // expected count on every QEMU release.
    assert!(
        COMMANDS.len() >= 250,
        "expected >=250 commands, got {}",
        COMMANDS.len()
    );
}

/// Every command of `COMMANDS` is a struct of the protocol's `commands`
/// module and every event one of its `events`, as the generated sources
/// `files` (`mod.rs`, `commands.rs`, `events.rs`) declare them.
fn assert_in_their_modules(
    protocol: &str,
    commands: &[&str],
    files: [&str; 3],
) {
    let [types, command_structs, events] = files;
    for name in commands {
        assert!(
            command_structs.contains(&format!("\npub struct {name} {{")),
            "{protocol} command {name} is not in commands"
        );
    }
    let count = |source: &str, attribute: &str| {
        source.lines().filter(|line| line.trim() == attribute).count()
    };
    assert_eq!(
        count(command_structs, "#[qapi(command)]"),
        commands.len(),
        "{protocol}"
    );
    for (module, source) in [("mod.rs", types), ("events.rs", events)] {
        assert_eq!(count(source, "#[qapi(command)]"), 0, "{protocol} {module}");
    }
    for (module, source) in
        [("mod.rs", types), ("commands.rs", command_structs)]
    {
        assert_eq!(count(source, "#[qapi(event)]"), 0, "{protocol} {module}");
    }
}

/// `qapi::commands` holds every command, also those whose structs data
/// types name, such as the actions of `transaction`; `qapi::events` and the
/// QGA modules likewise.
#[test]
fn every_command_and_event_is_in_its_module() {
    use qapi::{
        QapiCommand,
        commands::{
            BlockDirtyBitmapAdd, BlockdevBackup, BlockdevSnapshotSync,
            DriveBackup,
        },
    };
    assert_eq!(BlockDirtyBitmapAdd::NAME, "block-dirty-bitmap-add");
    assert_eq!(BlockdevBackup::NAME, "blockdev-backup");
    assert_eq!(BlockdevSnapshotSync::NAME, "blockdev-snapshot-sync");
    assert_eq!(DriveBackup::NAME, "drive-backup");
    // The transaction action's data is that same struct.
    let _: fn(qapi::DriveBackupWrapper) -> DriveBackup = |wrapper| wrapper.data;

    let names: Vec<_> = COMMANDS.iter().map(|command| command.name()).collect();
    assert_in_their_modules(
        "QMP",
        &names,
        [
            include_str!("../src/generated/mod.rs"),
            include_str!("../src/generated/commands.rs"),
            include_str!("../src/generated/events.rs"),
        ],
    );
    let names: Vec<_> = qapi::qga::metadata::COMMANDS
        .iter()
        .map(|command| command.name())
        .collect();
    assert_in_their_modules(
        "QGA",
        &names,
        [
            include_str!("../src/qga/mod.rs"),
            include_str!("../src/qga/commands.rs"),
            include_str!("../src/qga/events.rs"),
        ],
    );
}

#[test]
fn structs_and_enums_populated() {
    assert!(
        STRUCTS.len() >= 800,
        "expected >=800 structs, got {}",
        STRUCTS.len()
    );
    assert!(ENUMS.len() >= 100, "expected >=100 enums, got {}", ENUMS.len());
}

#[test]
fn blockdev_add_is_a_flat_union() {
    let s = STRUCTS.get("BlockdevAdd").expect("BlockdevAdd missing");
    assert!(s.is_union(), "BlockdevAdd should be detected as a union");
    assert!(
        s.discriminator_field().is_some(),
        "BlockdevAdd should expose its discriminator field"
    );
    assert!(
        s.branch_enum().is_some(),
        "BlockdevAdd should reference a branch enum"
    );
}

#[test]
fn empty_args_command_present() {
    // `query-status` returns StatusInfo and takes no arguments.
    let cmd = COMMANDS
        .iter()
        .find(|c| c.wire_name() == "query-status")
        .expect("query-status command missing");
    assert_eq!(
        cmd.args().fields().len(),
        0,
        "query-status should have zero fields, got {}",
        cmd.args().fields().len()
    );
    assert!(cmd.returns().is_some(), "query-status should have a return type");
}

#[test]
fn alternate_enum_classified() {
    let e =
        ENUMS.get("StrOrNull").expect("StrOrNull alternate missing from ENUMS");
    assert_eq!(
        e.kind(),
        EnumKind::Alternate,
        "StrOrNull should be classified as Alternate"
    );
}

#[test]
fn flat_union_branch_enum_classified() {
    let e = ENUMS
        .get("BlockdevOptionsBranch")
        .expect("BlockdevOptionsBranch missing from ENUMS");
    assert_eq!(
        e.kind(),
        EnumKind::UnionBranch,
        "BlockdevOptionsBranch should be classified as UnionBranch"
    );
    assert!(
        e.variants().iter().any(|v| v.payload_struct().is_some()),
        "at least one BlockdevOptionsBranch variant should carry a payload struct"
    );
}

#[test]
fn unit_enum_classified() {
    // Pick a known unit-only enum.
    let e = ENUMS
        .get("RunState")
        .or_else(|| ENUMS.get("BlockdevDriver"))
        .expect("expected at least one unit enum");
    assert_eq!(e.kind(), EnumKind::Unit);
}

#[test]
fn changed_branch_payloads_expose_distinct_lifetimes_and_original_wire_tags() {
    let branches = ENUMS.get("SocketAddressLegacyBranch").unwrap();
    let current = branches
        .variants()
        .iter()
        .find(|variant| variant.name() == "Inet")
        .unwrap();
    let historical = branches
        .variants()
        .iter()
        .find(|variant| variant.name() == "InetLegacy")
        .unwrap();
    assert_eq!(current.wire_name(), "inet");
    assert_eq!(historical.wire_name(), "inet");
    assert_eq!(current.since(), (6, 2, 0));
    assert_eq!(historical.until(), Some((6, 2, 0)));
    assert_eq!(current.payload_struct(), Some("InetSocketAddressWrapper"));
    assert_eq!(historical.payload_struct(), Some("InetSocketAddress"));
}

#[test]
fn required_wire_fields_stay_required_in_metadata_after_version_merging() {
    let info = STRUCTS.get("VersionInfo").unwrap();
    let qemu =
        info.fields().iter().find(|field| field.name() == "qemu").unwrap();
    assert!(
        !qemu.is_optional(),
        "version merging must not turn wire-required into schema-optional"
    );
    let value: qapi::VersionInfo = serde_json::from_value(serde_json::json!({
        "qemu":{"major":11,"minor":1,"micro":3},"package":""
    }))
    .unwrap();
    assert!(
        value.qemu.is_some(),
        "Rust uses Option to accommodate the older inline representation"
    );
}

#[test]
fn flags_and_features_describe_every_release() {
    use qapi::metadata::{Releases, TYPE_REFS};
    let command = |wire_name| {
        COMMANDS
            .iter()
            .find(|command| command.wire_name() == wire_name)
            .unwrap()
    };
    let runs = |runs: &[Releases]| -> Vec<_> {
        runs.iter().map(|run| (run.since(), run.until())).collect()
    };
    // QEMU 3.0 added migrate-pause, out of band from the start, and 7.1
    // allowed block-job-complete before the machine is configured. 10.1
    // deprecates it.
    let pause = command("migrate-pause");
    assert_eq!(runs(pause.allow_oob()), [((3, 0, 0), None)]);
    let complete = command("block-job-complete");
    assert!(
        !complete.allows_preconfig((7, 0, 0))
            && complete.allows_preconfig((7, 1, 0))
    );
    assert!(
        !complete.is_deprecated((10, 0, 0))
            && complete.is_deprecated((10, 1, 0))
    );
    assert!(
        complete.sends_success_response((1, 3, 0))
            && !complete.is_unstable((11, 1, 3))
    );
    // migrate's detach was deprecated in 10.1 and removed in 11.0.
    let detach = STRUCTS
        .get("Migrate")
        .unwrap()
        .fields()
        .iter()
        .find(|field| field.wire_name() == "detach")
        .unwrap();
    assert_eq!(runs(detach.deprecated()), [((10, 1, 0), Some((11, 0, 0)))]);
    assert!(
        !detach.is_deprecated((10, 0, 0)) && detach.is_deprecated((10, 2, 0))
    );
    // The guest agent replied to guest-shutdown until 1.1, and its replies
    // name their builtins as the newest release does.
    let qga = |wire_name| {
        qapi::qga::metadata::COMMANDS
            .iter()
            .find(|command| command.wire_name() == wire_name)
            .unwrap()
    };
    let shutdown = qga("guest-shutdown");
    assert!(shutdown.sends_success_response((1, 0, 0)));
    assert!(!shutdown.sends_success_response((1, 1, 0)));
    let time = qga("guest-get-time").returns().unwrap();
    assert_eq!(time.name(), "int");
    // It resolves against the guest agent's pool, not that of QMP.
    assert!(
        qapi::qga::metadata::TYPE_REFS
            .iter()
            .any(|entry| std::ptr::eq(entry, time))
    );
    assert!(!TYPE_REFS.iter().any(|entry| std::ptr::eq(entry, time)));
    let status = command("query-status").returns().unwrap();
    assert_eq!(status.name(), "StatusInfo");
    // One set of types describes both protocols.
    let _: &qapi::metadata::CommandMeta = qga("guest-get-time");
}
