//! Rust `#[deprecated]` types, compiled with that lint denied: the types and
//! members that hold such a type do not warn where they declare it, only
//! code that names the type does.
#![deny(deprecated)]

mod qapi;

use qapi_macros::qapi;
use serde_json::json;

#[qapi(feature = "deprecated")]
enum OldKind {
    #[qapi(name = "a")]
    A,
}

#[qapi]
enum Choice {
    Kind(OldKind),
    Number(u32),
}

#[test]
#[allow(deprecated, reason = "naming the deprecated type warns")]
fn an_alternate_holds_a_deprecated_type() {
    let choice: Choice = serde_json::from_value(json!("a")).unwrap();
    assert!(matches!(choice, Choice::Kind(OldKind::A)));
}

#[qapi(feature = "deprecated")]
struct Old {
    a: u32,
}

#[qapi(union_branch)]
enum Branch {
    #[qapi(name = "a")]
    A(Old),
}

#[qapi]
struct Holder {
    old: Option<Old>,
    #[qapi(discriminator, name = "type")]
    kind: OldKind,
    #[qapi(flatten, union)]
    u: Option<Branch>,
}

#[test]
#[allow(deprecated, reason = "naming the deprecated types warns")]
fn a_struct_and_a_union_branch_hold_a_deprecated_type() {
    let holder: Holder = serde_json::from_value(json!({"type": "a", "a": 1})).unwrap();
    assert!(holder.old.is_none());
    assert!(matches!(holder.kind, OldKind::A));
    assert!(matches!(holder.u, Some(Branch::A(Old { a: 1 }))));
}
