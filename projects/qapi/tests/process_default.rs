//! The process default version from `set_qemu_version`. It can be set once
//! per process, so this test binary holds the one test that sets it; the
//! other tests scope their versions with `with_version`.

use qapi::{
    VersionAlreadySet, qemu_version, scoped_qemu_version, set_qemu_version,
    with_version,
};
use serde_json::json;

#[qapi::qapi]
struct Gated {
    stable: u32,
    #[qapi(added = "11.0.0")]
    newer: Option<u32>,
}

#[qapi::qapi]
enum LayoutKind {
    #[qapi(name = "text")]
    Text,
}

#[qapi::qapi]
struct LayoutPayload {
    value: String,
}

#[qapi::qapi(union_branch)]
enum LayoutBranch {
    #[qapi(name = "text")]
    Text(LayoutPayload),
}

#[qapi::qapi(simple_union(removed = "11.0.0"))]
struct Layout {
    #[qapi(name = "type", discriminator)]
    kind: LayoutKind,
    #[qapi(flatten, union)]
    u: Option<LayoutBranch>,
}

#[test]
fn the_process_default_is_set_once_and_serves_serialization_only() {
    assert_eq!(qemu_version(), (0, 0, 0));
    assert_eq!(set_qemu_version((10, 1, 0)), Ok(()));
    for requested in [(7, 0, 0), (10, 1, 0)] {
        let error: VersionAlreadySet = set_qemu_version(requested).unwrap_err();
        assert_eq!((error.existing, error.requested), ((10, 1, 0), requested));
        assert!(error.to_string().contains("already 10.1.0"), "{error}");
    }
    assert_eq!(qemu_version(), (10, 1, 0));
    assert_eq!(scoped_qemu_version(), None);

    // Unscoped serialization uses the default; a scope overrides it.
    let gated = Gated { stable: 1, newer: Some(2) };
    assert_eq!(
        serde_json::to_value(&gated).unwrap_err().to_string(),
        "Gated.newer is not available in QEMU 10.1.0"
    );
    assert_eq!(
        qapi::to_value((11, 0, 0), &gated).unwrap(),
        json!({"stable": 1, "newer": 2})
    );
    assert_eq!(qemu_version(), (10, 1, 0));

    // Unscoped decoding takes the newest layout even though the default is
    // older; only an explicit scope selects the historical one.
    let flat: Layout =
        serde_json::from_value(json!({"type": "text", "value": "flat"}))
            .unwrap();
    assert!(
        matches!(flat.u, Some(LayoutBranch::Text(ref p)) if p.value == "flat")
    );
    let nested: Layout = with_version((10, 1, 0), || {
        serde_json::from_value(
            json!({"type": "text", "data": {"value": "nested"}}),
        )
    })
    .unwrap();
    assert!(
        matches!(nested.u, Some(LayoutBranch::Text(ref p)) if p.value == "nested")
    );
}
