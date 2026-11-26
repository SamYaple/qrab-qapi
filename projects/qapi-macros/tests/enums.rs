//! The expansion of enums, compiled and run against the stand-in runtime in
//! `runtime/`: string enums, alternates and union branches, with the lifetimes
//! of their types and variants.

// The tests use items and members of older releases, which the newest
// release no longer has and which are therefore deprecated.
#![allow(deprecated)]

mod child;
// The stand-in runtime, which the expansion names `::qapi`.
extern crate self as qapi;
mod runtime;
use runtime::*;
mod record;

use qapi::{QapiFlatUnion, take_unavailable, with_default, with_version};
use qapi_macros::qapi;
use serde_json::json;

#[qapi(added = "2.0.0")]
enum Driver {
    #[qapi(name = "qcow2")]
    Qcow2,
    #[qapi(name = "archipelago", added = "2.2.0", removed = "2.9.0")]
    Archipelago,
    #[qapi(name = "dep", deprecated(added = "9.2.0"))]
    Dep,
}

#[qapi(name = "ACPISlotType")]
enum AcpiSlotType {
    #[qapi(name = "DIMM")]
    Dimm,
    #[qapi(name = "CPU")]
    Cpu,
}

/// Old schemas have an enum member `''`.
#[qapi]
enum LegacyNames {
    #[qapi(name = "")]
    Empty,
    #[qapi(name = "1-foo")]
    OneFoo,
}

#[test]
fn a_member_may_have_an_empty_wire_name() {
    assert_eq!(LegacyNames::Empty.to_string(), "");
    assert_eq!(encode((1, 0, 0), &LegacyNames::Empty).unwrap(), r#""""#);
    assert!(matches!(decode(None, json!("")), Ok(LegacyNames::Empty)));
    assert!(matches!(
        decode(None, json!("1-foo")),
        Ok(LegacyNames::OneFoo)
    ));
}

/// One wire name with a representation per lifetime.
#[qapi]
enum Mode {
    #[qapi(name = "fast", removed = "3.0.0")]
    FastOld,
    #[qapi(name = "fast", added = "3.0.0")]
    Fast,
}

fn encode<T: serde::Serialize>(version: (u32, u32, u32), value: &T) -> Result<String, String> {
    with_version(version, || serde_json::to_string(value)).map_err(|error| error.to_string())
}

fn decode<T: serde::de::DeserializeOwned>(
    version: Option<(u32, u32, u32)>,
    json: serde_json::Value,
) -> Result<T, String> {
    let decode = || serde_json::from_value(json).map_err(|error| error.to_string());
    match version {
        Some(version) => with_version(version, decode),
        None => decode(),
    }
}

#[test]
fn string_enums_write_and_read_their_wire_names() {
    assert_eq!(Driver::Qcow2.to_string(), "qcow2");
    assert_eq!(AcpiSlotType::Cpu.to_string(), "CPU");
    assert_eq!(encode((9, 2, 0), &Driver::Qcow2).unwrap(), r#""qcow2""#);
    assert!(matches!(
        decode(Some((9, 2, 0)), json!("qcow2")),
        Ok(Driver::Qcow2)
    ));
    // The QAPI name, or else the Rust name, names the enum in Serde's model.
    assert_eq!(
        record::record(&AcpiSlotType::Cpu).unwrap(),
        "unit_variant(ACPISlotType, 1, CPU)"
    );
    let recorded = with_version((9, 2, 0), || record::record(&Driver::Dep)).unwrap();
    assert_eq!(recorded, "unit_variant(Driver, 2, dep)");
}

#[test]
fn variants_and_types_serialize_only_within_their_lifetime() {
    for (version, available) in [
        ((2, 1, 9), false),
        ((2, 2, 0), true),
        ((2, 8, 9), true),
        ((2, 9, 0), false),
    ] {
        let result = encode(version, &Driver::Archipelago);
        assert_eq!(result.is_ok(), available, "{version:?}: {result:?}");
        if !available {
            let (major, minor, patch) = version;
            assert_eq!(
                result.unwrap_err(),
                format!("Driver::Archipelago is not available in QEMU {major}.{minor}.{patch}")
            );
            assert_eq!(take_unavailable(), ["Driver::Archipelago"]);
        }
    }
    assert_eq!(
        encode((1, 9, 9), &Driver::Qcow2).unwrap_err(),
        "Driver is not available in QEMU 1.9.9"
    );
    assert_eq!(take_unavailable(), ["Driver"]);
    assert!(encode((2, 0, 0), &Driver::Qcow2).is_ok());
    // Outside any scope the default version applies.
    assert!(with_default((2, 5, 0), || serde_json::to_string(&Driver::Archipelago)).is_ok());
}

#[test]
fn scoped_decoding_names_the_unavailable_variant_and_lists_the_active_ones() {
    assert_eq!(
        decode::<Driver>(Some((9, 2, 0)), json!("archipelago")).unwrap_err(),
        "Driver::Archipelago is not available in QEMU 9.2.0"
    );
    assert!(matches!(
        decode(Some((2, 2, 0)), json!("archipelago")),
        Ok(Driver::Archipelago)
    ));
    assert_eq!(
        decode::<Driver>(Some((9, 2, 0)), json!("bogus")).unwrap_err(),
        "unknown variant `bogus`, expected `dep` or `qcow2`"
    );
    assert_eq!(
        decode::<Driver>(Some((2, 5, 0)), json!("bogus")).unwrap_err(),
        "unknown variant `bogus`, expected one of `archipelago`, `dep`, `qcow2`"
    );
    assert_eq!(
        decode::<Driver>(Some((1, 9, 0)), json!("qcow2")).unwrap_err(),
        "Driver is not available in QEMU 1.9.0"
    );
    // Unscoped decoding reads recorded values of any release, whatever the
    // default version.
    for decoded in [
        decode(None, json!("archipelago")),
        with_default((9, 2, 0), || decode(None, json!("archipelago"))),
    ] {
        assert!(matches!(decoded, Ok(Driver::Archipelago)));
    }
    assert_eq!(
        decode::<Driver>(None, json!("bogus")).unwrap_err(),
        "unknown variant `bogus`, expected one of `archipelago`, `dep`, `qcow2`"
    );
}

#[test]
fn a_wire_name_decodes_into_the_variant_of_the_version() {
    assert!(matches!(
        decode(Some((2, 12, 0)), json!("fast")),
        Ok(Mode::FastOld)
    ));
    assert!(matches!(
        decode(Some((3, 0, 0)), json!("fast")),
        Ok(Mode::Fast)
    ));
    assert!(matches!(decode(None, json!("fast")), Ok(Mode::Fast)));
    assert!(matches!(
        with_default((2, 0, 0), || decode(None, json!("fast"))),
        Ok(Mode::Fast)
    ));
    assert_eq!(encode((2, 12, 0), &Mode::FastOld).unwrap(), r#""fast""#);
    assert_eq!(
        encode((3, 0, 0), &Mode::FastOld).unwrap_err(),
        "Mode::FastOld is not available in QEMU 3.0.0"
    );
}

#[test]
fn deprecated_variants_warn_once_from_their_deprecation() {
    if child::is_child() {
        encode((9, 1, 0), &Driver::Dep).unwrap();
        eprintln!("from the deprecation");
        for _ in 0..2 {
            encode((9, 2, 0), &Driver::Dep).unwrap();
        }
        return;
    }
    let stderr = child::stderr_of("deprecated_variants_warn_once_from_their_deprecation");
    let warning = "qapi warning: Driver::Dep is deprecated since QEMU 9.2.0";
    assert_eq!(stderr.matches(warning).count(), 1, "{stderr}");
    let (before, after) = stderr.split_once("from the deprecation").unwrap();
    assert!(
        !before.contains(warning) && after.contains(warning),
        "{stderr}"
    );
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct Payload {
    text: String,
}

#[qapi(name = "StrOrNull")]
enum StrOrNull {
    S(String),
    N,
}

#[test]
fn alternates_pass_their_qapi_name_and_payload_to_serde() {
    assert_eq!(
        record::record(&StrOrNull::S("x".into())).unwrap(),
        "newtype_struct(StrOrNull)"
    );
    assert_eq!(
        record::record(&Alternate::Number(1)).unwrap(),
        "newtype_struct(Alternate)"
    );
    assert_eq!(encode((9, 2, 0), &StrOrNull::N).unwrap(), "null");
    // A union branch has no name of its own and serializes as its payload.
    assert_eq!(record::record(&Branch::U).unwrap(), "unit");
}

#[qapi]
enum Alternate {
    Record(Payload),
    Text(String),
    Number(i64),
    Absent,
}

#[test]
fn alternates_take_the_narrowest_variant_that_accepts_the_value() {
    for (wire, matches) in [
        (json!(3), "Number"),
        (json!("x"), "Text"),
        (json!({"text": "a"}), "Record"),
        (json!(null), "Absent"),
    ] {
        let value: Alternate = decode(Some((9, 2, 0)), wire.clone()).unwrap();
        let found = match value {
            Alternate::Record(_) => "Record",
            Alternate::Text(_) => "Text",
            Alternate::Number(_) => "Number",
            Alternate::Absent => "Absent",
        };
        assert_eq!(found, matches);
        assert_eq!(encode((9, 2, 0), &value).unwrap(), wire.to_string());
    }
    let error = decode::<Alternate>(Some((9, 2, 0)), json!([1])).unwrap_err();
    assert!(
        error.starts_with(
            "no variant of Alternate accepts [1] in QEMU 9.2.0 (Absent: expected null; Number: "
        ),
        "{error}"
    );
    assert!(
        error.contains("; Text: ") && error.contains("; Record: "),
        "{error}"
    );
    let error = decode::<Alternate>(None, json!([1])).unwrap_err();
    assert!(
        error.starts_with("no variant of Alternate accepts [1] (Absent"),
        "{error}"
    );
}

#[qapi]
enum GatedAlternate {
    #[qapi(added = "5.0.0")]
    Text(String),
    Number(f64),
}

#[test]
fn an_alternate_names_a_variant_that_the_scoped_version_lacks() {
    assert_eq!(
        decode::<GatedAlternate>(Some((4, 2, 0)), json!("x")).unwrap_err(),
        "GatedAlternate::Text is not available in QEMU 4.2.0"
    );
    for version in [Some((5, 0, 0)), None] {
        assert!(matches!(
            decode(version, json!("x")),
            Ok(GatedAlternate::Text(_))
        ));
    }
    assert_eq!(
        encode((4, 2, 0), &GatedAlternate::Text("x".into())).unwrap_err(),
        "GatedAlternate::Text is not available in QEMU 4.2.0"
    );
    assert_eq!(take_unavailable(), ["GatedAlternate::Text"]);
}

#[qapi]
enum Numeric {
    #[qapi(removed = "2.10.0")]
    Integer(i8),
    Floating(f64),
}

#[test]
fn an_integer_goes_to_a_float_only_without_an_integer_variant() {
    assert!(matches!(
        decode(Some((2, 9, 0)), json!(42)),
        Ok(Numeric::Integer(42))
    ));
    let error = decode::<Numeric>(Some((2, 9, 0)), json!(128)).unwrap_err();
    assert!(
        error.starts_with("no variant of Numeric accepts 128 in QEMU 2.9.0 (Integer: "),
        "{error}"
    );
    assert!(matches!(
        decode(Some((2, 9, 0)), json!(42.5)),
        Ok(Numeric::Floating(_))
    ));
    assert!(matches!(
        decode(Some((2, 10, 0)), json!(128)),
        Ok(Numeric::Floating(_))
    ));
    // Unscoped decoding stays lenient.
    assert!(matches!(decode(None, json!(128)), Ok(Numeric::Floating(_))));
}

/// `is_union_branch()` is the inherent method, `true`, exactly when `T`
/// implements `QapiFlatUnion`; otherwise the trait method, `false`.
struct Probe<T>(std::marker::PhantomData<T>);

trait NotUnionBranch {
    fn is_union_branch(&self) -> bool {
        false
    }
}

impl<T> NotUnionBranch for Probe<T> {}

impl<T: QapiFlatUnion> Probe<T> {
    fn is_union_branch(&self) -> bool {
        true
    }
}

#[test]
fn only_union_branch_enums_dispatch_by_wire_tag() {
    assert!(Probe::<Branch>(std::marker::PhantomData).is_union_branch());
    assert!(!Probe::<Alternate>(std::marker::PhantomData).is_union_branch());
    assert!(!Probe::<Driver>(std::marker::PhantomData).is_union_branch());
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct Old {
    old: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct New {
    new: u32,
}

#[qapi(union_branch)]
enum Branch {
    #[qapi(name = "a")]
    A(Payload),
    #[qapi(name = "b", added = "5.0.0")]
    B(Payload),
    #[qapi(name = "c", removed = "3.0.0")]
    COld(Old),
    #[qapi(name = "c", added = "3.0.0")]
    C(New),
    #[qapi(name = "u")]
    U,
}

#[qapi(union_branch, added = "2.0.0")]
enum LateBranch {
    #[qapi(name = "x")]
    X(Payload),
}

fn branch<T: QapiFlatUnion>(
    version: Option<(u32, u32, u32)>,
    tag: &str,
    payload: serde_json::Value,
) -> Result<T, String> {
    let decode = || T::deserialize_variant_value(tag, payload).map_err(|error| error.to_string());
    match version {
        Some(version) => with_version(version, decode),
        None => decode(),
    }
}

#[test]
fn union_branches_dispatch_by_tag_and_version() {
    let text = json!({"text": "t"});
    assert_eq!(
        Branch::A(Payload { text: "t".into() }).wire_tag(),
        Some("a")
    );
    assert_eq!(Branch::U.wire_tag(), Some("u"));
    assert_eq!(Branch::COld(Old { old: 1 }).wire_tag(), Some("c"));
    assert!(matches!(
        branch(Some((5, 0, 0)), "b", text.clone()),
        Ok(Branch::B(_))
    ));
    assert_eq!(
        branch::<Branch>(Some((4, 2, 0)), "b", text.clone()).unwrap_err(),
        "Branch::B is not available in QEMU 4.2.0"
    );
    assert!(matches!(
        branch(Some((2, 12, 0)), "c", json!({"old": 1})),
        Ok(Branch::COld(Old { old: 1 }))
    ));
    assert!(matches!(
        branch(Some((3, 0, 0)), "c", json!({"new": 2})),
        Ok(Branch::C(New { new: 2 }))
    ));
    assert!(matches!(
        branch(None, "c", json!({"new": 2})),
        Ok(Branch::C(_))
    ));
    assert!(matches!(
        branch(Some((9, 2, 0)), "u", json!({})),
        Ok(Branch::U)
    ));
    assert_eq!(
        branch::<Branch>(Some((4, 2, 0)), "bogus", json!({})).unwrap_err(),
        "unknown Branch discriminator `bogus`, expected one of `a`, `c`, `u`"
    );
    let error = branch::<Branch>(Some((9, 2, 0)), "a", json!({})).unwrap_err();
    assert_eq!(error, "Branch::A: missing field `text`");
    let map = json!({"text": "t"}).as_object().unwrap().clone();
    assert!(matches!(
        with_version((9, 2, 0), || Branch::deserialize_variant("a", map)),
        Ok(Branch::A(_))
    ));
}

#[test]
fn a_union_branch_type_outside_its_lifetime_is_refused_in_a_scope() {
    let text = json!({"text": "t"});
    assert_eq!(
        branch::<LateBranch>(Some((1, 0, 0)), "x", text.clone()).unwrap_err(),
        "LateBranch is not available in QEMU 1.0.0"
    );
    assert_eq!(
        decode::<LateBranch>(Some((1, 0, 0)), text.clone()).unwrap_err(),
        "LateBranch is not available in QEMU 1.0.0"
    );
    assert!(branch::<LateBranch>(Some((2, 0, 0)), "x", text.clone()).is_ok());
    assert!(branch::<LateBranch>(None, "x", text).is_ok());
}

#[qapi]
enum NoVariants {}

#[qapi(union_branch)]
enum NoBranches {}

#[test]
fn empty_enums_have_no_names_to_decode() {
    assert_eq!(
        decode::<NoVariants>(None, json!("x")).unwrap_err(),
        "unknown variant `x`, there are no variants"
    );
    assert_eq!(
        branch::<NoBranches>(None, "x", json!({})).unwrap_err(),
        "unknown NoBranches discriminator `x`, there are no variants"
    );
}

#[allow(non_camel_case_types)]
#[qapi]
enum r#match {
    r#type,
    #[qapi(added = "2.0.0")]
    r#loop,
}

#[test]
fn raw_identifiers_are_unraw_in_names_and_errors() {
    assert_eq!(r#match::r#type.to_string(), "type");
    assert_eq!(
        encode((1, 0, 0), &r#match::r#loop).unwrap_err(),
        "match::loop is not available in QEMU 1.0.0"
    );
}

/// Items in scope must not capture the expansion's own names.
#[allow(non_upper_case_globals, dead_code)]
mod shadowed {
    use super::{Payload, qapi};

    const other: u8 = 0;
    const value: u8 = 0;
    const inner: u8 = 0;
    const error: u8 = 0;
    const errors: u8 = 0;
    const message: u8 = 0;
    const version: u8 = 0;
    const wire: u8 = 0;
    const selected: u8 = 0;
    const discriminator: u8 = 0;
    const map: u8 = 0;
    const formatter: u8 = 0;
    const f: u8 = 0;
    const serializer: u8 = 0;
    const deserializer: u8 = 0;
    struct Serializer;
    struct Deserializer;
    type Result = ();
    type Option = ();

    #[qapi]
    pub enum Kind {
        #[qapi(name = "one")]
        One,
        #[qapi(name = "two", deprecated(added = "1.0.0"))]
        Two,
    }

    #[qapi]
    pub enum Alternate {
        Number(i64),
        Text(::std::string::String),
        Record(Payload),
    }

    #[qapi(union_branch)]
    pub enum Branch {
        #[qapi(name = "one")]
        One(Payload),
    }
}

#[test]
fn the_expansion_does_not_depend_on_names_in_scope() {
    assert_eq!(shadowed::Kind::One.to_string(), "one");
    assert!(matches!(
        decode(Some((1, 0, 0)), json!("two")),
        Ok(shadowed::Kind::Two)
    ));
    assert_eq!(
        decode::<shadowed::Kind>(Some((1, 0, 0)), json!("three")).unwrap_err(),
        "unknown variant `three`, expected `one` or `two`"
    );
    for wire in [json!(3), json!("text"), json!({"text": "record"})] {
        let value: shadowed::Alternate = decode(Some((1, 0, 0)), wire.clone()).unwrap();
        assert_eq!(encode((1, 0, 0), &value).unwrap(), wire.to_string());
    }
    assert!(matches!(
        branch(Some((1, 0, 0)), "one", json!({"text": "t"})),
        Ok(shadowed::Branch::One(_))
    ));
}
