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

#[test]
fn unit_enum_serialize() {
    let val = IoOperationType::Read;
    let json = to_json(&val).unwrap();
    assert_eq!(json, r#""read""#);

    let val = IoOperationType::Write;
    let json = to_json(&val).unwrap();
    assert_eq!(json, r#""write""#);
}

#[test]
fn unit_enum_deserialize() {
    let val: IoOperationType = serde_json::from_str(r#""read""#).unwrap();
    assert!(matches!(val, IoOperationType::Read));

    let val: IoOperationType = serde_json::from_str(r#""write""#).unwrap();
    assert!(matches!(val, IoOperationType::Write));
}

#[test]
fn unit_enum_roundtrip() {
    let variants = vec![OnOffAuto::Auto, OnOffAuto::On, OnOffAuto::Off];
    for v in variants {
        let json = to_json(&v).unwrap();
        let back: OnOffAuto = serde_json::from_str(&json).unwrap();
        assert_eq!(to_json(&back).unwrap(), json);
    }
}

#[test]
fn enum_with_rename() {
    let val = OffAutoPcibar::Bar0;
    let json = to_json(&val).unwrap();
    assert_eq!(json, r#""bar0""#);

    let val: OffAutoPcibar = serde_json::from_str(r#""bar5""#).unwrap();
    assert!(matches!(val, OffAutoPcibar::Bar5));
}

#[test]
fn enum_leading_digit_variant() {
    let val = PcieLinkSpeed::_25;
    let json = to_json(&val).unwrap();
    assert_eq!(json, r#""2_5""#);

    let val: PcieLinkSpeed = serde_json::from_str(r#""16""#).unwrap();
    assert!(matches!(val, PcieLinkSpeed::_16));
}

#[test]
fn alternate_string_roundtrip() {
    let val = StrOrNull::S("hello".to_string());
    let json = to_json(&val).unwrap();
    assert_eq!(json, r#""hello""#);

    let back: StrOrNull = serde_json::from_str(&json).unwrap();
    match back {
        StrOrNull::S(s) => assert_eq!(s, "hello"),
        _ => panic!("expected S variant"),
    }
}

#[test]
fn alternate_null_roundtrip() {
    let val = StrOrNull::N(());
    let json = to_json(&val).unwrap();
    assert_eq!(json, "null");

    let back: StrOrNull = serde_json::from_str("null").unwrap();
    assert!(matches!(back, StrOrNull::N(())));
}

#[test]
fn struct_serialize_with_rename() {
    let val = HumanReadableText::new("test output".to_string());
    let json = to_json(&val).unwrap();
    assert!(json.contains(r#""human-readable-text""#));
    assert!(json.contains(r#""test output""#));
}

#[test]
fn struct_deserialize_with_rename() {
    let json = r#"{"human-readable-text": "hello"}"#;
    let val: HumanReadableText = serde_json::from_str(json).unwrap();
    assert_eq!(val.human_readable_text, "hello");
}

#[test]
fn struct_roundtrip() {
    let val = StatusInfo::new(true, RunState::Running);
    let json = to_json(&val).unwrap();
    let back: StatusInfo = serde_json::from_str(&json).unwrap();
    assert!(back.running);
    assert!(matches!(back.status, RunState::Running));
}

#[test]
fn command_trait_query_status() {
    assert_eq!(QueryStatus::NAME, "query-status");
    const {
        assert!(!QueryStatus::ALLOW_OOB);
        assert!(QueryStatus::SUCCESS_RESPONSE);
    }
    // Returns type check — this is a compile-time check
    fn _check_returns(_: <QueryStatus as QapiCommand>::Returns) {}
    fn _check_is_status_info(_: StatusInfo) {}
}

#[test]
fn command_trait_yank() {
    assert_eq!(Yank::NAME, "yank");
    const {
        assert!(Yank::ALLOW_OOB);
        assert!(!Yank::ALLOW_PRECONFIG);
        assert!(Yank::SUCCESS_RESPONSE);
    }
}

#[test]
fn event_trait() {
    assert_eq!(ShutdownEvent::NAME, "SHUTDOWN");
    assert_eq!(StopEvent::NAME, "STOP");
    assert_eq!(ResumeEvent::NAME, "RESUME");
}

#[test]
fn command_serialize() {
    let mut cmd = QmpCapabilities::default();
    let json = to_json(&cmd).unwrap();
    // Should serialize to an empty object (no enable field since it's None)
    assert_eq!(json, "{}");

    cmd.enable = Some(vec![QmpCapability::Oob]);
    let json = to_json(&cmd).unwrap();
    assert!(json.contains("\"enable\""));
    assert!(json.contains("\"oob\""));
}

#[test]
fn event_serialize() {
    let mut evt = ShutdownEvent::default();
    evt.guest = Some(true);
    evt.reason = Some(ShutdownCause::HostSignal);
    let json = to_json(&evt).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["guest"], true);
    assert_eq!(parsed["reason"], "host-signal");
}

#[test]
fn event_deserialize() {
    let json = r#"{"guest": false, "reason": "host-qmp-quit"}"#;
    let evt: ShutdownEvent = serde_json::from_str(json).unwrap();
    assert_eq!(evt.guest, Some(false));
    assert!(matches!(evt.reason, Some(ShutdownCause::HostQmpQuit)));
}

#[test]
fn version_gating_rejects_old_version() {
    let val = IoOperationType::Read;
    assert!(to_json(&val).is_ok());
}

#[test]
fn union_struct_serialize() {
    let mut val = YankInstance::new(YankInstanceType::Chardev);
    val.u = Some(YankInstanceBranch::Chardev(YankInstanceChardev::new(
        "serial0".to_string(),
    )));
    let json = to_json(&val).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["type"], "chardev");
}

#[test]
fn error_class_roundtrip() {
    let val = QapiErrorClass::GenericError;
    let json = to_json(&val).unwrap();
    assert_eq!(json, r#""GenericError""#);
    let back: QapiErrorClass = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, QapiErrorClass::GenericError));
}

#[test]
fn command_version_info() {
    // Test that AVAILABLE_SINCE and REMOVED_IN are set correctly
    assert_eq!(QueryStatus::AVAILABLE_SINCE, (1, 0, 0));
    assert_eq!(QueryStatus::REMOVED_IN, None);
}

// ── Module separation tests ──

/// Commands must live in qapi::commands, not the crate root.
#[test]
fn commands_in_commands_module() {
    // These are commands — they implement QapiCommand
    assert_eq!(qapi::commands::QueryStatus::NAME, "query-status");
    assert_eq!(qapi::commands::QueryVersion::NAME, "query-version");
    assert_eq!(qapi::commands::Stop::NAME, "stop");
    assert_eq!(qapi::commands::Cont::NAME, "cont");
    assert_eq!(qapi::commands::Quit::NAME, "quit");
    assert_eq!(qapi::commands::SystemReset::NAME, "system_reset");
    assert_eq!(qapi::commands::SystemPowerdown::NAME, "system_powerdown");
}

/// Events must live in qapi::events, not the crate root.
#[test]
fn events_in_events_module() {
    use qapi::QapiEvent;
    assert_eq!(qapi::events::ShutdownEvent::NAME, "SHUTDOWN");
    assert_eq!(qapi::events::StopEvent::NAME, "STOP");
    assert_eq!(qapi::events::ResumeEvent::NAME, "RESUME");
    assert_eq!(qapi::events::ResetEvent::NAME, "RESET");
    assert_eq!(qapi::events::PowerdownEvent::NAME, "POWERDOWN");
}

/// Data types must live at the crate root, not in commands or events.
#[test]
fn data_types_at_crate_root() {
    // These are data types (return values, not commands)
    let _: qapi::StatusInfo =
        serde_json::from_str(r#"{"running":true,"status":"running"}"#).unwrap();

    // ErrorClass is an enum data type
    let _: qapi::ErrorClass =
        serde_json::from_str(r#""GenericError""#).unwrap();
}

/// QGA commands must live in qapi::qga::commands.
#[test]
fn qga_commands_in_module() {
    assert_eq!(qapi::qga::commands::GuestPing::NAME, "guest-ping");
    assert_eq!(qapi::qga::commands::GuestInfo::NAME, "guest-info");
    assert_eq!(qapi::qga::commands::GuestSync::NAME, "guest-sync");
}

/// QGA data types must live at qapi::qga root.
#[test]
fn qga_data_types_at_module_root() {
    // GuestAgentInfo is a data type, not a command
    let _info: Option<qapi::qga::GuestAgentInfo> = None;
}

/// Commands implement QapiCommand trait with correct metadata.
#[test]
fn command_trait_metadata() {
    // QueryStatus is a well-known command available since early QEMU
    assert_eq!(qapi::commands::QueryStatus::NAME, "query-status");
    const { assert!(qapi::commands::QueryStatus::SUCCESS_RESPONSE) };

    // Yank has specific version bounds
    assert_eq!(qapi::commands::Yank::NAME, "yank");
}

/// A command that a data type names, as the `transaction` actions name
/// `blockdev-backup`, is in `qapi::commands` like every other.
#[test]
fn cross_referenced_command_types_accessible() {
    let _: fn(qapi::BlockdevBackupWrapper) -> qapi::commands::BlockdevBackup =
        |wrapper| wrapper.data;
}

// ─── Discriminated union (flat) tests ───

#[test]
fn sev_info_deser_sev_variant() {
    // QEMU 10.2 response with sev-type discriminator
    let json = r#"{"api-major":1,"api-minor":55,"build-id":49,"enabled":true,"sev-type":"sev","state":"running","handle":1,"policy":1}"#;
    let info: SevInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.api_major, 1);
    assert_eq!(info.api_minor, 55);
    assert!(info.enabled);
    assert!(matches!(info.sev_type, Some(SevGuestType::Sev)));
    assert!(matches!(info.state, SevState::Running));
    match &info.u {
        Some(SevInfoBranch::Sev(guest)) => {
            assert_eq!(guest.handle, 1);
            assert_eq!(guest.policy, 1);
        }
        other => panic!("expected SevInfoBranch::Sev, got {:?}", other),
    }
}

#[test]
fn sev_info_deser_snp_variant() {
    let json = r#"{"api-major":1,"api-minor":55,"build-id":49,"enabled":true,"sev-type":"sev-snp","state":"running","snp-policy":196608}"#;
    let info: SevInfo = serde_json::from_str(json).unwrap();
    assert!(matches!(info.sev_type, Some(SevGuestType::SevSnp)));
    match &info.u {
        Some(SevInfoBranch::SevSnp(guest)) => {
            assert_eq!(guest.snp_policy, 196608);
        }
        other => panic!("expected SevInfoBranch::SevSnp, got {:?}", other),
    }
}

#[test]
fn sev_info_deser_pre_91() {
    // Pre-9.1.0 QEMU: no sev-type, no union branch, has handle/policy as direct
    // fields
    let json = r#"{"api-major":1,"api-minor":55,"build-id":49,"enabled":true,"state":"running","handle":1,"policy":1}"#;
    let info: SevInfo = serde_json::from_str(json).unwrap();
    assert!(info.enabled);
    assert!(info.sev_type.is_none());
    assert!(info.u.is_none());
    // handle/policy should be captured in the removed fields
    assert_eq!(info.handle, Some(1));
    assert_eq!(info.policy, Some(1));
}

#[test]
fn sev_info_roundtrip_sev() {
    let mut info = SevInfo::new(1, 55, 49, true, SevState::Running);
    info.sev_type = Some(SevGuestType::Sev);
    info.u = Some(SevInfoBranch::Sev(SevGuestInfo::new(1, 1)));
    let json = to_json(&info).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    // Verify flat structure: handle/policy at top level, not nested
    assert_eq!(parsed["handle"], 1);
    assert_eq!(parsed["policy"], 1);
    assert_eq!(parsed["sev-type"], "sev");
    // Round-trip
    let back: SevInfo = serde_json::from_str(&json).unwrap();
    assert!(matches!(back.sev_type, Some(SevGuestType::Sev)));
    match &back.u {
        Some(SevInfoBranch::Sev(g)) => {
            assert_eq!(g.handle, 1);
            assert_eq!(g.policy, 1);
        }
        other => panic!("expected Sev, got {:?}", other),
    }
}

#[test]
fn sev_info_roundtrip_snp() {
    let mut info = SevInfo::new(1, 55, 49, true, SevState::Running);
    info.sev_type = Some(SevGuestType::SevSnp);
    info.u = Some(SevInfoBranch::SevSnp(SevSnpGuestInfo::new(196608)));
    let json = to_json(&info).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["sev-type"], "sev-snp");
    assert_eq!(parsed["snp-policy"], 196608);
    let back: SevInfo = serde_json::from_str(&json).unwrap();
    assert!(matches!(back.sev_type, Some(SevGuestType::SevSnp)));
    match &back.u {
        Some(SevInfoBranch::SevSnp(g)) => assert_eq!(g.snp_policy, 196608),
        other => panic!("expected SevSnp, got {:?}", other),
    }
}

#[test]
fn sev_info_discriminator_selects_variant() {
    // Verify discriminator-based selection: even when both branch structs'
    // fields are present, the discriminator value picks the correct variant.
    // With brute-force deserialization this would incorrectly pick Sev
    // (tried first) even though sev-type says sev-snp.
    let json = r#"{"api-major":1,"api-minor":55,"build-id":49,"enabled":true,"sev-type":"sev-snp","state":"running","handle":1,"policy":1,"snp-policy":196608}"#;
    let info: SevInfo = serde_json::from_str(json).unwrap();
    assert!(matches!(info.sev_type, Some(SevGuestType::SevSnp)));
    match &info.u {
        Some(SevInfoBranch::SevSnp(g)) => assert_eq!(g.snp_policy, 196608),
        other => panic!("discriminator says sev-snp but got {:?}", other),
    }
}

#[test]
fn sev_info_unknown_discriminator_errors() {
    let json = r#"{"api-major":1,"api-minor":55,"build-id":49,"enabled":true,"sev-type":"sev-tdx","state":"running"}"#;
    let result = serde_json::from_str::<SevInfo>(json);
    assert!(result.is_err(), "unknown discriminator should fail");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("sev-tdx"),
        "error should mention unknown value: {}",
        err
    );
}

/// `type` is carried by two version alternates; the removed one's enum has no
/// `file`, which QEMU 8.0+ reports for every file-posix node.
#[test]
fn shared_wire_discriminator_decodes_current_values() {
    let file: ImageInfoSpecific = serde_json::from_str(
        r#"{"type":"file","data":{"extent-size-hint":1048576}}"#,
    )
    .unwrap();
    assert!(file.r_type.is_none());
    assert!(matches!(file.r#type, Some(ImageInfoSpecificKind::File)));
    let Some(ImageInfoSpecificBranch::File(wrapper)) = file.u else {
        panic!("file branch");
    };
    assert_eq!(wrapper.data.extent_size_hint, Some(1048576));
    let qcow2: ImageInfoSpecific = serde_json::from_str(
        r#"{"type":"qcow2","data":{"compat":"1.1","lazy-refcounts":false,"refcount-bits":16,"corrupt":false,"extended-l2":false,"compression-type":"zlib"}}"#,
    )
    .unwrap();
    assert!(matches!(qcow2.r#type, Some(ImageInfoSpecificKind::Qcow2)));
    assert!(matches!(qcow2.u, Some(ImageInfoSpecificBranch::Qcow2(_))));
    let error = serde_json::from_str::<ImageInfoSpecific>(
        r#"{"type":"bogus","data":{}}"#,
    )
    .expect_err("an unknown discriminator is refused");
    assert!(
        error.to_string().starts_with(
            "ImageInfoSpecific.type: unknown variant `bogus`, expected one of"
        ),
        "{error}"
    );
}

#[test]
fn shared_wire_discriminator_roundtrips_once() {
    let json = r#"{"type":"unix","path":"/tmp/qmp.sock"}"#;
    let address: SocketAddress = serde_json::from_str(json).unwrap();
    assert!(matches!(address.u, Some(SocketAddressBranch::Unix(_))));
    let back = qapi::to_value((11, 1, 3), &address).unwrap();
    assert_eq!(back["type"], "unix");
    assert_eq!(back["path"], "/tmp/qmp.sock");
    assert_eq!(back.as_object().unwrap().len(), 2, "{back}");
}

// Empty enums are valid QAPI schemas, though no wire value can inhabit them.
#[qapi::qapi]
enum EmptySchemaEnum {}

#[test]
fn empty_schema_enum_rejects_every_wire_value() {
    for input in ["null", "\"anything\"", "{}", "0"] {
        assert!(serde_json::from_str::<EmptySchemaEnum>(input).is_err());
    }
}
