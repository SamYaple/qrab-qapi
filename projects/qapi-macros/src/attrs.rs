//! The `#[qapi(...)]` attribute model and its parser.
//!
//! The keys are the interface between `qrab-qapi-codegen` and this crate, so
//! a key, value or form that the parser does not know is a compile error
//! spanned at it rather than something to skip. Values are string literals
//! without a suffix, and versions are `MAJOR.MINOR.PATCH` ones.
//!
//! On a type, in the attribute's arguments or in further `#[qapi(...)]`
//! attributes of the item:
//!
//! - `name = "Name"` (the QAPI name, where it is not the Rust one),
//!   `added = "V"`, `removed = "V"`, `condition = "C"`;
//! - the repeatable `deprecated(added = "V", removed = "V")` (`removed`
//!   may be left out) and `unstable(added = "V", removed = "V")` (either
//!   bound may be left out);
//! - the repeatable `feature = "deprecated"` and `feature = "unstable"`:
//!   QAPI's built-in features;
//! - on structs: `command`, `event`, the repeatable `simple_union`, and on
//!   commands `returns = "Type"`, the repeatable `returns(added = "V", value
//!   = "Type")` with `returns_enum = "Name"`, the name of the enum of those
//!   return types, and the repeatable flags `allow_oob`, `allow_preconfig`
//!   and `no_success_response`; each flag and `simple_union` is bare or has
//!   a version range;
//! - on enums: `union_branch`.
//!
//! On a member of a type: `name = "wire"`, `added`, `removed`,
//! `condition`, `deprecated(...)` and `feature` as above.
//! A struct field also takes `flatten`, the repeatable `discriminator`,
//! `union` and `required`. A bare `discriminator` makes the field its
//! union's discriminator in every release that has it, and
//! `discriminator(added = "V", removed = "V")` (either bound may be left
//! out, to start or end with the field) in that run of releases only, as
//! `CpuInfoFast.arch` is until 3.0, where `target` takes over while `arch`
//! lasts until 6.0. A bound of a run is not the field's own, which leaving
//! it out gives. The runs lie within the field's lifetime and the union
//! field's, and in each release of the union field exactly one run is the
//! discriminator.
//! A `flatten`ed field has no member of its own, so it takes
//! only `added`, `removed` and `union`.
//!
//! Only the keys called repeatable above may appear more than once on one
//! item, counting all of its `#[qapi(...)]` attributes together, and a
//! feature only once. A `condition` is not empty. A lifetime ends after it
//! starts, and a member's lies within its type's. Each deprecation lies
//! within its item's lifetime, after the deprecations before it, and so
//! does each range of a type's flag, such as `unstable`, after the ranges
//! of that flag before it; a bare flag holds for the whole lifetime.
//! Further keys are written `#[qapi(...)]`, on the type or member they
//! describe, and never on the payload of a variant.
//!
//! The features apply where no version says more: `feature = "deprecated"`
//! without `deprecated(...)` makes the item Rust `#[deprecated]`, and
//! `feature = "unstable"` on a type without `unstable(...)` documents it as
//! unstable, as it does a member. QAPI's other features, and the build
//! conditions of features, would change nothing in the expansion, so they
//! are errors like any other unknown key or value.

use crate::support::Version;
use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Expr, ExprLit, Ident, Lit, LitStr, MacroDelimiter, Meta, MetaList, MetaNameValue, Path, Token,
    parenthesized,
};

/// The lifetime of a flag such as `unstable`: from `added` (or the start)
/// until `removed` (or for good). A bare flag has neither.
pub(crate) struct AttrRange {
    /// The key, for errors.
    pub(crate) span: Span,
    pub(crate) added: Option<Version>,
    pub(crate) removed: Option<Version>,
}

/// A run of releases that deprecate an item: from `added` until `removed`,
/// or while the item lasts.
pub(crate) struct Deprecation {
    /// The `deprecated(...)` item, for errors.
    span: Span,
    pub(crate) added: Version,
    pub(crate) removed: Option<Version>,
}

/// The keys of a type. A flag holds the span of its key.
#[derive(Default)]
pub(crate) struct TypeAttrs {
    pub(crate) name: Option<LitStr>,
    pub(crate) since: Option<Version>,
    pub(crate) until: Option<Version>,
    pub(crate) deprecated: Vec<Deprecation>,
    pub(crate) unstable: Vec<AttrRange>,
    pub(crate) condition: Option<String>,
    pub(crate) features: Vec<String>,
    pub(crate) returns: Option<LitStr>,
    pub(crate) returns_history: Vec<(Version, LitStr)>,
    pub(crate) returns_enum: Option<LitStr>,
    pub(crate) allow_oob: Vec<AttrRange>,
    pub(crate) allow_preconfig: Vec<AttrRange>,
    pub(crate) no_success_response: Vec<AttrRange>,
    pub(crate) command: Option<Span>,
    pub(crate) event: Option<Span>,
    pub(crate) union_branch: Option<Span>,
    pub(crate) simple_union: Vec<AttrRange>,
    spans: LifetimeSpans,
}

/// The keys of a member of a type.
#[derive(Default)]
pub(crate) struct ItemAttrs {
    pub(crate) name: Option<String>,
    pub(crate) since: Option<Version>,
    pub(crate) until: Option<Version>,
    pub(crate) deprecated: Vec<Deprecation>,
    pub(crate) condition: Option<String>,
    pub(crate) features: Vec<String>,
    /// The flags of a field, each the span of its key.
    pub(crate) flatten: Option<Span>,
    /// The runs of releases in which the field is its union's
    /// discriminator; a bare `discriminator` is one without bounds.
    pub(crate) discriminator: Vec<AttrRange>,
    pub(crate) union_field: Option<Span>,
    pub(crate) required: Option<Span>,
    spans: LifetimeSpans,
}

/// Where `added` and `removed` were given, for their errors.
#[derive(Default)]
struct LifetimeSpans {
    added: Option<Span>,
    removed: Option<Span>,
}

impl ItemAttrs {
    pub(crate) fn has_feature(&self, name: &str) -> bool {
        self.features.iter().any(|feature| feature == name)
    }

    /// The first key that only a struct field takes, for a variant's error.
    pub(crate) fn field_key(&self) -> Option<(Span, &'static str)> {
        [
            (self.flatten, "flatten"),
            (
                self.discriminator.first().map(|range| range.span),
                "discriminator",
            ),
            (self.union_field, "union"),
            (self.required, "required"),
        ]
        .into_iter()
        .find_map(|(span, key)| Some((span?, key)))
    }
}

/// The comma-separated items of a `#[qapi(...)]` attribute or a nested list.
pub(crate) type MetaArgs = Punctuated<Meta, Token![,]>;

/// Parse comma-separated attribute items. Unlike `Meta`'s own parser, this
/// reads a Rust keyword as a key, so that one such as `if` is reported as an
/// unknown key rather than a syntax error.
pub(crate) fn parse_items(input: ParseStream) -> syn::Result<MetaArgs> {
    Punctuated::parse_terminated_with(input, parse_item)
}

fn parse_item(input: ParseStream) -> syn::Result<Meta> {
    let path = Path::from(Ident::parse_any(input)?);
    if input.peek(Token![=]) {
        Ok(Meta::NameValue(MetaNameValue {
            path,
            eq_token: input.parse()?,
            value: input.parse()?,
        }))
    } else if input.peek(syn::token::Paren) {
        let content;
        let paren = parenthesized!(content in input);
        Ok(Meta::List(MetaList {
            path,
            delimiter: MacroDelimiter::Paren(paren),
            tokens: content.parse()?,
        }))
    } else {
        Ok(Meta::Path(path))
    }
}

/// The items of a list such as `qapi(...)` or `allow_oob(...)`.
pub(crate) fn list_items(list: &MetaList) -> syn::Result<MetaArgs> {
    list.parse_args_with(parse_items)
}

/// The `#[qapi(...)]` attributes among `attrs`, parsed into their items.
/// The attribute's keys are written `#[qapi(...)]` alone: under a path such
/// as `#[qapi_macros::qapi(...)]` they would be another expansion of the
/// attribute, or on a member an attribute that nothing reads.
pub(crate) fn qapi_attr_items(
    attrs: &[syn::Attribute],
) -> impl Iterator<Item = syn::Result<MetaArgs>> + '_ {
    attrs
        .iter()
        .filter(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|last| last.ident == "qapi")
        })
        .map(|attr| match &attr.meta {
            _ if !attr.path().is_ident("qapi") => Err(syn::Error::new_spanned(
                attr.path(),
                "qapi: write the keys of a type or member as `#[qapi(...)]`",
            )),
            Meta::List(list) if matches!(list.delimiter, MacroDelimiter::Paren(_)) => {
                list_items(list)
            }
            meta => Err(syn::Error::new_spanned(
                meta,
                "qapi: expected `#[qapi(key = \"value\", ...)]`",
            )),
        })
}

/// The key of an attribute item, as written (`if` included).
fn key(meta: &Meta) -> syn::Result<String> {
    meta.path()
        .get_ident()
        .map(|ident| ident.unraw().to_string())
        .ok_or_else(|| syn::Error::new_spanned(meta.path(), "qapi: expected a key"))
}

fn unknown(meta: &Meta, key: &str) -> syn::Error {
    syn::Error::new_spanned(meta.path(), format!("qapi: unknown key `{key}`"))
}

/// Store the value of a key that may appear only once.
fn once<T>(slot: &mut Option<T>, meta: &Meta, value: T) -> syn::Result<()> {
    if slot.is_some() {
        return Err(duplicate(meta));
    }
    *slot = Some(value);
    Ok(())
}

/// Set a bare flag that may appear only once.
fn once_flag(slot: &mut Option<Span>, meta: &Meta, key: &str) -> syn::Result<()> {
    flag(meta, key)?;
    once(slot, meta, meta.path().span())
}

fn duplicate(meta: &Meta) -> syn::Error {
    let key = meta
        .path()
        .get_ident()
        .map(|ident| ident.unraw().to_string());
    syn::Error::new_spanned(
        meta.path(),
        format!("qapi: duplicate key `{}`", key.unwrap_or_default()),
    )
}

/// The string literal of `key = "value"`.
fn lit_str<'a>(meta: &'a Meta, key: &str) -> syn::Result<&'a LitStr> {
    match meta {
        Meta::NameValue(MetaNameValue {
            value:
                Expr::Lit(ExprLit {
                    lit: Lit::Str(text),
                    ..
                }),
            ..
        }) if text.suffix().is_empty() => Ok(text),
        Meta::NameValue(nv) => Err(syn::Error::new_spanned(
            &nv.value,
            format!("qapi: `{key}` takes a string literal"),
        )),
        _ => Err(syn::Error::new_spanned(
            meta,
            format!("qapi: expected `{key} = \"...\"`"),
        )),
    }
}

/// The string of a type's `name = "..."`, which must not be empty. A member
/// may have an empty name, as the legacy enum member `''` of old schemas.
fn type_name(meta: &Meta) -> syn::Result<&LitStr> {
    let text = lit_str(meta, "name")?;
    if text.value().is_empty() {
        return Err(syn::Error::new_spanned(
            text,
            "qapi: the `name` of a type must not be empty",
        ));
    }
    Ok(text)
}

/// The build condition of `condition = "..."`, which must not be empty.
fn condition(meta: &Meta) -> syn::Result<String> {
    let text = lit_str(meta, "condition")?;
    if text.value().is_empty() {
        return Err(syn::Error::new_spanned(
            text,
            "qapi: a `condition` must not be empty",
        ));
    }
    Ok(text.value())
}

/// The version of `key = "MAJOR.MINOR.PATCH"`.
fn version(meta: &Meta, key: &str) -> syn::Result<Version> {
    let text = lit_str(meta, key)?;
    Version::parse(&text.value()).ok_or_else(|| {
        syn::Error::new_spanned(
            text,
            format!("qapi: `{key}` must be a QEMU version such as \"9.2.0\""),
        )
    })
}

/// A bare flag such as `union_branch`.
fn flag(meta: &Meta, key: &str) -> syn::Result<()> {
    match meta {
        Meta::Path(_) => Ok(()),
        _ => Err(syn::Error::new_spanned(
            meta,
            format!("qapi: `{key}` takes no value"),
        )),
    }
}

/// The items of `key(...)`.
fn nested<'a>(meta: &'a Meta, key: &str) -> syn::Result<(&'a MetaList, MetaArgs)> {
    match meta {
        Meta::List(list) => Ok((list, list_items(list)?)),
        _ => Err(syn::Error::new_spanned(
            meta,
            format!("qapi: expected `{key}(...)`"),
        )),
    }
}

/// An error at `removed` unless it is later than `added`.
fn check_order(
    added: Option<Version>,
    removed: Option<Version>,
    span: Option<Span>,
) -> syn::Result<()> {
    if let (Some(added), Some(removed), Some(span)) = (added, removed, span)
        && removed <= added
    {
        return Err(syn::Error::new(
            span,
            format!("qapi: `removed` ({removed}) must be later than `added` ({added})"),
        ));
    }
    Ok(())
}

/// Parse `key(added = "X", removed = "Y")`; either bound may be left out,
/// but not both.
fn parse_attr_range(meta: &Meta, key: &str) -> syn::Result<AttrRange> {
    let (list, items) = nested(meta, key)?;
    let mut range = AttrRange {
        span: meta.path().span(),
        added: None,
        removed: None,
    };
    let mut removed_span = None;
    for item in &items {
        match self::key(item)?.as_str() {
            "added" => once(&mut range.added, item, version(item, "added")?)?,
            "removed" => {
                once(&mut range.removed, item, version(item, "removed")?)?;
                removed_span = Some(item.span());
            }
            other => return Err(unknown(item, other)),
        }
    }
    if range.added.is_none() && range.removed.is_none() {
        return Err(syn::Error::new_spanned(
            list,
            format!("qapi: `{key}(...)` needs `added` or `removed`"),
        ));
    }
    check_order(range.added, range.removed, removed_span)?;
    Ok(range)
}

/// A bare flag, or a flag with a version range.
fn flag_range(meta: &Meta, key: &str) -> syn::Result<AttrRange> {
    match meta {
        Meta::Path(path) => Ok(AttrRange {
            span: path.span(),
            added: None,
            removed: None,
        }),
        _ => parse_attr_range(meta, key),
    }
}

/// Parse `deprecated(added = "X", removed = "Y")`, where `removed` ends a
/// deprecation that a later release drops while the item lasts.
fn parse_deprecated(meta: &Meta) -> syn::Result<Deprecation> {
    let (list, items) = nested(meta, "deprecated")?;
    let (mut added, mut removed, mut removed_span) = (None, None, None);
    for item in &items {
        match key(item)?.as_str() {
            "added" => once(&mut added, item, version(item, "added")?)?,
            "removed" => {
                once(&mut removed, item, version(item, "removed")?)?;
                removed_span = Some(item.span());
            }
            other => return Err(unknown(item, other)),
        }
    }
    let added = added
        .ok_or_else(|| syn::Error::new_spanned(list, "qapi: `deprecated(...)` needs `added`"))?;
    check_order(Some(added), removed, removed_span)?;
    Ok(Deprecation {
        span: meta.span(),
        added,
        removed,
    })
}

/// The built-in QAPI features that change the expansion.
const FEATURES: [&str; 2] = ["deprecated", "unstable"];

/// Add the feature of `feature = "deprecated"` or `feature = "unstable"`.
fn feature(features: &mut Vec<String>, meta: &Meta) -> syn::Result<()> {
    let text = lit_str(meta, "feature")?;
    let feature = text.value();
    if !FEATURES.contains(&feature.as_str()) {
        return Err(syn::Error::new_spanned(
            text,
            "qapi: `feature` is `deprecated` or `unstable`; other QAPI features \
             do not change the expansion",
        ));
    }
    if features.contains(&feature) {
        return Err(syn::Error::new_spanned(
            text,
            format!("qapi: duplicate feature `{feature}`"),
        ));
    }
    features.push(feature);
    Ok(())
}

impl LifetimeSpans {
    /// The lifetime and deprecation keys that every item takes.
    fn parse(
        &mut self,
        item: &Meta,
        key: &str,
        since: &mut Option<Version>,
        until: &mut Option<Version>,
        deprecated: &mut Vec<Deprecation>,
    ) -> syn::Result<bool> {
        match key {
            "added" => {
                once(since, item, version(item, key)?)?;
                self.added = Some(item.span());
            }
            "removed" => {
                once(until, item, version(item, key)?)?;
                self.removed = Some(item.span());
            }
            "deprecated" => deprecated.push(parse_deprecated(item)?),
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Check that the lifetime ends after it starts, that each deprecation
    /// lies within it and that the deprecations do not overlap. Sorts the
    /// deprecations by release.
    fn check(
        &self,
        since: Option<Version>,
        until: Option<Version>,
        deprecated: &mut [Deprecation],
    ) -> syn::Result<()> {
        check_order(since, until, self.removed)?;
        deprecated.sort_by_key(|run| run.added);
        let mut previous_end = None;
        for run in deprecated.iter() {
            let added = run.added;
            let error = |message: String| Err(syn::Error::new(run.span, message));
            if let Some(since) = since
                && added < since
            {
                return error(format!(
                    "qapi: `deprecated` ({added}) must not be earlier than `added` ({since})"
                ));
            }
            if let Some(until) = until {
                if added >= until {
                    return error(format!(
                        "qapi: `deprecated` ({added}) must be earlier than `removed` ({until})"
                    ));
                }
                if let Some(end) = run.removed.filter(|end| *end > until) {
                    return error(format!(
                        "qapi: a deprecation must end by `removed` ({until}), not at {end}"
                    ));
                }
            }
            match previous_end {
                Some(None) => {
                    return error(format!(
                        "qapi: `deprecated` ({added}) follows a deprecation without `removed`"
                    ));
                }
                Some(Some(end)) if added < end => {
                    return error(format!(
                        "qapi: `deprecated` ({added}) must not be earlier than the `removed` \
                         ({end}) of the deprecation before it"
                    ));
                }
                _ => {}
            }
            previous_end = Some(run.removed);
        }
        Ok(())
    }
}

/// Check that each range of a type's flag lies within the type's lifetime,
/// from `since` until `until`, and that no two of them overlap: one starts
/// once the one before it has ended, and a bare flag, which holds for the
/// whole lifetime, is the only range of its flag.
fn check_ranges(
    key: &str,
    ranges: &[AttrRange],
    since: Option<Version>,
    until: Option<Version>,
) -> syn::Result<()> {
    let mut sorted: Vec<&AttrRange> = ranges.iter().collect();
    sorted.sort_by_key(|range| range.added);
    let mut previous_end = None;
    for range in sorted {
        let error = |message: String| Err(syn::Error::new(range.span, message));
        if let (Some(added), Some(since)) = (range.added, since)
            && added < since
        {
            return error(format!(
                "qapi: `{key}` (added {added}) must not start before the type's `added` ({since})"
            ));
        }
        if let (Some(added), Some(until)) = (range.added, until)
            && added >= until
        {
            return error(format!(
                "qapi: `{key}` (added {added}) must start before the type's `removed` ({until})"
            ));
        }
        if let (Some(removed), Some(since)) = (range.removed, since)
            && removed <= since
        {
            return error(format!(
                "qapi: `{key}` (removed {removed}) must end after the type's `added` ({since})"
            ));
        }
        if let (Some(removed), Some(until)) = (range.removed, until)
            && removed > until
        {
            return error(format!(
                "qapi: `{key}` (removed {removed}) must end by the type's `removed` ({until})"
            ));
        }
        match previous_end {
            Some(None) => {
                return error(format!(
                    "qapi: `{key}` overlaps the `{key}` before it, which does not end"
                ));
            }
            Some(Some(end)) if range.added.is_none_or(|added| added < end) => {
                return error(format!(
                    "qapi: `{key}` must not start before the `removed` ({end}) of the `{key}` \
                     before it"
                ));
            }
            _ => {}
        }
        previous_end = Some(range.removed);
    }
    Ok(())
}

impl TypeAttrs {
    /// Parse the attribute's arguments and every `#[qapi(...)]` attribute of
    /// the item, and check them together.
    pub(crate) fn parse(args: &MetaArgs, attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut parsed = Self::default();
        parsed.parse_list(args)?;
        for items in qapi_attr_items(attrs) {
            parsed.parse_list(&items?)?;
        }
        parsed
            .spans
            .check(parsed.since, parsed.until, &mut parsed.deprecated)?;
        let ranges = |key, list| check_ranges(key, list, parsed.since, parsed.until);
        ranges("unstable", &parsed.unstable)?;
        ranges("simple_union", &parsed.simple_union)?;
        ranges("allow_oob", &parsed.allow_oob)?;
        ranges("allow_preconfig", &parsed.allow_preconfig)?;
        ranges("no_success_response", &parsed.no_success_response)?;
        Ok(parsed)
    }

    /// Add the items of one `#[qapi(...)]` list.
    fn parse_list(&mut self, args: &MetaArgs) -> syn::Result<()> {
        for item in args {
            let key = key(item)?;
            let lifetime = &mut self.spans;
            if lifetime.parse(
                item,
                &key,
                &mut self.since,
                &mut self.until,
                &mut self.deprecated,
            )? {
                continue;
            }
            match key.as_str() {
                "feature" => feature(&mut self.features, item)?,
                "name" => once(&mut self.name, item, type_name(item)?.clone())?,
                "condition" => once(&mut self.condition, item, condition(item)?)?,
                "unstable" => self.unstable.push(parse_attr_range(item, &key)?),
                "union_branch" => once_flag(&mut self.union_branch, item, &key)?,
                "simple_union" => self.simple_union.push(flag_range(item, &key)?),
                "allow_oob" => self.allow_oob.push(flag_range(item, &key)?),
                "allow_preconfig" => self.allow_preconfig.push(flag_range(item, &key)?),
                "no_success_response" => self.no_success_response.push(flag_range(item, &key)?),
                "command" => once_flag(&mut self.command, item, &key)?,
                "event" => once_flag(&mut self.event, item, &key)?,
                "returns" => match item {
                    Meta::List(_) => self.returns_history.push(parse_returns_history(item)?),
                    _ => once(&mut self.returns, item, lit_str(item, &key)?.clone())?,
                },
                "returns_enum" => {
                    once(&mut self.returns_enum, item, lit_str(item, &key)?.clone())?;
                }
                _ => return Err(unknown(item, &key)),
            }
        }
        Ok(())
    }

    pub(crate) fn has_feature(&self, name: &str) -> bool {
        self.features.iter().any(|feature| feature == name)
    }

    /// The keys that only a command takes, by the span of each.
    pub(crate) fn command_keys(&self) -> impl Iterator<Item = (Span, &'static str)> + '_ {
        let ranges = |ranges: &'static str, list: &[AttrRange]| {
            list.first().map(|range| (range.span, ranges))
        };
        [
            self.returns.as_ref().map(|text| (text.span(), "returns")),
            self.returns_history
                .first()
                .map(|(_, text)| (text.span(), "returns")),
            self.returns_enum
                .as_ref()
                .map(|text| (text.span(), "returns_enum")),
            ranges("allow_oob", &self.allow_oob),
            ranges("allow_preconfig", &self.allow_preconfig),
            ranges("no_success_response", &self.no_success_response),
        ]
        .into_iter()
        .flatten()
    }

    /// The first key that only a struct takes, for an enum's error.
    pub(crate) fn struct_key(&self) -> Option<(Span, &'static str)> {
        let command = self.command.map(|span| (span, "command"));
        let event = self.event.map(|span| (span, "event"));
        let simple_union = self
            .simple_union
            .first()
            .map(|range| (range.span, "simple_union"));
        command
            .into_iter()
            .chain(self.command_keys())
            .chain(event)
            .chain(simple_union)
            .next()
    }
}

/// Parse `returns(added = "X", value = "Type")`.
fn parse_returns_history(meta: &Meta) -> syn::Result<(Version, LitStr)> {
    let (list, items) = nested(meta, "returns")?;
    let mut added = None;
    let mut value = None;
    for item in &items {
        match key(item)?.as_str() {
            "added" => once(&mut added, item, version(item, "added")?)?,
            "value" => once(&mut value, item, lit_str(item, "value")?.clone())?,
            other => return Err(unknown(item, other)),
        }
    }
    match (added, value) {
        (Some(added), Some(value)) => Ok((added, value)),
        _ => Err(syn::Error::new_spanned(
            list,
            "qapi: `returns(...)` needs `added` and `value`",
        )),
    }
}

impl ItemAttrs {
    /// Add the items of one `#[qapi(...)]` list of a member.
    fn parse_list(&mut self, items: &MetaArgs) -> syn::Result<()> {
        for item in items {
            let key = key(item)?;
            let lifetime = &mut self.spans;
            if lifetime.parse(
                item,
                &key,
                &mut self.since,
                &mut self.until,
                &mut self.deprecated,
            )? {
                continue;
            }
            match key.as_str() {
                "feature" => feature(&mut self.features, item)?,
                "name" => once(&mut self.name, item, lit_str(item, &key)?.value())?,
                "condition" => once(&mut self.condition, item, condition(item)?)?,
                "flatten" => once_flag(&mut self.flatten, item, &key)?,
                "discriminator" => self.discriminator.push(flag_range(item, &key)?),
                "union" => once_flag(&mut self.union_field, item, &key)?,
                "required" => once_flag(&mut self.required, item, &key)?,
                _ => return Err(unknown(item, &key)),
            }
        }
        Ok(())
    }

    /// Parse and check all `#[qapi(...)]` attributes of a member of the
    /// type `ta`.
    pub(crate) fn parse(attrs: &[syn::Attribute], ta: &TypeAttrs) -> syn::Result<Self> {
        let mut parsed = Self::default();
        for items in qapi_attr_items(attrs) {
            parsed.parse_list(&items?)?;
        }
        parsed
            .spans
            .check(parsed.since, parsed.until, &mut parsed.deprecated)?;
        parsed.check_within(ta)?;
        Ok(parsed)
    }

    /// Check that the member's lifetime lies within its type's.
    fn check_within(&self, ta: &TypeAttrs) -> syn::Result<()> {
        if let (Some(added), Some(span)) = (self.since, self.spans.added) {
            let error = |message: String| Err(syn::Error::new(span, message));
            if let Some(since) = ta.since
                && added < since
            {
                return error(format!(
                    "qapi: `added` ({added}) must not be earlier than the type's `added` ({since})"
                ));
            }
            if let Some(until) = ta.until
                && added >= until
            {
                return error(format!(
                    "qapi: `added` ({added}) must be earlier than the type's `removed` ({until})"
                ));
            }
        }
        if let (Some(removed), Some(span)) = (self.until, self.spans.removed) {
            let error = |message: String| Err(syn::Error::new(span, message));
            if let Some(since) = ta.since
                && removed <= since
            {
                return error(format!(
                    "qapi: `removed` ({removed}) must be later than the type's `added` ({since})"
                ));
            }
            if let Some(until) = ta.until
                && removed > until
            {
                return error(format!(
                    "qapi: `removed` ({removed}) must not be later than the type's `removed` \
                     ({until})"
                ));
            }
        }
        Ok(())
    }
}
