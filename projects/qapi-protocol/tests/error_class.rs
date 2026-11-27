//! The protocol's own `ErrorClass` against the error classes that the
//! generated types of qrab-qapi know from the pinned QEMU releases.

use qapi_protocol::{ErrorClass, RemoteError};

#[test]
fn every_error_class_of_the_pinned_releases_is_known() {
    let classes = qapi::metadata::ENUMS
        .get("QapiErrorClass")
        .expect("the generated error classes");
    assert!(!classes.variants().is_empty());
    for variant in classes.variants() {
        let wire = variant.wire_name();
        let class = RemoteError::new(wire, "").known_class();
        assert_eq!(class.map(ErrorClass::as_str), Some(wire), "{wire}");
    }
}

#[test]
fn an_unknown_class_stays_a_string() {
    let error = RemoteError::new("FutureClass", "from a newer QEMU");
    assert_eq!(error.known_class(), None);
    assert_eq!(error.class, "FutureClass");
}
