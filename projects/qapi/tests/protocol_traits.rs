//! Trait metadata, enum coverage and field renames of the generated types.
//! None of these tests runs QEMU.

// The tests use items and members of older releases, which the newest
// release no longer has and which are therefore deprecated.
#![allow(deprecated)]

use qapi::{commands::*, events::*, *};

/// Serialize for a peer newer than every release, so that every
/// since-gated field is included. Decoding stays unscoped (newest layout).
fn to_json<T: serde::Serialize + ?Sized>(
    value: &T,
) -> serde_json::Result<String> {
    with_version((99, 0, 0), || serde_json::to_string(value))
}

// ── Command trait metadata ───────────────────────────────────────────────────

#[test]
fn command_trait_metadata() {
    assert_eq!(QueryStatus::NAME, "query-status");
    const {
        assert!(!QueryStatus::ALLOW_OOB);
        assert!(QueryStatus::SUCCESS_RESPONSE);
    }

    assert_eq!(QueryVersion::NAME, "query-version");
    assert_eq!(QueryName::NAME, "query-name");
    assert_eq!(QueryKvm::NAME, "query-kvm");
    assert_eq!(QueryUuid::NAME, "query-uuid");
    assert_eq!(QueryCommands::NAME, "query-commands");
    assert_eq!(QueryChardev::NAME, "query-chardev");
    assert_eq!(QueryBlock::NAME, "query-block");
    assert_eq!(QueryBlockJobs::NAME, "query-block-jobs");
    assert_eq!(QueryMigrate::NAME, "query-migrate");
    assert_eq!(QueryMigrateCapabilities::NAME, "query-migrate-capabilities");
    assert_eq!(QueryMigrateParameters::NAME, "query-migrate-parameters");
    assert_eq!(QueryPci::NAME, "query-pci");
    assert_eq!(QueryTpmModels::NAME, "query-tpm-models");
    assert_eq!(QueryTpmTypes::NAME, "query-tpm-types");
    assert_eq!(QueryQmpSchema::NAME, "query-qmp-schema");
    assert_eq!(QueryDirtyRate::NAME, "query-dirty-rate");
    assert_eq!(QueryYank::NAME, "query-yank");

    const { assert!(Yank::ALLOW_OOB) };
}

// ── Event trait metadata ─────────────────────────────────────────────────────

#[test]
fn event_trait_metadata() {
    assert_eq!(ShutdownEvent::NAME, "SHUTDOWN");
    assert_eq!(StopEvent::NAME, "STOP");
    assert_eq!(ResumeEvent::NAME, "RESUME");
    assert_eq!(ResetEvent::NAME, "RESET");
    assert_eq!(PowerdownEvent::NAME, "POWERDOWN");
    assert_eq!(SuspendEvent::NAME, "SUSPEND");
    assert_eq!(WakeupEvent::NAME, "WAKEUP");
    assert_eq!(WatchdogEvent::NAME, "WATCHDOG");
    assert_eq!(GuestPanickedEvent::NAME, "GUEST_PANICKED");
}

// ── Enum serialization coverage ──────────────────────────────────────────────

#[test]
fn enum_run_state_all_variants() {
    let variants: Vec<(&str, RunState)> = vec![
        ("\"debug\"", RunState::Debug),
        ("\"finish-migrate\"", RunState::FinishMigrate),
        ("\"inmigrate\"", RunState::Inmigrate),
        ("\"internal-error\"", RunState::InternalError),
        ("\"io-error\"", RunState::IoError),
        ("\"paused\"", RunState::Paused),
        ("\"postmigrate\"", RunState::Postmigrate),
        ("\"prelaunch\"", RunState::Prelaunch),
        ("\"restore-vm\"", RunState::RestoreVm),
        ("\"running\"", RunState::Running),
        ("\"save-vm\"", RunState::SaveVm),
        ("\"shutdown\"", RunState::Shutdown),
        ("\"suspended\"", RunState::Suspended),
        ("\"watchdog\"", RunState::Watchdog),
        ("\"guest-panicked\"", RunState::GuestPanicked),
        ("\"colo\"", RunState::Colo),
    ];
    for (expected_json, variant) in variants {
        let json = to_json(&variant).unwrap();
        assert_eq!(
            json, expected_json,
            "RunState serialized wrong for {}",
            expected_json
        );
        let back: RunState = serde_json::from_str(&json).unwrap();
        assert_eq!(to_json(&back).unwrap(), expected_json);
    }
}

#[test]
fn enum_on_off_auto() {
    for (expected, variant) in [
        ("\"auto\"", OnOffAuto::Auto),
        ("\"on\"", OnOffAuto::On),
        ("\"off\"", OnOffAuto::Off),
    ] {
        let json = to_json(&variant).unwrap();
        assert_eq!(json, expected);
        let back: OnOffAuto = serde_json::from_str(&json).unwrap();
        assert_eq!(to_json(&back).unwrap(), expected);
    }
}

#[test]
fn enum_io_operation_type() {
    assert_eq!(to_json(&IoOperationType::Read).unwrap(), "\"read\"");
    assert_eq!(to_json(&IoOperationType::Write).unwrap(), "\"write\"");
}

#[test]
fn enum_error_class() {
    let json = to_json(&QapiErrorClass::GenericError).unwrap();
    assert_eq!(json, "\"GenericError\"");
    let json = to_json(&QapiErrorClass::CommandNotFound).unwrap();
    assert_eq!(json, "\"CommandNotFound\"");
}

#[test]
fn enum_migration_capability_all_variants() {
    let variants = [
        "xbzrle",
        "rdma-pin-all",
        "auto-converge",
        "zero-blocks",
        "events",
        "postcopy-ram",
        "x-colo",
        "release-ram",
        "return-path",
        "pause-before-switchover",
        "multifd",
        "dirty-bitmaps",
        "postcopy-blocktime",
        "late-block-activate",
        "x-ignore-shared",
        "validate-uuid",
        "background-snapshot",
        "zero-copy-send",
        "postcopy-preempt",
        "switchover-ack",
        "dirty-limit",
        "mapped-ram",
    ];
    for name in &variants {
        let version =
            if *name == "zero-blocks" { (10, 2, 0) } else { (11, 1, 3) };
        let json_str = format!("\"{}\"", name);
        let cap = serde_json::from_str::<MigrationCapability>(&json_str)
            .unwrap_or_else(|error| {
                panic!("Missing MigrationCapability {name}: {error}")
            });
        let back = qapi::with_version(version, || serde_json::to_string(&cap))
            .unwrap();
        assert_eq!(
            back, json_str,
            "Roundtrip failed for MigrationCapability {name}"
        );
    }
}

#[test]
fn enum_dump_guest_memory_format() {
    for name in &[
        "elf",
        "kdump-zlib",
        "kdump-lzo",
        "kdump-snappy",
        "kdump-raw-zlib",
        "kdump-raw-lzo",
        "kdump-raw-snappy",
        "win-dmp",
    ] {
        let json_str = format!("\"{}\"", name);
        if let Ok(fmt) =
            serde_json::from_str::<DumpGuestMemoryFormat>(&json_str)
        {
            let back = to_json(&fmt).unwrap();
            assert_eq!(back, json_str);
        }
    }
}

// ── Alternate types ──────────────────────────────────────────────────────────

#[test]
fn alternate_str_or_null() {
    let val = StrOrNull::S("hello".to_string());
    let json = to_json(&val).unwrap();
    assert_eq!(json, "\"hello\"");
    let back: StrOrNull = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, StrOrNull::S(s) if s == "hello"));

    let val = StrOrNull::N(());
    let json = to_json(&val).unwrap();
    assert_eq!(json, "null");
    let back: StrOrNull = serde_json::from_str("null").unwrap();
    assert!(matches!(back, StrOrNull::N(())));
}

// ── Struct field renames ─────────────────────────────────────────────────────

#[test]
fn struct_field_renames() {
    let val = HumanReadableText::new("test".to_string());
    let json = to_json(&val).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(
        parsed.get("human-readable-text").is_some(),
        "Expected 'human-readable-text' key"
    );

    let val = UuidInfo::new("abc".to_string());
    let json = to_json(&val).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.get("UUID").is_some(), "Expected 'UUID' key");
}
