//! The `#[qapi(...)]` attribute macro behind the generated types of
//! `qrab-qapi`.
//!
//! The attribute expands the QAPI types that `qrab-qapi-codegen` generates
//! into Serde implementations that check the QEMU version lifetimes of each
//! type and its members against the version of the peer:
//!
//! - String enums, alternates and the enums of union branches.
//! - Structs, with their flattened bases and unions, and command and event
//!   types, which also implement `QapiCommand` or `QapiEvent`.
//!
//! The expansion calls the run-time support of `qrab-qapi` by the absolute
//! path `::qapi`, so it compiles only where the crate `qapi` is that
//! library: in a crate that depends on `qrab-qapi` under its library name,
//! and in `qrab-qapi` itself, which declares `extern crate self as qapi`.
//!
//! Every build expands the complete history: a type or member that only
//! some releases have exists in every build, its lifetime checked against
//! the peer at run time. One that the newest release no longer has, or
//! deprecates, is Rust `#[deprecated]`, with a note naming the release. The
//! documentation of an unstable type and the flag constants of a command
//! describe the newest release, while its return types, and whether it is
//! `PreconfigSafe`, cover every release.
#![forbid(unsafe_code)]

// The modules: `attrs`, the authoritative list of the keys and their
// parser; `support`, what the expansions share; and an expansion per kind
// of item.
mod attrs;
mod command;
mod enums;
mod structs;
mod support;
#[cfg(test)]
mod tests;

use crate::{
    attrs::{MetaArgs, TypeAttrs, parse_items},
    enums::gen_enum,
    structs::gen_struct,
    support::{
        condition_note, deprecated_attr, flag_at_newest, push_doc_notes,
        ranges_at_newest,
    },
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Expand a QAPI type, as `qrab-qapi-codegen` generates it, into its
/// version-aware Serde and trait implementations.
///
/// The keys of the attribute are those that the generator writes; a key,
/// value or form that the attribute does not define is a compile error. The
/// item may be given further `#[qapi(...)]` attributes, as may its members.
///
/// The expansion derives `Debug` and `Clone` and implements `Serialize` and
/// `Deserialize`, so the item must not derive them itself: a derive below the
/// attribute is an error at its path, and one above it conflicts with the
/// expansion.
///
/// The item is `#[non_exhaustive]`, so that a new QEMU release can add fields
/// and variants without breaking code outside the crate: such code matches an
/// enum with a wildcard arm and builds a struct with one of the constructors
/// the expansion gives it. A struct whose fields are all `Option`, or that has
/// none, implements `Default`, with every field `None`, and the fields are then
/// assigned. Any other struct has `new`, which takes its non-`Option` fields in
/// declaration order and leaves the others `None`; a struct does not derive
/// `Default` itself. Enum variants are constructed as usual. String enums also
/// implement `Display`, writing the wire name. Structs implement it as their
/// QAPI name; a struct with fields that is not a command follows it with the
/// fields that are set, as `name(wire=value, ...)` with each value in its
/// `Debug` form. Generic items are not supported.
///
/// ```
/// # // A stand-in for qrab-qapi, which this crate cannot depend on: the
/// # // example crate names itself `qapi`.
/// # extern crate self as qapi;
/// # use std::cell::Cell;
/// # thread_local! {
/// #     static VERSION: Cell<Option<(u32, u32, u32)>> =
/// #         const { Cell::new(None) };
/// # }
/// # pub fn with_version<T>(
/// #     version: (u32, u32, u32),
/// #     f: impl FnOnce() -> T,
/// # ) -> T {
/// #     let outer = VERSION.replace(Some(version));
/// #     let result = f();
/// #     VERSION.set(outer);
/// #     result
/// # }
/// # pub fn scoped_qemu_version() -> Option<(u32, u32, u32)> { VERSION.get() }
/// # pub fn qemu_version() -> (u32, u32, u32) {
/// #     VERSION.get().unwrap_or((0, 0, 0))
/// # }
/// # pub mod __private {
/// #     pub use serde;
/// #     pub fn unavailable_type_message(name: &str) -> String {
/// #         let (major, minor, patch) = super::qemu_version();
/// #         format!("{name} is not available in QEMU {major}.{minor}.{patch}")
/// #     }
/// #     pub fn unavailable_error<E: serde::ser::Error>(name: &str) -> E {
/// #         E::custom(unavailable_type_message(name))
/// #     }
/// # }
/// # fn main() {
/// use qapi_macros::qapi;
///
/// #[qapi(added = "2.0.0")]
/// pub enum BlockdevDriver {
///     #[qapi(name = "qcow2")]
///     Qcow2,
///     #[qapi(name = "archipelago", added = "2.2.0", removed = "2.9.0")]
///     Archipelago,
/// }
///
/// let encode = |version, driver: BlockdevDriver| {
///     qapi::with_version(version, || serde_json::to_string(&driver))
/// };
/// assert_eq!(encode((9, 2, 0), BlockdevDriver::Qcow2).unwrap(), r#""qcow2""#);
/// assert_eq!(
///     encode((9, 2, 0), BlockdevDriver::Archipelago).unwrap_err().to_string(),
///     "BlockdevDriver::Archipelago is not available in QEMU 9.2.0",
/// );
///
/// let decode = |version, json| {
///     qapi::with_version(version, || {
///         serde_json::from_str::<BlockdevDriver>(json)
///     })
/// };
/// assert!(matches!(
///     decode((2, 5, 0), r#""archipelago""#),
///     Ok(BlockdevDriver::Archipelago)
/// ));
/// assert_eq!(
///     decode((9, 2, 0), r#""archipelago""#).unwrap_err().to_string(),
///     "BlockdevDriver::Archipelago is not available in QEMU 9.2.0",
/// );
/// # }
/// ```
#[proc_macro_attribute]
pub fn qapi(args: TokenStream, input: TokenStream) -> TokenStream {
    let qapi_args = parse_macro_input!(args with parse_items);
    let input_ast = parse_macro_input!(input as DeriveInput);

    TokenStream::from(
        expand(qapi_args, input_ast)
            .unwrap_or_else(|error| error.to_compile_error()),
    )
}

/// The expansion of an item, after every key and member is checked.
pub(crate) fn expand(
    qapi_args: MetaArgs,
    mut input_ast: DeriveInput,
) -> syn::Result<TokenStream2> {
    let ta = TypeAttrs::parse(&qapi_args, &input_ast.attrs)?;
    input_ast.attrs.retain(|a| !a.path().is_ident("qapi"));
    reject_generics(&input_ast.generics)?;
    reject_own_derives(
        &input_ast.attrs,
        matches!(input_ast.data, syn::Data::Struct(_)),
    )?;

    let impls = match input_ast.data {
        syn::Data::Enum(_) => gen_enum(&mut input_ast, &ta)?,
        syn::Data::Struct(_) => gen_struct(&mut input_ast, &ta)?,
        _ => {
            return Err(syn::Error::new_spanned(
                &input_ast.ident,
                "qapi: #[qapi] does not support this kind of item",
            ));
        }
    };
    annotate_type(&mut input_ast.attrs, &ta);
    Ok(quote! {
        #input_ast
        #impls
    })
}

/// Document an unstable type, the type it is the same as on the wire and its
/// build condition, mark it deprecated, derive `Debug` and `Clone`, which
/// every QAPI type has, and mark it `#[non_exhaustive]`, so that a QEMU
/// release may add fields and variants.
fn annotate_type(attrs: &mut Vec<syn::Attribute>, ta: &TypeAttrs) {
    let mut notes = Vec::new();
    // Unstable is not deprecated, so it is a doc line, not #[deprecated].
    if flag_at_newest(&ta.unstable)
        || (ta.unstable.is_empty() && ta.has_feature("unstable"))
    {
        // The ranges do not overlap, so at most one holds there.
        let since =
            ranges_at_newest(&ta.unstable).next().and_then(|range| range.added);
        notes.push(match since {
            Some(since) => format!(
                "**Unstable** since QEMU {since} — this interface may change without notice."
            ),
            None => "**Unstable** — this interface may change without notice.".to_string(),
        });
    }
    notes.extend(crate::structs::alias_note(ta));
    notes.extend(ta.condition.as_deref().map(condition_note));
    push_doc_notes(attrs, notes);
    attrs.extend(deprecated_attr(
        &ta.deprecated,
        ta.until,
        ta.has_feature("deprecated"),
        "type",
    ));
    attrs.push(syn::parse_quote! { #[derive(::core::fmt::Debug, ::core::clone::Clone)] });
    if !attrs.iter().any(|attr| attr.path().is_ident("non_exhaustive")) {
        attrs.push(syn::parse_quote! { #[non_exhaustive] });
    }
}

fn reject_generics(generics: &syn::Generics) -> syn::Result<()> {
    if let Some(param) = generics.params.first() {
        return Err(syn::Error::new_spanned(
            param,
            "qapi: #[qapi] does not support generic items",
        ));
    }
    if let Some(clause) = &generics.where_clause {
        return Err(syn::Error::new_spanned(
            clause,
            "qapi: #[qapi] does not support generic items",
        ));
    }
    Ok(())
}

/// Refuse a derive of a trait that the expansion provides, at the derive's
/// path rather than at the conflicting impl: for a struct also `Default`,
/// which it implements where every field is an `Option` and leaves out
/// where it has `new`.
fn reject_own_derives(
    attrs: &[syn::Attribute],
    is_struct: bool,
) -> syn::Result<()> {
    let provided: &[&str] = if is_struct {
        &["Debug", "Clone", "Serialize", "Deserialize", "Default"]
    } else {
        &["Debug", "Clone", "Serialize", "Deserialize"]
    };
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("derive")) {
        let paths = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
        )?;
        for path in paths {
            if let Some(last) = path.segments.last()
                && provided.iter().any(|name| last.ident == name)
            {
                return Err(syn::Error::new_spanned(
                    &path,
                    format!("qapi: #[qapi] provides `{}` itself", last.ident),
                ));
            }
        }
    }
    Ok(())
}
