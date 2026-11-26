//! The attribute's compile errors, which these tests read with the source
//! text they point at, and the parts of the expansion that only the
//! attributes it emits show: documentation, deprecation and the bounds.

use crate::bounds::{
    VersionBounds, bound, checked_bounds, excluded_by_bounds, report_once,
    rust_deprecated_by_bounds,
};
use crate::support::{Version, flag_at_newest};
use proc_macro2::TokenStream as TokenStream2;
use std::sync::atomic::AtomicBool;
use syn::DeriveInput;
use syn::parse::Parser;

fn version(text: &str) -> Version {
    Version::parse(text).unwrap()
}

fn bounds(min: Option<&str>, max: Option<&str>) -> VersionBounds {
    VersionBounds {
        min: min.map(version),
        max: max.map(version),
    }
}

/// Expand `item` with the attribute arguments `args`, both parsed from
/// source text so that errors carry the location they point at.
pub(crate) fn expand(args: &str, item: &str, bounds: &VersionBounds) -> syn::Result<TokenStream2> {
    let args = crate::attrs::parse_items.parse_str(args)?;
    let item: DeriveInput = syn::parse_str(item)?;
    crate::expand(args, item, bounds)
}

/// The message of the expansion's error and the source text it points at.
pub(crate) fn error_in(args: &str, item: &str, bounds: &VersionBounds) -> (String, String) {
    match expand(args, item, bounds) {
        Ok(tokens) => panic!("expected a compile error, got {tokens}"),
        Err(error) => (
            error.to_string(),
            error.span().source_text().unwrap_or_default(),
        ),
    }
}

pub(crate) fn error(args: &str, item: &str) -> (String, String) {
    error_in(args, item, &VersionBounds::default())
}

#[track_caller]
pub(crate) fn assert_error(args: &str, item: &str, message: &str, at: &str) {
    assert_eq!(error(args, item), (message.to_owned(), at.to_owned()));
}

/// The expanded item, with its attributes, and the rest of the expansion.
pub(crate) fn expanded(args: &str, item: &str, bounds: &VersionBounds) -> (DeriveInput, String) {
    let tokens = expand(args, item, bounds).unwrap();
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
            syn::Meta::NameValue(value) if value.path.is_ident("doc") => match &value.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(text),
                    ..
                }) => Some(text.value()),
                _ => None,
            },
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
        "abc", "9.2", "9.2.0.1", "9..0", "9.2.x", "+9.2.0", "", "09.2.0", "9.02.0", "9.2.00",
    ] {
        assert_eq!(Version::parse(text), None, "{text}");
        let item = format!(r#"enum E {{ #[qapi(added = "{text}")] A }}"#);
        assert_eq!(
            error("", &item),
            (
                "qapi: `added` must be a QEMU version such as \"9.2.0\"".to_owned(),
                format!("{text:?}")
            )
        );
    }
    assert_eq!(Version::parse("10.1.3"), Some(Version(10, 1, 3)));
    assert_eq!(Version::parse("0.15.0"), Some(Version(0, 15, 0)));
    assert_eq!(Version(10, 1, 3).to_string(), "10.1.3");
}

#[test]
fn invalid_version_bounds_name_their_variable_once() {
    assert_eq!(
        bound("QEMU_MIN_VERSION", Ok("9.2.0".into())).unwrap(),
        Some(Version(9, 2, 0))
    );
    assert_eq!(bound("QEMU_MIN_VERSION", Ok(String::new())).unwrap(), None);
    assert_eq!(
        bound("QEMU_MIN_VERSION", Err(std::env::VarError::NotPresent)).unwrap(),
        None
    );
    let error = bound("QEMU_MAX_VERSION", Ok("nine".into())).unwrap_err();
    assert_eq!(
        error.to_string(),
        "qapi: QEMU_MAX_VERSION must be a QEMU version such as 9.2.0, found \"nine\""
    );
    let error = checked_bounds(bounds(Some("11.0.0"), Some("9.2.0")))
        .err()
        .unwrap();
    assert_eq!(
        error.to_string(),
        "qapi: QEMU_MIN_VERSION 11.0.0 is later than QEMU_MAX_VERSION 9.2.0"
    );
    // Only the first expansion of a build reports an invalid bound; every
    // expansion goes on without bounds.
    let reported = AtomicBool::new(false);
    let invalid = || bound("QEMU_MIN_VERSION", Ok("x".into())).map(|_| VersionBounds::default());
    let (first, error) = report_once(invalid(), &reported);
    assert!(error.is_some() && first.min.is_none() && first.max.is_none());
    let (second, error) = report_once(invalid(), &reported);
    assert!(error.is_none() && second.min.is_none());
    let (valid, error) = report_once(Ok(bounds(Some("9.2.0"), None)), &reported);
    assert!(error.is_none() && valid.min == Some(Version(9, 2, 0)));
}

#[test]
fn the_bounds_leave_out_what_no_release_of_their_range_has() {
    let min = bounds(Some("9.2.0"), None);
    assert!(excluded_by_bounds(&min, None, Some(version("9.2.0"))));
    assert!(!excluded_by_bounds(&min, None, Some(version("9.2.1"))));
    assert!(!excluded_by_bounds(&min, Some(version("11.0.0")), None));
    let max = bounds(None, Some("9.2.0"));
    assert!(!excluded_by_bounds(&max, Some(version("9.2.0")), None));
    assert!(excluded_by_bounds(&max, Some(version("9.2.1")), None));
    assert!(!excluded_by_bounds(&max, None, Some(version("1.0.0"))));
    assert!(!excluded_by_bounds(
        &VersionBounds::default(),
        Some(version("99.0.0")),
        Some(version("99.1.0"))
    ));

    assert!(rust_deprecated_by_bounds(&min, version("9.2.0"), None));
    assert!(rust_deprecated_by_bounds(&min, version("9.1.0"), None));
    assert!(!rust_deprecated_by_bounds(&min, version("9.2.1"), None));
    assert!(!rust_deprecated_by_bounds(&max, version("1.0.0"), None));
    // A deprecation that ends covers the build only if it ends after the
    // maximum.
    let both = bounds(Some("9.2.0"), Some("10.0.0"));
    assert!(rust_deprecated_by_bounds(
        &both,
        version("9.0.0"),
        Some(version("10.1.0"))
    ));
    assert!(!rust_deprecated_by_bounds(
        &both,
        version("9.0.0"),
        Some(version("10.0.0"))
    ));
    assert!(!rust_deprecated_by_bounds(
        &min,
        version("9.0.0"),
        Some(version("99.0.0"))
    ));

    // Variants, and the whole item, follow the same rule.
    let item = r#"enum E {
        A,
        #[qapi(removed = "9.2.0")] Gone,
        #[qapi(removed = "9.2.1")] Kept,
    }"#;
    let (item, _) = expanded("", item, &min);
    let syn::Data::Enum(data) = &item.data else {
        panic!("not an enum");
    };
    let names: Vec<_> = data.variants.iter().map(|v| v.ident.to_string()).collect();
    assert_eq!(names, ["A", "Kept"]);
    let item = "enum E { A }";
    assert!(expand(r#"added = "9.2.1""#, item, &max).unwrap().is_empty());
    assert!(!expand(r#"added = "9.2.0""#, item, &max).unwrap().is_empty());
}

#[test]
fn flags_of_the_newest_release_follow_the_maximum() {
    let holds = |range: &str, max: Option<&str>| {
        let args = crate::attrs::parse_items.parse_str(range).unwrap();
        let ta = crate::attrs::TypeAttrs::parse(&args, &[]).unwrap();
        flag_at_newest(&ta.unstable, &bounds(None, max))
    };
    let max = Some("9.0.0");
    assert!(holds(r#"unstable(added = "9.0.0")"#, max));
    assert!(!holds(r#"unstable(added = "9.0.1")"#, max));
    assert!(!holds(r#"unstable(removed = "9.0.0")"#, max));
    assert!(holds(r#"unstable(removed = "9.0.1")"#, max));
    assert!(holds(r#"unstable(added = "99.0.0")"#, None));
    assert!(!holds(r#"unstable(removed = "99.0.0")"#, None));
    assert!(!holds("", None));
}

#[test]
fn every_key_is_checked_whatever_the_bounds_leave_out() {
    let old = bounds(None, Some("8.0.0"));
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
        for bounds in [&old, &VersionBounds::default()] {
            assert_eq!(error_in(args, item, bounds).0, message, "{item}");
        }
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
    let item =
        r#"enum E { #[qapi(added = "9.0.0", deprecated(added = "9.0.0"), removed = "9.1.0")] A }"#;
    assert!(expand("", item, &VersionBounds::default()).is_ok());
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
    assert!(expand("", item, &VersionBounds::default()).is_ok());
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
    assert!(expand(args, "enum E { A }", &VersionBounds::default()).is_ok());
    let args = r#"deprecated(added = "1.0.0", removed = "2.0.0")"#;
    assert!(
        expand(
            args,
            r#"#[qapi(deprecated(added = "3.0.0"))] enum E { A }"#,
            &VersionBounds::default()
        )
        .is_ok()
    );
    let args = r#"unstable(added = "1.0.0", removed = "2.0.0"), unstable(added = "3.0.0"),
        feature = "deprecated", feature = "unstable""#;
    assert!(expand(args, "enum E { A }", &VersionBounds::default()).is_ok());
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
    assert!(expand("", item, &VersionBounds::default()).is_ok());
}

#[test]
fn own_derives_of_the_expansions_traits_are_errors() {
    for (derive, name, at) in [
        ("#[derive(PartialEq, Debug)]", "Debug", "Debug"),
        (
            "#[derive(::core::clone::Clone)]",
            "Clone",
            "::core::clone::Clone",
        ),
        (
            "#[derive(serde::Serialize)]",
            "Serialize",
            "serde::Serialize",
        ),
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
    assert!(
        expand(
            "",
            "#[derive(Default)] enum E { #[default] A }",
            &VersionBounds::default()
        )
        .is_ok()
    );
    let (item, rest) = expanded(
        "",
        "#[derive(PartialEq, Eq)] enum E { A }",
        &VersionBounds::default(),
    );
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
    let (item, _) = expanded("", item, &VersionBounds::default());
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
fn unstable_types_are_documented_as_in_the_newest_release_of_the_bounds() {
    let doc = |args: &str, max: Option<&str>| {
        let (item, _) = expanded(args, "enum E { A }", &bounds(None, max));
        docs(&item.attrs)
    };
    let since = " **Unstable** since QEMU 9.0.0 — this interface may change without notice.";
    let plain = " **Unstable** — this interface may change without notice.";
    assert!(doc(r#"unstable(added = "9.0.0")"#, Some("8.0.0")).is_empty());
    assert_eq!(doc(r#"unstable(added = "9.0.0")"#, Some("9.0.0")), [since]);
    assert_eq!(doc(r#"unstable(added = "9.0.0")"#, None), [since]);
    assert_eq!(
        doc(r#"unstable(removed = "10.0.0")"#, Some("9.2.0")),
        [plain]
    );
    assert!(doc(r#"unstable(removed = "10.0.0")"#, None).is_empty());
    assert_eq!(doc(r#"feature = "unstable""#, None), [plain]);
    // The release that the range holding at the maximum began.
    let ranges = r#"unstable(added = "5.0.0", removed = "10.0.0"), unstable(added = "11.0.0")"#;
    assert_eq!(
        doc(ranges, Some("9.0.0")),
        [" **Unstable** since QEMU 5.0.0 — this interface may change without notice."]
    );
    assert_eq!(
        doc(ranges, None),
        [" **Unstable** since QEMU 11.0.0 — this interface may change without notice."]
    );
    // The versions decide where there are any.
    assert!(
        doc(
            r#"feature = "unstable", unstable(removed = "10.0.0")"#,
            None
        )
        .is_empty()
    );
}

#[test]
fn deprecation_marks_types_and_variants_by_feature_or_minimum() {
    let (item, _) = expanded(
        r#"feature = "deprecated", feature = "unstable""#,
        "enum E { A }",
        &VersionBounds::default(),
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
        let (item, _) = expanded(
            "",
            &format!("enum E {{ {attrs} A }}"),
            &VersionBounds::default(),
        );
        let attrs = variant_attrs(&item, 0);
        assert_eq!(
            deprecated_note(attrs).as_deref(),
            Some("# [deprecated (note = \"deprecated QAPI variant\")]")
        );
        assert!(docs(attrs)[0].contains("**Unstable**"), "{split}");
    }
    let item = r#"enum E { #[qapi(feature = "deprecated", deprecated(added = "9.2.0"))] A }"#;
    for (min, deprecated) in [("9.2.0", true), ("9.1.0", false), ("10.0.0", true)] {
        let (item, rest) = expanded("", item, &bounds(Some(min), None));
        // The version, not the feature, decides.
        let note = deprecated_note(variant_attrs(&item, 0));
        assert_eq!(note.is_some(), deprecated, "{min}: {note:?}");
        assert!(note.is_none_or(|note| note.contains("deprecated since QEMU 9.2.0")));
        assert!(rest.contains("qapi warning: E::A is deprecated since QEMU 9.2.0"));
    }
    // A deprecation that a later release drops is Rust `#[deprecated]` only
    // where every release of the bounds deprecates it, and it warns only
    // within its releases.
    let item = r#"enum E { #[qapi(deprecated(added = "9.0.0", removed = "10.0.0"))] A }"#;
    for (max, deprecated) in [
        (Some("9.2.0"), true),
        (Some("10.0.0"), false),
        (None, false),
    ] {
        let (item, rest) = expanded("", item, &bounds(Some("9.1.0"), max));
        assert_eq!(
            deprecated_note(variant_attrs(&item, 0)).is_some(),
            deprecated,
            "{max:?}"
        );
        assert!(
            rest.contains(":: qapi :: qemu_version () < (10u32 , 0u32 , 0u32)"),
            "{rest}"
        );
    }
}

#[test]
fn decoding_errors_name_the_bound_that_the_peers_release_lies_outside() {
    // What the bounds leave out of a release outside them is no error of
    // the peer's: the errors that the release decides say so.
    let earlier = ", which is earlier than this build's QEMU_MIN_VERSION 3.0.0";
    let later = ", which is later than this build's QEMU_MAX_VERSION 5.0.0";
    let items = [
        r#"enum E { #[qapi(removed = "3.0.0")] A, B }"#,
        r#"#[qapi(union_branch)] enum E { #[qapi(removed = "3.0.0")] A, B }"#,
        r#"enum E { A(u8), B(String) }"#,
    ];
    for item in items {
        for (bounds, clauses) in [
            (bounds(Some("3.0.0"), Some("5.0.0")), [true, true]),
            (bounds(Some("3.0.0"), None), [true, false]),
            (bounds(None, Some("5.0.0")), [false, true]),
            (VersionBounds::default(), [false, false]),
        ] {
            let (_, rest) = expanded(r#"added = "1.0.0""#, item, &bounds);
            for (clause, given) in [earlier, later].into_iter().zip(clauses) {
                // Each decoding error that the version decides carries it,
                // the type's and each variant's unavailability among them.
                let count = rest.matches(clause).count();
                assert_eq!(count > 0, given, "{item}: {clause}: {rest}");
                let unavailable = rest.matches("unavailable_type_message").count();
                assert!(!given || count > unavailable, "{item}: {clause}: {rest}");
            }
        }
    }
    // The bounds themselves are releases within them.
    let (_, rest) = expanded("", "enum E { A }", &bounds(Some("3.0.0"), Some("5.0.0")));
    for comparison in [
        ":: qapi :: scoped_qemu_version () < :: core :: option :: Option :: Some ((3u32 , 0u32 , 0u32))",
        ":: qapi :: scoped_qemu_version () > :: core :: option :: Option :: Some ((5u32 , 0u32 , 0u32))",
    ] {
        assert!(rest.contains(comparison), "{comparison}: {rest}");
    }
    // An unknown wire name names the release instead of the variants.
    let (_, rest) = expanded("", "enum E { A }", &bounds(Some("3.0.0"), None));
    assert!(
        rest.contains(r#""unknown variant `{}` in QEMU {}.{}.{}{}""#),
        "{rest}"
    );
    assert!(rest.contains(r#""unknown variant `{}`, {}""#), "{rest}");
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
        (
            r#"#[qapi::qapi(added = "1.0.0")] enum E { A }"#,
            "qapi::qapi",
        ),
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
    assert!(expand(type_life, item, &VersionBounds::default()).is_ok());
    let item = r#"enum E { #[qapi(added = "1.0.0", removed = "99.0.0")] A, B }"#;
    assert!(expand("", item, &VersionBounds::default()).is_ok());
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
    assert!(expand(ranges, "enum E { A }", &VersionBounds::default()).is_ok());
    let ranges =
        r#"added = "5.0.0", removed = "8.0.0", unstable(added = "5.0.0", removed = "8.0.0")"#;
    assert!(expand(ranges, "enum E { A }", &VersionBounds::default()).is_ok());
}

#[test]
fn a_payload_declares_no_use_of_a_deprecated_type() {
    let (item, _) = expanded(
        "",
        r#"enum E { #[qapi(name = "a")] A(Old), B }"#,
        &VersionBounds::default(),
    );
    let allows = |attrs: &[syn::Attribute]| {
        attrs
            .iter()
            .any(|attr| quote::quote!(#attr).to_string() == "# [allow (deprecated)]")
    };
    assert!(allows(variant_attrs(&item, 0)));
    assert!(!allows(variant_attrs(&item, 1)));
}
