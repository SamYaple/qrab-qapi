//! Expansion of string enums, alternates and union branch enums.

use crate::attrs::{ItemAttrs, TypeAttrs};
use crate::bounds::{VersionBounds, excluded_by_bounds};
use crate::support::{
    HelperNames, active_check, annotate_item, check_disjoint_wires, decoding_selects,
    deprecation_warning, newest_first, outside_bounds_clause, type_deserialization_guard,
    type_serialization_guard, unavailable_decode_error, unavailable_error, version_range_check,
    wire_name,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::BTreeMap;
use syn::ext::IdentExt;
use syn::{DeriveInput, Fields, Type};

struct VariantInfo {
    ident: syn::Ident,
    attrs: ItemAttrs,
    kind: VariantKind,
}

enum VariantKind {
    Unit,
    Tuple(Box<Type>),
}

impl VariantInfo {
    fn wire(&self) -> String {
        wire_name(&self.attrs.name, &self.ident)
    }

    /// The Rust name, `Type::Variant`, that errors and warnings use.
    fn path(&self, enum_name: &syn::Ident) -> String {
        format!("{}::{}", enum_name.unraw(), self.ident.unraw())
    }
}

/// The implementations of an enum, whose variants this checks and then
/// reduces to those within the bounds.
pub(crate) fn gen_enum(
    input_ast: &mut DeriveInput,
    ta: &TypeAttrs,
    bounds: &VersionBounds,
) -> syn::Result<TokenStream2> {
    let enum_name = input_ast.ident.clone();
    if let Some((span, key)) = ta.struct_key() {
        return Err(syn::Error::new(
            span,
            format!("qapi: `{key}` applies only to structs"),
        ));
    }

    // Entry-point dispatches on the same `input_ast.data` variant, so this
    // branch is an internal invariant, not user-reachable.
    let syn::Data::Enum(data_enum) = &mut input_ast.data else {
        unreachable!("gen_enum called with non-enum DeriveInput");
    };

    let mut parsed = Vec::new();
    for variant in std::mem::take(&mut data_enum.variants) {
        let attrs = ItemAttrs::parse(&variant.attrs, ta)?;
        if let Some((span, key)) = attrs.field_key() {
            return Err(syn::Error::new(
                span,
                format!("qapi: `{key}` applies only to struct fields"),
            ));
        }
        let kind = match &variant.fields {
            Fields::Unit => VariantKind::Unit,
            Fields::Unnamed(f) if f.unnamed.len() == 1 => {
                if let Some(attr) = f.unnamed[0].attrs.iter().find(|attr| {
                    attr.path()
                        .segments
                        .last()
                        .is_some_and(|last| last.ident == "qapi")
                }) {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "qapi: the keys of a variant go on the variant, not on its payload",
                    ));
                }
                VariantKind::Tuple(Box::new(f.unnamed[0].ty.clone()))
            }
            Fields::Unnamed(f) => {
                return Err(syn::Error::new_spanned(
                    f,
                    "qapi: tuple enum variants must wrap exactly one type",
                ));
            }
            Fields::Named(f) => {
                return Err(syn::Error::new_spanned(
                    f,
                    "qapi: enum variants are unit or wrap one type; use a struct for named fields",
                ));
            }
        };
        let info = VariantInfo {
            ident: variant.ident.clone(),
            attrs,
            kind,
        };
        parsed.push((variant, info));
    }
    check_disjoint_wires(
        parsed
            .iter()
            .map(|(_, info)| (info.wire(), &info.attrs, &info.ident)),
    )?;

    let is_string_enum = ta.union_branch.is_none()
        && parsed
            .iter()
            .all(|(_, info)| matches!(info.kind, VariantKind::Unit));
    if ta.union_branch.is_some()
        && let Some(name) = &ta.name
    {
        return Err(syn::Error::new_spanned(
            name,
            "qapi: a union branch enum has no QAPI name; it serializes inside its union",
        ));
    }

    let mut variants = Vec::new();
    let mut kept = syn::punctuated::Punctuated::new();
    for (mut variant, info) in parsed {
        if excluded_by_bounds(bounds, info.attrs.since, info.attrs.until) {
            continue;
        }
        annotate_item(&mut variant.attrs, bounds, &info.attrs, "variant");
        if let VariantKind::Tuple(_) = info.kind {
            // The payload's type may be Rust `#[deprecated]`; declaring the
            // variant is no use of it, so it warns only where code names it.
            variant
                .attrs
                .push(syn::parse_quote! { #[allow(deprecated)] });
        }
        variants.push(info);
        kept.push(variant);
    }
    data_enum.variants = kept;

    let types = variants.iter().filter_map(|variant| match &variant.kind {
        VariantKind::Tuple(ty) => Some(ty.as_ref()),
        VariantKind::Unit => None,
    });
    let mut expansion = EnumExpansion {
        names: HelperNames::new(&enum_name, [], types),
        name: &enum_name,
        name_str: enum_name.unraw().to_string(),
        ta,
        bounds,
        variants: &variants,
    };
    // The QAPI name of the type, which Serde's data model carries.
    let qapi_name = ta
        .name
        .as_ref()
        .map_or_else(|| expansion.name_str.clone(), |name| name.value());
    Ok(if is_string_enum {
        let ser = expansion.string_ser(&qapi_name);
        let de = expansion.string_de();
        let display = expansion.display();
        quote! { #ser #de #display }
    } else {
        let ser = expansion.alternate_ser(ta.union_branch.is_none().then_some(qapi_name.as_str()));
        let de = expansion.alternate_de();
        let flat_union = if ta.union_branch.is_some() {
            expansion.flat_union()
        } else {
            TokenStream2::new()
        };
        quote! { #ser #de #flat_union }
    })
}

struct EnumExpansion<'a> {
    names: HelperNames,
    name: &'a syn::Ident,
    /// The unraw Rust name, for errors.
    name_str: String,
    ta: &'a TypeAttrs,
    bounds: &'a VersionBounds,
    variants: &'a [VariantInfo],
}

impl EnumExpansion<'_> {
    /// Refuse a variant outside its lifetime, and warn once where the peer's
    /// version deprecates it.
    fn variant_serialization_guard(&self, variant: &VariantInfo) -> TokenStream2 {
        let path = variant.path(self.name);
        let check =
            version_range_check(variant.attrs.since, variant.attrs.until).map(|available| {
                let unavailable = unavailable_error(&path);
                quote! {
                    if !#available {
                        return ::core::result::Result::Err(#unavailable);
                    }
                }
            });
        let warning = deprecation_warning(&path, &variant.attrs.deprecated);
        quote! { #check #warning }
    }

    /// Whether decoding takes `variant`: the one active for the scoped
    /// version, or the newest of its wire name unscoped.
    fn selected(&self, variant: &VariantInfo) -> TokenStream2 {
        let mut same_wire: Vec<_> = self
            .variants
            .iter()
            .filter(|candidate| candidate.wire() == variant.wire())
            .collect();
        newest_first(&mut same_wire, |variant| &variant.attrs);
        let latest = same_wire[0].ident == variant.ident;
        decoding_selects(
            quote! { #latest },
            active_check(variant.attrs.since, variant.attrs.until),
        )
    }

    /// The variants of each wire name, newest first.
    fn wire_groups(&self) -> BTreeMap<String, Vec<&VariantInfo>> {
        let mut groups = BTreeMap::<String, Vec<&VariantInfo>>::new();
        for variant in self.variants {
            groups.entry(variant.wire()).or_default().push(variant);
        }
        for group in groups.values_mut() {
            newest_first(group, |variant| &variant.attrs);
        }
        groups
    }

    /// The error for a wire name that no variant has, listing those that
    /// decoding takes in the current version context. In a scope outside
    /// the bounds, whose own variants the build may have left out, it names
    /// the bound instead.
    fn unknown_error(
        &self,
        kind: &str,
        found: &syn::Ident,
        bindings: [syn::Ident; 5],
    ) -> TokenStream2 {
        let [expected_var, wire_var, selected_var, list, version] = bindings;
        let groups = self.wire_groups();
        let count = groups.len();
        let expected = groups.into_iter().map(|(wire, group)| {
            let selected = group.iter().map(|variant| self.selected(variant));
            quote! { (#wire, false #(|| #selected)*) }
        });
        let message = format!("unknown {kind} `{{}}`, {{}}");
        // Serde's wording for an unknown variant.
        let known = quote! {{
            let #expected_var: ::std::vec::Vec<&str> = <[(&str, bool); #count]>::into_iter([#(#expected),*])
                .filter_map(|(#wire_var, #selected_var)| #selected_var.then_some(#wire_var))
                .collect();
            let #list = match #expected_var.as_slice() {
                [] => ::std::string::String::from("there are no variants"),
                [#wire_var] => ::std::format!("expected `{}`", #wire_var),
                [#wire_var, #selected_var] => ::std::format!("expected `{}` or `{}`", #wire_var, #selected_var),
                _ => ::std::format!("expected one of `{}`", #expected_var.join("`, `")),
            };
            ::core::result::Result::Err(qapi::serde::de::Error::custom(::std::format!(#message, #found, #list)))
        }};
        let Some(clause) = outside_bounds_clause(self.bounds) else {
            return known;
        };
        let outside = format!("unknown {kind} `{{}}` in QEMU {{}}.{{}}.{{}}{{}}");
        quote! {{
            let #list: &str = #clause;
            if #list.is_empty() {
                #known
            } else {
                let #version = qapi::qemu_version();
                ::core::result::Result::Err(qapi::serde::de::Error::custom(::std::format!(
                    #outside, #found, #version.0, #version.1, #version.2, #list,
                )))
            }
        }}
    }

    fn string_ser(&mut self, qapi_name: &str) -> TokenStream2 {
        let name = self.name;
        let serializer_ty = self.names.ty("Serializer");
        let serializer = self.names.ident("serializer");
        let type_guard = type_serialization_guard(name, self.ta);
        let arms = self.variants.iter().enumerate().map(|(index, variant)| {
            let ident = &variant.ident;
            let wire = variant.wire();
            let index = index as u32;
            let guard = self.variant_serialization_guard(variant);
            quote! {
                #name::#ident => {
                    #guard
                    qapi::serde::Serializer::serialize_unit_variant(#serializer, #qapi_name, #index, #wire)
                }
            }
        });
        quote! {
            #[allow(deprecated)]
            impl qapi::serde::Serialize for #name {
                fn serialize<#serializer_ty>(&self, #serializer: #serializer_ty) -> ::core::result::Result<#serializer_ty::Ok, #serializer_ty::Error>
                where #serializer_ty: qapi::serde::Serializer {
                    #type_guard
                    match *self { #(#arms)* }
                }
            }
        }
    }

    fn string_de(&mut self) -> TokenStream2 {
        let name = self.name;
        let deserializer_ty = self.names.ty("Deserializer");
        let deserializer = self.names.ident("deserializer");
        let wire_var = self.names.ident("wire");
        let other = self.names.ident("other");
        let bindings = [
            self.names.ident("expected"),
            self.names.ident("wire"),
            self.names.ident("selected"),
            self.names.ident("list"),
            self.names.ident("version"),
        ];
        let type_guard = type_deserialization_guard(name, self.ta, self.bounds);
        let arms = self.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let wire = variant.wire();
            let selected = self.selected(variant);
            quote! { #wire if #selected => ::core::result::Result::Ok(#name::#ident), }
        });
        // In a scope, a wire name whose variants are all outside it.
        let unavailable = self.wire_groups().into_iter().map(|(wire, group)| {
            let error = unavailable_decode_error(&group[0].path(name), self.bounds);
            quote! { #wire => ::core::result::Result::Err(#error), }
        });
        let unknown = self.unknown_error("variant", &other, bindings);
        quote! {
            #[allow(deprecated)]
            impl<'de> qapi::serde::Deserialize<'de> for #name {
                fn deserialize<#deserializer_ty>(#deserializer: #deserializer_ty) -> ::core::result::Result<Self, #deserializer_ty::Error>
                where #deserializer_ty: qapi::serde::Deserializer<'de> {
                    #type_guard
                    let #wire_var = <::std::string::String as qapi::serde::Deserialize>::deserialize(#deserializer)?;
                    match #wire_var.as_str() {
                        #(#arms)*
                        #(#unavailable)*
                        #other => #unknown,
                    }
                }
            }
        }
    }

    /// `Display` writes a string enum's wire name.
    fn display(&mut self) -> TokenStream2 {
        let name = self.name;
        let formatter = self.names.ident("formatter");
        let arms = self.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let wire = variant.wire();
            quote! { #name::#ident => ::core::fmt::Formatter::write_str(#formatter, #wire), }
        });
        quote! {
            #[allow(deprecated)]
            impl ::core::fmt::Display for #name {
                fn fmt(&self, #formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match *self { #(#arms)* }
                }
            }
        }
    }

    /// Alternates and union branches serialize as their payload. An
    /// alternate passes it to Serde as a newtype struct named by its QAPI
    /// name, which JSON leaves out; a union branch has no name of its own,
    /// since it serializes inside its union.
    fn alternate_ser(&mut self, qapi_name: Option<&str>) -> TokenStream2 {
        let name = self.name;
        let serializer_ty = self.names.ty("Serializer");
        let serializer = self.names.ident("serializer");
        let inner = self.names.ident("inner");
        let type_guard = type_serialization_guard(name, self.ta);
        let arms = self.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let guard = self.variant_serialization_guard(variant);
            let (pattern, value) = match &variant.kind {
                VariantKind::Tuple(_) => (quote! { #name::#ident(ref #inner) }, quote! { #inner }),
                VariantKind::Unit => (quote! { #name::#ident }, quote! { &() }),
            };
            let serialize = match qapi_name {
                Some(qapi_name) => quote! {
                    qapi::serde::Serializer::serialize_newtype_struct(#serializer, #qapi_name, #value)
                },
                None => quote! { qapi::serde::Serialize::serialize(#value, #serializer) },
            };
            quote! {
                #pattern => {
                    #guard
                    #serialize
                }
            }
        });
        quote! {
            #[allow(deprecated)]
            impl qapi::serde::Serialize for #name {
                fn serialize<#serializer_ty>(&self, #serializer: #serializer_ty) -> ::core::result::Result<#serializer_ty::Ok, #serializer_ty::Error>
                where #serializer_ty: qapi::serde::Serializer {
                    #type_guard
                    match *self { #(#arms)* }
                }
            }
        }
    }

    /// Alternates decode by the shape of the value: the narrowest variant
    /// that decoding takes and that accepts it.
    fn alternate_de(&mut self) -> TokenStream2 {
        let name = self.name;
        let name_str = &self.name_str;
        let deserializer_ty = self.names.ty("Deserializer");
        let deserializer = self.names.ident("deserializer");
        let value = self.names.ident("value");
        let inner = self.names.ident("inner");
        let errors = self.names.ident("errors");
        let error = self.names.ident("error");
        let message = self.names.ident("message");
        let version = self.names.ident("version");
        let type_guard = type_deserialization_guard(name, self.ta, self.bounds);
        let outside =
            outside_bounds_clause(self.bounds).map(|clause| quote! { #message.push_str(#clause); });
        // Stable-sort variants by specificity so overlapping shapes (e.g. a
        // primitive vs an all-optional-fields struct) resolve to the least
        // ambiguous match instead of silently picking whichever happens to
        // come first in declaration order.
        let mut ordered: Vec<&VariantInfo> = self.variants.iter().collect();
        ordered.sort_by_key(|variant| alternate_shape(&variant.kind));
        let integer_selected: Vec<_> = ordered
            .iter()
            .filter(|variant| alternate_shape(&variant.kind) == AlternateShape::Integer)
            .map(|variant| self.selected(variant))
            .collect();
        let mut tries = Vec::new();
        let mut unavailable = Vec::new();
        for variant in ordered {
            let ident = &variant.ident;
            let label = variant.ident.unraw().to_string();
            let selected = self.selected(variant);
            // QEMU gives an integer to an integer variant whenever there is
            // one, and to a number variant only otherwise, so in a version
            // scope the range error of an integer variant does not fall
            // through to a float. Unscoped decoding stays lenient and takes
            // the float, so that recorded values remain readable.
            let eligible = if alternate_shape(&variant.kind) == AlternateShape::Float
                && !integer_selected.is_empty()
            {
                quote! {
                    !(qapi::scoped_qemu_version().is_some()
                        && (#value.is_i64() || #value.is_u64())
                        && (false #(|| #integer_selected)*))
                }
            } else {
                quote! { true }
            };
            let accepts = match &variant.kind {
                VariantKind::Tuple(ty) => quote! {
                    qapi::serde_json::from_value::<#ty>(::core::clone::Clone::clone(&#value)).is_ok()
                },
                VariantKind::Unit => quote! { #value.is_null() },
            };
            tries.push(match &variant.kind {
                VariantKind::Tuple(ty) => quote! {
                    if #selected && #eligible {
                        match qapi::serde_json::from_value::<#ty>(::core::clone::Clone::clone(&#value)) {
                            ::core::result::Result::Ok(#inner) => return ::core::result::Result::Ok(#name::#ident(#inner)),
                            ::core::result::Result::Err(#error) => #errors.push(::std::format!("{}: {}", #label, #error)),
                        }
                    }
                },
                VariantKind::Unit => quote! {
                    if #selected && #eligible {
                        if #value.is_null() {
                            return ::core::result::Result::Ok(#name::#ident);
                        }
                        #errors.push(::std::format!("{}: expected null", #label));
                    }
                },
            });
            let error = unavailable_decode_error(&variant.path(name), self.bounds);
            unavailable.push(quote! {
                if qapi::scoped_qemu_version().is_some() && !#selected && #eligible && #accepts {
                    return ::core::result::Result::Err(#error);
                }
            });
        }
        quote! {
            #[allow(deprecated)]
            impl<'de> qapi::serde::Deserialize<'de> for #name {
                fn deserialize<#deserializer_ty>(#deserializer: #deserializer_ty) -> ::core::result::Result<Self, #deserializer_ty::Error>
                where #deserializer_ty: qapi::serde::Deserializer<'de> {
                    #type_guard
                    let #value = <qapi::serde_json::Value as qapi::serde::Deserialize>::deserialize(#deserializer)?;
                    let mut #errors: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
                    #(#tries)*
                    #(#unavailable)*
                    let mut #message = ::std::format!("no variant of {} accepts {}", #name_str, #value);
                    if let ::core::option::Option::Some(#version) = qapi::scoped_qemu_version() {
                        #message.push_str(&::std::format!(" in QEMU {}.{}.{}", #version.0, #version.1, #version.2));
                        #outside
                    }
                    if !#errors.is_empty() {
                        #message.push_str(&::std::format!(" ({})", #errors.join("; ")));
                    }
                    ::core::result::Result::Err(qapi::serde::de::Error::custom(#message))
                }
            }
        }
    }

    /// A union branch enum dispatches by its wire tag, taking the payload
    /// shape that decoding selects for the version.
    fn flat_union(&mut self) -> TokenStream2 {
        let name = self.name;
        let discriminator = self.names.ident("discriminator");
        let map = self.names.ident("map");
        let value = self.names.ident("value");
        let error = self.names.ident("error");
        let other = self.names.ident("other");
        let bindings = [
            self.names.ident("expected"),
            self.names.ident("wire"),
            self.names.ident("selected"),
            self.names.ident("list"),
            self.names.ident("version"),
        ];
        let type_guard = type_deserialization_guard(name, self.ta, self.bounds);
        let tags = self.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let wire = variant.wire();
            match variant.kind {
                VariantKind::Tuple(_) => {
                    quote! { #name::#ident(..) => ::core::option::Option::Some(#wire), }
                }
                VariantKind::Unit => {
                    quote! { #name::#ident => ::core::option::Option::Some(#wire), }
                }
            }
        });
        let arms = self.wire_groups().into_iter().map(|(wire, group)| {
            let candidates = group.iter().map(|variant| {
                let ident = &variant.ident;
                let selected = self.selected(variant);
                let path = variant.path(name);
                let decode = match variant.kind {
                    VariantKind::Tuple(_) => quote! {
                        qapi::serde_json::from_value(#value)
                            .map(#name::#ident)
                            .map_err(|#error| qapi::serde::de::Error::custom(::std::format!("{}: {}", #path, #error)))
                    },
                    VariantKind::Unit => quote! { ::core::result::Result::Ok(#name::#ident) },
                };
                quote! { if #selected { return #decode; } }
            });
            let unavailable = unavailable_decode_error(&group[0].path(name), self.bounds);
            quote! {
                #wire => {
                    #(#candidates)*
                    ::core::result::Result::Err(#unavailable)
                }
            }
        });
        let unknown = self.unknown_error(
            &format!("{} discriminator", self.name_str),
            &other,
            bindings,
        );
        quote! {
            #[allow(deprecated)]
            impl qapi::QapiFlatUnion for #name {
                fn wire_tag(&self) -> ::core::option::Option<&'static str> {
                    match *self { #(#tags)* }
                }
                fn deserialize_variant(#discriminator: &str, #map: qapi::serde_json::Map<::std::string::String, qapi::serde_json::Value>) -> ::core::result::Result<Self, qapi::serde_json::Error> {
                    <Self as qapi::QapiFlatUnion>::deserialize_variant_value(#discriminator, qapi::serde_json::Value::Object(#map))
                }
                fn deserialize_variant_value(#discriminator: &str, #value: qapi::serde_json::Value) -> ::core::result::Result<Self, qapi::serde_json::Error> {
                    #type_guard
                    match #discriminator {
                        #(#arms)*
                        #other => #unknown,
                    }
                }
            }
        }
    }
}

/// Try narrow JSON shapes before permissive payloads. Integers precede floats
/// because serde accepts integer JSON as f64, losing precision for large values.
/// Pre-2.10 QAPI alternates distinguish those two numeric branches.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AlternateShape {
    Null,
    Integer,
    Scalar,
    Float,
    Array,
    Object,
}

fn alternate_shape(kind: &VariantKind) -> AlternateShape {
    use AlternateShape::*;
    let ty = match kind {
        VariantKind::Unit => return Null,
        VariantKind::Tuple(ty) => ty.as_ref(),
    };
    match ty {
        // `Null(())` — empty tuple type, matches only JSON null
        Type::Tuple(t) if t.elems.is_empty() => Null,
        Type::Array(_) | Type::Slice(_) => Array,
        Type::Path(tp) => {
            let name = match tp.path.segments.last() {
                Some(seg) => seg.ident.to_string(),
                None => return Object,
            };
            match name.as_str() {
                // Rust primitives that map to JSON scalars
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
                | "u128" | "usize" => Integer,
                "bool" | "char" | "String" | "str" => Scalar,
                "f32" | "f64" => Float,
                // Array-like
                "Vec" => Array,
                // Everything else is object-like (structs, tagged unions, …)
                _ => Object,
            }
        }
        _ => Object,
    }
}
