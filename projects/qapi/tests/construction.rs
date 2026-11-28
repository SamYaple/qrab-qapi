//! Code outside `qrab-qapi` builds structs with their constructors and
//! matches enums with a wildcard arm, so that the fields and variants that a
//! later QEMU baseline adds break none of it.

use qapi::{
    MigrationCapability, MigrationCapabilityStatus, RunState, StatusInfo,
    commands::{MigrateSetCapabilities, QueryStatus},
};

fn status(running: bool) -> StatusInfo {
    StatusInfo::new(running, RunState::Running)
}

fn describe(capability: &MigrationCapability) -> &'static str {
    match capability {
        MigrationCapability::PostcopyRam => "postcopy",
        MigrationCapability::Xbzrle => "xbzrle",
        _ => "another capability",
    }
}

#[test]
fn constructors_and_wildcard_arms_compile() {
    let info = status(true);
    assert_eq!(
        qapi::to_value((11, 1, 0), &info).unwrap(),
        serde_json::json!({"running": true, "status": "running"})
    );
    let command =
        MigrateSetCapabilities::new(vec![MigrationCapabilityStatus::new(
            MigrationCapability::PostcopyRam,
            true,
        )]);
    assert_eq!(describe(&command.capabilities[0].capability), "postcopy");
    assert!(matches!(RunState::Paused, RunState::Paused | RunState::Running));
    let query = QueryStatus::default();
    assert_eq!(
        qapi::to_value((11, 1, 0), &query).unwrap(),
        serde_json::json!({})
    );
}
