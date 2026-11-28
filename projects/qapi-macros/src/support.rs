//! Helpers shared by the expansions of every kind of item.

use crate::attrs::{AttrRange, Deprecation, ItemAttrs, TypeAttrs};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::ext::IdentExt;

/// A QEMU release, written `MAJOR.MINOR.PATCH`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Version(pub(crate) u32, pub(crate) u32, pub(crate) u32);

impl Version {
    pub(crate) const ZERO: Self = Self(0, 0, 0);
    const NEVER: Self = Self(u32::MAX, u32::MAX, u32::MAX);

    /// Parse exactly three decimal components without leading zeros.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('.').map(|part| {
            let digits = !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit());
            (digits && (part == "0" || !part.starts_with('0')))
                .then(|| part.parse().ok())
                .flatten()
        });
        let version = Self(parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(version)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// The `(major, minor, patch)` tuple that the runtime compares.
impl ToTokens for Version {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self(major, minor, patch) = self;
        tokens.extend(quote! { (#major, #minor, #patch) });
    }
}

pub(crate) fn wire_name(
    attrs_name: &Option<String>,
    ident: &syn::Ident,
) -> String {
    attrs_name.clone().unwrap_or_else(|| ident.unraw().to_string())
}

/// Whether the current version context lies in `[since, until)`, or `None`
/// for a lifetime without bounds.
pub(crate) fn version_range_check(
    since: Option<Version>,
    until: Option<Version>,
) -> Option<TokenStream2> {
    match (since, until) {
        (Some(s), Some(u)) => Some(
            quote! { (::qapi::qemu_version() >= #s && ::qapi::qemu_version() < #u) },
        ),
        (Some(s), None) => Some(quote! { (::qapi::qemu_version() >= #s) }),
        (None, Some(u)) => Some(quote! { (::qapi::qemu_version() < #u) }),
        (None, None) => None,
    }
}

/// [`version_range_check`], `true` for a lifetime without bounds.
pub(crate) fn active_check(
    since: Option<Version>,
    until: Option<Version>,
) -> TokenStream2 {
    version_range_check(since, until).unwrap_or_else(|| quote! { true })
}

/// Whether decoding takes a representation. Inside `qapi::with_version` it
/// takes the one `active` for that peer. Unscoped decoding, including under
/// the process-wide default, which only serialization uses, takes the newest
/// (`latest`), so that recorded and newer replies stay readable.
pub(crate) fn decoding_selects(
    latest: TokenStream2,
    active: TokenStream2,
) -> TokenStream2 {
    quote! { (if ::qapi::scoped_qemu_version().is_none() { #latest } else { #active }) }
}

/// Order the representations of one wire name newest first: those never
/// removed, then by latest removal, then by latest addition.
pub(crate) fn newest_first<T>(
    items: &mut [&T],
    attrs: impl Fn(&T) -> &ItemAttrs,
) {
    use std::cmp::Reverse;
    items.sort_by_key(|item| {
        let attrs = attrs(item);
        (
            attrs.until.is_some(),
            Reverse(attrs.until.unwrap_or(Version::NEVER)),
            Reverse(attrs.since.unwrap_or(Version::ZERO)),
        )
    });
}

/// Refuse a representation whose wire name an earlier one already has for
/// part of its lifetime, since decoding could not tell them apart.
pub(crate) fn check_disjoint_wires<'a>(
    items: impl IntoIterator<Item = (String, &'a ItemAttrs, &'a syn::Ident)>,
) -> syn::Result<()> {
    let start = |attrs: &ItemAttrs| attrs.since.unwrap_or(Version::ZERO);
    let end = |attrs: &ItemAttrs| attrs.until.unwrap_or(Version::NEVER);
    let mut seen: Vec<(String, &ItemAttrs)> = Vec::new();
    for (wire, attrs, ident) in items {
        if seen.iter().any(|(other, earlier)| {
            *other == wire
                && start(earlier) < end(attrs)
                && start(attrs) < end(earlier)
        }) {
            return Err(syn::Error::new_spanned(
                ident,
                format!(
                    "qapi: wire name `{wire}` is already taken in part of this lifetime; \
                     representations of one wire name need disjoint `added`/`removed` ranges"
                ),
            ));
        }
        seen.push((wire, attrs));
    }
    Ok(())
}

/// Add doc lines to an item, as a paragraph of their own after its docs.
pub(crate) fn push_doc_notes(
    attrs: &mut Vec<syn::Attribute>,
    notes: Vec<String>,
) {
    if notes.is_empty() {
        return;
    }
    if attrs.iter().any(|attr| attr.path().is_ident("doc")) {
        attrs.push(syn::parse_quote! { #[doc = ""] });
    }
    for note in notes {
        let note = format!(" {note}");
        attrs.push(syn::parse_quote! { #[doc = #note] });
    }
}

/// The doc line of a QAPI build condition.
pub(crate) fn condition_note(condition: &str) -> String {
    format!("QEMU/guest build condition: `{condition}`.")
}

/// Rust `#[deprecated]` for a type or member that the newest release no
/// longer offers: one removed at `until`, or one with a deprecation that
/// lasts to the newest release, or, without any versions to compare, one
/// with the `deprecated` feature. The note names the release. Code for an
/// older peer may still use it; the warning points newer code elsewhere.
pub(crate) fn deprecated_attr(
    deprecated: &[Deprecation],
    until: Option<Version>,
    feature: bool,
    what: &str,
) -> Option<syn::Attribute> {
    let note = if let Some(until) = until {
        format!("removed in QEMU {until}")
    } else if let Some(run) =
        deprecated.iter().find(|run| run.removed.is_none())
    {
        format!("deprecated since QEMU {}", run.added)
    } else if deprecated.is_empty() && feature {
        format!("deprecated QAPI {what}")
    } else {
        return None;
    };
    Some(syn::parse_quote! { #[deprecated(note = #note)] })
}

/// Replace the `#[qapi(...)]` attributes of a member by its QAPI
/// metadata: doc lines for the `unstable` feature and the build condition,
/// and Rust `#[deprecated]`.
pub(crate) fn annotate_item(
    item_attrs: &mut Vec<syn::Attribute>,
    attrs: &ItemAttrs,
    what: &str,
) {
    item_attrs.retain(|attr| !attr.path().is_ident("qapi"));
    let mut notes = Vec::new();
    // Unstable is not deprecated, so it is a doc line, not #[deprecated].
    if attrs.has_feature("unstable") {
        notes.push(format!(
            "**Unstable** — this {what} may change without notice."
        ));
    }
    notes.extend(attrs.condition.as_deref().map(condition_note));
    push_doc_notes(item_attrs, notes);
    item_attrs.extend(deprecated_attr(
        &attrs.deprecated,
        attrs.until,
        attrs.has_feature("deprecated"),
        what,
    ));
}

/// The start of a type's serializer: refuse it outside its lifetime, and
/// warn once where the peer's version deprecates it.
pub(crate) fn type_serialization_guard(
    name: &syn::Ident,
    ta: &TypeAttrs,
) -> TokenStream2 {
    let name = name.unraw().to_string();
    let check = version_range_check(ta.since, ta.until).map(|available| {
        let unavailable = unavailable_error(&name);
        quote! {
            if !#available {
                return ::core::result::Result::Err(#unavailable);
            }
        }
    });
    let warning = deprecation_warning(&name, &ta.deprecated);
    quote! { #check #warning }
}

/// The serialization error of a type or member outside its lifetime,
/// which `qapi::to_value` reports as unavailable under this name.
pub(crate) fn unavailable_error(name: &str) -> TokenStream2 {
    quote! { ::qapi::__private::unavailable_error(#name) }
}

/// The decoding error of a type or member outside its lifetime in the
/// scoped version.
pub(crate) fn unavailable_decode_error(name: &str) -> TokenStream2 {
    quote! { ::qapi::__private::serde::de::Error::custom(::qapi::__private::unavailable_type_message(#name)) }
}

/// The start of a type's decoder: in a peer's version scope, refuse a type
/// outside its lifetime.
pub(crate) fn type_deserialization_guard(
    name: &syn::Ident,
    attrs: &TypeAttrs,
) -> TokenStream2 {
    version_range_check(attrs.since, attrs.until)
        .map(|active| {
            let error = unavailable_decode_error(&name.unraw().to_string());
            quote! {
                if ::qapi::scoped_qemu_version().is_some() && !#active {
                    return ::core::result::Result::Err(#error);
                }
            }
        })
        .unwrap_or_default()
}

/// Once-guarded run-time warnings, one per deprecation, that `name`, the
/// Rust path of a type or member, is in use with a peer whose release
/// deprecates it; nothing for an item without deprecations.
pub(crate) fn deprecation_warning(
    name: &str,
    deprecated: &[Deprecation],
) -> TokenStream2 {
    let warnings = deprecated.iter().map(|run| {
        let msg = format!(
            "qapi warning: {name} is deprecated since QEMU {}",
            run.added
        );
        let active = active_check(Some(run.added), run.removed);
        quote! {
            {
                static __QAPI_DEP_WARNED: ::std::sync::Once = ::std::sync::Once::new();
                if #active {
                    __QAPI_DEP_WARNED.call_once(|| {
                        ::std::eprintln!(#msg);
                    });
                }
            }
        }
    });
    quote! { #(#warnings)* }
}

/// The ranges of a flag such as `unstable` that hold in the newest
/// release: those without a removal.
pub(crate) fn ranges_at_newest(
    ranges: &[AttrRange],
) -> impl Iterator<Item = &AttrRange> {
    ranges.iter().filter(|range| range.removed.is_none())
}

/// Whether a flag holds in the newest release.
pub(crate) fn flag_at_newest(ranges: &[AttrRange]) -> bool {
    ranges_at_newest(ranges).next().is_some()
}

/// Collision-free names for the locals, helper types and generic parameters
/// of an expansion: none may capture or shadow a name of the item, of its
/// members or of the types it refers to, such as `Serializer`.
pub(crate) struct HelperNames {
    used: std::collections::BTreeSet<String>,
}

impl HelperNames {
    /// Reserve the item's name, its members' names and every identifier of
    /// the types it refers to.
    pub(crate) fn new<'a>(
        parent: &'a syn::Ident,
        members: impl IntoIterator<Item = &'a syn::Ident>,
        types: impl IntoIterator<Item = &'a syn::Type>,
    ) -> Self {
        let mut names = Self {
            used: std::iter::once(parent)
                .chain(members)
                .map(|ident| ident.unraw().to_string())
                .collect(),
        };
        for ty in types {
            names.reserve(quote! { #ty });
        }
        names
    }

    fn reserve(&mut self, tokens: TokenStream2) {
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(ident) => {
                    self.used.insert(ident.unraw().to_string());
                }
                proc_macro2::TokenTree::Group(group) => {
                    self.reserve(group.stream())
                }
                _ => {}
            }
        }
    }

    fn allocate(&mut self, base: String) -> syn::Ident {
        let mut candidate = base.clone();
        let mut suffix = 2;
        while !self.used.insert(candidate.clone()) {
            candidate = format!("{base}{suffix}");
            suffix += 1;
        }
        format_ident!("{candidate}")
    }

    /// A local variable or parameter.
    pub(crate) fn ident(&mut self, stem: &str) -> syn::Ident {
        self.allocate(format!("__qapi_{stem}"))
    }

    /// A helper type or generic parameter.
    pub(crate) fn ty(&mut self, stem: &str) -> syn::Ident {
        self.allocate(format!("__Qapi{stem}"))
    }
}
