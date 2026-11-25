//! Version-aware `Serialize` implementations for structs.

use crate::structs::{StructExpansion, discriminators, inner_option_type};
use crate::support::{deprecation_warning, type_serialization_guard, unavailable_error};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

impl StructExpansion<'_> {
    /// Before anything is written: refuse a field set outside its lifetime
    /// and a required one left unset, and warn once about a deprecated field
    /// that is set.
    fn field_checks(&self) -> TokenStream2 {
        let name_str = &self.name_str;
        let checks = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let path = self.path(field);
            let present = if field.is_option() {
                quote! { self.#ident.is_some() }
            } else {
                quote! { true }
            };
            let available = field.is_gated().then(|| {
                let active = field.active();
                let unavailable = unavailable_error(&path);
                quote! {
                    if #present && !#active {
                        return ::core::result::Result::Err(#unavailable);
                    }
                }
            });
            let required = field.attrs.required.map(|_| {
                let active = field.active();
                let message = format!("{name_str}: missing required field `{}`", field.wire());
                quote! {
                    if #active && self.#ident.is_none() {
                        return ::core::result::Result::Err(::qapi::__private::serde::ser::Error::custom(#message));
                    }
                }
            });
            let deprecated = (!field.attrs.deprecated.is_empty()).then(|| {
                let warning = deprecation_warning(&path, &field.attrs.deprecated);
                quote! { if #present { #warning } }
            });
            quote! { #available #required #deprecated }
        });
        quote! { #(#checks)* }
    }

    /// A struct without flattened fields: a Serde struct of its members.
    pub(crate) fn ser_plain(&mut self) -> TokenStream2 {
        let name = self.name;
        let wire_name = &self.wire_name;
        let serializer_ty = self.names.ty("Serializer");
        let serializer = self.names.ident("serializer");
        let state = self.names.ident("state");
        let value = self.names.ident("value");
        let type_guard = type_serialization_guard(name, self.ta);
        let checks = self.field_checks();
        let counts = self.fields.iter().map(|field| {
            let ident = &field.ident;
            if field.is_option() {
                quote! { + ::core::primitive::usize::from(self.#ident.is_some()) }
            } else {
                quote! { + 1usize }
            }
        });
        let fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let wire = field.wire();
            if field.is_option() {
                quote! {
                    if let ::core::option::Option::Some(ref #value) = self.#ident {
                        ::qapi::__private::serde::ser::SerializeStruct::serialize_field(&mut #state, #wire, #value)?;
                    }
                }
            } else {
                quote! {
                    ::qapi::__private::serde::ser::SerializeStruct::serialize_field(&mut #state, #wire, &self.#ident)?;
                }
            }
        });
        let state_mut = (!self.fields.is_empty()).then(|| quote! { mut });
        quote! {
            #[allow(deprecated)]
            impl ::qapi::__private::serde::Serialize for #name {
                fn serialize<#serializer_ty>(&self, #serializer: #serializer_ty) -> ::core::result::Result<#serializer_ty::Ok, #serializer_ty::Error>
                where #serializer_ty: ::qapi::__private::serde::Serializer {
                    #type_guard
                    #checks
                    let #state_mut #state = ::qapi::__private::serde::Serializer::serialize_struct(#serializer, #wire_name, 0usize #(#counts)*)?;
                    #(#fields)*
                    ::qapi::__private::serde::ser::SerializeStruct::end(#state)
                }
            }
        }
    }

    /// A struct with flattened bases or a union: a map of its members and
    /// the members of its flattened fields, each key once.
    pub(crate) fn ser_flattened(&mut self) -> TokenStream2 {
        let name = self.name;
        let name_str = self.name_str.clone();
        let serializer_ty = self.names.ty("Serializer");
        let serializer = self.names.ident("serializer");
        let state = self.names.ident("state");
        let keys = self.names.ident("keys");
        let value = self.names.ident("value");
        let flat = self.names.ident("flat");
        let key = self.names.ident("key");
        let member = self.names.ident("member");
        let type_guard = type_serialization_guard(name, self.ta);
        let checks = self.field_checks();
        let union_checks = self.union_discriminator_checks();
        let nested = self.simple_union_layout(false);
        let duplicate = format!("{name_str}: duplicate member `{{}}`");
        let emit = |key: TokenStream2, value: TokenStream2| {
            quote! {
                if !#keys.insert(::std::string::String::from(#key)) {
                    return ::core::result::Result::Err(::qapi::__private::serde::ser::Error::custom(::std::format!(#duplicate, #key)));
                }
                ::qapi::__private::serde::ser::SerializeMap::serialize_entry(&mut #state, #key, #value)?;
            }
        };
        let fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let body = if field.is_flattened() {
                let union = field.attrs.union_field.is_some();
                let nested = if union { nested.clone() } else { quote! { false } };
                let into_data = emit(quote! { "data" }, quote! { &#flat });
                let each = emit(quote! { #key.as_str() }, quote! { &#member });
                let not_object = format!(
                    "{}: a flattened payload must serialize to an object",
                    self.path(field)
                );
                quote! {
                    let #flat = ::qapi::__private::serde_json::to_value(#value).map_err(::qapi::__private::serde::ser::Error::custom)?;
                    if #nested {
                        #into_data
                    } else if let ::qapi::__private::serde_json::Value::Object(#flat) = #flat {
                        for (#key, #member) in #flat {
                            #each
                        }
                    } else if !#flat.is_null() {
                        return ::core::result::Result::Err(::qapi::__private::serde::ser::Error::custom(#not_object));
                    }
                }
            } else {
                let wire = field.wire();
                emit(quote! { #wire }, quote! { #value })
            };
            if field.is_option() {
                let missing_payload = (field.attrs.union_field.is_some()
                    && !self.ta.simple_union.is_empty())
                .then(|| {
                    let message = format!("{name_str}: missing simple union payload");
                    quote! {
                        if #nested && self.#ident.is_none() {
                            return ::core::result::Result::Err(::qapi::__private::serde::ser::Error::custom(#message));
                        }
                    }
                });
                quote! {
                    #missing_payload
                    if let ::core::option::Option::Some(ref #value) = self.#ident {
                        #body
                    }
                }
            } else {
                quote! { { let #value = &self.#ident; #body } }
            }
        });
        quote! {
            #[allow(deprecated)]
            impl ::qapi::__private::serde::Serialize for #name {
                fn serialize<#serializer_ty>(&self, #serializer: #serializer_ty) -> ::core::result::Result<#serializer_ty::Ok, #serializer_ty::Error>
                where #serializer_ty: ::qapi::__private::serde::Serializer {
                    #type_guard
                    #checks
                    #union_checks
                    let mut #state = ::qapi::__private::serde::Serializer::serialize_map(#serializer, ::core::option::Option::None)?;
                    let mut #keys = ::std::collections::BTreeSet::<::std::string::String>::new();
                    #(#fields)*
                    ::qapi::__private::serde::ser::SerializeMap::end(#state)
                }
            }
        }
    }

    /// Refuse a union whose payload is a different branch from the one its
    /// discriminator names: the member that is the discriminator in the
    /// version being serialized.
    fn union_discriminator_checks(&mut self) -> TokenStream2 {
        let name_str = self.name_str.clone();
        let mut checks = Vec::new();
        for branch in self
            .fields
            .iter()
            .filter(|field| field.attrs.union_field.is_some())
        {
            let ident = &branch.ident;
            let payload = self.names.ident("union_payload");
            let tag = self.names.ident("union_tag");
            let mut checks_of_discriminators = Vec::new();
            for field in discriminators(self.fields) {
                let disc = &field.ident;
                let value = self.names.ident("discriminator_value");
                let discriminates = field.discriminates();
                let present = if field.is_option() {
                    quote! { #discriminates && self.#disc.is_some() }
                } else {
                    discriminates
                };
                let message = format!(
                    "{name_str}: discriminator `{}` is `{{}}`, but the payload is branch `{{}}`",
                    field.wire()
                );
                // A discriminator set outside its lifetime is refused
                // before, so those that are set are active.
                checks_of_discriminators.push(quote! {
                    if #present {
                        let #value = ::qapi::__private::serde_json::to_value(&self.#disc).map_err(::qapi::__private::serde::ser::Error::custom)?;
                        if #value.as_str() != ::core::option::Option::Some(#tag) {
                            return ::core::result::Result::Err(::qapi::__private::serde::ser::Error::custom(::std::format!(
                                #message,
                                #value.as_str().map_or_else(|| #value.to_string(), ::std::string::ToString::to_string),
                                #tag,
                            )));
                        }
                    } else
                });
            }
            let discriminators = quote! { #(#checks_of_discriminators)* {} };
            let check = if let Some(inner) = inner_option_type(&branch.ty) {
                quote! {
                    if let ::core::option::Option::Some(ref #payload) = self.#ident {
                        if let ::core::option::Option::Some(#tag) = <#inner as ::qapi::QapiFlatUnion>::wire_tag(#payload) {
                            #discriminators
                        }
                    }
                }
            } else {
                let ty = &branch.ty;
                quote! {
                    if let ::core::option::Option::Some(#tag) = <#ty as ::qapi::QapiFlatUnion>::wire_tag(&self.#ident) {
                        #discriminators
                    }
                }
            };
            // A branch set outside its lifetime is refused before.
            checks.push(check);
        }
        quote! { #(#checks)* }
    }
}
