//! The expansion of commands and events, compiled and run against the
//! stand-in runtime in `qapi/`: their trait constants and per-release
//! flags, and the replies of a command whose return type changed.

mod qapi;

use qapi::{PreconfigSafe, QapiCommand, QapiEvent, take_unavailable, with_version};
use qapi_macros::qapi;
use serde_json::{Value, json};

#[qapi]
struct Thing {
    name: String,
}

#[qapi(
    command,
    name = "query-thing",
    returns = "Thing",
    added = "1.0.0",
    removed = "9.0.0",
    allow_oob(added = "2.12.0"),
    allow_preconfig(added = "3.0.0", removed = "5.0.0"),
    allow_preconfig(added = "6.0.0"),
    no_success_response(added = "4.0.0")
)]
struct QueryThing {
    #[qapi(added = "2.0.0")]
    verbose: Option<bool>,
}

#[qapi(command, name = "stop")]
struct Stop {}

#[qapi(command, added = "2.0.0", allow_preconfig)]
struct Preconfig {}

#[qapi(command, feature = "unstable")]
struct Experimental {}

#[qapi(command, unstable(added = "8.0.0"))]
struct Unstable {}

#[qapi(command, unstable(removed = "8.0.0"))]
struct Stabilized {}

fn returns<C: QapiCommand<Returns = R>, R>() {}

#[test]
fn commands_describe_the_newest_schema_in_their_constants() {
    const {
        assert!(QueryThing::ALLOW_OOB);
        assert!(QueryThing::ALLOW_PRECONFIG);
        assert!(!QueryThing::SUCCESS_RESPONSE);
        assert!(!QueryThing::UNSTABLE);
        assert!(!Stop::ALLOW_OOB && !Stop::ALLOW_PRECONFIG && Stop::SUCCESS_RESPONSE);
        assert!(Experimental::UNSTABLE && Unstable::UNSTABLE && !Stabilized::UNSTABLE);
    }
    assert_eq!(QueryThing::NAME, "query-thing");
    assert_eq!(QueryThing::AVAILABLE_SINCE, (1, 0, 0));
    assert_eq!(QueryThing::REMOVED_IN, Some((9, 0, 0)));
    assert_eq!(Stop::AVAILABLE_SINCE, (0, 0, 0));
    assert_eq!(Stop::REMOVED_IN, None);
    returns::<QueryThing, Thing>();
    returns::<Stop, ()>();
}

#[test]
fn command_flags_hold_in_their_releases() {
    let oob = [((2, 11, 9), false), ((2, 12, 0), true), ((8, 0, 0), true)];
    for (version, expected) in oob {
        assert_eq!(QueryThing::allows_oob(version), expected, "{version:?}");
    }
    let preconfig = [
        ((2, 9, 9), false),
        ((3, 0, 0), true),
        ((4, 9, 9), true),
        ((5, 0, 0), false),
        ((6, 0, 0), true),
    ];
    for (version, expected) in preconfig {
        assert_eq!(
            QueryThing::allows_preconfig(version),
            expected,
            "{version:?}"
        );
    }
    for (version, expected) in [((3, 9, 9), true), ((4, 0, 0), false)] {
        assert_eq!(QueryThing::sends_success_response(version), expected);
    }
    assert!(!Stop::allows_oob((11, 0, 0)) && Stop::sends_success_response((11, 0, 0)));
    assert!(Experimental::is_unstable((1, 0, 0)));
    assert!(!Unstable::is_unstable((7, 9, 9)) && Unstable::is_unstable((8, 0, 0)));
    assert!(Stabilized::is_unstable((7, 9, 9)) && !Stabilized::is_unstable((8, 0, 0)));
    assert!(!QueryThing::is_unstable((5, 0, 0)));
}

/// `is_preconfig_safe()` is the inherent method, `true`, exactly when `T`
/// implements `PreconfigSafe`; otherwise the trait method, `false`.
struct Probe<T>(std::marker::PhantomData<T>);

trait NotPreconfigSafe {
    fn is_preconfig_safe(&self) -> bool {
        false
    }
}

impl<T> NotPreconfigSafe for Probe<T> {}

impl<T: PreconfigSafe> Probe<T> {
    fn is_preconfig_safe(&self) -> bool {
        true
    }
}

#[test]
fn commands_with_preconfig_releases_are_preconfig_safe() {
    assert!(Probe::<QueryThing>(std::marker::PhantomData).is_preconfig_safe());
    assert!(Probe::<Preconfig>(std::marker::PhantomData).is_preconfig_safe());
    assert!(!Probe::<Stop>(std::marker::PhantomData).is_preconfig_safe());
    // The first release that allows it, or the command's own.
    assert_eq!(QueryThing::PRECONFIG_SINCE, Some((3, 0, 0)));
    assert_eq!(Preconfig::PRECONFIG_SINCE, Some((2, 0, 0)));
    assert_eq!(Stop::PRECONFIG_SINCE, None);
}

#[test]
fn commands_serialize_their_arguments_and_display_their_name() {
    let command = QueryThing {
        verbose: Some(true),
    };
    assert_eq!(command.to_string(), "query-thing");
    assert_eq!(
        with_version((2, 0, 0), || serde_json::to_value(&command)).unwrap(),
        json!({"verbose": true})
    );
    let error = with_version((1, 9, 0), || serde_json::to_value(&command)).unwrap_err();
    assert_eq!(
        error.to_string(),
        "QueryThing.verbose is not available in QEMU 1.9.0"
    );
    assert_eq!(take_unavailable(), ["QueryThing.verbose"]);
}

#[qapi(removed = "10.1.0")]
struct TargetInfo {
    arch: Option<String>,
}

#[qapi(added = "10.1.0")]
struct QemuTargetInfo {
    arch: String,
}

#[qapi(
    command,
    name = "query-target",
    returns = "QemuTargetInfo",
    added = "1.2.0",
    returns(added = "1.2.0", value = "TargetInfo"),
    returns(added = "10.1.0", value = "QemuTargetInfo")
)]
struct QueryTarget {}

#[qapi]
struct TrimResponse {
    paths: Vec<String>,
}

#[qapi(
    command,
    name = "fstrim",
    returns = "TrimResponse",
    added = "1.2.0",
    returns(added = "1.2.0", value = "()"),
    returns(added = "2.4.0", value = "TrimResponse")
)]
struct Fstrim {}

fn reply<C: QapiCommand>(
    version: Option<(u32, u32, u32)>,
    json: Value,
) -> Result<C::Returns, String> {
    let decode = || serde_json::from_value(json).map_err(|error| error.to_string());
    match version {
        Some(version) => with_version(version, decode),
        None => decode(),
    }
}

#[test]
fn a_reply_decodes_into_the_return_type_of_the_peers_release() {
    returns::<QueryTarget, QueryTargetReturns>();
    let arch = json!({"arch": "x86_64"});
    for version in [(1, 2, 0), (10, 0, 9)] {
        match reply::<QueryTarget>(Some(version), arch.clone()) {
            Ok(QueryTargetReturns::V1_2_0(info)) => {
                assert_eq!(info.arch.as_deref(), Some("x86_64"))
            }
            other => panic!("{version:?}: {other:?}"),
        }
    }
    for version in [Some((10, 1, 0)), None] {
        match reply::<QueryTarget>(version, arch.clone()) {
            Ok(QueryTargetReturns::V10_1_0(info)) => assert_eq!(info.arch, "x86_64"),
            other => panic!("{version:?}: {other:?}"),
        }
    }
    assert_eq!(
        reply::<QueryTarget>(Some((1, 1, 0)), arch).unwrap_err(),
        "QueryTargetReturns is not available in QEMU 1.1.0"
    );
}

#[test]
fn a_release_without_a_return_value_replies_an_empty_object() {
    for wire in [json!({}), json!(null)] {
        assert!(matches!(
            reply::<Fstrim>(Some((2, 3, 0)), wire),
            Ok(FstrimReturns::V1_2_0(()))
        ));
    }
    assert_eq!(
        reply::<Fstrim>(Some((2, 3, 0)), json!({"paths": []})).unwrap_err(),
        r#"FstrimReturns::V1_2_0: expected an empty object, found {"paths":[]}"#
    );
    assert!(matches!(
        reply::<Fstrim>(Some((2, 4, 0)), json!({"paths": ["/"]})),
        Ok(FstrimReturns::V2_4_0(TrimResponse { .. }))
    ));
}

#[test]
fn a_reply_serializes_only_for_the_releases_of_its_return_type() {
    let old = QueryTargetReturns::V1_2_0(TargetInfo {
        arch: Some("x86_64".into()),
    });
    assert_eq!(
        with_version((9, 2, 0), || serde_json::to_value(&old)).unwrap(),
        json!({"arch": "x86_64"})
    );
    let error = with_version((10, 1, 0), || serde_json::to_value(&old)).unwrap_err();
    assert_eq!(
        error.to_string(),
        "QueryTargetReturns::V1_2_0 is not available in QEMU 10.1.0"
    );
    assert_eq!(take_unavailable(), ["QueryTargetReturns::V1_2_0"]);
    let new = QueryTargetReturns::V10_1_0(QemuTargetInfo {
        arch: "x86_64".into(),
    });
    assert!(with_version((10, 1, 0), || serde_json::to_value(&new)).is_ok());
}

#[qapi(event, name = "SHUTDOWN")]
struct ShutdownEvent {
    guest: bool,
}

#[test]
fn events_decode_their_data_under_their_name() {
    assert_eq!(<ShutdownEvent as QapiEvent>::NAME, "SHUTDOWN");
    let event: ShutdownEvent = serde_json::from_value(json!({"guest": true})).unwrap();
    assert!(event.guest);
    assert_eq!(event.to_string(), "SHUTDOWN(guest=true)");
}
