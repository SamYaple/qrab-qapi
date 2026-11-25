//! Version-aware `Deserialize` implementations for structs.

use crate::structs::{FieldInfo, StructExpansion, discriminators, inner_option_type};
use crate::support::{
    decoding_selects, outside_bounds_clause, type_deserialization_guard, unavailable_decode_error,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::BTreeMap;
use syn::ext::IdentExt;

/// The local variables of one decoding: a slot per field, and a raw JSON
/// value per wire name that several fields share.
struct Slots {
    fields: BTreeMap<String, syn::Ident>,
    raw: BTreeMap<String, syn::Ident>,
}

impl Slots {
    fn field(&self, field: &FieldInfo) -> &syn::Ident {
        &self.fields[&field.ident.to_string()]
    }
}

impl StructExpansion<'_> {
    fn slots(&mut self) -> Slots {
        let fields = self
            .fields
            .iter()
            .map(|field| {
                let slot = self.names.ident(&format!("field_{}", field.ident.unraw()));
                (field.ident.to_string(), slot)
            })
            .collect();
        let raw = self
            .shared_wires()
            .into_keys()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|wire| {
                let encoded: String = wire.bytes().map(|byte| format!("{byte:02x}")).collect();
                let raw = self.names.ident(&format!("raw_{encoded}"));
                (wire, raw)
            })
            .collect();
        Slots { fields, raw }
    }

    /// In a peer's version scope, refuse a value without a field that the
    /// version requires.
    fn required_checks(&self, result: &syn::Ident, error_ty: &TokenStream2) -> TokenStream2 {
        let checks = self
            .fields
            .iter()
            .filter(|field| field.attrs.required.is_some())
            .map(|field| {
                let ident = &field.ident;
                let message = format!(
                    "{}: missing required field `{}`",
                    self.name_str,
                    field.wire()
                );
                let active = field.active();
                quote! {
                    if qapi::scoped_qemu_version().is_some() && #active && #result.#ident.is_none() {
                        return ::core::result::Result::Err(<#error_ty as qapi::serde::de::Error>::custom(#message));
                    }
                }
            });
        quote! { #(#checks)* }
    }

    /// Decode the raw value of each shared wire name into the one version
    /// alternate that decoding selects, keeping that alternate's error. In a
    /// scope that none of them is active in, the value is refused.
    fn decode_alternates(
        &self,
        slots: &Slots,
        raw_value: &syn::Ident,
        error: &syn::Ident,
        error_ty: &TokenStream2,
    ) -> TokenStream2 {
        let decodes = self.shared_wires().into_iter().map(|(wire, group)| {
            let raw = &slots.raw[&wire];
            let branches = group.iter().enumerate().map(|(index, field)| {
                let slot = slots.field(field);
                let inner = inner_option_type(&field.ty).expect("alternates are options");
                let latest = index == 0;
                let selected = decoding_selects(quote! { #latest }, field.active());
                let context = format!("{}.{}: {{}}", self.name_str, wire);
                quote! {
                    if #selected {
                        #slot = ::core::option::Option::Some(
                            qapi::serde_json::from_value::<#inner>(#raw_value).map_err(|#error| {
                                <#error_ty as qapi::serde::de::Error>::custom(::std::format!(#context, #error))
                            })?,
                        );
                    } else
                }
            });
            let unavailable = unavailable_decode_error(&self.path(group[0]), self.bounds);
            quote! {
                if let ::core::option::Option::Some(#raw_value) = #raw {
                    #(#branches)* {
                        return ::core::result::Result::Err(#unavailable);
                    }
                }
            }
        });
        quote! { #(#decodes)* }
    }

    /// Declare the slots of the shared wire names' alternates.
    fn alternate_slots(&self, slots: &Slots) -> TokenStream2 {
        let declarations = self.shared_wires().into_values().flatten().map(|field| {
            let slot = slots.field(field);
            let ty = &field.ty;
            quote! { let mut #slot: #ty = ::core::option::Option::None; }
        });
        quote! { #(#declarations)* }
    }

    /// The value of a field once its slot is filled: an `Option` as it is,
    /// any other type required.
    fn finish_field(
        &self,
        slots: &Slots,
        field: &FieldInfo,
        error_ty: &TokenStream2,
    ) -> TokenStream2 {
        let slot = slots.field(field);
        let ident = &field.ident;
        if field.is_option() {
            quote! { #ident: #slot }
        } else {
            let missing = self.missing_field(&field.wire(), error_ty);
            quote! {
                #ident: #slot.ok_or_else(|| #missing)?
            }
        }
    }

    /// The error for a member that the value lacks, named with the type as
    /// the expansion's other errors are.
    fn missing_field(&self, wire: &str, error_ty: &TokenStream2) -> TokenStream2 {
        let message = format!("{}: missing field `{wire}`", self.name_str);
        quote! { <#error_ty as qapi::serde::de::Error>::custom(#message) }
    }

    /// A struct without flattened fields: decoded as a map, key by key.
    pub(crate) fn de_plain(&mut self) -> syn::Result<TokenStream2> {
        let name = self.name;
        let name_str = self.name_str.clone();
        let deserializer_ty = self.names.ty("Deserializer");
        let deserializer = self.names.ident("deserializer");
        let visitor_ty = self.names.ty("Visitor");
        let access_ty = self.names.ty("MapAccess");
        let access = self.names.ident("access");
        let formatter = self.names.ident("formatter");
        let key = self.names.ident("key");
        let result = self.names.ident("result");
        let raw_value = self.names.ident("raw_value");
        let error = self.names.ident("error");
        let slots = self.slots();
        let error_ty = quote! { #access_ty::Error };
        let type_guard = type_deserialization_guard(name, self.ta, self.bounds);
        let shared = self.shared_wires();

        let mut declarations = Vec::new();
        let mut arms = Vec::new();
        for field in self.fields {
            if shared.contains_key(&field.wire()) {
                continue;
            }
            let slot = slots.field(field);
            let wire = field.wire();
            let ty = inner_option_type(&field.ty).unwrap_or(&field.ty);
            declarations.push(quote! { let mut #slot: ::core::option::Option<#ty> = ::core::option::Option::None; });
            arms.push(quote! {
                #wire => { #slot = ::core::option::Option::Some(qapi::serde::de::MapAccess::next_value::<#ty>(&mut #access)?); }
            });
        }
        for (wire, raw) in &slots.raw {
            declarations.push(quote! { let mut #raw: ::core::option::Option<qapi::serde_json::Value> = ::core::option::Option::None; });
            arms.push(quote! {
                #wire => { #raw = ::core::option::Option::Some(qapi::serde::de::MapAccess::next_value::<qapi::serde_json::Value>(&mut #access)?); }
            });
        }
        let alternate_slots = self.alternate_slots(&slots);
        let alternates = self.decode_alternates(&slots, &raw_value, &error, &error_ty);
        let values: Vec<_> = self
            .fields
            .iter()
            .map(|field| self.finish_field(&slots, field, &error_ty))
            .collect();
        let required = self.required_checks(&result, &error_ty);
        Ok(quote! {
            #[allow(deprecated)]
            impl<'de> qapi::serde::Deserialize<'de> for #name {
                fn deserialize<#deserializer_ty>(#deserializer: #deserializer_ty) -> ::core::result::Result<Self, #deserializer_ty::Error>
                where #deserializer_ty: qapi::serde::Deserializer<'de> {
                    #type_guard
                    struct #visitor_ty;
                    impl<'de> qapi::serde::de::Visitor<'de> for #visitor_ty {
                        type Value = #name;
                        fn expecting(&self, #formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                            ::core::fmt::Formatter::write_str(#formatter, #name_str)
                        }
                        fn visit_map<#access_ty>(self, mut #access: #access_ty) -> ::core::result::Result<Self::Value, #access_ty::Error>
                        where #access_ty: qapi::serde::de::MapAccess<'de> {
                            #(#declarations)*
                            while let ::core::option::Option::Some(#key) = qapi::serde::de::MapAccess::next_key::<::std::string::String>(&mut #access)? {
                                match #key.as_str() {
                                    #(#arms)*
                                    _ => { let _ = qapi::serde::de::MapAccess::next_value::<qapi::serde::de::IgnoredAny>(&mut #access)?; }
                                }
                            }
                            #alternate_slots
                            #alternates
                            let #result = #name { #(#values),* };
                            #required
                            ::core::result::Result::Ok(#result)
                        }
                    }
                    qapi::serde::Deserializer::deserialize_map(#deserializer, #visitor_ty)
                }
            }
        })
    }

    /// A struct with flattened bases or a union: decoded from the whole JSON
    /// object, whose members its fields take out before the bases decode
    /// the rest and the union its branch.
    pub(crate) fn de_flattened(&mut self) -> syn::Result<TokenStream2> {
        let name = self.name;
        let name_str = self.name_str.clone();
        let deserializer_ty = self.names.ty("Deserializer");
        let deserializer = self.names.ident("deserializer");
        let value = self.names.ident("value");
        let map = self.names.ident("map");
        let other = self.names.ident("other");
        let payload = self.names.ident("payload");
        let tag = self.names.ident("tag");
        let result = self.names.ident("result");
        let raw_value = self.names.ident("raw_value");
        let error = self.names.ident("error");
        let version = self.names.ident("version");
        let slots = self.slots();
        let error_ty = quote! { #deserializer_ty::Error };
        let type_guard = type_deserialization_guard(name, self.ta, self.bounds);
        let custom = |message: TokenStream2| {
            quote! { <#error_ty as qapi::serde::de::Error>::custom(#message) }
        };
        let shared = self.shared_wires();

        let has_union = self
            .fields
            .iter()
            .any(|field| field.attrs.union_field.is_some());
        let union_payload = has_union.then(|| {
            let simple = self.simple_union_layout(true);
            let missing = self.missing_field("data", &error_ty);
            quote! {
                let #payload = if #simple {
                    #map.get("data").cloned().ok_or_else(|| #missing)?
                } else {
                    qapi::serde_json::Value::Object(::core::clone::Clone::clone(&#map))
                };
            }
        });

        let mut members = Vec::new();
        for field in self.members() {
            let wire = field.wire();
            if shared.contains_key(&wire) {
                continue;
            }
            let slot = slots.field(field);
            let context = format!("{name_str}.{wire}: {{}}");
            let failed = custom(quote! { ::std::format!(#context, #error) });
            members.push(if let Some(inner) = inner_option_type(&field.ty) {
                quote! {
                    let #slot: ::core::option::Option<#inner> = match #map.remove(#wire) {
                        ::core::option::Option::Some(#raw_value) => ::core::option::Option::Some(
                            qapi::serde_json::from_value::<#inner>(#raw_value).map_err(|#error| #failed)?,
                        ),
                        ::core::option::Option::None => ::core::option::Option::None,
                    };
                }
            } else {
                let ty = &field.ty;
                quote! {
                    let #slot: ::core::option::Option<#ty> = match #map.remove(#wire) {
                        ::core::option::Option::Some(#raw_value) => ::core::option::Option::Some(
                            qapi::serde_json::from_value::<#ty>(#raw_value).map_err(|#error| #failed)?,
                        ),
                        ::core::option::Option::None => ::core::option::Option::None,
                    };
                }
            });
        }
        for (wire, raw) in &slots.raw {
            members.push(quote! { let #raw = #map.remove(#wire); });
        }
        let alternate_slots = self.alternate_slots(&slots);
        let alternates = self.decode_alternates(&slots, &raw_value, &error, &error_ty);

        let discriminators = discriminators(self.fields);
        let mut flattened = Vec::new();
        for field in self.fields.iter().filter(|field| field.is_flattened()) {
            let slot = slots.field(field);
            let ty = &field.ty;
            let decode = if field.attrs.union_field.is_some() {
                let branch_ty = inner_option_type(ty).unwrap_or(ty);
                let dispatch = |tag_value: TokenStream2| {
                    let failed = custom(quote! { #error });
                    quote! {
                        <#branch_ty as qapi::QapiFlatUnion>::deserialize_variant_value(
                            #tag_value,
                            ::core::clone::Clone::clone(&#payload),
                        ).map_err(|#error| #failed)?
                    }
                };
                let optional_tag = match discriminators.as_slice() {
                    [single] if !single.is_option() => None,
                    [single] => {
                        let disc = slots.field(single);
                        Some(quote! { #disc.as_ref().map(::std::string::ToString::to_string) })
                    }
                    // The member that is the discriminator in the peer's
                    // release, or unscoped in the newest the build
                    // represents.
                    discriminators => {
                        let tags = discriminators.iter().map(|field| {
                            let disc = slots.field(field);
                            let selected = decoding_selects(
                                {
                                    let newest = field.discriminates_at_newest(self.bounds);
                                    quote! { #newest }
                                },
                                field.discriminates(),
                            );
                            quote! {
                                .or_else(|| if #selected {
                                    #disc.as_ref().map(::std::string::ToString::to_string)
                                } else {
                                    ::core::option::Option::None
                                })
                            }
                        });
                        Some(quote! {
                            ::core::option::Option::<::std::string::String>::None #(#tags)*
                        })
                    }
                };
                // In a scope outside the bounds, the release's discriminator
                // may be one that the build left out.
                let left_out = self.discriminator_left_out.then(|| {
                    let discriminates = discriminators.iter().map(|field| field.discriminates());
                    let clause =
                        outside_bounds_clause(self.bounds).unwrap_or_else(|| quote! { "" });
                    let message = format!(
                        "{}: no discriminator selects the branch in QEMU {{}}.{{}}.{{}}{{}}",
                        self.path(field)
                    );
                    let failed = custom(quote! {
                        ::std::format!(#message, #version.0, #version.1, #version.2, #clause)
                    });
                    quote! {
                        if qapi::scoped_qemu_version().is_some() && !(false #(|| #discriminates)*) {
                            let #version = qapi::qemu_version();
                            return ::core::result::Result::Err(#failed);
                        }
                    }
                });
                let decode = match optional_tag {
                    Some(tag_value) => {
                        let decode = dispatch(quote! { &#tag });
                        quote! {
                            match #tag_value {
                                ::core::option::Option::Some(#tag) => ::core::option::Option::Some(#decode),
                                ::core::option::Option::None => ::core::option::Option::None,
                            }
                        }
                    }
                    None => {
                        let disc = slots.field(discriminators[0]);
                        let missing = self.missing_field(&discriminators[0].wire(), &error_ty);
                        let decode =
                            dispatch(quote! { &::std::string::ToString::to_string(&#tag) });
                        let decode = if field.is_option() {
                            quote! { ::core::option::Option::Some(#decode) }
                        } else {
                            decode
                        };
                        quote! {
                            match &#disc {
                                ::core::option::Option::Some(#tag) => #decode,
                                ::core::option::Option::None => return ::core::result::Result::Err(#missing),
                            }
                        }
                    }
                };
                quote! { { #left_out #decode } }
            } else {
                let context = format!("{name_str}: {{}}");
                let failed = custom(quote! { ::std::format!(#context, #error) });
                quote! {
                    qapi::serde_json::from_value::<#ty>(qapi::serde_json::Value::Object(::core::clone::Clone::clone(&#map)))
                        .map_err(|#error| #failed)?
                }
            };
            // A gated field is an Option (see `gen_struct`).
            let decode = match self.flattened_selected(field) {
                Some(selected) => quote! {
                    if #selected { #decode } else { ::core::option::Option::None }
                },
                None => decode,
            };
            flattened.push(quote! { let #slot: #ty = #decode; });
        }

        let values: Vec<_> = self
            .fields
            .iter()
            .map(|field| {
                let ident = &field.ident;
                let slot = slots.field(field);
                if field.is_flattened() || shared.contains_key(&field.wire()) {
                    quote! { #ident: #slot }
                } else {
                    self.finish_field(&slots, field, &error_ty)
                }
            })
            .collect();
        let required = self.required_checks(&result, &error_ty);
        let map_mut = self.members().next().is_some().then(|| quote! { mut });
        let not_object = custom(quote! {
            ::std::format!("{}: expected an object, found {}", #name_str, #other)
        });
        Ok(quote! {
            #[allow(deprecated)]
            impl<'de> qapi::serde::Deserialize<'de> for #name {
                fn deserialize<#deserializer_ty>(#deserializer: #deserializer_ty) -> ::core::result::Result<Self, #deserializer_ty::Error>
                where #deserializer_ty: qapi::serde::Deserializer<'de> {
                    #type_guard
                    let #value = <qapi::serde_json::Value as qapi::serde::Deserialize>::deserialize(#deserializer)?;
                    let #map_mut #map = match #value {
                        qapi::serde_json::Value::Object(#map) => #map,
                        #other => return ::core::result::Result::Err(#not_object),
                    };
                    #union_payload
                    #(#members)*
                    #alternate_slots
                    #alternates
                    #(#flattened)*
                    let #result = #name { #(#values),* };
                    #required
                    ::core::result::Result::Ok(#result)
                }
            }
        })
    }
}
