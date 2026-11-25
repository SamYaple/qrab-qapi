//! The expansion of structs, compiled and run against the stand-in runtime
//! in `qapi/`: members and their lifetimes, version alternates, flattened
//! bases, unions in both layouts, and `Display`.

mod child;
mod qapi;
mod record;

use qapi::{take_unavailable, with_default, with_version};
use qapi_macros::qapi;
use serde_json::{Value, json};
use std::collections::HashMap;

#[qapi(name = "image-info")]
struct ImageInfo {
    filename: String,
    #[qapi(name = "virtual-size")]
    virtual_size: i64,
    #[qapi(added = "10.2.0")]
    limits: Option<u32>,
    #[qapi(removed = "9.0.0")]
    legacy: Option<bool>,
    #[qapi(added = "8.0.0", removed = "8.5.0")]
    window: Option<u8>,
    #[qapi(deprecated(added = "9.2.0"))]
    dep: Option<String>,
}

fn image() -> ImageInfo {
    ImageInfo {
        filename: "f".into(),
        virtual_size: 1,
        limits: None,
        legacy: None,
        window: None,
        dep: None,
    }
}

fn encode<T: serde::Serialize>(version: (u32, u32, u32), value: &T) -> Result<Value, String> {
    with_version(version, || serde_json::to_value(value)).map_err(|error| error.to_string())
}

fn decode<T: serde::de::DeserializeOwned>(
    version: Option<(u32, u32, u32)>,
    json: Value,
) -> Result<T, String> {
    let decode = || serde_json::from_value(json).map_err(|error| error.to_string());
    match version {
        Some(version) => with_version(version, decode),
        None => decode(),
    }
}

#[test]
fn members_serialize_under_their_wire_names() {
    assert_eq!(
        encode((10, 2, 0), &image()).unwrap(),
        json!({"filename": "f", "virtual-size": 1})
    );
    let recorded = with_version((10, 2, 0), || record::record(&image())).unwrap();
    assert_eq!(recorded, "struct(image-info, 2) filename virtual-size");
    let mut info = image();
    info.limits = Some(2);
    let recorded = with_version((10, 2, 0), || record::record(&info)).unwrap();
    assert_eq!(
        recorded,
        "struct(image-info, 3) filename virtual-size limits"
    );
    assert_eq!(encode((10, 2, 0), &info).unwrap()["limits"], 2);
}

/// Serialize `info` for each version, which may or may not have `name`.
fn check_lifetime(info: &ImageInfo, name: &str, versions: &[((u32, u32, u32), bool)]) {
    for &(version, available) in versions {
        let result = encode(version, info);
        if available {
            assert!(result.is_ok(), "{name} {version:?}: {result:?}");
        } else {
            let (major, minor, patch) = version;
            assert_eq!(
                result.unwrap_err(),
                format!("{name} is not available in QEMU {major}.{minor}.{patch}")
            );
            assert_eq!(take_unavailable(), [name]);
        }
    }
}

#[test]
fn a_member_set_outside_its_lifetime_is_unavailable() {
    let mut info = image();
    info.limits = Some(2);
    check_lifetime(
        &info,
        "ImageInfo.limits",
        &[((10, 1, 9), false), ((10, 2, 0), true)],
    );
    let mut info = image();
    info.legacy = Some(true);
    check_lifetime(
        &info,
        "ImageInfo.legacy",
        &[((8, 9, 9), true), ((9, 0, 0), false)],
    );
    let mut info = image();
    info.window = Some(1);
    check_lifetime(
        &info,
        "ImageInfo.window",
        &[
            ((7, 0, 0), false),
            ((8, 0, 0), true),
            ((8, 4, 9), true),
            ((8, 5, 0), false),
        ],
    );
    // Outside any scope the default version applies.
    let mut info = image();
    info.limits = Some(2);
    let unscoped = with_default((10, 1, 0), || serde_json::to_value(&info));
    assert_eq!(
        unscoped.unwrap_err().to_string(),
        "ImageInfo.limits is not available in QEMU 10.1.0"
    );
}

#[test]
fn deprecated_members_that_are_set_warn_once_from_their_deprecation() {
    if child::is_child() {
        let mut info = image();
        encode((9, 2, 0), &info).unwrap();
        info.dep = Some("d".into());
        encode((9, 1, 0), &info).unwrap();
        eprintln!("from the deprecation");
        for _ in 0..2 {
            encode((9, 2, 0), &info).unwrap();
        }
        return;
    }
    let stderr =
        child::stderr_of("deprecated_members_that_are_set_warn_once_from_their_deprecation");
    let warning = "qapi warning: ImageInfo.dep is deprecated since QEMU 9.2.0";
    assert_eq!(stderr.matches(warning).count(), 1, "{stderr}");
    let (before, after) = stderr.split_once("from the deprecation").unwrap();
    assert!(
        !before.contains(warning) && after.contains(warning),
        "{stderr}"
    );
}

#[test]
fn decoding_ignores_unknown_members_and_requires_the_others() {
    let info: ImageInfo = decode(
        Some((10, 2, 0)),
        json!({"filename": "f", "virtual-size": 1, "limits": 3, "unknown": [1]}),
    )
    .unwrap();
    assert_eq!(
        (info.filename.as_str(), info.virtual_size, info.limits),
        ("f", 1, Some(3))
    );
    assert_eq!(
        decode::<ImageInfo>(None, json!({"filename": "f"})).unwrap_err(),
        "ImageInfo: missing field `virtual-size`"
    );
    assert_eq!(
        decode::<ImageInfo>(None, json!(3)).unwrap_err(),
        "invalid type: integer `3`, expected ImageInfo"
    );
}

#[qapi]
struct Required {
    #[qapi(added = "2.0.0", required)]
    value: Option<i64>,
}

#[test]
fn members_that_a_version_requires_are_checked_in_its_scope() {
    assert_eq!(
        encode((2, 0, 0), &Required { value: None }).unwrap_err(),
        "Required: missing required field `value`"
    );
    assert_eq!(
        encode((1, 9, 9), &Required { value: None }).unwrap(),
        json!({})
    );
    assert_eq!(
        decode::<Required>(Some((2, 0, 0)), json!({})).unwrap_err(),
        "Required: missing required field `value`"
    );
    assert!(decode::<Required>(Some((1, 9, 9)), json!({})).is_ok());
    // Unscoped decoding stays lenient, whatever the default.
    assert!(decode::<Required>(None, json!({})).is_ok());
    assert!(with_default((2, 0, 0), || decode::<Required>(None, json!({}))).is_ok());
}

#[qapi(added = "5.0.0")]
struct Late {
    a: u8,
}

#[test]
fn a_struct_outside_its_lifetime_is_refused() {
    assert_eq!(
        encode((4, 0, 0), &Late { a: 1 }).unwrap_err(),
        "Late is not available in QEMU 4.0.0"
    );
    assert_eq!(take_unavailable(), ["Late"]);
    assert_eq!(
        decode::<Late>(Some((4, 0, 0)), json!({"a": 1})).unwrap_err(),
        "Late is not available in QEMU 4.0.0"
    );
    assert!(decode::<Late>(None, json!({"a": 1})).is_ok());
}

/// A member whose type changed, and so its version alternates.
#[qapi]
struct Shared {
    #[qapi(name = "value", added = "3.0.0")]
    value: Option<String>,
    #[qapi(name = "value", removed = "2.0.0")]
    value_legacy: Option<i64>,
    v: Option<u8>,
}

#[test]
fn a_shared_wire_name_decodes_into_the_alternate_of_the_version() {
    let shared: Shared = decode(Some((1, 0, 0)), json!({"value": 3, "v": 1})).unwrap();
    assert_eq!(
        (shared.value, shared.value_legacy, shared.v),
        (None, Some(3), Some(1))
    );
    for version in [Some((3, 0, 0)), None] {
        let shared: Shared = decode(version, json!({"value": "x"})).unwrap();
        assert_eq!(
            (shared.value.as_deref(), shared.value_legacy),
            (Some("x"), None)
        );
    }
    assert_eq!(
        decode::<Shared>(Some((3, 0, 0)), json!({"value": 3})).unwrap_err(),
        "Shared.value: invalid type: integer `3`, expected a string"
    );
    assert_eq!(
        decode::<Shared>(Some((2, 5, 0)), json!({"value": "x"})).unwrap_err(),
        "Shared.value is not available in QEMU 2.5.0"
    );
    assert!(decode::<Shared>(Some((2, 5, 0)), json!({})).is_ok());
    let legacy = Shared {
        value: None,
        value_legacy: Some(1),
        v: None,
    };
    assert_eq!(encode((1, 0, 0), &legacy).unwrap(), json!({"value": 1}));
    assert_eq!(
        encode((3, 0, 0), &legacy).unwrap_err(),
        "Shared.value_legacy is not available in QEMU 3.0.0"
    );
}

#[qapi]
struct Base {
    id: String,
    #[qapi(added = "3.0.0")]
    tag: Option<String>,
}

#[qapi]
enum Kind {
    #[qapi(name = "file")]
    File,
    #[qapi(name = "nbd")]
    Nbd,
    #[qapi(name = "old", removed = "2.0.0")]
    Old,
}

#[qapi]
struct FileOptions {
    path: String,
}

#[qapi]
struct NbdOptions {
    host: String,
}

#[qapi(union_branch)]
enum Branch {
    #[qapi(name = "file")]
    File(FileOptions),
    #[qapi(name = "nbd")]
    Nbd(NbdOptions),
    #[qapi(name = "old", removed = "2.0.0")]
    Old,
}

#[qapi(simple_union(removed = "2.10.0"))]
struct Options {
    #[qapi(flatten)]
    base: Base,
    #[qapi(name = "type", discriminator)]
    kind: Kind,
    #[qapi(flatten, union)]
    u: Option<Branch>,
}

fn file_options() -> Options {
    Options {
        base: Base {
            id: "a".into(),
            tag: None,
        },
        kind: Kind::File,
        u: Some(Branch::File(FileOptions { path: "/p".into() })),
    }
}

#[test]
fn unions_take_the_layout_of_the_version() {
    let flat = json!({"id": "a", "type": "file", "path": "/p"});
    let nested = json!({"id": "a", "type": "file", "data": {"path": "/p"}});
    for (version, wire) in [
        (Some((9, 2, 0)), &flat),
        (Some((2, 9, 0)), &nested),
        (None, &flat),
    ] {
        let options: Options = decode(version, wire.clone()).unwrap();
        assert_eq!(options.base.id, "a");
        assert!(matches!(options.kind, Kind::File));
        assert!(matches!(options.u, Some(Branch::File(ref file)) if file.path == "/p"));
        if let Some(version) = version {
            assert_eq!(&encode(version, &options).unwrap(), wire);
        }
    }
    // Unscoped decoding takes the newest layout, whatever the default.
    for decoded in [
        decode::<Options>(None, nested.clone()),
        with_default((2, 9, 0), || decode::<Options>(None, nested.clone())),
    ] {
        assert_eq!(
            decoded.unwrap_err(),
            "Branch::File: FileOptions: missing field `path`"
        );
    }
    // In a release of the simple layout, the payload is under `data`.
    for wire in [json!({"id": "a", "type": "file"}), flat.clone()] {
        assert_eq!(
            decode::<Options>(Some((2, 9, 0)), wire).unwrap_err(),
            "Options: missing field `data`"
        );
    }
    let mut unset = file_options();
    unset.u = None;
    assert_eq!(
        encode((2, 9, 0), &unset).unwrap_err(),
        "Options: missing simple union payload"
    );
    assert_eq!(
        encode((9, 2, 0), &unset).unwrap(),
        json!({"id": "a", "type": "file"})
    );
    let old: Options = decode(
        Some((1, 0, 0)),
        json!({"id": "a", "type": "old", "data": {}}),
    )
    .unwrap();
    assert!(matches!(old.u, Some(Branch::Old)));
}

#[test]
fn unions_refuse_a_payload_of_another_branch() {
    let mut options = file_options();
    options.kind = Kind::Nbd;
    assert_eq!(
        encode((9, 2, 0), &options).unwrap_err(),
        "Options: discriminator `type` is `nbd`, but the payload is branch `file`"
    );
}

#[test]
fn flattened_decoding_keeps_the_context_of_its_errors() {
    assert_eq!(
        decode::<Options>(None, json!(3)).unwrap_err(),
        "Options: expected an object, found 3"
    );
    assert_eq!(
        decode::<Options>(Some((9, 2, 0)), json!({"id": "a", "type": 5})).unwrap_err(),
        "Options.type: invalid type: integer `5`, expected a string"
    );
    assert_eq!(
        decode::<Options>(Some((9, 2, 0)), json!({"id": "a", "type": "old"})).unwrap_err(),
        "Options.type: Kind::Old is not available in QEMU 9.2.0"
    );
    assert_eq!(
        decode::<Options>(Some((9, 2, 0)), json!({"type": "file", "path": "/p"})).unwrap_err(),
        "Options: Base: missing field `id`"
    );
    assert_eq!(
        decode::<Options>(Some((9, 2, 0)), json!({"id": "a"})).unwrap_err(),
        "Options: missing field `type`"
    );
    assert_eq!(
        decode::<DriverOptions>(None, json!({"path": "/p"})).unwrap_err(),
        "DriverOptions: missing field `driver`"
    );
}

#[qapi]
enum Target {
    #[qapi(name = "x86_64")]
    X86_64,
    #[qapi(name = "s390x")]
    S390x,
}

#[qapi(union_branch)]
enum CpuBranch {
    #[qapi(name = "x86", removed = "3.0.0")]
    X86,
    #[qapi(name = "x86_64", added = "3.0.0")]
    X86_64,
    #[qapi(name = "s390x", added = "3.0.0")]
    S390x(NbdOptions),
}

#[qapi]
enum Arch {
    #[qapi(name = "x86")]
    X86,
}

/// `CpuInfoFast`, whose discriminator is `arch` in 2.12 and `target` from
/// 3.0, which keeps `arch` until 6.0.
#[qapi(added = "2.12.0")]
struct Cpu {
    #[qapi(discriminator(removed = "3.0.0"), removed = "6.0.0")]
    arch: Option<Arch>,
    #[qapi(discriminator, added = "3.0.0")]
    target: Option<Target>,
    #[qapi(flatten, union)]
    u: Option<CpuBranch>,
}

#[test]
fn each_release_has_the_discriminator_of_its_range() {
    for version in [Some((5, 2, 0)), None] {
        let cpu: Cpu = decode(
            version,
            json!({"arch": "x86", "target": "s390x", "host": "h"}),
        )
        .unwrap();
        assert!(matches!(cpu.u, Some(CpuBranch::S390x(ref nbd)) if nbd.host == "h"));
        assert!(matches!(cpu.arch, Some(Arch::X86)));
        assert_eq!(
            encode((5, 2, 0), &cpu).unwrap(),
            json!({"arch": "x86", "target": "s390x", "host": "h"})
        );
    }
    let cpu: Cpu = decode(Some((2, 12, 0)), json!({"arch": "x86"})).unwrap();
    assert!(matches!(cpu.u, Some(CpuBranch::X86)));
    // From 3.0 arch is a member only.
    for version in [Some((5, 2, 0)), None] {
        let cpu: Cpu = decode(version, json!({"arch": "x86"})).unwrap();
        assert!(cpu.u.is_none() && cpu.arch.is_some());
    }
    assert_eq!(encode((2, 12, 0), &cpu).unwrap(), json!({"arch": "x86"}));
    let mismatch = Cpu {
        arch: Some(Arch::X86),
        target: Some(Target::X86_64),
        u: Some(CpuBranch::S390x(NbdOptions { host: "h".into() })),
    };
    assert_eq!(
        encode((5, 2, 0), &mismatch).unwrap_err(),
        "Cpu: discriminator `target` is `x86_64`, but the payload is branch `s390x`"
    );
}

/// A discriminator in two runs of releases, around the one of another, and
/// one that stays a member, without a lifetime, once another takes over.
#[qapi(added = "2.12.0")]
struct Handover {
    #[qapi(discriminator(removed = "3.0.0"), discriminator(added = "5.0.0"))]
    arch: Option<Arch>,
    #[qapi(discriminator(added = "3.0.0", removed = "5.0.0"))]
    target: Option<Target>,
    #[qapi(flatten, union)]
    u: Option<CpuBranch>,
}

#[qapi(added = "2.12.0")]
struct Kept {
    #[qapi(discriminator(removed = "3.0.0"))]
    arch: Arch,
    #[qapi(discriminator, added = "3.0.0")]
    target: Option<Target>,
    #[qapi(flatten, union)]
    u: Option<CpuBranch>,
}

#[test]
fn a_discriminator_selects_the_branch_in_each_of_its_runs() {
    // Unscoped decoding takes the discriminator of the newest release.
    let wire = json!({"arch": "x86", "target": "s390x", "host": "h"});
    for version in [Some((2, 12, 0)), None] {
        let handover: Handover = decode(version, wire.clone()).unwrap();
        assert!(matches!(handover.u, Some(CpuBranch::X86)), "{version:?}");
    }
    let handover: Handover = decode(Some((4, 0, 0)), wire.clone()).unwrap();
    assert!(matches!(handover.u, Some(CpuBranch::S390x(_))));
    // Outside its run, a member that is no Option is no discriminator.
    let kept = Kept {
        arch: Arch::X86,
        target: Some(Target::S390x),
        u: Some(CpuBranch::S390x(NbdOptions { host: "h".into() })),
    };
    assert_eq!(encode((5, 2, 0), &kept).unwrap(), wire);
    let kept: Kept = decode(Some((5, 2, 0)), wire).unwrap();
    assert!(matches!(kept.u, Some(CpuBranch::S390x(_))));
}

#[qapi]
struct DriverOptions {
    #[qapi(discriminator)]
    driver: Kind,
    #[qapi(flatten, union)]
    u: Option<Branch>,
}

/// A union field without `Option`, whose member discriminator every
/// release sends.
#[qapi]
struct PlainUnion {
    #[qapi(discriminator)]
    driver: Kind,
    #[qapi(flatten, union)]
    u: Branch,
}

#[test]
fn a_union_without_option_needs_its_discriminator() {
    let wire = json!({"driver": "file", "path": "/p"});
    let required: PlainUnion = decode(Some((9, 2, 0)), wire.clone()).unwrap();
    assert!(matches!(required.u, Branch::File(ref file) if file.path == "/p"));
    assert_eq!(encode((9, 2, 0), &required).unwrap(), wire);
    assert_eq!(
        decode::<PlainUnion>(None, json!({"path": "/p"})).unwrap_err(),
        "PlainUnion: missing field `driver`"
    );
}

#[qapi]
struct FlatScalar {
    #[qapi(flatten)]
    n: u32,
}

#[test]
fn a_flattened_payload_serializes_to_an_object() {
    assert_eq!(
        encode((9, 2, 0), &FlatScalar { n: 1 }).unwrap_err(),
        "FlatScalar.n: a flattened payload must serialize to an object"
    );
}

#[qapi]
struct Catchall {
    id: String,
    size: Option<u32>,
    #[qapi(flatten)]
    arguments: Option<HashMap<String, Value>>,
}

#[test]
fn a_flattened_map_takes_the_unclaimed_members_and_claims_none_twice() {
    let catchall: Catchall = decode(None, json!({"id": "a", "size": 2, "x": 1})).unwrap();
    assert_eq!(catchall.size, Some(2));
    assert_eq!(
        catchall.arguments.unwrap(),
        HashMap::from([("x".into(), json!(1))])
    );
    let mut catchall = Catchall {
        id: "a".into(),
        size: None,
        arguments: Some(HashMap::from([("x".into(), json!(1))])),
    };
    assert_eq!(
        encode((9, 2, 0), &catchall).unwrap(),
        json!({"id": "a", "x": 1})
    );
    catchall.arguments = Some(HashMap::from([("id".into(), json!("b"))]));
    assert_eq!(
        encode((9, 2, 0), &catchall).unwrap_err(),
        "Catchall: duplicate member `id`"
    );
}

#[qapi]
struct GatedFlat {
    id: String,
    #[qapi(flatten, added = "5.0.0")]
    extra: Option<HashMap<String, Value>>,
    #[qapi(flatten, removed = "5.0.0")]
    old: Option<HashMap<String, Value>>,
}

#[test]
fn flattened_fields_follow_their_lifetime() {
    let wire = json!({"id": "a", "x": 1});
    let new: GatedFlat = decode(Some((5, 0, 0)), wire.clone()).unwrap();
    assert!(new.extra.is_some() && new.old.is_none());
    let old: GatedFlat = decode(Some((4, 0, 0)), wire.clone()).unwrap();
    assert!(old.extra.is_none() && old.old.is_some());
    let unscoped: GatedFlat = decode(None, wire).unwrap();
    assert!(unscoped.extra.is_some() && unscoped.old.is_none());
    assert_eq!(
        encode((4, 0, 0), &new).unwrap_err(),
        "GatedFlat.extra is not available in QEMU 4.0.0"
    );
    assert_eq!(take_unavailable(), ["GatedFlat.extra"]);
    assert_eq!(encode((4, 0, 0), &old).unwrap(), json!({"id": "a", "x": 1}));
}

#[qapi]
enum NewKind {
    #[qapi(name = "a")]
    A,
}

#[qapi]
enum OldKind {
    #[qapi(name = "a")]
    A,
}

#[qapi(union_branch)]
enum AltBranch {
    #[qapi(name = "a")]
    A(FileOptions),
}

#[qapi]
struct AlternateDiscriminator {
    #[qapi(name = "type", discriminator, added = "2.0.0")]
    kind: Option<NewKind>,
    #[qapi(name = "type", discriminator, removed = "2.0.0")]
    kind_legacy: Option<OldKind>,
    #[qapi(flatten, union)]
    u: Option<AltBranch>,
}

#[qapi]
struct OptionalDiscriminator {
    #[qapi(discriminator)]
    kind: Option<NewKind>,
    #[qapi(flatten, union)]
    u: Option<AltBranch>,
}

#[test]
fn union_branches_follow_an_alternate_or_optional_discriminator() {
    let wire = json!({"type": "a", "path": "/p"});
    let old: AlternateDiscriminator = decode(Some((1, 0, 0)), wire.clone()).unwrap();
    assert!(old.kind.is_none() && old.kind_legacy.is_some() && old.u.is_some());
    let new: AlternateDiscriminator = decode(Some((2, 0, 0)), wire.clone()).unwrap();
    assert!(new.kind.is_some() && new.kind_legacy.is_none() && new.u.is_some());
    assert_eq!(encode((1, 0, 0), &old).unwrap(), wire);
    let absent: OptionalDiscriminator = decode(None, json!({})).unwrap();
    assert!(absent.u.is_none());
    let present: OptionalDiscriminator = decode(None, json!({"kind": "a", "path": "/p"})).unwrap();
    assert!(matches!(present.u, Some(AltBranch::A(_))));
}

#[qapi]
struct Empty {}

#[qapi]
struct Unit;

#[test]
fn display_writes_the_qapi_name_and_the_fields_that_are_set() {
    let mut info = image();
    info.limits = Some(2);
    assert_eq!(
        info.to_string(),
        r#"image-info(filename="f", virtual-size=1, limits=2)"#
    );
    assert_eq!(Empty {}.to_string(), "Empty");
    assert_eq!(Unit.to_string(), "Unit");
    assert_eq!(
        file_options().to_string(),
        r#"Options(base=Base { id: "a", tag: None }, type=File, u=File(FileOptions { path: "/p" }))"#
    );
    assert_eq!(encode((1, 0, 0), &Unit).unwrap(), json!({}));
    assert!(decode::<Unit>(None, json!({"x": 1})).is_ok());
}

#[allow(non_camel_case_types)]
#[qapi]
struct r#match {
    r#type: String,
    #[qapi(added = "2.0.0")]
    r#loop: Option<u8>,
}

#[test]
fn raw_identifiers_are_unraw_in_names_and_errors() {
    let value = r#match {
        r#type: "t".into(),
        r#loop: Some(1),
    };
    assert_eq!(value.to_string(), r#"match(type="t", loop=1)"#);
    assert_eq!(
        encode((1, 0, 0), &value).unwrap_err(),
        "match.loop is not available in QEMU 1.0.0"
    );
    assert_eq!(
        decode::<r#match>(None, json!(1)).unwrap_err(),
        "invalid type: integer `1`, expected match"
    );
}

/// Items in scope, and fields with the names of the expansion's locals,
/// must not capture them.
#[allow(non_upper_case_globals, dead_code)]
mod shadowed {
    use super::{Kind, qapi};

    const value: u8 = 0;
    const key: u8 = 0;
    const state: u8 = 0;
    const keys: u8 = 0;
    const flat: u8 = 0;
    const member: u8 = 0;
    const result: u8 = 0;
    const raw_value: u8 = 0;
    const error: u8 = 0;
    const access: u8 = 0;
    const formatter: u8 = 0;
    const separator: u8 = 0;
    const payload: u8 = 0;
    const tag: u8 = 0;
    const other: u8 = 0;
    const d: u8 = 0;
    struct Visitor;
    struct MapAccess;
    struct Serializer;
    struct Deserializer;
    type Result = ();
    type Option = ();

    #[qapi]
    pub struct Payload {
        pub text: ::std::string::String,
    }

    #[qapi(union_branch)]
    pub enum Branch {
        #[qapi(name = "file")]
        File(Payload),
    }

    #[qapi]
    pub struct Fields {
        pub v: ::core::option::Option<i64>,
        pub map: ::core::option::Option<i64>,
        pub serializer: ::core::option::Option<i64>,
        #[qapi(name = "value", added = "2.0.0")]
        pub value_new: ::core::option::Option<::std::string::String>,
        #[qapi(name = "value", removed = "2.0.0")]
        pub value_old: ::core::option::Option<i64>,
    }

    #[qapi(simple_union(removed = "2.0.0"))]
    pub struct Union {
        #[qapi(name = "type", discriminator)]
        pub kind: Kind,
        #[qapi(flatten, union)]
        pub u: ::core::option::Option<Branch>,
        #[qapi(flatten)]
        pub rest: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, super::Value>,
        >,
    }
}

#[test]
fn the_expansion_does_not_depend_on_names_in_scope() {
    let wire = json!({"v": 1, "map": 2, "serializer": 3, "value": "x"});
    let fields: shadowed::Fields = decode(Some((2, 0, 0)), wire.clone()).unwrap();
    assert_eq!(
        (
            fields.v,
            fields.map,
            fields.serializer,
            fields.value_new.as_deref()
        ),
        (Some(1), Some(2), Some(3), Some("x"))
    );
    assert_eq!(encode((2, 0, 0), &fields).unwrap(), wire);
    for (version, wire) in [
        ((1, 0, 0), json!({"type": "file", "data": {"text": "t"}})),
        ((2, 0, 0), json!({"type": "file", "text": "t"})),
    ] {
        let union: shadowed::Union = decode(Some(version), wire.clone()).unwrap();
        assert!(matches!(union.u, Some(shadowed::Branch::File(_))));
        let mut union = union;
        union.rest = None;
        assert_eq!(encode(version, &union).unwrap(), wire);
    }
}
