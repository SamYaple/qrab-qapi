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
