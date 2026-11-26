//! The alias form of a struct that is the same on the wire as another type
//! from a release: `alias(added = "V", value = "Type")`.

use crate::attrs::TypeAttrs;
use crate::bounds::VersionBounds;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Type};

/// Refuse `alias` on a command or an event, and one whose release lies
/// outside the struct's lifetime or whose value is no type.
pub(crate) fn check_alias(ta: &TypeAttrs) -> syn::Result<()> {
    let Some((added, value, span)) = &ta.alias else {
        return Ok(());
    };
    if ta.command.is_some() || ta.event.is_some() {
        return Err(syn::Error::new(
            *span,
            "qapi: `alias` applies only to structs that are neither commands nor events",
        ));
    }
    let error = |message: String| Err(syn::Error::new_spanned(value, message));
    if let Some(since) = ta.since
        && *added < since
    {
        return error(format!(
            "qapi: `alias` (added {added}) must not be earlier than the type's `added` ({since})"
        ));
    }
    if let Some(until) = ta.until
        && *added >= until
    {
        return error(format!(
            "qapi: `alias` (added {added}) must be earlier than the type's `removed` ({until})"
        ));
    }
    alias_type(value).map(drop)
}

fn alias_type(value: &syn::LitStr) -> syn::Result<Type> {
    value.parse().map_err(|error| {
        syn::Error::new_spanned(value, format!("qapi: `alias` is not a Rust type: {error}"))
    })
}

/// The item in place of the struct in a build whose minimum is at or after
/// the alias's release: a deprecated alias of the type, with the struct's
/// documentation. `None` where the build keeps the struct.
pub(crate) fn alias_item(
    input_ast: &DeriveInput,
    ta: &TypeAttrs,
    bounds: &VersionBounds,
) -> syn::Result<Option<TokenStream2>> {
    let Some((added, value, _)) = &ta.alias else {
        return Ok(None);
    };
    if bounds.min.is_none_or(|min| min < *added) {
        return Ok(None);
    }
    let target = alias_type(value)?;
    let target_name = value.value();
    let docs = input_ast
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"));
    let note = format!(" The same on the wire as `{target_name}` from QEMU {added}.");
    let deprecated = format!("the same on the wire as `{target_name}` from QEMU {added}; use it");
    let (vis, ident) = (&input_ast.vis, &input_ast.ident);
    Ok(Some(quote! {
        #(#docs)*
        #[doc = ""]
        #[doc = #note]
        #[deprecated(note = #deprecated)]
        #vis type #ident = #target;
    }))
}
