//! The version contexts, with hand-written version-gated values standing in
//! for the generated ones of qrab-qapi.

use qapi_core::{
    EncodeError, qemu_version, scoped_qemu_version, to_value, unavailable_error,
    unavailable_type_message, with_version,
};
use serde::{Serialize, Serializer};

/// A value that only QEMU `since` and later have, as the `#[qapi]` expansion
/// gates a type.
struct Gated {
    name: &'static str,
    since: (u32, u32, u32),
    value: u32,
}

impl Serialize for Gated {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if qemu_version() < self.since {
            return Err(unavailable_error(self.name));
        }
        self.value.serialize(serializer)
    }
}

fn gated(name: &'static str, since: (u32, u32, u32)) -> Gated {
    Gated {
        name,
        since,
        value: 1,
    }
}

#[test]
fn a_scope_nests_and_restores_the_outer_version() {
    assert_eq!(scoped_qemu_version(), None);
    with_version((10, 0, 0), || {
        assert_eq!(qemu_version(), (10, 0, 0));
        with_version((11, 1, 3), || {
            assert_eq!(scoped_qemu_version(), Some((11, 1, 3)))
        });
        assert_eq!(qemu_version(), (10, 0, 0));
    });
    assert_eq!(scoped_qemu_version(), None);
    assert_eq!(qemu_version(), (0, 0, 0));
}

#[test]
fn a_scope_is_restored_when_its_closure_panics() {
    let result = std::panic::catch_unwind(|| with_version((9, 2, 0), || panic!("inside")));
    assert!(result.is_err());
    assert_eq!(scoped_qemu_version(), None);
}

#[test]
fn to_value_names_what_the_release_lacks_wherever_it_is_nested() {
    assert_eq!(
        to_value((11, 1, 3), &gated("Gated", (10, 0, 0))).unwrap(),
        1
    );
    let nested = vec![gated("Outer", (1, 0, 0)), gated("Inner", (10, 0, 0))];
    match to_value((9, 2, 0), &nested) {
        Err(EncodeError::Unavailable { name, version }) => {
            assert_eq!((name.as_str(), version), ("Inner", (9, 2, 0)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_unrelated_error_is_not_reported_as_unavailable() {
    struct Fails;
    impl Serialize for Fails {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("Gated is broken"))
        }
    }
    assert!(matches!(
        to_value((11, 1, 3), &Fails),
        Err(EncodeError::Serde(_))
    ));
}

#[test]
fn a_name_is_matched_whole_not_as_the_end_of_a_longer_one() {
    // The value fails with the error of "OtherGated" after recovering from a
    // failure of "Gated", whose message the error text contains as a tail.
    // The newest recorded failure is "Gated", but the error does not carry it
    // whole, so the report names "OtherGated".
    struct Recovers;
    impl Serialize for Recovers {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            let kept = serde_json::to_value(gated("OtherGated", (10, 0, 0))).unwrap_err();
            let _ = serde_json::to_value(gated("Gated", (10, 0, 0)));
            Err(serde::ser::Error::custom(kept))
        }
    }
    match to_value((9, 2, 0), &Recovers) {
        Err(EncodeError::Unavailable { name, .. }) => assert_eq!(name, "OtherGated"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn without_a_context_the_error_says_how_to_give_one() {
    let error = serde_json::to_value(gated("Gated", (10, 0, 0))).unwrap_err();
    assert!(error.to_string().contains("qapi::with_version"), "{error}");
    assert!(unavailable_type_message("Gated").contains("without a QEMU version"));
    assert_eq!(
        with_version((9, 2, 0), || unavailable_type_message("Gated")),
        "Gated is not available in QEMU 9.2.0"
    );
}
