//! The compile errors of structs, commands and events, read with the source
//! text they point at, and the struct keys on other items.

use crate::bounds::VersionBounds;
use crate::tests::{assert_error, deprecated_note, docs, error, expand, expanded};

fn field_attrs(item: &syn::DeriveInput, index: usize) -> &[syn::Attribute] {
    match &item.data {
        syn::Data::Struct(data) => &data.fields.iter().nth(index).unwrap().attrs,
        _ => panic!("not a struct"),
    }
}

#[test]
fn struct_keys_apply_only_to_structs_and_fields() {
    for (args, at) in [
        ("command", "command"),
        ("event", "event"),
        (r#"returns = "()""#, r#""()""#),
        ("allow_oob", "allow_oob"),
        (r#"simple_union(removed = "6.2.0")"#, "simple_union"),
    ] {
        let (message, source) = error(args, "enum E { A }");
        assert!(
            message.ends_with("` applies only to structs"),
            "{args}: {message}"
        );
        assert_eq!(source, at);
    }
    for key in ["flatten", "discriminator", "union", "required"] {
        assert_error(
            "",
            &format!("enum E {{ #[qapi({key})] A(T) }}"),
            &format!("qapi: `{key}` applies only to struct fields"),
            key,
        );
    }
    assert_error(
        "union_branch",
        "struct S {}",
        "qapi: `union_branch` applies only to enums",
        "union_branch",
    );
    assert_error(
        "",
        r#"struct S { #[qapi(flatten = "yes")] a: Option<T> }"#,
        "qapi: `flatten` takes no value",
        r#"flatten = "yes""#,
    );
    assert_error(
        "",
        "struct S { #[qapi(flatten)] #[qapi(flatten)] a: Option<T> }",
        "qapi: duplicate key `flatten`",
        "flatten",
    );
    assert_error(
        r#"command, allow_oob(added = "1.0.0", added = "2.0.0")"#,
        "struct S {}",
        "qapi: duplicate key `added`",
        "added",
    );
    assert_error(
        "command",
        "#[qapi(command)] struct S {}",
        "qapi: duplicate key `command`",
        "command",
    );
    assert_error(
        r#"command, allow_oob(added = "9.0.0", removed = "8.0.0")"#,
        "struct S {}",
        "qapi: `removed` (8.0.0) must be later than `added` (9.0.0)",
        r#"removed = "8.0.0""#,
    );
}

#[test]
fn unsupported_struct_shapes_are_errors() {
    assert_error(
        "",
        "struct S(String);",
        "qapi: tuple structs are not supported; use named fields",
        "(String)",
    );
    assert_error(
        "",
        "struct G<T> { a: T }",
        "qapi: #[qapi] does not support generic items",
        "T",
    );
    assert!(expand("", "struct Unit;", &VersionBounds::default()).is_ok());
}

#[test]
fn fields_with_a_lifetime_or_requirement_are_options() {
    for (item, at) in [
        (
            r#"struct S { #[qapi(added = "9.0.0")] a: String }"#,
            "String",
        ),
        (
            r#"struct S { #[qapi(removed = "9.0.0")] a: String }"#,
            "String",
        ),
        (
            r#"struct S { #[qapi(flatten, added = "9.0.0")] a: Base }"#,
            "Base",
        ),
    ] {
        assert_error(
            "",
            item,
            "qapi: a field with `added` or `removed` must be an `Option`, \
             since peers outside its lifetime do not send it",
            at,
        );
    }
    assert_error(
        "",
        r#"struct S { #[qapi(name = "")] a: String }"#,
        "qapi: the `name` of a field must not be empty",
        "a",
    );
    assert_error(
        "",
        "struct S { #[qapi(required)] a: String }",
        "qapi: `required` marks an `Option` field that the peer's version requires",
        "required",
    );
}

#[test]
fn unions_need_their_flatten_discriminator_and_union_field() {
    assert_error(
        "",
        "struct S { #[qapi(discriminator)] kind: Kind, #[qapi(union)] u: Option<Branch> }",
        "qapi: a `union` field is `flatten`ed into the struct",
        "union",
    );
    assert_error(
        "",
        "struct S { kind: Kind, #[qapi(flatten, union)] u: Option<Branch> }",
        "qapi: a `union` field needs a `discriminator` field to select its branch",
        "union",
    );
    assert_error(
        "",
        "struct S { #[qapi(discriminator)] kind: Kind }",
        "qapi: a `discriminator` selects the branch of a `union` field, which the struct lacks",
        "kind",
    );
    assert_error(
        "",
        "struct S { #[qapi(flatten, discriminator)] kind: Kind, #[qapi(flatten, union)] u: Option<Branch> }",
        "qapi: a `discriminator` is a member of the struct, not `flatten`ed",
        "discriminator",
    );
    for item in [
        r#"struct S { #[qapi(flatten, name = "rest")] rest: Option<HashMap<String, Value>> }"#,
        r#"struct S { #[qapi(discriminator)] kind: Kind, #[qapi(flatten, union)] #[qapi(name = "u")] u: Option<Branch> }"#,
    ] {
        assert_error(
            "",
            item,
            "qapi: a `flatten`ed field has no member of its own, so it takes no `name`",
            "flatten",
        );
    }
    assert_error(
        "simple_union",
        "struct S { #[qapi(flatten)] base: Base }",
        "qapi: `simple_union` describes the layout of a `union` field, which the struct lacks",
        "simple_union",
    );
    assert_error(
        "",
        r#"struct S {
            #[qapi(name = "type", discriminator, added = "6.2.0")] kind: Option<Kind>,
            #[qapi(name = "type", discriminator, removed = "6.2.0")] kind_legacy: Option<KindLegacy>,
            #[qapi(flatten, union)] branch: Branch,
        }"#,
        "qapi: a union field with version-alternate discriminators must be `Option<T>`",
        "Branch",
    );
    assert_error(
        "",
        "struct S { #[qapi(discriminator)] kind: Option<Kind>, #[qapi(flatten, union)] branch: Branch }",
        "qapi: a union field with an optional discriminator must be `Option<T>`",
        "Branch",
    );
    assert_error(
        "",
        r#"struct S {
            #[qapi(name = "value", removed = "3.0.0")] old: Option<i64>,
            #[qapi(name = "value", added = "2.0.0")] new: Option<String>,
        }"#,
        "qapi: wire name `value` is already taken in part of this lifetime; \
         representations of one wire name need disjoint `added`/`removed` ranges",
        "new",
    );
}

#[test]
fn command_keys_belong_to_commands_alone() {
    assert_error(
        "command, event",
        "struct S {}",
        "qapi: an item is a `command` or an `event`, not both",
        "event",
    );
    for (args, key, at) in [
        ("allow_oob", "allow_oob", "allow_oob"),
        (
            "event, allow_preconfig",
            "allow_preconfig",
            "allow_preconfig",
        ),
        (
            r#"no_success_response(added = "1.0.0")"#,
            "no_success_response",
            "no_success_response",
        ),
        (r#"returns = "bool""#, "returns", r#""bool""#),
        (
            r#"returns(added = "1.0.0", value = "bool")"#,
            "returns",
            r#""bool""#,
        ),
    ] {
        assert_error(
            args,
            "struct S {}",
            &format!("qapi: `{key}` applies only to commands"),
            at,
        );
    }
}

#[test]
fn a_return_type_history_is_ordered_and_ends_at_returns() {
    let (message, at) = error(r#"command, returns = "Vec<""#, "struct S {}");
    assert!(
        message.starts_with("qapi: `returns` is not a Rust type: unexpected end of input"),
        "{message}"
    );
    assert_eq!(at, r#""Vec<""#);
    let (message, at) = error(
        r#"command, added = "1.0.0", returns = "B", returns(added = "1.0.0", value = "Vec<"),
            returns(added = "2.0.0", value = "B")"#,
        "struct S {}",
    );
    assert!(
        message.starts_with("qapi: `returns` is not a Rust type"),
        "{message}"
    );
    assert_eq!(at, r#""Vec<""#);
    assert_error(
        r#"command, added = "1.0.0", returns(added = "1.0.0", value = "A"), returns(added = "2.0.0", value = "B")"#,
        "struct S {}",
        "qapi: `returns(...)` comes with `returns = \"Type\"`, the newest return type",
        r#""B""#,
    );
    assert_error(
        r#"command, added = "1.0.0", returns = "A", returns(added = "1.0.0", value = "A"),
            returns(added = "2.0.0", value = "B")"#,
        "struct S {}",
        "qapi: `returns` is the newest `returns(...)` value",
        r#""A""#,
    );
    assert_error(
        r#"command, added = "2.0.0", returns = "A", returns(added = "2.0.0", value = "B"),
            returns(added = "1.0.0", value = "A")"#,
        "struct S {}",
        "qapi: `returns(...)` lists the return types in release order, each added after the one before",
        r#""A""#,
    );
    assert_error(
        r#"command, added = "1.0.0", returns = "B", returns(added = "1.0.0", value = "A"),
            returns(added = "1.0.0", value = "B")"#,
        "struct S {}",
        "qapi: `returns(...)` lists the return types in release order, each added after the one before",
        r#""B""#,
    );
    assert_error(
        r#"command, added = "1.0.0", returns(added = "1.0.0")"#,
        "struct S {}",
        "qapi: `returns(...)` needs `added` and `value`",
        r#"returns(added = "1.0.0")"#,
    );
    // The same type, spelled with other whitespace, is the same.
    let args = r#"command, added = "1.0.0", returns = "Vec<A>", returns(added = "1.0.0", value = "B"),
        returns(added = "2.0.0", value = "Vec < A >")"#;
    assert!(expand(args, "struct S {}", &VersionBounds::default()).is_ok());
    assert_error(
        r#"command, added = "1.0.0", returns = "A", returns(added = "1.0.0", value = "A")"#,
        "struct S {}",
        "qapi: `returns(...)` is the history of a return type that changed, so it lists two or \
         more; give one type as `returns = \"Type\"`",
        r#""A""#,
    );
    assert_error(
        r#"command, added = "1.0.0", returns = "B", returns(added = "1.0.0", value = "Vec<A>"),
            returns(added = "2.0.0", value = "Vec < A >"), returns(added = "3.0.0", value = "B")"#,
        "struct S {}",
        "qapi: `returns(...)` lists a return type once it changes, not the same type again",
        r#""Vec < A >""#,
    );
    // A type may come back after another.
    let args = r#"command, added = "1.0.0", returns = "A", returns(added = "1.0.0", value = "A"),
        returns(added = "2.0.0", value = "B"), returns(added = "3.0.0", value = "A")"#;
    assert!(expand(args, "struct S {}", &VersionBounds::default()).is_ok());
    // The history covers the command's lifetime: it starts with the
    // command and changes before its removal.
    let history = r#"returns = "B", returns(added = "1.0.0", value = "A"),
        returns(added = "2.0.0", value = "B")"#;
    assert_error(
        &format!(r#"command, added = "0.15.0", {history}"#),
        "struct S {}",
        "qapi: the first `returns(...)` is added with the command, at 0.15.0",
        r#""A""#,
    );
    assert_error(
        &format!(r#"command, added = "1.1.0", {history}"#),
        "struct S {}",
        "qapi: the first `returns(...)` is added with the command, at 1.1.0",
        r#""A""#,
    );
    assert_error(
        &format!(r#"command, added = "1.0.0", removed = "2.0.0", {history}"#),
        "struct S {}",
        "qapi: `returns(...)` (added 2.0.0) must be earlier than the command's `removed` (2.0.0)",
        r#""B""#,
    );
    let args = format!(r#"command, added = "1.0.0", removed = "2.1.0", {history}"#);
    assert!(expand(&args, "struct S {}", &VersionBounds::default()).is_ok());
    // Without its `added`, the command has releases before the first type.
    assert_error(
        &format!(r#"command, removed = "2.1.0", {history}"#),
        "struct S {}",
        "qapi: the first `returns(...)` is added with the command, which needs `added`",
        r#""A""#,
    );
    // Each type is checked before the order.
    let (message, at) = error(
        r#"command, added = "2.0.0", returns = "B", returns(added = "2.0.0", value = "Vec<"),
            returns(added = "1.0.0", value = "B")"#,
        "struct S {}",
    );
    assert!(
        message.starts_with("qapi: `returns` is not a Rust type"),
        "{message}"
    );
    assert_eq!(at, r#""Vec<""#);
    // Every type of the history is checked, whatever the bounds keep.
    let args = r#"command, added = "1.0.0", returns = "B", returns(added = "1.0.0", value = "Vec<"),
        returns(added = "2.0.0", value = "B")"#;
    let (message, at) = crate::tests::error_in(
        args,
        "struct S {}",
        &VersionBounds {
            min: Some(crate::support::Version(3, 0, 0)),
            max: None,
        },
    );
    assert!(
        message.starts_with("qapi: `returns` is not a Rust type"),
        "{message}"
    );
    assert_eq!(at, r#""Vec<""#);
}

#[test]
fn deprecation_of_fields_takes_effect_by_feature_or_minimum() {
    use crate::support::Version;
    let item = r#"struct Example { #[qapi(deprecated(added = "9.2.0"))] value: String }"#;
    for (min, deprecated) in [(Version(10, 0, 0), true), (Version(9, 0, 0), false)] {
        let bounds = VersionBounds {
            min: Some(min),
            max: None,
        };
        let (item, rest) = expanded("", item, &bounds);
        assert!(rest.contains("qapi warning: Example.value is deprecated since QEMU 9.2.0"));
        let note = deprecated_note(field_attrs(&item, 0));
        assert_eq!(note.is_some(), deprecated, "{note:?}");
        assert!(note.is_none_or(|note| note.contains("deprecated since QEMU 9.2.0")));
    }
    // Each deprecation warns in its own releases.
    let item = r#"struct Example {
        #[qapi(deprecated(added = "8.0.0", removed = "9.0.0"), deprecated(added = "10.0.0"))]
        value: String,
    }"#;
    let (_, rest) = expanded("", item, &VersionBounds::default());
    for warning in [
        "qapi warning: Example.value is deprecated since QEMU 8.0.0",
        "qapi warning: Example.value is deprecated since QEMU 10.0.0",
        "qapi :: qemu_version () < (9u32 , 0u32 , 0u32)",
    ] {
        assert!(rest.contains(warning), "{warning}: {rest}");
    }
    for attrs in [
        r#"#[qapi(feature = "deprecated", feature = "unstable")]"#,
        r#"#[qapi(feature = "deprecated")] #[qapi(feature = "unstable")]"#,
    ] {
        let (item, _) = expanded(
            "",
            &format!("struct S {{ {attrs} value: String }}"),
            &VersionBounds::default(),
        );
        let attrs = field_attrs(&item, 0);
        assert_eq!(
            deprecated_note(attrs).as_deref(),
            Some("# [deprecated (note = \"deprecated QAPI field\")]")
        );
        assert_eq!(
            docs(attrs),
            [" **Unstable** — this field may change without notice."]
        );
    }
}

#[test]
fn the_bounds_keep_the_return_types_of_their_releases() {
    use crate::support::Version;
    let args = r#"command, added = "1.2.0", name = "query-target", returns = "New",
        returns(added = "1.2.0", value = "Old"), returns(added = "10.1.0", value = "New")"#;
    let returns = |min: Option<Version>, max: Option<Version>| {
        let (_, rest) = expanded(args, "struct QueryTarget {}", &VersionBounds { min, max });
        let enum_kept = rest.contains("enum QueryTargetReturns");
        let returns = rest
            .split("type Returns = ")
            .nth(1)
            .and_then(|tail| tail.split(" ;").next())
            .unwrap()
            .to_owned();
        (returns, enum_kept)
    };
    assert_eq!(returns(None, None), ("QueryTargetReturns".into(), true));
    assert_eq!(
        returns(Some(Version(9, 0, 0)), Some(Version(10, 1, 0))),
        ("QueryTargetReturns".into(), true)
    );
    // A single release's type needs no enum.
    assert_eq!(
        returns(None, Some(Version(10, 0, 9))),
        ("Old".into(), false)
    );
    assert_eq!(
        returns(Some(Version(10, 1, 0)), None),
        ("New".into(), false)
    );
}

#[test]
fn a_flattened_field_takes_no_keys_of_a_member() {
    for key in [
        "required",
        r#"deprecated(added = "1.0.0")"#,
        r#"feature = "deprecated""#,
        r#"feature = "unstable""#,
        r#"condition = "CONFIG_X""#,
    ] {
        let name = key.split(['(', ' ']).next().unwrap();
        assert_error(
            "",
            &format!("struct S {{ #[qapi(flatten, {key})] base: Option<Base> }}"),
            &format!("qapi: a `flatten`ed field has no member of its own, so it takes no `{name}`"),
            "flatten",
        );
    }
    let item =
        r#"struct S { #[qapi(flatten, added = "2.0.0", removed = "3.0.0")] base: Option<Base> }"#;
    assert!(expand("", item, &VersionBounds::default()).is_ok());
}

#[test]
fn a_union_has_one_branch_field_and_one_discriminator_member() {
    assert_error(
        "",
        r#"struct S {
            #[qapi(discriminator)] kind: Kind,
            #[qapi(flatten, union)] u: Option<Branch>,
            #[qapi(flatten, union)] v: Option<Branch>,
        }"#,
        "qapi: a struct has at most one `union` field",
        "union",
    );
    // In each release one member is the discriminator.
    for keys in [
        ["discriminator", "discriminator"],
        [
            r#"discriminator, removed = "6.0.0""#,
            r#"discriminator, added = "3.0.0""#,
        ],
        [
            r#"discriminator(removed = "4.0.0"), removed = "6.0.0""#,
            r#"discriminator, added = "3.0.0""#,
        ],
        [
            r#"discriminator(removed = "3.0.0"), discriminator(added = "5.0.0")"#,
            r#"discriminator(added = "3.0.0", removed = "6.0.0")"#,
        ],
    ] {
        assert_error(
            "",
            &format!(
                r#"struct S {{
                    #[qapi({})] arch: Option<Kind>,
                    #[qapi({})] target: Option<Kind>,
                    #[qapi(flatten, union)] u: Option<Branch>,
                }}"#,
                keys[0], keys[1]
            ),
            "qapi: discriminators `arch` and `target` overlap; in each release one member \
             selects the branch",
            "discriminator",
        );
    }
    for keys in [
        [
            r#"discriminator(removed = "3.0.0"), removed = "6.0.0""#,
            r#"discriminator, added = "3.0.0""#,
        ],
        [
            r#"discriminator(removed = "3.0.0"), discriminator(added = "5.0.0")"#,
            r#"discriminator(added = "3.0.0", removed = "5.0.0")"#,
        ],
    ] {
        let item = format!(
            r#"struct S {{
                #[qapi({})] arch: Option<Kind>,
                #[qapi({})] target: Option<Kind>,
                #[qapi(flatten, union)] u: Option<Branch>,
            }}"#,
            keys[0], keys[1]
        );
        assert!(
            expand("", &item, &VersionBounds::default()).is_ok(),
            "{item}"
        );
    }
    // A discriminator's runs lie within its field's lifetime, give no bound
    // that is the field's own, and do not overlap each other.
    for (keys, message) in [
        (
            r#"discriminator(removed = "7.0.0"), removed = "6.0.0""#,
            "qapi: `discriminator` (removed 7.0.0) must end by the field's `removed` (6.0.0)",
        ),
        (
            r#"discriminator(added = "1.0.0"), added = "2.0.0""#,
            "qapi: `discriminator` (added 1.0.0) must not start before the field's `added` \
             (2.0.0)",
        ),
        (
            r#"discriminator(added = "6.0.0"), removed = "6.0.0""#,
            "qapi: `discriminator` (added 6.0.0) must start before the field's `removed` (6.0.0)",
        ),
        (
            r#"discriminator(removed = "2.0.0"), added = "2.0.0""#,
            "qapi: `discriminator` (removed 2.0.0) must end after the field's `added` (2.0.0)",
        ),
        (
            r#"discriminator(removed = "6.0.0"), removed = "6.0.0""#,
            "qapi: `discriminator` (removed 6.0.0) ends with the field; leave its `removed` out",
        ),
        (
            r#"discriminator(added = "2.0.0"), added = "2.0.0""#,
            "qapi: `discriminator` (added 2.0.0) starts with the field; leave its `added` out",
        ),
        (
            "discriminator, discriminator(added = \"2.0.0\")",
            "qapi: the `discriminator` runs of `arch` overlap",
        ),
    ] {
        assert_error(
            "",
            &format!(
                r#"struct S {{
                    #[qapi({keys})] arch: Option<Kind>,
                    #[qapi(flatten, union)] u: Option<Branch>,
                }}"#
            ),
            message,
            "discriminator",
        );
    }
    // Without bounds of its own, the field has the type's.
    for (keys, message) in [
        (
            r#"discriminator(added = "1.0.0")"#,
            "qapi: `discriminator` (added 1.0.0) must not start before the type's `added` (2.0.0)",
        ),
        (
            r#"discriminator(removed = "7.0.0")"#,
            "qapi: `discriminator` (removed 7.0.0) must end by the type's `removed` (6.0.0)",
        ),
        (
            r#"discriminator(removed = "6.0.0")"#,
            "qapi: `discriminator` (removed 6.0.0) ends with the field; leave its `removed` out",
        ),
    ] {
        assert_error(
            r#"added = "2.0.0", removed = "6.0.0""#,
            &format!(
                r#"struct S {{
                    #[qapi({keys})] arch: Option<Kind>,
                    #[qapi(flatten, union)] u: Option<Branch>,
                }}"#
            ),
            message,
            "discriminator",
        );
    }
    // The runs cover the union field's lifetime and no more: in each
    // release that has the union, one member selects its branch.
    for (keys, union, message) in [
        (
            [
                r#"discriminator(removed = "3.0.0")"#,
                r#"discriminator(added = "4.0.0")"#,
            ],
            "",
            "qapi: no `discriminator` selects the branch of `u` from 3.0.0 until 4.0.0",
        ),
        (
            [
                r#"discriminator(removed = "3.0.0"), removed = "6.0.0""#,
                r#"discriminator(added = "4.0.0", removed = "5.0.0")"#,
            ],
            "",
            "qapi: no `discriminator` selects the branch of `u` from 3.0.0 until 4.0.0",
        ),
        (
            [
                r#"discriminator(added = "2.0.0")"#,
                r#"discriminator, added = "3.0.0""#,
            ],
            "",
            "qapi: no `discriminator` selects the branch of `u` before 2.0.0",
        ),
        (
            [
                r#"discriminator(removed = "3.0.0")"#,
                r#"discriminator(added = "3.0.0", removed = "5.0.0")"#,
            ],
            "",
            "qapi: no `discriminator` selects the branch of `u` from 5.0.0 on",
        ),
        (
            [
                r#"discriminator(removed = "3.0.0")"#,
                r#"discriminator(added = "3.0.0", removed = "5.0.0")"#,
            ],
            r#", removed = "6.0.0""#,
            "qapi: no `discriminator` selects the branch of `u` from 5.0.0 until 6.0.0",
        ),
        (
            [
                r#"discriminator(removed = "2.0.0"), added = "1.0.0""#,
                r#"discriminator(added = "4.0.0")"#,
            ],
            r#", added = "3.0.0""#,
            "qapi: `discriminator` must not start before the `union` field's `added` (3.0.0)",
        ),
        (
            [
                r#"discriminator(removed = "3.0.0")"#,
                "discriminator, added = \"3.0.0\"",
            ],
            r#", removed = "5.0.0""#,
            "qapi: `discriminator` must end by the `union` field's `removed` (5.0.0)",
        ),
    ] {
        assert_error(
            "",
            &format!(
                r#"struct S {{
                    #[qapi({})] arch: Option<Kind>,
                    #[qapi({})] target: Option<Kind>,
                    #[qapi(flatten, union{union})] u: Option<Branch>,
                }}"#,
                keys[0], keys[1]
            ),
            message,
            "discriminator",
        );
    }
    // A union field that some releases lack has a discriminator in those
    // releases, which may be a member in the others.
    for (discriminator, union) in [
        (
            r#"discriminator, added = "3.0.0")] kind: Option<Kind>"#,
            r#"added = "3.0.0""#,
        ),
        (
            r#"discriminator(added = "3.0.0"))] kind: Kind"#,
            r#"added = "3.0.0""#,
        ),
        (
            r#"discriminator(removed = "3.0.0"))] kind: Kind"#,
            r#"removed = "3.0.0""#,
        ),
    ] {
        let item = format!(
            r#"struct S {{
                #[qapi({discriminator},
                #[qapi(flatten, union, {union})] u: Option<Branch>,
            }}"#
        );
        assert!(
            expand("", &item, &VersionBounds::default()).is_ok(),
            "{item}"
        );
    }
    let item = r#"struct S {
        #[qapi(discriminator)] kind: Kind,
        #[qapi(flatten, union, added = "3.0.0")] u: Option<Branch>,
    }"#;
    assert_error(
        "",
        item,
        "qapi: `discriminator` must not start before the `union` field's `added` (3.0.0)",
        "discriminator",
    );
    // Alternates of one wire name may not overlap either, even where only
    // one of them is the discriminator.
    let (message, at) = error(
        "",
        r#"struct S {
            #[qapi(discriminator(removed = "2.0.0"), name = "type", removed = "3.0.0")] old: Option<Kind>,
            #[qapi(discriminator, name = "type", added = "2.0.0")] new: Option<Kind>,
            #[qapi(flatten, union)] u: Option<Branch>,
        }"#,
    );
    assert!(
        message.starts_with("qapi: wire name `type` is already taken"),
        "{message}"
    );
    assert_eq!(at, "new");
}

#[test]
fn the_ranges_of_a_structs_flags_lie_within_its_lifetime_and_do_not_overlap() {
    let union = r#"struct S { #[qapi(discriminator, name = "type")] kind: Kind, #[qapi(flatten, union)] u: Option<Branch> }"#;
    assert_error(
        r#"added = "5.0.0", simple_union(removed = "2.0.0")"#,
        union,
        "qapi: `simple_union` (removed 2.0.0) must end after the type's `added` (5.0.0)",
        "simple_union",
    );
    assert_error(
        "simple_union, simple_union",
        union,
        "qapi: `simple_union` overlaps the `simple_union` before it, which does not end",
        "simple_union",
    );
    for key in ["allow_oob", "allow_preconfig", "no_success_response"] {
        assert_error(
            &format!(r#"command, removed = "5.0.0", {key}(added = "6.0.0")"#),
            "struct S {}",
            &format!("qapi: `{key}` (added 6.0.0) must start before the type's `removed` (5.0.0)"),
            key,
        );
        assert_error(
            &format!(r#"command, {key}(removed = "4.0.0"), {key}(added = "3.0.0")"#),
            "struct S {}",
            &format!(
                "qapi: `{key}` must not start before the `removed` (4.0.0) of the `{key}` before it"
            ),
            key,
        );
    }
}

#[test]
fn a_member_of_a_struct_lives_within_it() {
    assert_error(
        r#"removed = "5.0.0""#,
        r#"struct S { #[qapi(added = "6.0.0")] a: Option<u32> }"#,
        "qapi: `added` (6.0.0) must be earlier than the type's `removed` (5.0.0)",
        r#"added = "6.0.0""#,
    );
}

#[test]
fn the_flags_of_a_command_follow_the_bounds() {
    use crate::support::Version;
    let constants = |args: &str, min: Option<Version>, max: Option<Version>| {
        let (_, rest) = expanded(args, "struct C {}", &VersionBounds { min, max });
        let constant = |name: &str| {
            rest.split(&format!("const {name} : "))
                .nth(1)
                .and_then(|tail| tail.split(';').next())
                .unwrap()
                .split(" = ")
                .nth(1)
                .unwrap()
                .trim()
                .to_owned()
        };
        (
            constant("ALLOW_PRECONFIG"),
            constant("PRECONFIG_SINCE"),
            rest.contains("PreconfigSafe"),
        )
    };
    let none = || {
        (
            "false".to_owned(),
            ":: core :: option :: Option :: None".to_owned(),
            false,
        )
    };
    let since = |version: &str| format!(":: core :: option :: Option :: Some (({version}))");
    let args = r#"command, allow_preconfig(added = "9.0.0")"#;
    assert_eq!(constants(args, None, Some(Version(8, 0, 0))), none());
    assert_eq!(
        constants(args, None, Some(Version(9, 0, 0))),
        ("true".into(), since("9u32 , 0u32 , 0u32"), true)
    );
    let args = r#"command, allow_preconfig(added = "3.0.0", removed = "5.0.0"),
        allow_preconfig(added = "7.0.0", removed = "8.0.0")"#;
    assert_eq!(constants(args, Some(Version(8, 0, 0)), None), none());
    // A run that a release of the bounds has makes the command
    // `PreconfigSafe`, even where the maximum lacks it.
    assert_eq!(
        constants(args, Some(Version(6, 0, 0)), None),
        ("false".into(), since("7u32 , 0u32 , 0u32"), true)
    );
    assert_eq!(
        constants(args, None, None),
        ("false".into(), since("3u32 , 0u32 , 0u32"), true)
    );
    // Without `added`, a run starts with the command.
    let args = r#"command, added = "2.0.0", allow_preconfig(removed = "5.0.0")"#;
    assert_eq!(
        constants(args, None, None),
        ("false".into(), since("2u32 , 0u32 , 0u32"), true)
    );
    assert_eq!(constants(args, Some(Version(5, 0, 0)), None), none());
    // The other flag constants describe the newest release of the bounds.
    let allow_oob = |args: &str, max: Option<Version>| {
        let (_, rest) = expanded(args, "struct C {}", &VersionBounds { min: None, max });
        rest.contains("const ALLOW_OOB : bool = true")
    };
    assert!(allow_oob(r#"command, allow_oob(added = "9.0.0")"#, None));
    assert!(!allow_oob(
        r#"command, allow_oob(added = "9.0.0")"#,
        Some(Version(8, 0, 0))
    ));
    assert!(!allow_oob(r#"command, allow_oob(removed = "9.0.0")"#, None));
    assert!(allow_oob(
        r#"command, allow_oob(removed = "9.0.0")"#,
        Some(Version(8, 0, 0))
    ));
}

#[test]
fn a_field_declares_no_use_of_a_deprecated_type() {
    let (item, _) = expanded("", "struct S { a: Old }", &VersionBounds::default());
    let attrs = field_attrs(&item, 0);
    assert!(
        attrs
            .iter()
            .any(|attr| quote::quote!(#attr).to_string() == "# [allow (deprecated)]"),
        "{}",
        attrs
            .iter()
            .map(|attr| quote::quote!(#attr).to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );
}

#[test]
fn a_union_whose_discriminator_the_bounds_leave_out_is_refused_in_its_releases() {
    // `CpuInfoFast`: `target` takes over from `arch` in 3.0.
    let item = r#"#[qapi(added = "2.12.0")] struct Cpu {
        #[qapi(discriminator(removed = "3.0.0"), removed = "6.0.0")] arch: Option<Arch>,
        #[qapi(discriminator, added = "3.0.0")] target: Option<Target>,
        #[qapi(flatten, union)] u: Option<Branch>,
    }"#;
    let refusal = "Cpu.u: no discriminator selects the branch in QEMU {}.{}.{}{}";
    for (min, max, refused) in [
        (None, Some("2.12.0"), true),
        (Some("6.0.0"), None, true),
        (Some("3.0.0"), Some("5.2.0"), false),
        (None, None, false),
    ] {
        let bounds = VersionBounds {
            min: min.map(|min| crate::support::Version::parse(min).unwrap()),
            max: max.map(|max| crate::support::Version::parse(max).unwrap()),
        };
        let (_, rest) = expanded("", item, &bounds);
        assert_eq!(rest.contains(refusal), refused, "{min:?}..{max:?}: {rest}");
    }
}

#[test]
fn unscoped_decoding_takes_the_discriminator_of_the_newest_release() {
    use crate::attrs::{ItemAttrs, TypeAttrs};
    use crate::structs::FieldInfo;
    use crate::support::Version;
    let field = |keys: &str| {
        let item: syn::DeriveInput = syn::parse_str(&format!(
            "struct S {{ #[qapi({keys})] arch: Option<Arch> }}"
        ))
        .unwrap();
        let syn::Data::Struct(data) = item.data else {
            unreachable!()
        };
        let field = data.fields.into_iter().next().unwrap();
        FieldInfo {
            attrs: ItemAttrs::parse(&field.attrs, &TypeAttrs::default()).unwrap(),
            ident: field.ident.unwrap(),
            ty: field.ty,
        }
    };
    let newest = |field: &FieldInfo, max: Option<&str>| {
        field.discriminates_at_newest(&VersionBounds {
            min: None,
            max: max.map(|max| Version::parse(max).unwrap()),
        })
    };
    // A run that starts at the maximum holds in it, one that ends there not.
    let from = field(r#"discriminator(added = "3.0.0")"#);
    assert!(newest(&from, Some("3.0.0")) && newest(&from, None));
    assert!(!newest(&from, Some("2.12.0")));
    let until = field(r#"discriminator(removed = "3.0.0")"#);
    assert!(newest(&until, Some("2.12.0")));
    assert!(!newest(&until, Some("3.0.0")) && !newest(&until, None));
    // Any run of the field may hold there.
    let runs = field(r#"discriminator(removed = "3.0.0"), discriminator(added = "5.0.0")"#);
    assert!(newest(&runs, Some("2.12.0")) && newest(&runs, Some("5.0.0")));
    assert!(!newest(&runs, Some("4.0.0")) && newest(&runs, None));
}
