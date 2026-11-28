// The tests use items and members of older releases, which the newest
// release no longer has and which are therefore deprecated.
#![allow(deprecated)]

use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[qapi::qapi(added = "2.0.0", removed = "3.0.0")]
struct Record {
    value: String,
}

/// A type whose name ends in another's.
#[qapi::qapi(added = "2.0.0", removed = "3.0.0")]
struct OldRecord {
    value: String,
}

#[qapi::qapi(added = "2.0.0", removed = "3.0.0")]
struct Flattened {
    #[qapi(flatten)]
    properties: Option<std::collections::HashMap<String, Value>>,
}

#[qapi::qapi(added = "2.0.0", removed = "3.0.0")]
enum Kind {
    #[qapi(name = "value")]
    Value,
}

#[qapi::qapi(added = "2.0.0", removed = "3.0.0")]
enum Alternate {
    Text(String),
    Null(()),
}

fn check<T: DeserializeOwned>(wire: Value) {
    for version in [(1, 9, 0), (3, 0, 0)] {
        let error = qapi::with_version(version, || {
            serde_json::from_value::<T>(wire.clone())
        })
        .err()
        .unwrap();
        assert!(error.to_string().contains("not available"), "{error}");
    }
    qapi::with_version((2, 0, 0), || serde_json::from_value::<T>(wire.clone()))
        .unwrap();
    // Recorded values can still be inspected without asserting a peer release.
    serde_json::from_value::<T>(wire).unwrap();
}

#[test]
fn explicit_peer_contexts_enforce_type_lifetimes_for_all_wire_shapes() {
    check::<Record>(json!({"value":"record"}));
    check::<Flattened>(json!({"value":"flattened"}));
    check::<Kind>(json!("value"));
    check::<Alternate>(json!("text"));
    check::<Alternate>(Value::Null);
}

#[qapi::qapi(added = "1.0.0")]
enum Mode {
    #[qapi(name = "old")]
    Old,
    #[qapi(name = "new", added = "2.0.0")]
    New,
}

#[qapi::qapi(added = "1.0.0")]
struct Shared {
    #[qapi(name = "value", added = "2.0.0")]
    value: Option<String>,
    #[qapi(name = "value", removed = "2.0.0")]
    value_legacy: Option<i64>,
}

#[qapi::qapi(added = "1.0.0")]
struct Extra {
    extra: bool,
}

#[qapi::qapi(added = "1.0.0")]
struct Outer {
    mode: Option<Mode>,
    record: Option<Record>,
    shared: Option<Shared>,
    #[qapi(flatten, added = "2.0.0")]
    extra: Option<Extra>,
    #[qapi(added = "2.0.0", required)]
    since_two: Option<bool>,
}

fn unavailable<T: serde::Serialize>(
    version: (u32, u32, u32),
    value: &T,
) -> String {
    match qapi::to_value(version, value) {
        Err(qapi::EncodeError::Unavailable { name, version: reported }) => {
            assert_eq!(reported, version, "{name}");
            name
        }
        other => panic!("{other:?}"),
    }
}

fn outer() -> Outer {
    Outer {
        mode: None,
        record: None,
        shared: None,
        extra: None,
        since_two: None,
    }
}

#[test]
fn encoding_names_the_unavailable_type_variant_or_field_wherever_it_is() {
    let record = Record { value: "record".into() };
    assert_eq!(unavailable((3, 0, 0), &record), "Record");
    assert_eq!(unavailable((1, 0, 0), &Mode::New), "Mode::New");
    let shared = Shared { value: Some("new".into()), value_legacy: None };
    assert_eq!(unavailable((1, 0, 0), &shared), "Shared.value");

    // Nested: in a field, in a list and in a flattened payload.
    let nested = Outer { record: Some(record.clone()), ..outer() };
    assert_eq!(unavailable((1, 9, 0), &nested), "Record");
    let nested = vec![Outer { mode: Some(Mode::New), ..outer() }];
    assert_eq!(unavailable((1, 5, 0), &nested), "Mode::New");
    let nested = Outer { shared: Some(shared), ..outer() };
    assert_eq!(unavailable((1, 5, 0), &nested), "Shared.value");
    let nested = Outer { extra: Some(Extra { extra: true }), ..outer() };
    assert_eq!(unavailable((1, 5, 0), &nested), "Outer.extra");

    // The error reads the same with and without `to_value`.
    let error = qapi::to_value((1, 0, 0), &Mode::New).unwrap_err();
    assert_eq!(error.to_string(), "Mode::New is not available in QEMU 1.0.0");
    let error =
        qapi::with_version((1, 0, 0), || serde_json::to_value(Mode::New))
            .unwrap_err();
    assert_eq!(error.to_string(), "Mode::New is not available in QEMU 1.0.0");

    assert_eq!(qapi::to_value((2, 0, 0), &Mode::New).unwrap(), "new");
    assert_eq!(qapi::to_value((1, 0, 0), &Mode::Old).unwrap(), "old");
}

#[test]
fn other_encoding_failures_are_serde_errors() {
    // `since_two` is required from 2.0.0.
    match qapi::to_value((2, 0, 0), &outer()) {
        Err(qapi::EncodeError::Serde(error)) => {
            assert!(error.to_string().contains("since_two"), "{error}");
        }
        other => panic!("{other:?}"),
    }
    // A failure outside `to_value` leaves nothing behind for the next call.
    qapi::with_version((1, 0, 0), || serde_json::to_value(Mode::New))
        .unwrap_err();
    assert!(matches!(
        qapi::to_value((2, 0, 0), &outer()),
        Err(qapi::EncodeError::Serde(_))
    ));
}

/// A hand-written value that serializes a part of itself for a peer.
struct ForPeer<T>((u32, u32, u32), T);

impl<T: serde::Serialize> serde::Serialize for ForPeer<T> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let value = qapi::to_value(self.0, &self.1)
            .map_err(serde::ser::Error::custom)?;
        serde::Serialize::serialize(&value, serializer)
    }
}

#[test]
fn an_enclosing_encoding_reports_what_a_nested_one_found() {
    match qapi::to_value((2, 0, 0), &ForPeer((1, 0, 0), Mode::New)) {
        Err(qapi::EncodeError::Unavailable { name, version }) => {
            assert_eq!((name.as_str(), version), ("Mode::New", (1, 0, 0)));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        qapi::to_value((2, 0, 0), &ForPeer((1, 0, 0), Mode::Old)).unwrap(),
        "old"
    );
}

/// A hand-written value that recovers from a failure: the old mode where
/// the peer lacks the new one, then the rest of the value.
struct Fallback<T>(T);

impl<T: serde::Serialize> serde::Serialize for Fallback<T> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeTuple;
        let mode = qapi::to_value(qapi::qemu_version(), &Mode::New)
            .or_else(|_| serde_json::to_value(Mode::Old))
            .map_err(serde::ser::Error::custom)?;
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&mode)?;
        tuple.serialize_element(&self.0)?;
        tuple.end()
    }
}

/// A hand-written value that serializes a second value, which fails too,
/// on its way out of the failure of the first.
struct Diagnosed<A, B>(A, B);

impl<A: serde::Serialize, B: serde::Serialize> serde::Serialize
    for Diagnosed<A, B>
{
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let first = serde_json::to_value(&self.0).map_err(|error| {
            let _ = serde_json::to_value(&self.1);
            serde::ser::Error::custom(error)
        })?;
        serde::Serialize::serialize(&first, serializer)
    }
}

/// A hand-written value that retries with its second value after its first
/// failed, and fails with both errors, the second first.
struct Retried<A, B>(A, B);

impl<A: serde::Serialize, B: serde::Serialize> serde::Serialize
    for Retried<A, B>
{
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let value = serde_json::to_value(&self.0).or_else(|first| {
            serde_json::to_value(&self.1).map_err(|second| {
                serde::ser::Error::custom(format!("{second}; first {first}"))
            })
        })?;
        serde::Serialize::serialize(&value, serializer)
    }
}

/// A value that never serializes.
struct Refused;

impl serde::Serialize for Refused {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("refused"))
    }
}

#[test]
fn encoding_reports_the_failure_that_the_error_carries() {
    let record = || Record { value: "record".into() };
    assert_eq!(
        qapi::to_value((1, 9, 0), &Fallback(true)).unwrap(),
        json!(["old", true])
    );
    // A failure that a value recovered from is not the one reported.
    assert_eq!(unavailable((1, 9, 0), &Fallback(record())), "Record");
    match qapi::to_value((1, 9, 0), &Fallback(Refused)) {
        Err(qapi::EncodeError::Serde(error)) => {
            assert_eq!(error.to_string(), "refused")
        }
        other => panic!("{other:?}"),
    }
    // Nor is one that came after it, in a longer name or not.
    assert_eq!(
        unavailable((1, 9, 0), &Diagnosed(record(), Mode::New)),
        "Record"
    );
    let old = OldRecord { value: "old".into() };
    assert_eq!(unavailable((1, 9, 0), &Diagnosed(old, record())), "OldRecord");
    // Nor one whose version begins the version of the failure.
    let later =
        Diagnosed(ForPeer((3, 0, 10), record()), ForPeer((3, 0, 1), record()));
    match qapi::to_value((2, 0, 0), &later) {
        Err(qapi::EncodeError::Unavailable { name, version }) => {
            assert_eq!((name.as_str(), version), ("Record", (3, 0, 10)));
        }
        other => panic!("{other:?}"),
    }
    // An error that carries several failures reports the newest.
    let retried = Retried(record(), ForPeer((3, 0, 10), record()));
    match qapi::to_value((3, 0, 1), &retried) {
        Err(qapi::EncodeError::Unavailable { name, version }) => {
            assert_eq!((name.as_str(), version), ("Record", (3, 0, 10)));
        }
        other => panic!("{other:?}"),
    }
}
