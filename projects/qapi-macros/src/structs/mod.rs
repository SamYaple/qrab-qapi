//! Expansion of structs: records, the arguments of commands and the data of
//! events, with their flattened bases and unions.

use crate::attrs::{AttrRange, ItemAttrs, TypeAttrs};
use crate::bounds::{VersionBounds, excluded_by_bounds};
use crate::command::{check_command_keys, gen_command_impl, gen_event_impl};
use crate::support::{
    HelperNames, Version, active_check, annotate_item, check_disjoint_wires, decoding_selects,
    flag_at_newest, lasts_to_newest, newest_first, version_range_check, wire_name,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ext::IdentExt;
use syn::{DeriveInput, Fields, Type};

mod de;
mod ser;
#[cfg(test)]
mod tests;

pub(crate) struct FieldInfo {
    pub(crate) ident: syn::Ident,
    pub(crate) ty: Type,
    pub(crate) attrs: ItemAttrs,
}

impl FieldInfo {
    pub(crate) fn wire(&self) -> String {
        wire_name(&self.attrs.name, &self.ident)
    }

    pub(crate) fn is_option(&self) -> bool {
        inner_option_type(&self.ty).is_some()
    }

    pub(crate) fn is_flattened(&self) -> bool {
        self.attrs.flatten.is_some()
    }

    /// Whether the field has a version lifetime.
    pub(crate) fn is_gated(&self) -> bool {
        self.attrs.since.is_some() || self.attrs.until.is_some()
    }

    pub(crate) fn active(&self) -> TokenStream2 {
        active_check(self.attrs.since, self.attrs.until)
    }

    /// The runs of releases in which the field is its union's
    /// discriminator, from `added` (or the field's `added`) until `removed`
    /// (or the field's `removed`); `None` is a bound that neither gives.
    pub(crate) fn discriminator_runs(&self) -> Vec<(Option<Version>, Option<Version>)> {
        self.attrs
            .discriminator
            .iter()
            .map(|range| {
                (
                    range.added.or(self.attrs.since),
                    range.removed.or(self.attrs.until),
                )
            })
            .collect()
    }

    /// Whether the field is the discriminator in `qapi::qemu_version()`.
    pub(crate) fn discriminates(&self) -> TokenStream2 {
        let checks = self
            .discriminator_runs()
            .into_iter()
            .map(|(added, removed)| active_check(added, removed));
        quote! { (false #(|| #checks)*) }
    }

    /// Whether the field is the discriminator in the newest release the
    /// build represents.
    pub(crate) fn discriminates_at_newest(&self, bounds: &VersionBounds) -> bool {
        self.discriminator_runs()
            .into_iter()
            .any(|(added, removed)| {
                bounds
                    .max
                    .is_none_or(|max| added.is_none_or(|added| added <= max))
                    && lasts_to_newest(removed, bounds)
            })
    }
}

/// The implementations of a struct, whose fields this checks and then
/// reduces to those within the bounds.
pub(crate) fn gen_struct(
    input_ast: &mut DeriveInput,
    ta: &TypeAttrs,
    bounds: &VersionBounds,
) -> syn::Result<TokenStream2> {
    let struct_name = input_ast.ident.clone();
    if let Some(span) = ta.union_branch {
        return Err(syn::Error::new(
            span,
            "qapi: `union_branch` applies only to enums",
        ));
    }
    check_command_keys(ta)?;

    // Entry-point dispatches on the same `input_ast.data` variant, so this
    // branch is an internal invariant, not user-reachable.
    let syn::Data::Struct(data_struct) = &mut input_ast.data else {
        unreachable!("gen_struct called with non-struct DeriveInput");
    };
    let mut named = match &mut data_struct.fields {
        Fields::Unnamed(unnamed) => {
            return Err(syn::Error::new_spanned(
                unnamed,
                "qapi: tuple structs are not supported; use named fields",
            ));
        }
        Fields::Named(named) => Some(named),
        Fields::Unit => None,
    };

    let mut parsed = Vec::new();
    for field in named
        .as_mut()
        .map(|named| std::mem::take(&mut named.named))
        .into_iter()
        .flatten()
    {
        let attrs = ItemAttrs::parse(&field.attrs, ta)?;
        let info = FieldInfo {
            // Every field of a `Fields::Named` carries an identifier.
            ident: field.ident.clone().expect("named field"),
            ty: field.ty.clone(),
            attrs,
        };
        check_field(&info)?;
        parsed.push((field, info));
    }
    let infos: Vec<&FieldInfo> = parsed.iter().map(|(_, info)| info).collect();
    check_fields(ta, &infos)?;
    let discriminator_left_out = parsed.iter().any(|(_, info)| {
        !info.attrs.discriminator.is_empty()
            && excluded_by_bounds(bounds, info.attrs.since, info.attrs.until)
    });

    let mut fields = Vec::new();
    let mut kept = syn::punctuated::Punctuated::new();
    for (mut field, info) in parsed {
        if excluded_by_bounds(bounds, info.attrs.since, info.attrs.until) {
            continue;
        }
        annotate_item(&mut field.attrs, bounds, &info.attrs, "field");
        // The field's type may be Rust `#[deprecated]`; declaring the field
        // is no use of it, so it warns only where code names it.
        field.attrs.push(syn::parse_quote! { #[allow(deprecated)] });
        fields.push(info);
        kept.push(field);
    }
    if let Some(named) = named {
        named.named = kept;
    }

    let mut expansion = StructExpansion {
        names: HelperNames::new(
            &struct_name,
            fields.iter().map(|field| &field.ident),
            fields.iter().map(|field| &field.ty),
        ),
        name: &struct_name,
        name_str: struct_name.unraw().to_string(),
        wire_name: ta
            .name
            .as_ref()
            .map_or_else(|| struct_name.unraw().to_string(), |name| name.value()),
        ta,
        bounds,
        fields: &fields,
        discriminator_left_out,
    };
    let flattened = fields.iter().any(FieldInfo::is_flattened);
    let ser = if flattened {
        expansion.ser_flattened()
    } else {
        expansion.ser_plain()
    };
    let de = if flattened {
        expansion.de_flattened()?
    } else {
        expansion.de_plain()?
    };
    let command = if ta.command.is_some() {
        gen_command_impl(
            &struct_name,
            &input_ast.vis,
            &expansion.wire_name,
            ta,
            bounds,
        )?
    } else {
        TokenStream2::new()
    };
    let event = if ta.event.is_some() {
        gen_event_impl(&struct_name, &expansion.wire_name)
    } else {
        TokenStream2::new()
    };
    let display = expansion.display();
    Ok(quote! {
        #ser
        #de
        #command
        #event
        #display
    })
}

/// The keys of one field that contradict each other or its type.
fn check_field(field: &FieldInfo) -> syn::Result<()> {
    let attrs = &field.attrs;
    if attrs.name.as_deref() == Some("") {
        return Err(syn::Error::new_spanned(
            &field.ident,
            "qapi: the `name` of a field must not be empty",
        ));
    }
    if field.is_gated() && !field.is_option() {
        return Err(syn::Error::new_spanned(
            &field.ty,
            "qapi: a field with `added` or `removed` must be an `Option`, \
             since peers outside its lifetime do not send it",
        ));
    }
    if let Some(span) = attrs.required
        && !field.is_option()
    {
        return Err(syn::Error::new(
            span,
            "qapi: `required` marks an `Option` field that the peer's version requires",
        ));
    }
    if let Some(span) = attrs.union_field
        && attrs.flatten.is_none()
    {
        return Err(syn::Error::new(
            span,
            "qapi: a `union` field is `flatten`ed into the struct",
        ));
    }
    if attrs.flatten.is_some()
        && let Some(range) = attrs.discriminator.first()
    {
        return Err(syn::Error::new(
            range.span,
            "qapi: a `discriminator` is a member of the struct, not `flatten`ed",
        ));
    }
    if let Some(span) = attrs.flatten {
        let member_keys = [
            (attrs.name.is_some(), "name"),
            (attrs.required.is_some(), "required"),
            (!attrs.deprecated.is_empty(), "deprecated"),
            (!attrs.features.is_empty(), "feature"),
            (attrs.condition.is_some(), "condition"),
        ];
        if let Some((_, key)) = member_keys.into_iter().find(|(given, _)| *given) {
            return Err(syn::Error::new(
                span,
                format!(
                    "qapi: a `flatten`ed field has no member of its own, so it takes no `{key}`"
                ),
            ));
        }
    }
    Ok(())
}

/// The keys that only make sense together across the fields.
fn check_fields(ta: &TypeAttrs, fields: &[&FieldInfo]) -> syn::Result<()> {
    let mut unions = fields
        .iter()
        .copied()
        .filter_map(|field| Some((field.attrs.union_field?, field)));
    let union = unions.next();
    if let Some((span, _)) = unions.next() {
        return Err(syn::Error::new(
            span,
            "qapi: a struct has at most one `union` field",
        ));
    }
    let discriminators: Vec<&FieldInfo> = fields
        .iter()
        .copied()
        .filter(|field| !field.attrs.discriminator.is_empty())
        .collect();
    match (union, discriminators.first()) {
        (Some((span, _)), None) => {
            return Err(syn::Error::new(
                span,
                "qapi: a `union` field needs a `discriminator` field to select its branch",
            ));
        }
        (None, Some(discriminator)) => {
            return Err(syn::Error::new_spanned(
                &discriminator.ident,
                "qapi: a `discriminator` selects the branch of a `union` field, which the struct lacks",
            ));
        }
        (Some((_, union)), Some(_)) => {
            check_discriminator_runs(ta, union, &discriminators)?;
            let why = if discriminators.len() > 1 {
                Some("a union field with version-alternate discriminators must be `Option<T>`")
            } else if discriminators[0].is_option() {
                Some("a union field with an optional discriminator must be `Option<T>`")
            } else {
                None
            };
            if let Some(why) = why {
                option_inner(&union.ty, why)?;
            }
        }
        (None, None) => {}
    }
    if union.is_none()
        && let Some(range) = ta.simple_union.first()
    {
        return Err(syn::Error::new(
            range.span,
            "qapi: `simple_union` describes the layout of a `union` field, which the struct lacks",
        ));
    }
    let members: Vec<&FieldInfo> = fields
        .iter()
        .copied()
        .filter(|field| !field.is_flattened())
        .collect();
    // Fields that share a wire name thereby have lifetimes, and so are
    // options (see `check_field`).
    check_disjoint_wires(
        members
            .iter()
            .map(|field| (field.wire(), &field.attrs, &field.ident)),
    )
}

/// A run of releases in which a field is the discriminator, with the
/// type's lifetime for a bound that the field leaves open.
struct DiscriminatorRun<'a> {
    field: &'a FieldInfo,
    /// The field's place among the discriminators, for errors.
    order: usize,
    range: &'a AttrRange,
    start: Option<Version>,
    end: Option<Version>,
}

/// Check the runs of the discriminators of `union`: each lies within its
/// field's lifetime and gives no bound that is the field's own, and
/// together they cover the union field's lifetime without overlapping, so
/// that in each release that has the union one member selects its branch,
/// as `CpuInfoFast.target` takes over from `arch` in 3.0. A bound that the
/// field or the union field leaves open is the type's.
fn check_discriminator_runs(
    ta: &TypeAttrs,
    union: &FieldInfo,
    discriminators: &[&FieldInfo],
) -> syn::Result<()> {
    let mut runs = Vec::new();
    for (order, field) in discriminators.iter().copied().enumerate() {
        let attrs = &field.attrs;
        let (since, until) = (attrs.since.or(ta.since), attrs.until.or(ta.until));
        let owner = |own: Option<Version>| if own.is_some() { "field" } else { "type" };
        let (added_owner, removed_owner) = (owner(attrs.since), owner(attrs.until));
        for range in &attrs.discriminator {
            let error = |message: String| Err(syn::Error::new(range.span, message));
            if let Some(added) = range.added {
                if since == Some(added) {
                    return error(format!(
                        "qapi: `discriminator` (added {added}) starts with the field; leave its \
                         `added` out"
                    ));
                }
                if let Some(since) = since.filter(|since| added < *since) {
                    return error(format!(
                        "qapi: `discriminator` (added {added}) must not start before the \
                         {added_owner}'s `added` ({since})"
                    ));
                }
                if let Some(until) = until.filter(|until| added >= *until) {
                    return error(format!(
                        "qapi: `discriminator` (added {added}) must start before the \
                         {removed_owner}'s `removed` ({until})"
                    ));
                }
            }
            if let Some(removed) = range.removed {
                if until == Some(removed) {
                    return error(format!(
                        "qapi: `discriminator` (removed {removed}) ends with the field; leave \
                         its `removed` out"
                    ));
                }
                if let Some(since) = since.filter(|since| removed <= *since) {
                    return error(format!(
                        "qapi: `discriminator` (removed {removed}) must end after the \
                         {added_owner}'s `added` ({since})"
                    ));
                }
                if let Some(until) = until.filter(|until| removed > *until) {
                    return error(format!(
                        "qapi: `discriminator` (removed {removed}) must end by the \
                         {removed_owner}'s `removed` ({until})"
                    ));
                }
            }
            runs.push(DiscriminatorRun {
                field,
                order,
                range,
                start: range.added.or(since),
                end: range.removed.or(until),
            });
        }
    }
    // An open start sorts first, as `None` does.
    runs.sort_by_key(|run| run.start);
    let union_name = union.ident.unraw();
    let (union_since, union_until) = (
        union.attrs.since.or(ta.since),
        union.attrs.until.or(ta.until),
    );
    let gap = |run: &DiscriminatorRun, from: Option<Version>, until: Option<Version>| {
        let releases = match (from, until) {
            (Some(from), Some(until)) => format!("from {from} until {until}"),
            (Some(from), None) => format!("from {from} on"),
            (None, Some(until)) => format!("before {until}"),
            (None, None) => "in any release".to_owned(),
        };
        Err(syn::Error::new(
            run.range.span,
            format!("qapi: no `discriminator` selects the branch of `{union_name}` {releases}"),
        ))
    };
    let mut covered = union_since;
    for (index, run) in runs.iter().enumerate() {
        match index.checked_sub(1).map(|previous| &runs[previous]) {
            None => {
                if let Some(since) = union_since.filter(|_| run.start < union_since) {
                    return Err(syn::Error::new(
                        run.range.span,
                        format!(
                            "qapi: `discriminator` must not start before the `union` field's \
                             `added` ({since})"
                        ),
                    ));
                }
            }
            Some(previous) => {
                let overlaps = match (previous.end, run.start) {
                    (Some(end), Some(start)) => start < end,
                    _ => true,
                };
                if overlaps {
                    let message = if std::ptr::eq(previous.field, run.field) {
                        format!(
                            "qapi: the `discriminator` runs of `{}` overlap",
                            run.field.ident.unraw()
                        )
                    } else {
                        let (first, second) = if previous.order < run.order {
                            (previous.field, run.field)
                        } else {
                            (run.field, previous.field)
                        };
                        format!(
                            "qapi: discriminators `{}` and `{}` overlap; in each release one \
                             member selects the branch",
                            first.ident.unraw(),
                            second.ident.unraw()
                        )
                    };
                    return Err(syn::Error::new(run.range.span, message));
                }
            }
        }
        if run.start != covered {
            return gap(run, covered, run.start);
        }
        covered = run.end;
    }
    // A union has a discriminator (see `check_fields`), so a run.
    let last = runs.last().expect("a discriminator has a run");
    let ends_after = match (last.end, union_until) {
        (Some(end), Some(until)) => end > until,
        (None, Some(_)) => true,
        (_, None) => false,
    };
    if let Some(until) = union_until.filter(|_| ends_after) {
        return Err(syn::Error::new(
            last.range.span,
            format!("qapi: `discriminator` must end by the `union` field's `removed` ({until})"),
        ));
    }
    if last.end != union_until {
        return gap(last, last.end, union_until);
    }
    Ok(())
}

pub(crate) struct StructExpansion<'a> {
    pub(crate) names: HelperNames,
    pub(crate) name: &'a syn::Ident,
    /// The unraw Rust name, for errors.
    pub(crate) name_str: String,
    /// The QAPI name.
    pub(crate) wire_name: String,
    pub(crate) ta: &'a TypeAttrs,
    pub(crate) bounds: &'a VersionBounds,
    pub(crate) fields: &'a [FieldInfo],
    /// Whether the bounds left out a discriminator, whose releases decoding
    /// then cannot select a branch in.
    pub(crate) discriminator_left_out: bool,
}

impl StructExpansion<'_> {
    /// The Rust name, `Type.field`, that errors and warnings use.
    pub(crate) fn path(&self, field: &FieldInfo) -> String {
        format!("{}.{}", self.name_str, field.ident.unraw())
    }

    /// The modeled members, which have a wire name of their own.
    pub(crate) fn members(&self) -> impl Iterator<Item = &FieldInfo> {
        self.fields.iter().filter(|field| !field.is_flattened())
    }

    /// The members that share a wire name, by that name: the version
    /// alternates of a member whose type changed, such as a discriminator
    /// (`NumaOptions.type` is `NumaOptionsKind` until 2.9 and
    /// `NumaOptionsType` after), newest first.
    pub(crate) fn shared_wires(&self) -> std::collections::BTreeMap<String, Vec<&FieldInfo>> {
        let mut groups = std::collections::BTreeMap::<String, Vec<&FieldInfo>>::new();
        for field in self.members() {
            groups.entry(field.wire()).or_default().push(field);
        }
        groups.retain(|_, group| group.len() > 1);
        for group in groups.values_mut() {
            newest_first(group, |field| &field.attrs);
        }
        groups
    }

    /// Whether decoding takes a gated flattened field: in a scope, where it
    /// is active, and unscoped, unless it was removed by the newest release
    /// the build represents.
    pub(crate) fn flattened_selected(&self, field: &FieldInfo) -> Option<TokenStream2> {
        version_range_check(field.attrs.since, field.attrs.until).map(|active| {
            let latest = lasts_to_newest(field.attrs.until, self.bounds);
            decoding_selects(quote! { #latest }, active)
        })
    }

    /// Whether the historical simple-union layout, with the branch payload
    /// under `data`, applies. Unscoped decoding takes the newest layout;
    /// serialization and scoped decoding the version's.
    pub(crate) fn simple_union_layout(&self, decoding: bool) -> TokenStream2 {
        let ranges = &self.ta.simple_union;
        if ranges.is_empty() {
            return quote! { false };
        }
        let checks = ranges
            .iter()
            .map(|range| active_check(range.added, range.removed));
        let active = quote! { (false #(|| #checks)*) };
        if decoding {
            let latest = flag_at_newest(ranges, self.bounds);
            decoding_selects(quote! { #latest }, active)
        } else {
            active
        }
    }

    /// `Display` writes the QAPI name, followed for a record by the fields
    /// that are set, as `name(field=value, ...)` with each value in its
    /// `Debug` form: a member under its wire name and a flattened field,
    /// which has no `name`, under its Rust name.
    fn display(&mut self) -> TokenStream2 {
        let name = self.name;
        let wire_name = &self.wire_name;
        let formatter = self.names.ident("formatter");
        if self.ta.command.is_some() || self.fields.is_empty() {
            return quote! {
                #[allow(deprecated)]
                impl ::core::fmt::Display for #name {
                    fn fmt(&self, #formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(#formatter, #wire_name)
                    }
                }
            };
        }
        let separator = self.names.ident("separator");
        let value = self.names.ident("value");
        let fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let label = field.wire();
            let write = quote! {
                ::core::write!(#formatter, "{}{}={:?}", #separator, #label, #value)?;
                #separator = ", ";
            };
            if field.is_option() {
                quote! {
                    if let ::core::option::Option::Some(ref #value) = self.#ident {
                        #write
                    }
                }
            } else {
                quote! { { let #value = &self.#ident; #write } }
            }
        });
        quote! {
            #[allow(deprecated, unused_assignments)]
            impl ::core::fmt::Display for #name {
                fn fmt(&self, #formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(#formatter, #wire_name)?;
                    ::core::fmt::Formatter::write_str(#formatter, "(")?;
                    let mut #separator = "";
                    #(#fields)*
                    ::core::fmt::Formatter::write_str(#formatter, ")")
                }
            }
        }
    }
}

/// The discriminators among `fields`, of which one selects the branch in
/// each release of the union field (see `check_discriminator_runs`), unless
/// the bounds left it out.
pub(crate) fn discriminators(fields: &[FieldInfo]) -> Vec<&FieldInfo> {
    fields
        .iter()
        .filter(|field| !field.attrs.discriminator.is_empty())
        .collect()
}

/// The `T` of `Option<T>`, or an error spanned at `ty` explaining `why` it
/// must be an `Option`.
pub(crate) fn option_inner<'a>(ty: &'a Type, why: &str) -> syn::Result<&'a Type> {
    inner_option_type(ty).ok_or_else(|| syn::Error::new_spanned(ty, format!("qapi: {why}")))
}

/// Extract the inner type `T` from `Option<T>`. Returns `None` if the type
/// is not `Option`.
pub(crate) fn inner_option_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
        && seg.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return Some(inner);
    }
    None
}
