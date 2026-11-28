//! The attribute's compile errors, which these tests read with the source
//! text they point at, and the parts of the expansion that only the
//! attributes it emits show: documentation and deprecation.

use crate::support::{Version, flag_at_newest};
use proc_macro2::TokenStream as TokenStream2;
use syn::{DeriveInput, parse::Parser};

/// Expand `item` with the attribute arguments `args`, both parsed from
/// source text so that errors carry the location they point at.
pub(crate) fn expand(args: &str, item: &str) -> syn::Result<TokenStream2> {
    let args = crate::attrs::parse_items.parse_str(args)?;
    let item: DeriveInput = syn::parse_str(item)?;
    crate::expand(args, item)
}

/// The message of the expansion's error and the source text it points at.
pub(crate) fn error(args: &str, item: &str) -> (String, String) {
    match expand(args, item) {
        Ok(tokens) => panic!("expected a compile error, got {tokens}"),
        Err(error) => {
            (error.to_string(), error.span().source_text().unwrap_or_default())
        }
    }
}

#[track_caller]
pub(crate) fn assert_error(args: &str, item: &str, message: &str, at: &str) {
    assert_eq!(error(args, item), (message.to_owned(), at.to_owned()));
}

/// The expanded item, with its attributes, and the rest of the expansion.
pub(crate) fn expanded(args: &str, item: &str) -> (DeriveInput, String) {
    let tokens = expand(args, item).unwrap();
    let file: syn::File = syn::parse2(tokens).unwrap();
    let mut items = file.items.into_iter();
    let item: DeriveInput = match items.next() {
        Some(syn::Item::Enum(item)) => item.into(),
        Some(syn::Item::Struct(item)) => item.into(),
        other => panic!("not an item: {}", quote::quote!(#other)),
    };
    let rest: TokenStream2 = items.map(|item| quote::quote!(#item)).collect();
    (item, rest.to_string())
}

pub(crate) fn docs(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            syn::Meta::NameValue(value) if value.path.is_ident("doc") => {
                match &value.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(text),
                        ..
                    }) => Some(text.value()),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

pub(crate) fn deprecated_note(attrs: &[syn::Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("deprecated"))
        .map(|attr| quote::quote!(#attr).to_string())
}

fn variant_attrs(item: &DeriveInput, index: usize) -> &[syn::Attribute] {
    match &item.data {
        syn::Data::Enum(data) => &data.variants[index].attrs,
        _ => panic!("not an enum"),
    }
}

#[test]
fn unknown_keys_and_forms_are_errors_at_them() {
    assert_error(
        r#"colour = "red""#,
        "enum E { A }",
        "qapi: unknown key `colour`",
        "colour",
    );
    assert_error(
        "",
        r#"enum E { #[qapi(since = "1.0.0")] A }"#,
        "qapi: unknown key `since`",
        "since",
    );
    assert_error(
        "",
        "enum E { #[qapi(name = 3)] A }",
        "qapi: `name` takes a string literal",
        "3",
    );
    assert_error(
        "",
        r#"enum E { #[qapi(name = "a"x)] A }"#,
        "qapi: `name` takes a string literal",
        r#""a"x"#,
    );
    assert_error(
        "",
        "enum E { #[qapi(name)] A }",
        "qapi: expected `name = \"...\"`",
        "name",
    );
    assert_error(
        r#"name = """#,
        "enum E { A }",
        "qapi: the `name` of a type must not be empty",
        r#""""#,
    );
    for attribute in [
        "#[qapi]",
        r#"#[qapi = "a"]"#,
        r#"#[qapi[name = "a"]]"#,
        r#"#[qapi{name = "a"}]"#,
    ] {
        let (message, at) = error("", &format!("enum E {{ {attribute} A }}"));
        assert_eq!(message, "qapi: expected `#[qapi(key = \"value\", ...)]`");
        assert_eq!(format!("#[{at}]"), attribute);
    }
    assert_error(
        r#"unstable(since = "1.0.0")"#,
        "enum E { A }",
        "qapi: unknown key `since`",
        "since",
    );
    assert_error(
        r#"feature = "ordinary""#,
        "enum E { A }",
        "qapi: `feature` is `deprecated` or `unstable`; other QAPI features do not change the expansion",
        r#""ordinary""#,
    );
    assert_error(
        r#"feature = "deprecated", if = "CONFIG_X""#,
        "enum E { A }",
        "qapi: unknown key `if`",
        "if",
    );
    assert_error(
        "",
        r#"enum E { #[qapi(feature = "unstable", if = "CONFIG_X")] A }"#,
        "qapi: unknown key `if`",
        "if",
    );
    let (message, _) = error("", r#"enum E { #[qapi(name "a")] A }"#);
    assert!(message.starts_with("expected"), "{message}");
}

#[test]
fn versions_have_three_decimal_components_without_leading_zeros() {
    for text in [
        "abc", "9.2", "9.2.0.1", "9..0", "9.2.x", "+9.2.0", "", "09.2.0",
        "9.02.0", "9.2.00",
    ] {
        assert_eq!(Version::parse(text), None, "{text}");
        let item = format!(r#"enum E {{ #[qapi(added = "{text}")] A }}"#);
        assert_eq!(
            error("", &item),
            (
                "qapi: `added` must be a QEMU version such as \"9.2.0\""
                    .to_owned(),
                format!("{text:?}")
            )
        );
    }
    assert_eq!(Version::parse("10.1.3"), Some(Version(10, 1, 3)));
    assert_eq!(Version::parse("0.15.0"), Some(Version(0, 15, 0)));
    assert_eq!(Version(10, 1, 3).to_string(), "10.1.3");
}

#[test]
fn flags_of_the_newest_release_are_those_without_a_removal() {
    let holds = |range: &str| {
        let args = crate::attrs::parse_items.parse_str(range).unwrap();
        let ta = crate::attrs::TypeAttrs::parse(&args, &[]).unwrap();
        flag_at_newest(&ta.unstable)
    };
    assert!(holds(r#"unstable(added = "9.0.0")"#));
    assert!(holds(r#"unstable(added = "99.0.0")"#));
    assert!(!holds(r#"unstable(removed = "99.0.0")"#));
    assert!(holds(
        r#"unstable(removed = "9.0.0"), unstable(added = "10.0.0")"#
    ));
    assert!(!holds(""));
}

#[test]
fn every_key_is_checked() {
    let cases = [
        (
            r#"added = "9.0.0""#,
            "#[derive(Debug)] enum E { A }",
            "qapi: #[qapi] provides `Debug` itself",
        ),
        (
            r#"added = "9.0.0""#,
            "enum E { #[qapi(bogus = 1)] A }",
            "qapi: unknown key `bogus`",
        ),
        (
            r#"added = "9.0.0""#,
            "enum E { B { x: u32 } }",
            "qapi: enum variants are unit or wrap one type; use a struct for named fields",
        ),
        (
            "",
            r#"enum E { #[qapi(added = "9.0.0", removed = "8.0.0")] A }"#,
            "qapi: `removed` (8.0.0) must be later than `added` (9.0.0)",
        ),
        (
            r#"added = "9.0.0""#,
            "union U { a: u32 }",
            "qapi: #[qapi] does not support this kind of item",
        ),
        (
            r#"added = "9.0.0""#,
            "enum G<T> { A(T) }",
            "qapi: #[qapi] does not support generic items",
        ),
    ];
    for (args, item, message) in cases {
        assert_eq!(error(args, item).0, message, "{item}");
    }
}

#[test]
fn lifetimes_end_after_they_start_and_hold_their_deprecation() {
    assert_error(
        "",
        r#"enum E { #[qapi(added = "9.0.0", removed = "8.0.0")] A }"#,
        "qapi: `removed` (8.0.0) must be later than `added` (9.0.0)",
        r#"removed = "8.0.0""#,
    );
    assert_error(
        r#"added = "9.0.0""#,
        r#"#[qapi(removed = "9.0.0")] enum E { A }"#,
        "qapi: `removed` (9.0.0) must be later than `added` (9.0.0)",
        r#"removed = "9.0.0""#,
    );
    assert_error(
        r#"unstable(added = "9.0.0", removed = "8.0.0")"#,
        "enum E { A }",
        "qapi: `removed` (8.0.0) must be later than `added` (9.0.0)",
        r#"removed = "8.0.0""#,
    );
    assert_error(
        "",
        r#"enum E { #[qapi(added = "9.0.0", deprecated(added = "8.0.0"))] A }"#,
        "qapi: `deprecated` (8.0.0) must not be earlier than `added` (9.0.0)",
        r#"deprecated(added = "8.0.0")"#,
    );
    assert_error(
        "",
        r#"enum E { #[qapi(removed = "9.0.0")] #[qapi(deprecated(added = "9.0.0"))] A }"#,
        "qapi: `deprecated` (9.0.0) must be earlier than `removed` (9.0.0)",
        r#"deprecated(added = "9.0.0")"#,
    );
    // A deprecation in the release that adds the item is within it.
    let item = r#"enum E { #[qapi(added = "9.0.0", deprecated(added = "9.0.0"), removed = "9.1.0")] A }"#;
    assert!(expand("", item).is_ok());
    // A deprecation that ends does so after it starts and by the item's end,
    // and the next starts after it.
    assert_error(
        "",
        r#"enum E { #[qapi(deprecated(added = "9.0.0", removed = "9.0.0"))] A }"#,
        "qapi: `removed` (9.0.0) must be later than `added` (9.0.0)",
        r#"removed = "9.0.0""#,
    );
    assert_error(
        "",
        r#"enum E { #[qapi(removed = "9.1.0", deprecated(added = "9.0.0", removed = "9.2.0"))] A }"#,
        "qapi: a deprecation must end by `removed` (9.1.0), not at 9.2.0",
        r#"deprecated(added = "9.0.0", removed = "9.2.0")"#,
    );
    assert_error(
        "",
        r#"enum E { #[qapi(deprecated(added = "9.1.0"))] #[qapi(deprecated(added = "8.0.0", removed = "9.2.0"))] A }"#,
        "qapi: `deprecated` (9.1.0) must not be earlier than the `removed` (9.2.0) of the deprecation before it",
        r#"deprecated(added = "9.1.0")"#,
    );
    assert_error(
        "",
        r#"enum E { #[qapi(deprecated(added = "8.0.0"), deprecated(added = "9.1.0"))] A }"#,
        "qapi: `deprecated` (9.1.0) follows a deprecation without `removed`",
        r#"deprecated(added = "9.1.0")"#,
    );
    let item = r#"enum E { #[qapi(deprecated(added = "8.0.0", removed = "9.0.0"), deprecated(added = "9.0.0", removed = "9.1.0"), removed = "9.1.0")] A }"#;
    assert!(expand("", item).is_ok());
}

#[test]
fn generic_items_are_errors() {
    assert_error(
        "",
        "enum G<T> { A(T) }",
        "qapi: #[qapi] does not support generic items",
        "T",
    );
    assert_error(
        "",
        "enum G where u8: Copy { A }",
        "qapi: #[qapi] does not support generic items",
        "where u8: Copy",
    );
}

#[test]
fn unsupported_item_shapes_are_errors() {
    assert_error(
        "",
        "enum E { A(String, i64) }",
        "qapi: tuple enum variants must wrap exactly one type",
        "(String, i64)",
    );
    assert_error(
        "",
        "enum E { A { value: String } }",
        "qapi: enum variants are unit or wrap one type; use a struct for named fields",
        "{ value: String }",
    );
    assert_error(
        "",
        "union U { a: u32 }",
        "qapi: #[qapi] does not support this kind of item",
        "U",
    );
}

#[test]
fn repeated_keys_and_features_are_errors() {
    assert_error(
        "",
        r#"enum E { #[qapi(name = "a", name = "b")] A }"#,
        "qapi: duplicate key `name`",
        "name",
    );
    assert_error(
        "",
        r#"enum E { #[qapi(added = "1.0.0")] #[qapi(added = "2.0.0")] A }"#,
        "qapi: duplicate key `added`",
        "added",
    );
    assert_error(
        r#"unstable(added = "1.0.0", added = "2.0.0")"#,
        "enum E { A }",
        "qapi: duplicate key `added`",
        "added",
    );
    assert_error(
        "",
        r#"enum E { #[qapi(feature = "deprecated")] #[qapi(feature = "deprecated")] A }"#,
        "qapi: duplicate feature `deprecated`",
        r#""deprecated""#,
    );
    // `deprecated(...)`, `unstable(...)` and `feature` with different
    // features repeat.
    let args = r#"unstable(added = "1.0.0", removed = "2.0.0"), unstable(added = "3.0.0"),
        deprecated(added = "1.0.0", removed = "2.0.0"),
        feature = "deprecated", feature = "unstable""#;
    assert!(expand(args, "enum E { A }").is_ok());
    let args = r#"deprecated(added = "1.0.0", removed = "2.0.0")"#;
    assert!(
        expand(args, r#"#[qapi(deprecated(added = "3.0.0"))] enum E { A }"#)
            .is_ok()
    );
    let args = r#"unstable(added = "1.0.0", removed = "2.0.0"), unstable(added = "3.0.0"),
        feature = "deprecated", feature = "unstable""#;
    assert!(expand(args, "enum E { A }").is_ok());
}

#[test]
fn keys_without_an_effect_are_errors() {
    assert_error(
        "",
        "enum E { #[qapi(deprecated)] A }",
        "qapi: expected `deprecated(...)`",
        "deprecated",
    );
    assert_error(
        "",
        r#"enum E { #[qapi(deprecated(removed = "9.0.0"))] A }"#,
        "qapi: `deprecated(...)` needs `added`",
        r#"deprecated(removed = "9.0.0")"#,
    );
    assert_error(
        "unstable",
        "enum E { A }",
        "qapi: expected `unstable(...)`",
        "unstable",
    );
    assert_error(
        "unstable()",
        "enum E { A }",
        "qapi: `unstable(...)` needs `added` or `removed`",
        "unstable()",
    );
    // A union branch enum serializes inside its union, under no name.
    assert_error(
        r#"name = "Wire""#,
        "#[qapi(union_branch)] enum E { A }",
        "qapi: a union branch enum has no QAPI name; it serializes inside its union",
        r#""Wire""#,
    );
}

#[test]
fn representations_of_one_wire_name_have_disjoint_lifetimes() {
    let message = "qapi: wire name `x` is already taken in part of this lifetime; \
                   representations of one wire name need disjoint `added`/`removed` ranges";
    for item in [
        r#"enum E { #[qapi(name = "x")] A, #[qapi(name = "x")] B }"#,
        r#"enum E { #[qapi(name = "x", removed = "3.0.0")] A, #[qapi(name = "x", added = "2.0.0")] B }"#,
        r#"enum E { #[qapi(name = "x", added = "2.0.0", removed = "4.0.0")] A, #[qapi(name = "x", added = "3.0.0")] B }"#,
        r#"enum E { x, #[qapi(name = "x")] B }"#,
    ] {
        assert_eq!(
            error("", item),
            (message.to_owned(), "B".to_owned()),
            "{item}"
        );
    }
    let item = r#"enum E { #[qapi(name = "x", removed = "3.0.0")] A, #[qapi(name = "x", added = "3.0.0")] B }"#;
    assert!(expand("", item).is_ok());
}

#[test]
fn own_derives_of_the_expansions_traits_are_errors() {
    for (derive, name, at) in [
        ("#[derive(PartialEq, Debug)]", "Debug", "Debug"),
        ("#[derive(::core::clone::Clone)]", "Clone", "::core::clone::Clone"),
        ("#[derive(serde::Serialize)]", "Serialize", "serde::Serialize"),
        ("#[derive(Deserialize)]", "Deserialize", "Deserialize"),
    ] {
        assert_error(
            "",
            &format!("{derive} enum E {{ A }}"),
            &format!("qapi: #[qapi] provides `{name}` itself"),
            at,
        );
    }
    // A struct implements `Default` or has `new`; an enum may derive it.
    assert_error(
        "",
        "#[derive(Default)] struct S { a: Option<u8> }",
        "qapi: #[qapi] provides `Default` itself",
        "Default",
    );
    assert!(expand("", "#[derive(Default)] enum E { #[default] A }").is_ok());
    let (item, rest) = expanded("", "#[derive(PartialEq, Eq)] enum E { A }");
    assert!(rest.contains("Serialize for E"), "{rest}");
    let derives: Vec<_> = item
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .map(|attr| quote::quote!(#attr).to_string())
        .collect();
    assert_eq!(
        derives,
        [
            "# [derive (PartialEq , Eq)]",
            "# [derive (:: core :: fmt :: Debug , :: core :: clone :: Clone)]"
        ]
    );
}

#[test]
fn notes_follow_the_docs_as_a_paragraph_of_their_own() {
    let item = r#"
        /// A type.
        #[qapi(unstable(added = "9.0.0"))]
        #[qapi(condition = "CONFIG_X")]
        enum E {
            /// A variant.
            #[qapi(feature = "unstable", condition = "CONFIG_Y")]
            A,
            #[qapi(condition = "CONFIG_Z")]
            B,
        }"#;
    let (item, _) = expanded("", item);
    assert_eq!(
        docs(&item.attrs),
        [
            " A type.",
            "",
            " **Unstable** since QEMU 9.0.0 — this interface may change without notice.",
            " QEMU/guest build condition: `CONFIG_X`.",
        ]
    );
    assert_eq!(
        docs(variant_attrs(&item, 0)),
        [
            " A variant.",
            "",
            " **Unstable** — this variant may change without notice.",
            " QEMU/guest build condition: `CONFIG_Y`.",
        ]
    );
    // Without docs of its own an item gets no blank line first.
    assert_eq!(
        docs(variant_attrs(&item, 1)),
        [" QEMU/guest build condition: `CONFIG_Z`."]
    );
}

#[test]
fn unstable_types_are_documented_as_in_the_newest_release() {
    let doc = |args: &str| {
        let (item, _) = expanded(args, "enum E { A }");
        docs(&item.attrs)
    };
    let since = " **Unstable** since QEMU 9.0.0 — this interface may change without notice.";
    let plain = " **Unstable** — this interface may change without notice.";
    assert_eq!(doc(r#"unstable(added = "9.0.0")"#), [since]);
    assert!(doc(r#"unstable(removed = "10.0.0")"#).is_empty());
    assert_eq!(doc(r#"feature = "unstable""#), [plain]);
    // The release that began the range holding in the newest release.
    let ranges = r#"unstable(added = "5.0.0", removed = "10.0.0"), unstable(added = "11.0.0")"#;
    assert_eq!(
        doc(ranges),
        [
            " **Unstable** since QEMU 11.0.0 — this interface may change without notice."
        ]
    );
    // The versions decide where there are any.
    assert!(
        doc(r#"feature = "unstable", unstable(removed = "10.0.0")"#).is_empty()
    );
}

#[test]
fn deprecation_marks_what_the_newest_release_deprecates_or_no_longer_has() {
    // Without versions, the `deprecated` feature decides.
    let (item, _) = expanded(
        r#"feature = "deprecated", feature = "unstable""#,
        "enum E { A }",
    );
    assert_eq!(
        deprecated_note(&item.attrs).as_deref(),
        Some("# [deprecated (note = \"deprecated QAPI type\")]")
    );
    assert!(docs(&item.attrs)[0].contains("**Unstable**"));
    for split in [false, true] {
        let attrs = if split {
            r#"#[qapi(feature = "deprecated")] #[qapi(feature = "unstable")]"#
        } else {
            r#"#[qapi(feature = "deprecated", feature = "unstable")]"#
        };
        let (item, _) = expanded("", &format!("enum E {{ {attrs} A }}"));
        let attrs = variant_attrs(&item, 0);
        assert_eq!(
            deprecated_note(attrs).as_deref(),
            Some("# [deprecated (note = \"deprecated QAPI variant\")]")
        );
        assert!(docs(attrs)[0].contains("**Unstable**"), "{split}");
    }
    let note = |text: &str| Some(format!("# [deprecated (note = \"{text}\")]"));
    let notes = |item: &str| {
        let (item, rest) = expanded("", item);
        let notes: Vec<Option<String>> = (0..2)
            .map(|index| deprecated_note(variant_attrs(&item, index)))
            .collect();
        (notes, rest)
    };
    // A deprecation that lasts to the newest release: the version, not the
    // feature, decides, and it warns within its releases.
    let (found, rest) = notes(
        r#"enum E { #[qapi(feature = "deprecated", deprecated(added = "9.2.0"))] A, B }"#,
    );
    assert_eq!(found, [note("deprecated since QEMU 9.2.0"), None]);
    assert!(rest.contains("qapi warning: E::A is deprecated since QEMU 9.2.0"));
    // One that a later release dropped marks nothing, and it still warns
    // within its releases.
    let (found, rest) = notes(
        r#"enum E { #[qapi(deprecated(added = "9.0.0", removed = "10.0.0"))] A, B }"#,
    );
    assert_eq!(found, [None, None]);
    assert!(
        rest.contains(":: qapi :: qemu_version () < (10u32 , 0u32 , 0u32)"),
        "{rest}"
    );
    // What the newest release no longer has names its removal, whether or
    // not a deprecation came first; what a later release added is current.
    let (found, _) = notes(
        r#"enum E { #[qapi(removed = "9.1.0")] A, #[qapi(added = "9.1.0")] B }"#,
    );
    assert_eq!(found, [note("removed in QEMU 9.1.0"), None]);
    let (found, _) = notes(
        r#"enum E { #[qapi(deprecated(added = "8.0.0"), removed = "9.1.0")] A, B }"#,
    );
    assert_eq!(found, [note("removed in QEMU 9.1.0"), None]);
    // A type and a field the same way.
    let (item, _) = expanded(r#"removed = "9.1.0""#, "enum E { A }");
    assert_eq!(deprecated_note(&item.attrs), note("removed in QEMU 9.1.0"));
    let (item, _) = expanded(
        "",
        r#"struct S { #[qapi(removed = "9.1.0")] old: Option<u8>, new: Option<u8> }"#,
    );
    let syn::Data::Struct(data) = &item.data else {
        unreachable!("a struct expands to a struct");
    };
    let fields: Vec<Option<String>> =
        data.fields.iter().map(|field| deprecated_note(&field.attrs)).collect();
    assert_eq!(fields, [note("removed in QEMU 9.1.0"), None]);
}

#[test]
fn a_condition_is_not_empty() {
    assert_error(
        r#"condition = """#,
        "enum E { A }",
        "qapi: a `condition` must not be empty",
        r#""""#,
    );
    assert_error(
        "",
        r#"enum E { #[qapi(condition = "")] A }"#,
        "qapi: a `condition` must not be empty",
        r#""""#,
    );
}

#[test]
fn keys_go_in_qapi_attributes_of_the_item_or_member_itself() {
    for (item, at) in [
        (
            r#"enum E { #[qapi_macros::qapi(added = "1.0.0")] A }"#,
            "qapi_macros::qapi",
        ),
        (r#"#[qapi::qapi(added = "1.0.0")] enum E { A }"#, "qapi::qapi"),
    ] {
        assert_error(
            "",
            item,
            "qapi: write the keys of a type or member as `#[qapi(...)]`",
            at,
        );
    }
    for payload in [r#"#[qapi(added = "1.0.0")]"#, "#[qapi_macros::qapi]"] {
        assert_error(
            "",
            &format!("enum E {{ A({payload} u32) }}"),
            "qapi: the keys of a variant go on the variant, not on its payload",
            payload,
        );
    }
}

#[test]
fn a_member_lives_within_its_type() {
    let type_life = r#"added = "5.0.0", removed = "8.0.0""#;
    for (variant, message, at) in [
        (
            r#"added = "4.0.0""#,
            "qapi: `added` (4.0.0) must not be earlier than the type's `added` (5.0.0)",
            r#"added = "4.0.0""#,
        ),
        (
            r#"added = "8.0.0""#,
            "qapi: `added` (8.0.0) must be earlier than the type's `removed` (8.0.0)",
            r#"added = "8.0.0""#,
        ),
        (
            r#"removed = "5.0.0""#,
            "qapi: `removed` (5.0.0) must be later than the type's `added` (5.0.0)",
            r#"removed = "5.0.0""#,
        ),
        (
            r#"removed = "8.0.1""#,
            "qapi: `removed` (8.0.1) must not be later than the type's `removed` (8.0.0)",
            r#"removed = "8.0.1""#,
        ),
    ] {
        assert_error(
            type_life,
            &format!("enum E {{ #[qapi({variant})] A, B }}"),
            message,
            at,
        );
    }
    // The lifetime of a member may start and end with its type's.
    let item = r#"enum E { #[qapi(added = "5.0.0", removed = "8.0.0")] A, B }"#;
    assert!(expand(type_life, item).is_ok());
    let item =
        r#"enum E { #[qapi(added = "1.0.0", removed = "99.0.0")] A, B }"#;
    assert!(expand("", item).is_ok());
}

#[test]
fn the_ranges_of_a_flag_lie_within_the_lifetime_and_do_not_overlap() {
    let type_life = r#"added = "5.0.0", removed = "8.0.0", "#;
    for (range, message) in [
        (
            r#"unstable(added = "4.0.0")"#,
            "qapi: `unstable` (added 4.0.0) must not start before the type's `added` (5.0.0)",
        ),
        (
            r#"unstable(added = "8.0.0")"#,
            "qapi: `unstable` (added 8.0.0) must start before the type's `removed` (8.0.0)",
        ),
        (
            r#"unstable(removed = "5.0.0")"#,
            "qapi: `unstable` (removed 5.0.0) must end after the type's `added` (5.0.0)",
        ),
        (
            r#"unstable(removed = "9.0.0")"#,
            "qapi: `unstable` (removed 9.0.0) must end by the type's `removed` (8.0.0)",
        ),
    ] {
        assert_error(
            &format!("{type_life}{range}"),
            "enum E { A }",
            message,
            "unstable",
        );
    }
    for (ranges, message) in [
        (
            r#"unstable(added = "1.0.0"), unstable(added = "2.0.0")"#,
            "qapi: `unstable` overlaps the `unstable` before it, which does not end",
        ),
        (
            r#"unstable(added = "3.0.0"), unstable(added = "1.0.0", removed = "4.0.0")"#,
            "qapi: `unstable` must not start before the `removed` (4.0.0) of the `unstable` before it",
        ),
        (
            r#"unstable(removed = "2.0.0"), unstable(removed = "4.0.0")"#,
            "qapi: `unstable` must not start before the `removed` (2.0.0) of the `unstable` before it",
        ),
    ] {
        assert_error(ranges, "enum E { A }", message, "unstable");
    }
    let ranges = r#"unstable(removed = "2.0.0"), unstable(added = "2.0.0", removed = "3.0.0"),
        unstable(added = "5.0.0")"#;
    assert!(expand(ranges, "enum E { A }").is_ok());
    let ranges = r#"added = "5.0.0", removed = "8.0.0", unstable(added = "5.0.0", removed = "8.0.0")"#;
    assert!(expand(ranges, "enum E { A }").is_ok());
}

#[test]
fn a_payload_declares_no_use_of_a_deprecated_type() {
    let (item, _) = expanded("", r#"enum E { #[qapi(name = "a")] A(Old), B }"#);
    let allows = |attrs: &[syn::Attribute]| {
        attrs.iter().any(|attr| {
            quote::quote!(#attr).to_string() == "# [allow (deprecated)]"
        })
    };
    assert!(allows(variant_attrs(&item, 0)));
    assert!(!allows(variant_attrs(&item, 1)));
}
