//! The names that the generator synthesizes: the enums of return types, the
//! inventory of every namespace with its collisions, and the names of
//! historical members when a patch release of an older series joins the
//! selection.
use super::registry_tests::registry;
use super::*;
use crate::namespace::{Pattern, inventory, plan_returns_enums_with};

/// The names of `registry` with `pattern`, as `namespace: name`.
fn names(registry: &UnifiedRegistry, pattern: Pattern) -> Vec<String> {
    inventory(registry, "QMP")
        .unwrap()
        .into_iter()
        .filter(|name| name.pattern == pattern)
        .map(|name| format!("{}: {}", name.namespace, name.name))
        .collect()
}

/// Two releases in which `ping` returns `Alpha`, then `Beta`, with `extra`
/// in both.
fn pings(extra: &str) -> Result<UnifiedRegistry, RegistryError> {
    registry(&[
        (
            "v9.0.0",
            &format!(
                "{{'struct':'Alpha','data':{{'a':'int'}}}} {{'command':'ping','returns':'Alpha'}} {extra}"
            ),
        ),
        (
            "v9.1.0",
            &format!(
                "{{'struct':'Beta','data':{{'b':'str'}}}} {{'command':'ping','returns':'Beta'}} {extra}"
            ),
        ),
    ])
}

#[test]
fn a_command_whose_return_type_changes_has_a_planned_enum() {
    let registry = pings("").unwrap();
    let UnifiedType::Struct(ping) = &registry.types["Ping"] else {
        panic!()
    };
    assert_eq!(ping.returns_enum.as_deref(), Some("PingReturns"));
    assert_eq!(names(&registry, Pattern::Returns), ["items: PingReturns"]);
    assert_eq!(
        names(&registry, Pattern::ReturnsVariant),
        [
            "variants of PingReturns: V9_0_0",
            "variants of PingReturns: V9_1_0"
        ]
    );
    // A command named like its response has the enum of its request type.
    let registry = super::registry_tests::registry(&[
        (
            "v9.0.0",
            "{'struct':'GuestExec','data':{'pid':'int'}}
             {'command':'guest-exec','returns':'GuestExec'}",
        ),
        (
            "v9.1.0",
            "{'struct':'GuestExec','data':{'pid':'int'}}
             {'struct':'GuestExecStatus','data':{'code':'int'}}
             {'command':'guest-exec','returns':'GuestExecStatus'}",
        ),
    ])
    .unwrap();
    assert_eq!(
        names(&registry, Pattern::Returns),
        ["items: GuestExecCommandReturns"]
    );
    // A command with one return type has none.
    let registry = super::registry_tests::registry(&[
        (
            "v9.0.0",
            "{'struct':'Alpha','data':{}} {'command':'ping','returns':'Alpha'}",
        ),
        (
            "v9.1.0",
            "{'struct':'Alpha','data':{}} {'command':'ping','returns':'Alpha'}",
        ),
    ])
    .unwrap();
    assert!(names(&registry, Pattern::Returns).is_empty());
}

/// A type, a command or a reserved name that holds `{Command}Returns` is
/// an error naming both, not a type that the generated code defines twice.
#[test]
fn a_held_name_of_an_enum_of_return_types_is_an_error() {
    for (extra, holder) in [
        ("{'struct':'PingReturns','data':{}}", "struct 'PingReturns'"),
        ("{'command':'ping-returns'}", "command 'ping-returns'"),
    ] {
        let error = pings(extra).unwrap_err();
        let RegistryError::ReturnsNameTaken {
            protocol,
            command,
            name,
            holder: held,
            tag,
        } = &error
        else {
            panic!("{error:?}")
        };
        assert_eq!(
            (
                *protocol,
                command.as_str(),
                name.as_str(),
                held.as_str(),
                tag.as_str()
            ),
            ("QMP", "ping", "PingReturns", holder, "v9.1.0")
        );
        assert_eq!(
            error.to_string(),
            format!(
                "QMP command ping needs an enum of return types from v9.1.0, PingReturns, \
                 which {holder} holds; name the enum in RETURNS_ENUM_NAMES"
            )
        );
    }
    // The naming table names the enum instead, and its name must be free.
    let mut registry = pings("").unwrap();
    let tags = BTreeMap::from([((9, 1, 0), "v9.1.0".to_owned())]);
    let table = [("QMP", "ping", "PingReplies")];
    plan_returns_enums_with(&mut registry, "QMP", &tags, &table).unwrap();
    let UnifiedType::Struct(ping) = &registry.types["Ping"] else {
        panic!()
    };
    assert_eq!(ping.returns_enum.as_deref(), Some("PingReplies"));
    let other = registry.types["Beta"].clone();
    registry.types.insert("PingReplies".into(), other);
    let error = plan_returns_enums_with(&mut registry, "QMP", &tags, &table).unwrap_err();
    assert!(
        matches!(&error, RegistryError::ReturnsNameTaken { name, .. } if name == "PingReplies"),
        "{error:?}"
    );
    // The table of one protocol does not name the other's enums.
    let mut registry = pings("").unwrap();
    plan_returns_enums_with(
        &mut registry,
        "QMP",
        &tags,
        &[("QGA", "ping", "PingReplies")],
    )
    .unwrap();
    let UnifiedType::Struct(ping) = &registry.types["Ping"] else {
        panic!()
    };
    assert_eq!(ping.returns_enum.as_deref(), Some("PingReturns"));
}

/// Every pattern of synthesized names in one history, each in its own
/// namespace with what it stands for.
#[test]
fn the_inventory_names_every_pattern_of_synthesized_names() {
    let modern = |extra: &str| {
        format!(
            "{{'enum':'Mode','data':['on','v2-4-0']}}
             {{'struct':'Holder','data':{{'type':'str','self':'int','foo':{extra}}}}}
             {{'struct':'String','data':{{}}}}
             {{'enum':'ShapeKind','data':['round']}}
             {{'union':'Shape','base':{{'kind':'ShapeKind'}},'discriminator':'kind',
               'data':{{'round':'Holder'}}}}
             {{'struct':'Qapi','data':{{}}}}
             {{'command':'qapi','returns':'Qapi'}}
             {{'event':'TICK'}}"
        )
    };
    let registry = registry(&[
        (
            "v2.3.0",
            "{'type':'Encode','data':{'error':{'code':'int'}}}
             {'union':'Simple','data':{'one':'int'}}
             {'type':'Record','data':{'pc':'str','PC':'int'}}
             {'enum':'Shift','data':['a']}",
        ),
        (
            "v9.0.0",
            &format!("{} {{'enum':'Shift','data':['a']}}", modern("'int'")),
        ),
        (
            "v9.1.0",
            &format!("{} {{'struct':'Shift','data':{{}}}}", modern("'str'")),
        ),
    ])
    .unwrap();
    let pattern = |pattern| names(&registry, pattern);
    assert_eq!(
        pattern(Pattern::Escaped),
        [
            "items: QapiString",
            "fields of Holder: r#type",
            "fields of Holder: self_",
        ]
    );
    assert_eq!(pattern(Pattern::Generated), ["items: ShiftGenerated"]);
    assert_eq!(
        pattern(Pattern::Branch),
        ["items: ShapeBranch", "items: SimpleBranch"]
    );
    assert_eq!(pattern(Pattern::ImplicitTag), ["items: SimpleKind"]);
    // `{Qapi}Command` would be qrab-qapi's trait `QapiCommand`.
    assert_eq!(pattern(Pattern::Command), ["items: QapiQapiCommand"]);
    assert_eq!(pattern(Pattern::Event), ["items: TickEvent"]);
    // `{Encode}{Error}` would be qrab-qapi's `EncodeError`.
    assert_eq!(pattern(Pattern::Inline), ["items: QapiEncodeError"]);
    assert_eq!(pattern(Pattern::Legacy), ["fields of Holder: foo_legacy"]);
    assert_eq!(pattern(Pattern::Alt), ["fields of Record: pc_alt"]);
    assert_eq!(
        pattern(Pattern::Lowering),
        [
            "fields of Shape: u",
            "fields of Simple: r_type",
            "fields of Simple: u"
        ]
    );
    // A schema value that spells a release is a variant of its enum alone.
    assert!(pattern(Pattern::Schema).contains(&"variants of Mode: V240".to_owned()));
    let reserved = pattern(Pattern::Reserved);
    assert!(
        reserved.contains(&"items: commands".to_owned()),
        "{reserved:?}"
    );
    assert!(
        reserved.contains(&"items: QapiCommand".to_owned()),
        "{reserved:?}"
    );
}

/// A member whose name the escapes of the historical members cannot avoid
/// is an error that names both.
#[test]
fn a_historical_member_whose_names_are_held_is_an_error() {
    let release = |foo: &str| {
        format!(
            "{{'struct':'Holder','data':{{'foo':'{foo}','foo-legacy':'int',
               'foo-legacy-v9-0':'int'}}}}"
        )
    };
    let error = registry(&[("v9.0.0", &release("int")), ("v9.1.0", &release("str"))]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "QMP fields of Holder: member 'foo-legacy-v9-0' and the earlier member 'foo' are both \
         named foo_legacy_v9_0"
    );
    // Without the member of the series name, the series name escapes the
    // numbered one.
    let release =
        |foo: &str| format!("{{'struct':'Holder','data':{{'foo':'{foo}','foo-legacy':'int'}}}}");
    let registry = registry(&[("v9.0.0", &release("int")), ("v9.1.0", &release("str"))]).unwrap();
    assert_eq!(
        names(&registry, Pattern::Legacy),
        ["fields of Holder: foo_legacy_v9_0"]
    );
}

/// A field `value` and a union branch `on` of each release, as `(tag,
/// type)`: none where the type is `None`, else of that type, which the
/// branch wraps in a struct.
fn history(releases: &[(&str, Option<&str>)]) -> UnifiedRegistry {
    let payloads = "{'struct':'OfInt','data':{'v':'int'}} {'struct':'OfStr','data':{'v':'str'}}
         {'struct':'OfNumber','data':{'v':'number'}} {'struct':'OfSize','data':{'v':'size'}}
         {'struct':'OfByte','data':{'v':'uint8'}}";
    let schemas: Vec<_> = releases
        .iter()
        .map(|(tag, ty)| {
            let (value, on, tags) = match ty {
                Some(ty) => {
                    let payload = match *ty {
                        "int" => "OfInt",
                        "str" => "OfStr",
                        "number" => "OfNumber",
                        "size" => "OfSize",
                        _ => "OfByte",
                    };
                    (
                        format!("'value':'{ty}',"),
                        format!("'on':'{payload}'"),
                        "'on',",
                    )
                }
                None => (String::new(), String::new(), ""),
            };
            let text = format!(
                "{payloads} {{'enum':'Tag','data':[{tags}'off']}}
                 {{'struct':'Record','data':{{{value}'other':'int'}}}}
                 {{'union':'Choice','base':{{'tag':'Tag'}},'discriminator':'tag','data':{{{on}}}}}"
            );
            (*tag, text)
        })
        .collect();
    let schemas: Vec<_> = schemas
        .iter()
        .map(|(tag, text)| (*tag, text.as_str()))
        .collect();
    registry(&schemas).unwrap()
}

/// The Rust names of the members of `Record` and of the branches of
/// `Choice`, each with its wire name and type.
fn members(registry: &UnifiedRegistry) -> BTreeMap<String, (String, String)> {
    let mut members = BTreeMap::new();
    let wire = |name: &str, meta: &Metadata| {
        meta.attributes
            .iter()
            .find_map(|attr| match attr {
                Attribute::Item { name, value } if name == "name" => Some(value.clone()),
                _ => None,
            })
            .unwrap_or_else(|| name.to_owned())
    };
    let UnifiedType::Struct(record) = &registry.types["Record"] else {
        panic!()
    };
    for field in &record.ir.fields {
        members.insert(
            format!("Record.{}", field.name),
            (wire(&field.name, &field.meta), field.r#type.clone()),
        );
    }
    let UnifiedType::Enum(choice) = &registry.types["ChoiceBranch"] else {
        panic!()
    };
    for variant in &choice.ir.variants {
        let payload = match &variant.kind {
            EnumVariantKind::Tuple(ty) => ty.clone(),
            EnumVariantKind::Unit => String::new(),
        };
        members.insert(
            format!("Choice.{}", variant.name),
            (wire(&variant.name, &variant.meta), payload),
        );
    }
    members
}

/// Adding a patch release of an older series to the selection keeps the
/// name of every member, with its wire name and type: a release that equals
/// its neighbours, one that backports a member or a shape of the next
/// series, and one with a shape of its own, which interrupts a run of
/// releases. Numbering the earlier shapes in release order alone would give
/// a later one the next number in the last case.
#[test]
fn a_patch_release_of_an_older_series_renames_no_member() {
    let base = [
        ("v8.0.0", Some("int")),
        ("v8.1.0", Some("int")),
        ("v8.2.0", Some("int")),
        ("v9.0.0", Some("str")),
        ("v9.1.0", Some("number")),
        ("v9.2.0", Some("size")),
    ];
    let before = members(&history(&base));
    assert!(before.contains_key("Record.value_legacy3"), "{before:?}");
    for (inserted, shape) in [
        // Equal to its neighbours.
        ("v8.2.1", Some("int")),
        // The shape of the next series, backported.
        ("v8.2.1", Some("str")),
        // The shape of a later series, which the next one does not have.
        ("v8.2.1", Some("number")),
        // A shape of its own.
        ("v8.1.1", Some("uint8")),
        // No member at all.
        ("v8.1.1", None),
    ] {
        let mut releases = base.to_vec();
        let at = releases
            .iter()
            .position(|(tag, _)| parse_version_tag_full(tag) > parse_version_tag_full(inserted))
            .unwrap();
        releases.insert(at, (inserted, shape));
        let after = members(&history(&releases));
        for (name, member) in &before {
            assert_eq!(
                after.get(name),
                Some(member),
                "{inserted} {shape:?} renames {name}: {after:?}"
            );
        }
    }
    // The interrupted run keeps its name for its first part; the rest, and
    // a run that only patch releases have, take the names of their series.
    let mut releases = base.to_vec();
    releases.insert(2, ("v8.1.1", Some("uint8")));
    let after = members(&history(&releases));
    let added: Vec<_> = after
        .keys()
        .filter(|name| !before.contains_key(*name))
        .collect();
    assert_eq!(
        added,
        [
            "Choice.OnLegacyV8p1",
            "Choice.OnLegacyV8p2",
            "Record.value_legacy_v8_1",
            "Record.value_legacy_v8_2"
        ]
    );
}

/// A member that only backports have, such as `delay-ns` of QEMU's
/// `BlkdebugInjectErrorOptions` (10.0.10, 10.2.3, then from 11.0.1), names
/// its runs by their series, so that a backport to another series adds a
/// name and renames none.
#[test]
fn runs_of_backports_are_named_by_their_series() {
    let releases = [
        ("v10.0.0", None),
        ("v10.0.10", Some("int")),
        ("v10.1.0", None),
        ("v10.2.0", None),
        ("v10.2.3", Some("int")),
        ("v11.0.0", None),
        ("v11.0.1", Some("int")),
        ("v11.1.0", Some("int")),
    ];
    let registry = history(&releases);
    assert_eq!(
        names(&registry, Pattern::Legacy),
        [
            "variants of ChoiceBranch: OnLegacyV10p0",
            "variants of ChoiceBranch: OnLegacyV10p2",
            "fields of Record: value_legacy_v10_0",
            "fields of Record: value_legacy_v10_2",
            "variants of Tag: OnLegacyV10p0",
            "variants of Tag: OnLegacyV10p2",
        ]
    );
    let mut more = releases.to_vec();
    more.insert(3, ("v10.1.5", Some("int")));
    let before = members(&registry);
    let after = members(&history(&more));
    for (name, member) in &before {
        assert_eq!(after.get(name), Some(member), "{name}");
    }
    assert!(after.contains_key("Record.value_legacy_v10_1"), "{after:?}");
}
