//! `QapiCommand` and `QapiEvent` implementations.

use crate::attrs::{AttrRange, TypeAttrs};
use crate::support::{
    HelperNames, Version, active_check, decoding_selects, flag_at_newest, unavailable_decode_error,
    unavailable_error, version_range_check,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::ext::IdentExt;
use syn::{LitStr, Type};

/// Refuse command keys on an item that is no command, an item that is both
/// a command and an event, and a return type history that is not one: a
/// single type, types out of order, outside the command's lifetime or
/// repeated by the next one, a first type not added with the command, which
/// therefore has `added`, or a newest one that disagrees with `returns`;
/// and a history without `returns_enum`, the name of its enum, or that name
/// without a history or where it is no identifier.
pub(crate) fn check_command_keys(ta: &TypeAttrs) -> syn::Result<()> {
    if let (Some(_), Some(event)) = (ta.command, ta.event) {
        return Err(syn::Error::new(
            event,
            "qapi: an item is a `command` or an `event`, not both",
        ));
    }
    if ta.command.is_none()
        && let Some((span, key)) = ta.command_keys().next()
    {
        return Err(syn::Error::new(
            span,
            format!("qapi: `{key}` applies only to commands"),
        ));
    }
    if let [(_, only)] = ta.returns_history.as_slice() {
        return Err(syn::Error::new_spanned(
            only,
            "qapi: `returns(...)` is the history of a return type that changed, \
             so it lists two or more; give one type as `returns = \"Type\"`",
        ));
    }
    let same = |text: &LitStr| return_type(text).map(|ty| ty.to_token_stream().to_string());
    let mut previous: Option<(Version, &LitStr)> = None;
    for (added, value) in &ta.returns_history {
        let error = |message: String| Err(syn::Error::new_spanned(value, message));
        return_type(value)?;
        match (previous, ta.since) {
            (Some((before, _)), _) if *added <= before => {
                return error(
                    "qapi: `returns(...)` lists the return types in release order, \
                     each added after the one before"
                        .to_owned(),
                );
            }
            (None, Some(since)) if *added != since => {
                return error(format!(
                    "qapi: the first `returns(...)` is added with the command, at {since}"
                ));
            }
            // Releases before the first type would have none.
            (None, None) => {
                return error(
                    "qapi: the first `returns(...)` is added with the command, which needs \
                     `added`"
                        .to_owned(),
                );
            }
            (Some((_, before)), _) if same(before)? == same(value)? => {
                return error(
                    "qapi: `returns(...)` lists a return type once it changes, \
                     not the same type again"
                        .to_owned(),
                );
            }
            _ => {}
        }
        if let Some(until) = ta.until
            && *added >= until
        {
            return error(format!(
                "qapi: `returns(...)` (added {added}) must be earlier than the command's \
                 `removed` ({until})"
            ));
        }
        previous = Some((*added, value));
    }
    if let Some((_, newest)) = ta.returns_history.last() {
        let Some(returns) = &ta.returns else {
            return Err(syn::Error::new_spanned(
                newest,
                "qapi: `returns(...)` comes with `returns = \"Type\"`, the newest return type",
            ));
        };
        if same(returns)? != same(newest)? {
            return Err(syn::Error::new_spanned(
                returns,
                "qapi: `returns` is the newest `returns(...)` value",
            ));
        }
    }
    match (&ta.returns_enum, ta.returns_history.first()) {
        (None, Some((_, first))) => {
            return Err(syn::Error::new_spanned(
                first,
                "qapi: a `returns(...)` history needs `returns_enum = \"Name\"`, \
                 the name of the enum of its return types",
            ));
        }
        (Some(name), None) => {
            return Err(syn::Error::new_spanned(
                name,
                "qapi: `returns_enum` names the enum of a `returns(...)` history",
            ));
        }
        (Some(name), Some(_)) => {
            returns_enum_ident(name)?;
        }
        (None, None) => {}
    }
    Ok(())
}

/// The enum of a command's return types that `returns_enum` names.
fn returns_enum_ident(text: &LitStr) -> syn::Result<syn::Ident> {
    text.parse().map_err(|_| {
        syn::Error::new_spanned(
            text,
            "qapi: `returns_enum` is the name of an enum, such as \"QueryTargetReturns\"",
        )
    })
}

/// The type of a `returns` literal. Its tokens keep the literal's span, and
/// so do its errors.
fn return_type(text: &LitStr) -> syn::Result<Type> {
    text.parse().map_err(|error| {
        syn::Error::new_spanned(text, format!("qapi: `returns` is not a Rust type: {error}"))
    })
}

pub(crate) fn gen_command_impl(
    name: &syn::Ident,
    vis: &syn::Visibility,
    wn: &str,
    ta: &TypeAttrs,
) -> syn::Result<TokenStream2> {
    let entries = return_entries(ta)?;
    let (returns, returns_enum) = match entries.as_slice() {
        [] => {
            let returns = match &ta.returns {
                Some(text) => return_type(text)?,
                None => syn::parse_quote! { () },
            };
            (returns.into_token_stream(), TokenStream2::new())
        }
        // One return type in the whole history.
        [(.., ty)] => (ty.into_token_stream(), TokenStream2::new()),
        _ => {
            let enum_name = returns_enum_ident(
                ta.returns_enum
                    .as_ref()
                    .expect("check_command_keys requires `returns_enum` with a history"),
            )?;
            let returns_enum = gen_returns_enum(&enum_name, vis, wn, &entries);
            (enum_name.into_token_stream(), returns_enum)
        }
    };
    let oob = flag_at_newest(&ta.allow_oob);
    let preconfig = flag_at_newest(&ta.allow_preconfig);
    let success = !flag_at_newest(&ta.no_success_response);
    // The `unstable` feature without versions marks every release.
    let unstable_feature = ta.unstable.is_empty() && ta.has_feature("unstable");
    let unstable = unstable_feature || flag_at_newest(&ta.unstable);
    let unstable_at = if unstable_feature {
        quote! { true }
    } else {
        command_flag_check(&ta.unstable)
    };
    let oob_at = command_flag_check(&ta.allow_oob);
    let preconfig_at = command_flag_check(&ta.allow_preconfig);
    let no_success_at = command_flag_check(&ta.no_success_response);

    let since = ta.since.unwrap_or(Version::ZERO);
    let until = match ta.until {
        Some(until) => quote! { ::core::option::Option::Some(#until) },
        None => quote! { ::core::option::Option::None },
    };
    // The runs of preconfig, each within the command's lifetime. Report the
    // first introduction even when there are several.
    let preconfig_runs: Vec<Version> = ta
        .allow_preconfig
        .iter()
        .map(|range| range.added.or(ta.since).unwrap_or(Version::ZERO))
        .collect();
    let preconfig_since = match preconfig_runs.iter().min() {
        Some(since) => quote! { ::core::option::Option::Some(#since) },
        None => quote! { ::core::option::Option::None },
    };
    let preconfig_impl = (!preconfig_runs.is_empty()).then(|| {
        quote! {
            #[allow(deprecated)]
            impl ::qapi::PreconfigSafe for #name {}
        }
    });

    Ok(quote! {
        #returns_enum

        #[allow(deprecated)]
        impl ::qapi::QapiCommand for #name {
            type Returns = #returns;
            const NAME: &'static str = #wn;
            const ALLOW_OOB: bool = #oob;
            const ALLOW_PRECONFIG: bool = #preconfig;
            const SUCCESS_RESPONSE: bool = #success;
            const UNSTABLE: bool = #unstable;
            const AVAILABLE_SINCE: (u32, u32, u32) = #since;
            const REMOVED_IN: ::core::option::Option<(u32, u32, u32)> = #until;
            const PRECONFIG_SINCE: ::core::option::Option<(u32, u32, u32)> = #preconfig_since;
            fn allows_oob(_version: (u32, u32, u32)) -> bool { #oob_at }
            fn allows_preconfig(_version: (u32, u32, u32)) -> bool { #preconfig_at }
            fn sends_success_response(_version: (u32, u32, u32)) -> bool { !(#no_success_at) }
            fn is_unstable(_version: (u32, u32, u32)) -> bool { #unstable_at }
        }

        #preconfig_impl
    })
}

fn command_flag_check(ranges: &[AttrRange]) -> TokenStream2 {
    let checks = ranges.iter().map(|flag| {
        let since = flag.added.unwrap_or(Version::ZERO);
        match flag.removed {
            Some(until) => quote! { (#since..#until).contains(&_version) },
            None => quote! { _version >= #since },
        }
    });
    quote! { false #(|| (#checks))* }
}

/// A return type of a command's history: from the release that introduced
/// it until the next one, the variant named by that release, and the type.
type ReturnEntry = (Version, Option<Version>, syn::Ident, Type);

fn return_entries(ta: &TypeAttrs) -> syn::Result<Vec<ReturnEntry>> {
    let mut entries = Vec::new();
    for (index, (added, text)) in ta.returns_history.iter().enumerate() {
        let until = ta.returns_history.get(index + 1).map(|(next, _)| *next);
        let variant = format_ident!("V{}_{}_{}", added.0, added.1, added.2);
        entries.push((*added, until, variant, return_type(text)?));
    }
    Ok(entries)
}

/// The reply type of a command whose return type changed across its
/// releases: an enum with a variant per return type,
/// named by the release that introduced it. A reply decodes into the variant
/// of the peer's release in a version scope, and into the newest unscoped.
fn gen_returns_enum(
    enum_name: &syn::Ident,
    vis: &syn::Visibility,
    wn: &str,
    entries: &[ReturnEntry],
) -> TokenStream2 {
    let name_str = enum_name.unraw().to_string();
    let mut names = HelperNames::new(enum_name, [], entries.iter().map(|(.., ty)| ty));
    let serializer_ty = names.ty("Serializer");
    let serializer = names.ident("serializer");
    let deserializer_ty = names.ty("Deserializer");
    let deserializer = names.ident("deserializer");
    let inner = names.ident("inner");
    let other = names.ident("other");
    let map = names.ident("map");

    let variants = entries.iter().map(|(added, until, variant, ty)| {
        let doc = match until {
            Some(until) => format!(" The return type from QEMU {added} until {until}."),
            None => format!(" The return type from QEMU {added}."),
        };
        quote! {
            #[doc = #doc]
            #variant(#ty),
        }
    });
    let ser_arms = entries.iter().map(|(added, until, variant, _)| {
        let available = version_range_check(Some(*added), *until);
        let unavailable = unavailable_error(&format!("{name_str}::{variant}"));
        quote! {
            #enum_name::#variant(ref #inner) => {
                if !#available {
                    return ::core::result::Result::Err(#unavailable);
                }
                ::qapi::__private::serde::Serialize::serialize(#inner, #serializer)
            }
        }
    });
    // Newest first, so that unscoped decoding takes the first.
    let decodes = entries.iter().rev().enumerate().map(|(index, (added, until, variant, ty))| {
        let latest = index == 0;
        let selected = decoding_selects(quote! { #latest }, active_check(Some(*added), *until));
        let decode = if matches!(ty, Type::Tuple(tuple) if tuple.elems.is_empty()) {
            // A command without a return value replies `{}`.
            let message = format!("{name_str}::{variant}: expected an empty object, found {{}}");
            quote! {
                match <::qapi::__private::serde_json::Value as ::qapi::__private::serde::Deserialize>::deserialize(#deserializer)? {
                    ::qapi::__private::serde_json::Value::Null => ::core::result::Result::Ok(#enum_name::#variant(())),
                    ::qapi::__private::serde_json::Value::Object(#map) if #map.is_empty() => ::core::result::Result::Ok(#enum_name::#variant(())),
                    #other => ::core::result::Result::Err(::qapi::__private::serde::de::Error::custom(::std::format!(#message, #other))),
                }
            }
        } else {
            quote! { <#ty as ::qapi::__private::serde::Deserialize>::deserialize(#deserializer).map(#enum_name::#variant) }
        };
        quote! {
            if #selected {
                return #decode;
            }
        }
    });
    let unavailable = unavailable_decode_error(&name_str);
    let doc = format!(
        " The reply of `{wn}`, whose return type changed across QEMU releases: the return type of the peer's release."
    );
    quote! {
        #[doc = #doc]
        #[allow(deprecated, clippy::large_enum_variant)]
        #[derive(::core::fmt::Debug, ::core::clone::Clone)]
        #[non_exhaustive]
        #vis enum #enum_name {
            #(#variants)*
        }

        #[allow(deprecated)]
        impl ::qapi::__private::serde::Serialize for #enum_name {
            fn serialize<#serializer_ty>(&self, #serializer: #serializer_ty) -> ::core::result::Result<#serializer_ty::Ok, #serializer_ty::Error>
            where #serializer_ty: ::qapi::__private::serde::Serializer {
                match *self { #(#ser_arms)* }
            }
        }

        #[allow(deprecated)]
        impl<'de> ::qapi::__private::serde::Deserialize<'de> for #enum_name {
            fn deserialize<#deserializer_ty>(#deserializer: #deserializer_ty) -> ::core::result::Result<Self, #deserializer_ty::Error>
            where #deserializer_ty: ::qapi::__private::serde::Deserializer<'de> {
                #(#decodes)*
                ::core::result::Result::Err(#unavailable)
            }
        }
    }
}

pub(crate) fn gen_event_impl(name: &syn::Ident, wn: &str) -> TokenStream2 {
    quote! {
        #[allow(deprecated)]
        impl ::qapi::QapiEvent for #name {
            const NAME: &'static str = #wn;
        }
    }
}
