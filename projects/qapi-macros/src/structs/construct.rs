//! The construction API of a struct, which is `#[non_exhaustive]` and so
//! cannot be built with a struct literal outside its crate.
//!
//! - A struct whose fields are all `Option`, or that has none, implements
//!   `Default` with every field `None`; fields are then assigned.
//! - Any other struct has `new`, which takes its non-`Option` fields in
//!   declaration order and leaves the `Option` fields `None`.

use super::FieldInfo;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ext::IdentExt;

/// `Default` or `new` for a struct with the fields that the bounds keep.
pub(crate) fn construction(
    name: &syn::Ident,
    vis: &syn::Visibility,
    fields: &[FieldInfo],
) -> TokenStream2 {
    let (required, optional): (Vec<&FieldInfo>, Vec<&FieldInfo>) =
        fields.iter().partition(|field| !field.is_option());
    let optional = optional.iter().map(|field| &field.ident);
    if required.is_empty() {
        return quote! {
            #[allow(deprecated)]
            impl ::core::default::Default for #name {
                fn default() -> Self {
                    Self { #(#optional: ::core::option::Option::None,)* }
                }
            }
        };
    }
    let list = required
        .iter()
        .map(|field| format!("`{}`", field.ident.unraw()))
        .collect::<Vec<_>>()
        .join(", ");
    let doc = format!(
        " A `{}` with its required fields, {list}, in declaration order, and every optional \
         field `None`.",
        name.unraw()
    );
    let params = required.iter().map(|field| {
        let (ident, ty) = (&field.ident, &field.ty);
        quote! { #ident: #ty }
    });
    let required = required.iter().map(|field| &field.ident);
    quote! {
        #[allow(deprecated)]
        impl #name {
            #[doc = #doc]
            #[allow(clippy::too_many_arguments)]
            #[must_use]
            #vis fn new(#(#params),*) -> Self {
                Self {
                    #(#required,)*
                    #(#optional: ::core::option::Option::None,)*
                }
            }
        }
    }
}
