//! The alias form of a struct that is the same on the wire as another type
//! from a release: `alias(added = "V", value = "Type")`.

use crate::attrs::TypeAttrs;
use syn::Type;

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
        syn::Error::new_spanned(
            value,
            format!("qapi: `alias` is not a Rust type: {error}"),
        )
    })
}

/// The doc line of a struct that is the same on the wire as another type
/// from a release: from there on, the newer type describes its replies.
pub(crate) fn alias_note(ta: &TypeAttrs) -> Option<String> {
    let (added, value, _) = ta.alias.as_ref()?;
    Some(format!(
        "The same on the wire as `{}` from QEMU {added}.",
        value.value()
    ))
}
