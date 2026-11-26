//! Code outside `qrab-qapi` that builds structs with their constructors and
//! matches enums with a wildcard arm compiles whichever older releases the
//! build keeps, as long as the shapes stay. `contrib/check-qapi.sh`
//! compiles it with the minimums 9.2.0 and 3.0.0: the older range adds
//! `StatusInfo.singlestep` (removed in 9.0) and `MigrationCapability::Block`
//! (removed in 9.1), and with them the fields and variants that a lower
//! floor adds to the API of a crate that another crate of the build uses.

use qapi::commands::{MigrateSetCapabilities, QueryStatus};
use qapi::{MigrationCapability, MigrationCapabilityStatus, RunState, StatusInfo};

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
fn constructors_and_wildcard_arms_compile_in_any_range() {
    let info = status(true);
    assert_eq!(
        qapi::to_value((11, 1, 0), &info).unwrap(),
        serde_json::json!({"running": true, "status": "running"})
    );
    let command = MigrateSetCapabilities::new(vec![MigrationCapabilityStatus::new(
        MigrationCapability::PostcopyRam,
        true,
    )]);
    assert_eq!(describe(&command.capabilities[0].capability), "postcopy");
    assert!(matches!(
        RunState::Paused,
        RunState::Paused | RunState::Running
    ));
    let query = QueryStatus::default();
    assert_eq!(
        qapi::to_value((11, 1, 0), &query).unwrap(),
        serde_json::json!({})
    );
}
