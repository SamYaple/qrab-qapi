//! Lowering of parsed QAPI definitions into the Rust IR.
//!
//! The lookup tables that these functions take hold the definitions lowered
//! so far, keyed by QAPI name.

use super::{
    ARGUMENTS_TYPE, Attribute, Enum, EnumVariant, EnumVariantKind, Metadata, SchemaItem, Struct,
    StructField, builtin_type,
};
use crate::qapi_ir::{rustify_field_name, rustify_name, rustify_variant};
use crate::{
    MembersOrRef, QapiAlternate, QapiAlternative, QapiAlternatives, QapiCommand, QapiEnum,
    QapiEnumValue, QapiEvent, QapiMember, QapiStruct, QapiTypeRef, QapiUnion,
    extract_since_from_comment,
};
use std::collections::{HashMap, HashSet};

/// A definition that the IR cannot represent, or a union passed to the
/// function for the other kind of union.
///
/// QEMU's generator checks schemas from 2.4 on (before, an anonymous struct
/// could stand anywhere), and the parser also accepts what it rejects. The
/// lowering functions are public, so they report a definition that the IR
/// cannot represent instead of assuming a well-formed schema; no pinned
/// QEMU schema contains one.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LowerError {
    /// A member, branch or alternative that names no type, or a command's
    /// `returns` that names none.
    #[error("QAPI {definition}: '{member}' has no type")]
    MissingType { definition: String, member: String },
    /// An anonymous struct outside a struct member, such as a pre-2.4 inline
    /// dictionary as a command's return type or as a union or alternate branch.
    #[error(
        "QAPI {definition}: '{member}' is an anonymous struct, which only struct members may be"
    )]
    AnonymousStruct { definition: String, member: String },
    /// A base, argument, event data or discriminator type that the lookup
    /// tables do not hold. A discriminator may also have a builtin type.
    #[error("QAPI {definition} references missing type {name}")]
    UnknownType { definition: String, name: String },
    /// A union discriminator that is not a member of the union's base.
    #[error("QAPI union {union} has no discriminator field {discriminator} in its base")]
    MissingDiscriminator {
        union: String,
        discriminator: String,
    },
    /// A simple union whose base has a member with the wire name, `type`,
    /// or the IR name, `r-type`, of the discriminator that the lowering adds.
    #[error("QAPI union {union}: member '{member}' clashes with the discriminator 'type'")]
    DiscriminatorClash { union: String, member: String },
    /// A legacy alternate (`'discriminator': {}`) passed to
    /// [`process_union`]; [`process_legacy_alternate`] lowers it.
    #[error("QAPI union {union} is a legacy alternate")]
    LegacyAlternate { union: String },
    /// A union passed to [`process_legacy_alternate`] that is not a legacy
    /// alternate, or one with a base.
    #[error("QAPI union {union} is not a legacy alternate")]
    NotLegacyAlternate { union: String },
}

/// The release number at the start of a `Since` text, as three components:
/// "1.2" gives "1.2.0", and trailing dots and text after the number are
/// dropped. Text that does not start with a release number of at most three
/// components, such as prose that begins with "since", gives `None`.
fn normalize_since_version(s: &str) -> Option<String> {
    let number: String = s
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let mut parts: Vec<_> = number.trim_end_matches('.').split('.').collect();
    if parts.len() > 3 || parts.iter().any(|part| part.is_empty()) {
        return None;
    }
    parts.resize(3, "0");
    Some(parts.join("."))
}

/// Add each feature as a list of `feature` and its own `condition`.
macro_rules! add_feat {
    ($meta:expr, $feat:expr) => {
        if let Some(features) = $feat {
            for feature in features {
                let mut attrs = Vec::new();
                attrs.push(("feature", Some(feature.name.to_string())));
                if let Some(condition) = &feature.r#if {
                    attrs.push(("condition", Some(condition.to_string())));
                }
                $meta.attributes.push(Attribute::with_values(attrs));
            }
        }
    };
}

/// Remove the schema indentation used for member-description continuations.
fn member_documentation(lines: &[&str]) -> String {
    let start = usize::from(
        lines
            .first()
            .is_some_and(|line| !line.starts_with([' ', '\t'])),
    );
    let indent = lines
        .iter()
        .skip(start)
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start_matches([' ', '\t']).len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            if index >= start && !line.trim().is_empty() {
                &line[indent..]
            } else {
                *line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn member_wire_name<'a>(name: &'a str, meta: &'a Metadata) -> &'a str {
    meta.attributes
        .iter()
        .find_map(|attr| match attr {
            Attribute::Item { name, value } if name == "name" => Some(value.as_str()),
            _ => None,
        })
        .unwrap_or(name)
}

/// Mark a schema-declared item, lift version facts from its doc comment and
/// keep its GPL-2.0-or-later prose apart in `schema_doc`.
macro_rules! add_schema_facts {
    ($meta:expr, $kind:expr, $docs:expr, $variants:expr) => {
        $meta.schema_item = Some($kind);
        if let Some(doc) = $docs {
            let body = if doc.body.is_empty() {
                &doc.description
            } else {
                &doc.body
            };
            if !body.is_empty() {
                $meta.schema_doc = Some(body.join("\n"));
            }
            if let Some(since) = doc.since.and_then(normalize_since_version) {
                $meta.attributes.push(Attribute::with_value("added", since));
            }
            let mut members = Vec::new();
            for (name, desc) in &doc.fields {
                if let Some(value) = $variants
                    .iter_mut()
                    .find(|value| member_wire_name(&value.name, &value.meta) == *name)
                {
                    let since = extract_since_from_comment(desc)
                        .and_then(|since| normalize_since_version(&since));
                    value.meta.schema_doc = Some(member_documentation(desc));
                    if let Some(since) = since {
                        value
                            .meta
                            .attributes
                            .push(Attribute::with_value("added", since));
                    }
                } else if let Some(member) = name.split_once('.') {
                    members.push((member, desc));
                }
                // Other names, such as a command's or a member of another
                // type, document nothing of this item.
            }
            // A member's member ("@bus.number") extends the member's own
            // description, wherever it stands.
            for ((member, field), desc) in members {
                if let Some(doc) = $variants
                    .iter_mut()
                    .find(|value| member_wire_name(&value.name, &value.meta) == member)
                    .and_then(|value| value.meta.schema_doc.as_mut())
                {
                    doc.push('\n');
                    doc.push_str(&format!("  .{}: {}", field, desc.join(" ")));
                }
            }
        }
    };
}

fn field_name_attr(name: &str) -> Option<Attribute> {
    let rust_name = rustify_field_name(name);
    if name != rust_name {
        Some(Attribute::with_value("name", name))
    } else {
        None
    }
}

fn name_attr(name: &str) -> Option<Attribute> {
    let rust_name = rustify_name(name);
    if name != rust_name {
        Some(Attribute::with_value("name", name))
    } else {
        None
    }
}

fn variant_name_attr(name: &str) -> Option<Attribute> {
    (name != rustify_variant(name)).then(|| Attribute::with_value("name", name))
}

/// Resolve a named or array type reference; anonymous structs are handled
/// by [`process_member_with_inline`], in struct members only.
fn process_type_ref<'a>(
    q: &'a QapiTypeRef<'a>,
    definition: &str,
    member: &str,
) -> Result<(&'a str, bool), LowerError> {
    match q {
        QapiTypeRef::Ref(v) => Ok((v, false)),
        QapiTypeRef::ArrayRef(v) => Ok((v, true)),
        QapiTypeRef::Unset => Err(LowerError::MissingType {
            definition: definition.to_owned(),
            member: member.to_owned(),
        }),
        QapiTypeRef::InlineStruct(_) => Err(LowerError::AnonymousStruct {
            definition: definition.to_owned(),
            member: member.to_owned(),
        }),
    }
}

fn process_alternative(q: QapiAlternative, definition: &str) -> Result<EnumVariant, LowerError> {
    let (r#type, array) = process_type_ref(&q.r#type, definition, &q.name)?;

    let mut meta = Metadata::default();
    if let Some(attr) = variant_name_attr(&q.name) {
        meta.attributes.push(attr);
    }
    if let Some(condition) = q.r#if {
        meta.attributes
            .push(Attribute::with_value("condition", condition));
    }

    Ok(EnumVariant {
        name: q.name.into(),
        kind: EnumVariantKind::Tuple(r#type.into()),
        meta,
        array,
    })
}

/// Lower an alternate into an enum with a variant for each alternative.
pub fn process_alternate(q: QapiAlternate) -> Result<Enum, LowerError> {
    let mut variants = Vec::new();
    for v in q.data {
        variants.push(process_alternative(v, &q.name)?);
    }

    let mut meta = Metadata::default();
    if let Some(attr) = name_attr(&q.name) {
        meta.attributes.push(attr);
    }
    if let Some(condition) = q.r#if {
        meta.attributes
            .push(Attribute::with_value("condition", condition));
    }
    add_feat! {meta, q.features};
    add_schema_facts! {meta, SchemaItem::Alternate, q.doc, &mut variants};

    Ok(Enum {
        name: q.name.into(),
        variants,
        meta,
    })
}

/// Lower a legacy alternate, the union with an empty discriminator
/// (`'discriminator': {}`) of QEMU 1.7 to 2.3, as the alternate it is.
pub fn process_legacy_alternate(q: QapiUnion) -> Result<Enum, LowerError> {
    if !q.legacy_alternate || q.discriminator.is_some() || q.base != MembersOrRef::Unset {
        return Err(LowerError::NotLegacyAlternate {
            union: q.name.to_string(),
        });
    }
    process_alternate(QapiAlternate {
        name: q.name,
        data: QapiAlternatives(q.data.0),
        r#if: q.r#if,
        features: q.features,
        doc: q.doc,
        malformed: q.malformed,
        keys: q.keys,
        text: q.text,
    })
}

fn process_enum_value(q: QapiEnumValue) -> EnumVariant {
    let mut meta = Metadata::default();
    if let Some(attr) = variant_name_attr(&q.name) {
        meta.attributes.push(attr);
    }
    if let Some(condition) = q.r#if {
        meta.attributes
            .push(Attribute::with_value("condition", condition));
    }
    add_feat! {meta, q.r#features};

    EnumVariant {
        name: q.name.into(),
        kind: EnumVariantKind::Unit,
        meta,
        array: false,
    }
}

/// Lower an enum.
pub fn process_enum(q: QapiEnum) -> Enum {
    let mut variants = Vec::new();
    for v in q.data {
        let variant = process_enum_value(v);
        variants.push(variant);
    }

    let mut meta = Metadata::default();
    if let Some(attr) = name_attr(&q.name) {
        meta.attributes.push(attr);
    }
    if let Some(condition) = q.r#if {
        meta.attributes
            .push(Attribute::with_value("condition", condition));
    }
    add_feat! {meta, q.r#features};
    add_schema_facts! {meta, SchemaItem::Enum, q.doc, &mut variants};

    Enum {
        name: q.name.into(),
        variants,
        meta,
    }
}

/// Lower a member of `definition`, which must have a named or array type.
pub fn process_member(q: QapiMember, definition: &str) -> Result<StructField, LowerError> {
    let (r#type, array) = process_type_ref(&q.r#type, definition, &q.name)?;

    let mut meta = Metadata::default();
    if let Some(attr) = field_name_attr(&q.name) {
        meta.attributes.push(attr);
    }
    if let Some(condition) = q.r#if {
        meta.attributes
            .push(Attribute::with_value("condition", condition));
    }
    add_feat! {meta, q.features};
    Ok(StructField {
        name: q.name.into(),
        meta,
        r#type: r#type.into(),
        optional: q.optional,
        array,
    })
}

/// Lower a member of `definition` that may have an anonymous struct type,
/// which becomes a struct named after `prefix` and the member, under
/// `condition` and the member's own condition. Return the field and the
/// extracted structs, inner ones first.
fn process_member_with_inline(
    q: QapiMember,
    definition: &str,
    prefix: &str,
    condition: Option<&str>,
    names: &mut HashSet<String>,
) -> Result<(StructField, Vec<Struct>), LowerError> {
    if let QapiTypeRef::InlineStruct(members) = q.r#type {
        let preferred = format!("{}{}", rustify_name(prefix), rustify_name(&q.name));
        let mut inline_name = preferred.clone();
        let mut suffix = 0;
        while !names.insert(inline_name.clone()) {
            suffix += 1;
            inline_name = if suffix == 1 {
                format!("{preferred}Inline")
            } else {
                format!("{preferred}Inline{suffix}")
            };
        }
        let member_condition = q.r#if.map(|condition| condition.to_string());
        let inline_condition = match (condition, member_condition.as_deref()) {
            (Some(outer), Some(inner)) => Some(format!("({outer} && {inner})")),
            (outer, inner) => outer.or(inner).map(str::to_owned),
        };
        let mut fields = Vec::new();
        let mut extracted = Vec::new();
        for member in members {
            let (field, nested) = process_member_with_inline(
                member,
                definition,
                &inline_name,
                inline_condition.as_deref(),
                names,
            )?;
            fields.push(field);
            extracted.extend(nested);
        }
        let mut inline_meta = Metadata::default();
        if let Some(condition) = inline_condition {
            inline_meta
                .attributes
                .push(Attribute::with_value("condition", condition));
        }
        extracted.push(Struct {
            name: inline_name.clone(),
            fields,
            meta: inline_meta,
        });

        let mut meta = Metadata::default();
        if let Some(attr) = field_name_attr(&q.name) {
            meta.attributes.push(attr);
        }
        if let Some(condition) = member_condition {
            meta.attributes
                .push(Attribute::with_value("condition", condition));
        }
        add_feat! {meta, q.features};
        let field = StructField {
            name: q.name.into(),
            meta,
            r#type: inline_name,
            optional: q.optional,
            array: false,
        };
        Ok((field, extracted))
    } else {
        Ok((process_member(q, definition)?, Vec::new()))
    }
}

/// Lower a struct, its base's fields first, and the anonymous struct types
/// of its members (before QEMU 2.4).
///
/// `structs_lookup` holds the base. `names` holds the Rust names that are
/// taken; the caller seeds it with the [`rustify_name`] of every definition
/// of the schema. An anonymous struct becomes a struct of its own, named
/// in Rust after its struct and member (`VersionInfo` and `qemu` give
/// `VersionInfoQemu`), with `Inline`, `Inline2` and so on appended while the
/// name is taken, and under the struct's and the member's conditions. The
/// struct's own name and the extracted names join `names`.
///
/// Return the struct and the extracted structs, inner ones first.
pub fn process_struct(
    q: QapiStruct,
    structs_lookup: &HashMap<String, Struct>,
    names: &mut HashSet<String>,
) -> Result<(Struct, Vec<Struct>), LowerError> {
    names.insert(rustify_name(&q.name));
    let mut fields = Vec::new();
    let mut extracted = Vec::new();
    if let Some(base) = q.base.as_ref() {
        fields.extend(lookup_struct(structs_lookup, &q.name, base)?.fields.clone());
    }
    let condition = q.r#if.as_ref().map(ToString::to_string);
    for member in q.data {
        let (field, inline) =
            process_member_with_inline(member, &q.name, &q.name, condition.as_deref(), names)?;
        fields.push(field);
        extracted.extend(inline);
    }
    let mut meta = Metadata::default();
    if let Some(attr) = name_attr(&q.name) {
        meta.attributes.push(attr);
    }
    if let Some(condition) = condition {
        meta.attributes
            .push(Attribute::with_value("condition", condition));
    }
    add_feat! {meta, q.r#features};
    add_schema_facts! {meta, SchemaItem::Struct, q.doc, &mut fields};

    Ok((
        Struct {
            name: q.name.into(),
            fields,
            meta,
        },
        extracted,
    ))
}

fn lookup_struct<'a>(
    structs_lookup: &'a HashMap<String, Struct>,
    definition: &str,
    name: &str,
) -> Result<&'a Struct, LowerError> {
    structs_lookup
        .get(name)
        .ok_or_else(|| LowerError::UnknownType {
            definition: definition.to_owned(),
            name: name.to_owned(),
        })
}

/// Lower a flat or simple union.
///
/// Return three items:
///
/// - The union as a struct: its base's fields, the discriminator marked
///   `discriminator`, and the branch as the flattened field `u`, which is
///   optional. A simple union, which names no discriminator, gets one of
///   its own, named `r-type` in the IR and `type` on the wire, and the
///   attribute `simple_union`.
/// - The enum of the branches, `<Union>Branch` in Rust names, marked
///   `union_branch`. A flat union's discriminator enum values without a
///   branch, which QAPI gives an empty one, are unit variants.
/// - For a simple union, the enum of its discriminator values,
///   `<Union>Type` in Rust names: one for each branch, under the branch's
///   condition.
///
/// The base is a struct of `structs_lookup` or the union's own members; the
/// discriminator's type is an enum of `enums_lookup` or a builtin type (the
/// `str` of QEMU 1.7's BlockdevOptions). A legacy alternate is an error:
/// [`process_legacy_alternate`] lowers it.
pub fn process_union(
    q: QapiUnion,
    structs_lookup: &HashMap<String, Struct>,
    enums_lookup: &HashMap<String, Enum>,
) -> Result<(Struct, Enum, Option<Enum>), LowerError> {
    let q_name = q.name.to_string();
    if q.legacy_alternate {
        return Err(LowerError::LegacyAlternate { union: q_name });
    }
    let rust_name = rustify_name(&q_name);
    let branch_name = format!("{rust_name}Branch");

    let mut fields = process_members_or_ref(q.base, &q_name, structs_lookup)?;

    let discriminator_type = if let Some(discriminator) = q.discriminator.as_deref() {
        let mut discriminator_type = None;
        for field in &mut fields {
            if field.name == discriminator {
                field.meta.attributes.push(Attribute::new("discriminator"));
                discriminator_type = Some(field.r#type.clone());
            }
        }
        let discriminator_type =
            discriminator_type.ok_or_else(|| LowerError::MissingDiscriminator {
                union: q_name.clone(),
                discriminator: discriminator.to_owned(),
            })?;
        Some(discriminator_type)
    } else {
        if let Some(clash) = fields.iter().find(|field| {
            field.name == "r-type" || member_wire_name(&field.name, &field.meta) == "type"
        }) {
            return Err(LowerError::DiscriminatorClash {
                union: q_name,
                member: clash.name.clone(),
            });
        }
        // Its wire name is QAPI's; its IR name keeps it apart from the `type`
        // member that a later release of the union may declare.
        let mut meta = Metadata::default();
        meta.attributes.push(Attribute::new("discriminator"));
        meta.attributes.push(Attribute::with_value("name", "type"));
        fields.push(StructField {
            name: "r-type".into(),
            meta,
            r#type: format!("{rust_name}Type"),
            optional: false,
            array: false,
        });
        None
    };
    let is_simple = discriminator_type.is_none();

    let mut variants = Vec::new();
    for branch in q.data {
        let (r#type, array) = process_type_ref(&branch.r#type, &q_name, &branch.name)?;
        let mut meta = Metadata::default();
        if let Some(attr) = variant_name_attr(&branch.name) {
            meta.attributes.push(attr);
        }
        if let Some(condition) = branch.r#if {
            meta.attributes
                .push(Attribute::with_value("condition", condition));
        }
        variants.push(EnumVariant {
            name: branch.name.into(),
            kind: EnumVariantKind::Tuple(r#type.into()),
            meta,
            array,
        });
    }
    // QAPI flat unions implicitly select an empty branch for valid
    // discriminator enum values omitted from `data` (QEMU's
    // docs/devel/qapi-code-gen.rst, "Union types"). Model these explicitly
    // rather than treating valid tags such as CpuInfoFast.x86_64 as unknown.
    if let Some(discriminator_type) = &discriminator_type {
        if let Some(tags) = enums_lookup.get(discriminator_type) {
            for tag in &tags.variants {
                if !variants.iter().any(|branch| branch.name == tag.name) {
                    variants.push(EnumVariant {
                        name: tag.name.clone(),
                        kind: EnumVariantKind::Unit,
                        meta: tag.meta.clone(),
                        array: false,
                    });
                }
            }
        } else if builtin_type(discriminator_type).is_none() {
            return Err(LowerError::UnknownType {
                definition: q_name,
                name: discriminator_type.clone(),
            });
        }
    }
    // Keep the synthesized branch's peer-build condition with its union.
    let mut branch_meta = Metadata::default();
    branch_meta.attributes.push(Attribute::new("union_branch"));
    if let Some(condition) = &q.r#if {
        branch_meta
            .attributes
            .push(Attribute::with_value("condition", condition));
    }
    let e = Enum {
        name: branch_name.clone(),
        variants,
        meta: branch_meta,
    };

    let mut meta = Metadata::default();
    meta.attributes.push(Attribute::new("union"));
    meta.attributes.push(Attribute::new("flatten"));
    fields.push(StructField {
        name: "u".into(),
        meta,
        r#type: branch_name,
        optional: true,
        array: false,
    });
    let mut meta = Metadata::default();
    if let Some(attr) = name_attr(&q.name) {
        meta.attributes.push(attr);
    }
    if let Some(condition) = &q.r#if {
        meta.attributes
            .push(Attribute::with_value("condition", condition));
    }
    add_feat! {meta, q.r#features};
    add_schema_facts! {meta, SchemaItem::Union, q.doc, &mut fields};
    if is_simple {
        meta.attributes.push(Attribute::new("simple_union"));
    }
    let s = Struct {
        name: q.name.into(),
        fields,
        meta,
    };
    let disc_enum = is_simple.then(|| {
        let variants = e
            .variants
            .iter()
            .map(|v| EnumVariant {
                name: v.name.clone(),
                kind: EnumVariantKind::Unit,
                // The branch's wire name and condition.
                meta: v.meta.clone(),
                array: false,
            })
            .collect();
        let mut meta = Metadata::default();
        if let Some(condition) = &q.r#if {
            meta.attributes
                .push(Attribute::with_value("condition", condition));
        }
        Enum {
            name: format!("{rust_name}Type"),
            variants,
            meta,
        }
    });

    Ok((s, e, disc_enum))
}

/// Resolve a `MembersOrRef` to a flat list of struct fields. When the source
/// is a `Ref(name)`, the referenced struct's fields are inlined (duplicated)
/// directly into the parent.
fn process_members_or_ref(
    q: MembersOrRef,
    definition: &str,
    structs_lookup: &HashMap<String, Struct>,
) -> Result<Vec<StructField>, LowerError> {
    match q {
        MembersOrRef::Unset => Ok(Vec::new()),
        MembersOrRef::Members(v) => v
            .into_iter()
            .map(|field| process_member(field, definition))
            .collect(),
        MembersOrRef::Ref(v) => Ok(lookup_struct(structs_lookup, definition, &v)?
            .fields
            .clone()),
    }
}

/// Lower an event into the struct of its data, named `<name>Event` so that
/// it does not share its name with a command, with the `event` attribute
/// and its wire name.
///
/// The data are members, or a struct of `structs_lookup` whose fields the
/// event takes: with boxed data, a union, so `structs_lookup` holds the
/// lowered unions too.
pub fn process_event(
    q: QapiEvent,
    structs_lookup: &HashMap<String, Struct>,
) -> Result<Struct, LowerError> {
    let mut fields = Vec::new();
    if let Some(data) = q.data {
        fields.extend(process_members_or_ref(data, &q.name, structs_lookup)?);
    }
    let mut meta = Metadata::default();
    meta.attributes.push(Attribute::new("event"));
    meta.attributes.push(Attribute::with_value("name", &q.name));
    if let Some(condition) = q.r#if {
        meta.attributes
            .push(Attribute::with_value("condition", condition));
    }
    add_feat! {meta, q.r#features};
    add_schema_facts! {meta, SchemaItem::Event, q.doc, &mut fields};

    let event_name = format!("{}Event", q.name);
    Ok(Struct {
        name: event_name,
        fields,
        meta,
    })
}

/// Lower a command into the struct of its arguments, with the `command`
/// attribute and its flags.
///
/// The arguments are members, or a struct of `structs_lookup` whose fields
/// the command takes: with boxed data, a union, so `structs_lookup` holds
/// the lowered unions too. `returns` holds the Rust type of the reply: `()`
/// without a return type, `Vec<T>` for a list. A gen:false command accepts
/// arguments beyond those it declares, and gets the flattened optional
/// member `arguments` for them, which `flatten` tells apart from a declared
/// argument of that name.
pub fn process_command(
    q: QapiCommand,
    structs_lookup: &HashMap<String, Struct>,
) -> Result<Struct, LowerError> {
    let mut fields = Vec::new();
    if let Some(data) = q.data {
        fields.extend(process_members_or_ref(data, &q.name, structs_lookup)?);
    }
    let mut meta = Metadata::default();
    meta.attributes.push(Attribute::new("command"));
    if let Some(attr) = name_attr(&q.name) {
        meta.attributes.push(attr);
    }
    if let Some(condition) = q.r#if {
        meta.attributes
            .push(Attribute::with_value("condition", condition));
    }
    add_feat! {meta, q.r#features};
    add_schema_facts! {meta, SchemaItem::Command, q.doc, &mut fields};
    if let Some(returns) = q.returns {
        let (r#type, array) = process_type_ref(&returns, &q.name, "returns")?;
        meta.qapi_returns = Some(if array {
            format!("[{}]", r#type)
        } else {
            r#type.to_owned()
        });
        let rust_type = super::rustify_type(r#type);
        let r#type = if array {
            format!("Vec<{}>", rust_type)
        } else {
            rust_type
        };
        meta.attributes
            .push(Attribute::with_value("returns", r#type));
    } else {
        meta.attributes.push(Attribute::with_value("returns", "()"));
    }
    if q.allow_oob == Some("true") {
        meta.attributes.push(Attribute::new("allow_oob"));
    }
    if q.allow_preconfig == Some("true") {
        meta.attributes.push(Attribute::new("allow_preconfig"));
    }
    if matches!(q.success_response.as_deref(), Some("false" | "no")) {
        meta.attributes.push(Attribute::new("no_success_response"));
    }

    if q.r#gen.as_deref() == Some("false") || q.r#gen.as_deref() == Some("no") {
        fields.push(StructField {
            name: "arguments".into(),
            r#type: ARGUMENTS_TYPE.into(),
            optional: true,
            array: false,
            meta: Metadata {
                doc: Some("Additional command arguments not modeled by this schema.".into()),
                attributes: vec![Attribute::new("flatten")],
                ..Metadata::default()
            },
        });
    }

    Ok(Struct {
        name: q.name.into(),
        fields,
        meta,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QapiCond, QapiDocumentation, QapiEnumValue, QapiFeature, QapiFeatures, QapiMembers,
        QapiSchemaToken, QapiTypeRef,
    };

    fn make_enum_value(name: &str) -> QapiEnumValue<'_> {
        QapiEnumValue {
            name: name.into(),
            ..Default::default()
        }
    }

    fn member<'a>(name: &'a str, r#type: QapiTypeRef<'a>) -> QapiMember<'a> {
        QapiMember {
            malformed: Vec::new(),
            name: name.into(),
            optional: false,
            r#type,
            r#if: None,
            features: None,
            keys: None,
        }
    }

    fn token(input: &str, version: (u32, u32, u32)) -> QapiSchemaToken<'_> {
        let schema = crate::parse_schema_with_version(input, version).unwrap();
        schema.0.into_iter().next().unwrap()
    }

    fn union(input: &str, version: (u32, u32, u32)) -> QapiUnion<'_> {
        let QapiSchemaToken::Union(value) = token(input, version) else {
            panic!("union token")
        };
        value
    }

    fn command(input: &str, version: (u32, u32, u32)) -> QapiCommand<'_> {
        let QapiSchemaToken::Command(value) = token(input, version) else {
            panic!("command token")
        };
        value
    }

    fn lower_first(input: &str, version: (u32, u32, u32)) -> Result<(), LowerError> {
        let lookup = HashMap::new();
        match token(input, version) {
            QapiSchemaToken::Struct(value) => {
                process_struct(value, &lookup, &mut HashSet::new()).map(drop)
            }
            QapiSchemaToken::Union(value) => {
                process_union(value, &lookup, &HashMap::new()).map(drop)
            }
            QapiSchemaToken::Alternate(value) => process_alternate(value).map(drop),
            QapiSchemaToken::Command(value) => process_command(value, &lookup).map(drop),
            QapiSchemaToken::Event(value) => process_event(value, &lookup).map(drop),
            token => panic!("unexpected definition {token:?}"),
        }
    }

    fn condition(meta: &Metadata) -> Option<&str> {
        meta.attributes.iter().find_map(|attr| match attr {
            Attribute::Item { name, value } if name == "condition" => Some(value.as_str()),
            _ => None,
        })
    }

    #[test]
    fn historical_simple_union_arrays_lower_to_list_branches() {
        let input = union(
            "{ 'union': 'Example', 'data': {'text': 'str', 'values': ['int']} }",
            (2, 0, 0),
        );
        let (value, branches, _) = process_union(input, &HashMap::new(), &HashMap::new()).unwrap();
        assert!(
            value
                .meta
                .attributes
                .contains(&Attribute::new("simple_union"))
        );
        let array = branches
            .variants
            .iter()
            .find(|variant| variant.name == "values")
            .unwrap();
        assert!(array.array);
        assert_eq!(array.kind, EnumVariantKind::Tuple("int".into()));
        assert!(!branches.variants[0].array);
    }

    #[test]
    fn simple_unions_get_a_discriminator_and_its_enum_under_the_branch_conditions() {
        let input = union(
            "{ 'union': 'Backend', 'if': 'defined(CONFIG_CHARDEV)', 'data': { 'file': 'FileOptions', 'spicevmc': { 'type': 'SpiceOptions', 'if': 'defined(CONFIG_SPICE)' } } }",
            (6, 1, 0),
        );
        let (value, branches, types) =
            process_union(input, &HashMap::new(), &HashMap::new()).unwrap();
        let condition_attr = |value: &str| Attribute::with_value("condition", value);

        assert_eq!(value.name, "Backend");
        assert_eq!(
            value.meta.attributes,
            [
                condition_attr("defined(CONFIG_CHARDEV)"),
                Attribute::new("simple_union")
            ]
        );
        let [discriminator, branch] = &value.fields[..] else {
            panic!("{:?}", value.fields)
        };
        assert_eq!(discriminator.name, "r-type");
        assert_eq!(discriminator.r#type, "BackendType");
        assert_eq!(
            discriminator.meta.attributes,
            [
                Attribute::new("discriminator"),
                Attribute::with_value("name", "type")
            ]
        );
        assert!(!discriminator.optional && !discriminator.array);
        assert_eq!(branch.name, "u");
        assert_eq!(branch.r#type, "BackendBranch");
        assert_eq!(
            branch.meta.attributes,
            [Attribute::new("union"), Attribute::new("flatten")]
        );
        assert!(branch.optional && !branch.array);

        assert_eq!(branches.name, "BackendBranch");
        assert_eq!(
            branches.meta.attributes,
            [
                Attribute::new("union_branch"),
                condition_attr("defined(CONFIG_CHARDEV)")
            ]
        );
        let spicevmc = [
            Attribute::with_value("name", "spicevmc"),
            condition_attr("defined(CONFIG_SPICE)"),
        ];
        assert_eq!(
            branches
                .variants
                .iter()
                .map(|v| (v.name.as_str(), &v.kind, &v.meta.attributes[..]))
                .collect::<Vec<_>>(),
            [
                (
                    "file",
                    &EnumVariantKind::Tuple("FileOptions".into()),
                    &[Attribute::with_value("name", "file")][..]
                ),
                (
                    "spicevmc",
                    &EnumVariantKind::Tuple("SpiceOptions".into()),
                    &spicevmc[..]
                ),
            ]
        );

        let types = types.unwrap();
        assert_eq!(types.name, "BackendType");
        assert_eq!(
            types.meta.attributes,
            [condition_attr("defined(CONFIG_CHARDEV)")]
        );
        assert_eq!(
            types
                .variants
                .iter()
                .map(|v| (v.name.as_str(), &v.kind, &v.meta.attributes[..]))
                .collect::<Vec<_>>(),
            [
                (
                    "file",
                    &EnumVariantKind::Unit,
                    &[Attribute::with_value("name", "file")][..]
                ),
                ("spicevmc", &EnumVariantKind::Unit, &spicevmc[..]),
            ]
        );
    }

    #[test]
    fn pre_2_4_anonymous_structs_outside_struct_members_are_errors() {
        for (input, definition, member) in [
            (
                "{ 'command': 'c', 'returns': { 'a': 'int' } }",
                "c",
                "returns",
            ),
            (
                "{ 'command': 'c', 'data': { 'm': { 'a': 'int' } } }",
                "c",
                "m",
            ),
            (
                "{ 'event': 'E', 'data': { 'm': { 'a': 'int' } } }",
                "E",
                "m",
            ),
            (
                "{ 'union': 'U', 'data': { 'b': { 'a': 'int' } } }",
                "U",
                "b",
            ),
        ] {
            assert_eq!(
                lower_first(input, (2, 3, 0)),
                Err(LowerError::AnonymousStruct {
                    definition: definition.into(),
                    member: member.into(),
                }),
                "{input}"
            );
        }
        // Struct members may hold anonymous structs, nested ones included.
        lower_first(
            "{ 'type': 'T', 'data': { 'm': { 'n': { 'a': 'int' } } } }",
            (2, 0, 0),
        )
        .unwrap();
    }

    #[test]
    fn members_and_alternatives_without_a_type_are_errors() {
        let missing = |definition: &str, member: &str| {
            Err(LowerError::MissingType {
                definition: definition.into(),
                member: member.into(),
            })
        };
        assert_eq!(
            process_member(member("m", QapiTypeRef::Unset), "S").map(drop),
            missing("S", "m")
        );
        let alternate = QapiAlternate {
            name: "A".into(),
            data: QapiAlternatives(vec![QapiAlternative {
                malformed: Vec::new(),
                name: "b".into(),
                r#type: QapiTypeRef::Unset,
                r#if: None,
                keys: None,
            }]),
            ..Default::default()
        };
        assert_eq!(process_alternate(alternate).map(drop), missing("A", "b"));
        // Inside an anonymous struct, the error names the QAPI definition.
        let inline = QapiStruct {
            name: "T".into(),
            data: QapiMembers(vec![member(
                "m",
                QapiTypeRef::InlineStruct(QapiMembers(vec![member("n", QapiTypeRef::Unset)])),
            )]),
            ..Default::default()
        };
        assert_eq!(
            process_struct(inline, &HashMap::new(), &mut HashSet::new()).map(drop),
            missing("T", "n")
        );
    }

    #[test]
    fn missing_types_and_discriminators_are_errors() {
        let unknown = |definition: &str, name: &str| {
            Err(LowerError::UnknownType {
                definition: definition.into(),
                name: name.into(),
            })
        };
        for (input, error) in [
            (
                "{ 'struct': 'S', 'base': 'Missing', 'data': {} }",
                unknown("S", "Missing"),
            ),
            (
                "{ 'command': 'c', 'data': 'Missing' }",
                unknown("c", "Missing"),
            ),
            (
                "{ 'event': 'E', 'data': 'Missing' }",
                unknown("E", "Missing"),
            ),
            (
                "{ 'union': 'U', 'base': 'Missing', 'discriminator': 'kind', 'data': {} }",
                unknown("U", "Missing"),
            ),
            (
                "{ 'union': 'U', 'base': { 'kind': 'Kind' }, 'discriminator': 'kind', 'data': {} }",
                unknown("U", "Kind"),
            ),
            (
                "{ 'union': 'U', 'base': { 'kind': 'str' }, 'discriminator': 'other', 'data': {} }",
                Err(LowerError::MissingDiscriminator {
                    union: "U".into(),
                    discriminator: "other".into(),
                }),
            ),
        ] {
            assert_eq!(lower_first(input, (11, 1, 3)), error, "{input}");
        }
    }

    #[test]
    fn simple_union_bases_must_not_declare_the_discriminator() {
        for (base, member) in [("'type': 'str'", "type"), ("'r-type': 'str'", "r-type")] {
            let input =
                format!("{{ 'union': 'U', 'base': {{ {base} }}, 'data': {{ 'a': 'int' }} }}");
            assert_eq!(
                lower_first(&input, (2, 0, 0)),
                Err(LowerError::DiscriminatorClash {
                    union: "U".into(),
                    member: member.into(),
                }),
                "{input}"
            );
        }
        lower_first(
            "{ 'union': 'U', 'base': { 'kind': 'str' }, 'data': { 'a': 'int' } }",
            (2, 0, 0),
        )
        .unwrap();
    }

    #[test]
    fn flat_unions_take_builtin_discriminators_without_implicit_branches() {
        // QEMU 1.7's BlockdevOptions has a `str` discriminator.
        let input = union(
            "{ 'union': 'Options', 'base': { 'driver': 'str' }, 'discriminator': 'driver', 'data': { 'file': 'FileOptions' } }",
            (1, 7, 0),
        );
        let (value, branches, types) =
            process_union(input, &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(types, None);
        assert_eq!(
            value.fields[0].meta.attributes,
            [Attribute::new("discriminator")]
        );
        assert_eq!(value.fields[0].r#type, "str");
        let names: Vec<_> = branches.variants.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(names, ["file"]);
    }

    #[test]
    fn legacy_alternates_lower_only_as_alternates() {
        let input = "{ 'union': 'BlockdevRef', 'discriminator': {}, 'data': { 'definition': 'BlockdevOptions', 'reference': 'str' } }";
        assert_eq!(
            process_union(union(input, (1, 7, 0)), &HashMap::new(), &HashMap::new()).map(drop),
            Err(LowerError::LegacyAlternate {
                union: "BlockdevRef".into()
            })
        );
        let value = process_legacy_alternate(union(input, (1, 7, 0))).unwrap();
        assert_eq!(value.name, "BlockdevRef");
        assert_eq!(value.meta.schema_item, Some(SchemaItem::Alternate));
        assert_eq!(
            value
                .variants
                .iter()
                .map(|v| (v.name.as_str(), &v.kind))
                .collect::<Vec<_>>(),
            [
                (
                    "definition",
                    &EnumVariantKind::Tuple("BlockdevOptions".into())
                ),
                ("reference", &EnumVariantKind::Tuple("str".into())),
            ]
        );

        let not_legacy = Err(LowerError::NotLegacyAlternate { union: "U".into() });
        let flat = union(
            "{ 'union': 'U', 'base': { 'kind': 'str' }, 'discriminator': 'kind', 'data': {} }",
            (6, 1, 0),
        );
        assert_eq!(process_legacy_alternate(flat), not_legacy);
        let based = QapiUnion {
            name: "U".into(),
            base: MembersOrRef::Ref("Base".into()),
            legacy_alternate: true,
            ..Default::default()
        };
        assert_eq!(process_legacy_alternate(based), not_legacy);
    }

    #[test]
    fn since_texts_yield_only_release_numbers() {
        for (text, expected) in [
            ("9.1", Some("9.1.0")),
            ("2.9.", Some("2.9.0")),
            (" 1.2.3 (experimental)", Some("1.2.3")),
            ("8", Some("8.0.0")),
            ("the device is realized", None),
            ("QEMU 2.5", None),
            ("", None),
            ("1..2", None),
            ("1.2.3.4", None),
        ] {
            assert_eq!(
                normalize_since_version(text).as_deref(),
                expected,
                "{text:?}"
            );
        }
        let input = "##\n# @Example:\n#\n# @member: Prose that ends with a line\n#     since the device is realized\n#\n# Since: 9.0\n##\n{ 'struct': 'Example', 'data': { 'member': 'int' } }";
        let QapiSchemaToken::Struct(value) = token(input, (9, 1, 0)) else {
            panic!("struct token")
        };
        let (value, _) = process_struct(value, &HashMap::new(), &mut HashSet::new()).unwrap();
        assert!(
            !value.fields[0]
                .meta
                .attributes
                .iter()
                .any(|attr| matches!(attr, Attribute::Item { name, .. } if name == "added")),
            "{:?}",
            value.fields[0].meta.attributes
        );
    }

    #[test]
    fn member_comments_remove_syntax_indent_but_keep_nested_literal_indent() {
        assert_eq!(
            member_documentation(&["intro", "    continuation", "", "        code", "    after"]),
            "intro\ncontinuation\n\n    code\nafter"
        );
        assert_eq!(
            member_documentation(&["    new-line description", "    continuation"]),
            "new-line description\ncontinuation"
        );
    }

    #[test]
    fn schema_doc_comments_yield_version_facts_and_separate_prose() {
        let input = "##\n# @Example:\n#\n# Item prose.\n#\n# @member: Member prose.\n#     Since 9.1\n#\n# Since: 9.0\n##\n{ 'struct': 'Example', 'data': { 'member': 'int' } }";
        let QapiSchemaToken::Struct(value) = token(input, (9, 1, 0)) else {
            panic!("struct token")
        };
        let (value, _) = process_struct(value, &HashMap::new(), &mut HashSet::new()).unwrap();
        assert_eq!(value.meta.doc, None);
        assert_eq!(value.meta.schema_doc.as_deref(), Some("Item prose."));
        assert_eq!(value.meta.schema_item, Some(SchemaItem::Struct));
        assert!(
            value
                .meta
                .attributes
                .contains(&Attribute::with_value("added", "9.0.0"))
        );
        let member = &value.fields[0];
        assert_eq!(member.meta.doc, None);
        assert_eq!(
            member.meta.schema_doc.as_deref(),
            Some("Member prose.\nSince 9.1")
        );
        assert_eq!(member.meta.schema_item, None);
        assert!(
            member
                .meta
                .attributes
                .contains(&Attribute::with_value("added", "9.1.0"))
        );
    }

    #[test]
    fn sub_member_descriptions_extend_their_member_in_any_order() {
        for fields in [
            vec![
                ("bus", vec!["The bus."]),
                ("bus.number", vec!["Its number."]),
            ],
            vec![
                ("bus.number", vec!["Its number."]),
                ("bus", vec!["The bus."]),
            ],
        ] {
            let value = QapiStruct {
                name: "Device".into(),
                data: QapiMembers(vec![member("bus", QapiTypeRef::Ref("Bus".into()))]),
                doc: Some(QapiDocumentation {
                    name: "Device",
                    fields: fields.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let (value, _) = process_struct(value, &HashMap::new(), &mut HashSet::new()).unwrap();
            assert_eq!(
                value.fields[0].meta.schema_doc.as_deref(),
                Some("The bus.\n  .number: Its number."),
                "{fields:?}"
            );
        }
    }

    #[test]
    fn test_process_enum_simple() {
        let q = QapiEnum {
            name: "StatusCode".into(),
            data: vec![make_enum_value("ok"), make_enum_value("error")],
            prefix: None,
            r#if: None,
            features: None,
            doc: None,
            malformed: Vec::new(),
            keys: Vec::new(),
            text: "",
        };
        let e = process_enum(q);
        assert_eq!(e.name, "StatusCode");
        assert_eq!(e.meta.attributes, []);
        assert_eq!(e.meta.schema_item, Some(SchemaItem::Enum));
        assert_eq!(e.variants.len(), 2);
        assert_eq!(e.variants[0].name, "ok");
        assert_eq!(e.variants[0].kind, EnumVariantKind::Unit);
        assert_eq!(
            e.variants[0].meta.attributes,
            [Attribute::with_value("name", "ok")]
        );
        assert_eq!(e.variants[1].name, "error");
    }

    #[test]
    fn enum_conditions_and_feature_conditions_are_kept() {
        let QapiSchemaToken::Enum(value) = token(
            "{ 'enum': 'my-enum', 'if': 'CONFIG_A', 'features': [ { 'name': 'f', 'if': 'CONFIG_F' } ], 'data': [ { 'name': 'Value', 'if': 'CONFIG_B' } ] }",
            (11, 1, 3),
        ) else {
            panic!("enum token")
        };
        let e = process_enum(value);
        assert_eq!(
            e.meta.attributes,
            [
                Attribute::with_value("name", "my-enum"),
                Attribute::with_value("condition", "CONFIG_A"),
                Attribute::List(vec![
                    Attribute::with_value("feature", "f"),
                    Attribute::with_value("condition", "CONFIG_F"),
                ]),
            ]
        );
        assert_eq!(
            e.variants[0].meta.attributes,
            [Attribute::with_value("condition", "CONFIG_B")]
        );
    }

    #[test]
    fn test_process_enum_with_features() {
        let q = QapiEnum {
            name: "MyEnum".into(),
            data: vec![QapiEnumValue {
                name: "deprecated-val".into(),
                features: Some(QapiFeatures(vec![QapiFeature {
                    malformed: Vec::new(),
                    name: "deprecated".into(),
                    r#if: None,
                    keys: None,
                }])),
                ..Default::default()
            }],
            prefix: None,
            r#if: None,
            features: None,
            doc: None,
            malformed: Vec::new(),
            keys: Vec::new(),
            text: "",
        };
        let e = process_enum(q);
        assert_eq!(e.variants.len(), 1);
        assert_eq!(
            e.variants[0].meta.attributes,
            [
                Attribute::with_value("name", "deprecated-val"),
                Attribute::List(vec![Attribute::with_value("feature", "deprecated")]),
            ]
        );
    }

    #[test]
    fn test_process_struct_simple() {
        let q = QapiStruct {
            name: "MyStruct".into(),
            data: QapiMembers(vec![member("field-name", QapiTypeRef::Ref("str".into()))]),
            base: None,
            r#if: None,
            features: None,
            doc: None,
            malformed: Vec::new(),
            keys: Vec::new(),
            text: "",
        };
        let mut names = HashSet::new();
        let (s, extracted) = process_struct(q, &HashMap::new(), &mut names).unwrap();
        assert_eq!(s.name, "MyStruct");
        assert!(extracted.is_empty());
        assert_eq!(names, HashSet::from(["MyStruct".to_owned()]));
        assert_eq!(s.fields.len(), 1);
        assert_eq!(s.fields[0].name, "field-name");
        assert_eq!(s.fields[0].r#type, "str");
        assert_eq!(
            s.fields[0].meta.attributes,
            [Attribute::with_value("name", "field-name")]
        );
        assert!(!s.fields[0].optional);
    }

    #[test]
    fn test_process_struct_with_base() {
        let mut lookup = HashMap::new();
        lookup.insert(
            "BaseStruct".to_string(),
            Struct {
                name: "BaseStruct".into(),
                fields: vec![StructField {
                    name: "base-field".into(),
                    r#type: "int".into(),
                    optional: false,
                    array: false,
                    meta: Metadata::default(),
                }],
                meta: Metadata::default(),
            },
        );

        let q = QapiStruct {
            name: "Child".into(),
            data: QapiMembers(vec![QapiMember {
                optional: true,
                ..member("extra", QapiTypeRef::Ref("bool".into()))
            }]),
            base: Some("BaseStruct".into()),
            r#if: None,
            features: None,
            doc: None,
            malformed: Vec::new(),
            keys: Vec::new(),
            text: "",
        };
        let (s, _) = process_struct(q, &lookup, &mut HashSet::new()).unwrap();
        assert_eq!(s.name, "Child");
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[0].name, "base-field");
        assert_eq!(s.fields[1].name, "extra");
        assert!(s.fields[1].optional);
    }

    #[test]
    fn test_process_member_optional_and_array() {
        let m = QapiMember {
            optional: true,
            ..member("items", QapiTypeRef::ArrayRef("SomeType".into()))
        };
        let f = process_member(m, "Example").unwrap();
        assert_eq!(f.name, "items");
        assert!(f.optional);
        assert!(f.array);
        assert_eq!(f.r#type, "SomeType");
    }

    #[test]
    fn test_process_struct_with_inline_struct() {
        // Simulates VersionInfo.qemu from v1.1.0:
        // { 'type': 'VersionInfo', 'data': { 'qemu': {'major': 'int', 'minor': 'int', 'micro': 'int'}, ... } }
        let int = || QapiTypeRef::Ref("int".into());
        let q = QapiStruct {
            name: "VersionInfo".into(),
            data: QapiMembers(vec![
                member(
                    "qemu",
                    QapiTypeRef::InlineStruct(QapiMembers(vec![
                        member("major", int()),
                        member("minor", int()),
                        member("micro", int()),
                    ])),
                ),
                QapiMember {
                    optional: true,
                    ..member("package", QapiTypeRef::Ref("str".into()))
                },
            ]),
            base: None,
            r#if: None,
            features: None,
            doc: None,
            malformed: Vec::new(),
            keys: Vec::new(),
            text: "",
        };
        let (s, extracted) = process_struct(q, &HashMap::new(), &mut HashSet::new()).unwrap();
        assert_eq!(s.name, "VersionInfo");
        assert_eq!(s.fields.len(), 2);
        // The inline struct field should now reference the extracted type
        assert_eq!(s.fields[0].name, "qemu");
        assert_eq!(s.fields[0].r#type, "VersionInfoQemu");
        assert!(!s.fields[0].optional);
        assert_eq!(s.fields[1].name, "package");

        // One inline struct should have been extracted
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].name, "VersionInfoQemu");
        assert_eq!(extracted[0].meta, Metadata::default());
        assert_eq!(extracted[0].fields.len(), 3);
        assert_eq!(extracted[0].fields[0].name, "major");
        assert_eq!(extracted[0].fields[0].r#type, "int");
    }

    #[test]
    fn nested_anonymous_structs_are_named_after_their_holders_and_avoid_taken_names() {
        let input = "{ 'type': 'Outer', 'data': { 'mid': { 'leaf': { 'a': 'int' } } } }";
        for (taken, outer, inner) in [
            (&[][..], "OuterMid", "OuterMidLeaf"),
            (&["OuterMid"][..], "OuterMidInline", "OuterMidInlineLeaf"),
            (
                &["OuterMid", "OuterMidInline"][..],
                "OuterMidInline2",
                "OuterMidInline2Leaf",
            ),
        ] {
            let QapiSchemaToken::Struct(value) = token(input, (2, 0, 0)) else {
                panic!("struct token")
            };
            let mut names: HashSet<String> = taken.iter().map(|name| name.to_string()).collect();
            let (value, extracted) = process_struct(value, &HashMap::new(), &mut names).unwrap();
            assert_eq!(value.fields[0].r#type, outer);
            let [first, second] = &extracted[..] else {
                panic!("{extracted:?}")
            };
            assert_eq!((first.name.as_str(), second.name.as_str()), (inner, outer));
            assert_eq!(second.fields[0].name, "leaf");
            assert_eq!(second.fields[0].r#type, inner);
            assert_eq!(first.fields[0].r#type, "int");
            for name in ["Outer", outer, inner] {
                assert!(names.contains(name), "{name}: {names:?}");
            }
        }
    }

    #[test]
    fn anonymous_structs_take_their_holders_conditions() {
        let config = |name: &'static str| Some(QapiCond::ConfigName(name.into()));
        let q = QapiStruct {
            name: "Outer".into(),
            r#if: config("CONFIG_A"),
            data: QapiMembers(vec![QapiMember {
                r#if: config("CONFIG_B"),
                ..member(
                    "m",
                    QapiTypeRef::InlineStruct(QapiMembers(vec![
                        QapiMember {
                            r#if: config("CONFIG_C"),
                            ..member(
                                "n",
                                QapiTypeRef::InlineStruct(QapiMembers(vec![member(
                                    "a",
                                    QapiTypeRef::Ref("int".into()),
                                )])),
                            )
                        },
                        member(
                            "o",
                            QapiTypeRef::InlineStruct(QapiMembers(vec![member(
                                "a",
                                QapiTypeRef::Ref("int".into()),
                            )])),
                        ),
                    ])),
                )
            }]),
            ..Default::default()
        };
        let (value, extracted) = process_struct(q, &HashMap::new(), &mut HashSet::new()).unwrap();
        assert_eq!(condition(&value.meta), Some("CONFIG_A"));
        assert_eq!(condition(&value.fields[0].meta), Some("CONFIG_B"));
        let conditions: Vec<_> = extracted
            .iter()
            .map(|value| (value.name.as_str(), condition(&value.meta)))
            .collect();
        assert_eq!(
            conditions,
            [
                ("OuterMN", Some("((CONFIG_A && CONFIG_B) && CONFIG_C)")),
                ("OuterMO", Some("(CONFIG_A && CONFIG_B)")),
                ("OuterM", Some("(CONFIG_A && CONFIG_B)")),
            ]
        );
        assert_eq!(condition(&extracted[2].fields[0].meta), Some("CONFIG_C"));
    }

    fn returns(value: &Struct) -> Vec<&str> {
        value
            .meta
            .attributes
            .iter()
            .filter_map(|attr| match attr {
                Attribute::Item { name, value } if name == "returns" => Some(value.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn test_process_command_simple() {
        let q = QapiCommand {
            name: "query-status".into(),
            data: Some(MembersOrRef::Members(QapiMembers(vec![]))),
            returns: Some(QapiTypeRef::Ref("StatusInfo".into())),
            ..Default::default()
        };
        let s = process_command(q, &HashMap::new()).unwrap();
        assert_eq!(s.name, "query-status");
        assert_eq!(s.fields, []);
        assert_eq!(s.meta.schema_item, Some(SchemaItem::Command));
        assert_eq!(
            s.meta.attributes,
            [
                Attribute::new("command"),
                Attribute::with_value("name", "query-status"),
                Attribute::with_value("returns", "StatusInfo"),
            ]
        );
    }

    #[test]
    fn command_return_types_become_rust_types() {
        for (returns_type, version, expected) in [
            ("", (11, 1, 3), "()"),
            ("'returns': 'str', ", (11, 1, 3), "String"),
            ("'returns': 'int', ", (11, 1, 3), "i64"),
            ("'returns': 'size', ", (11, 1, 3), "u64"),
            ("'returns': 'any', ", (11, 1, 3), "serde_json::Value"),
            ("'returns': 'visitor', ", (1, 1, 0), "serde_json::Value"),
            ("'returns': '**', ", (2, 4, 0), "serde_json::Value"),
            ("'returns': 'query-info', ", (11, 1, 3), "QueryInfo"),
            (
                "'returns': ['ChardevInfo'], ",
                (11, 1, 3),
                "Vec<ChardevInfo>",
            ),
            ("'returns': ['int'], ", (11, 1, 3), "Vec<i64>"),
        ] {
            let input = format!("{{ 'command': 'c', {returns_type}'data': {{}} }}");
            let value = process_command(command(&input, version), &HashMap::new()).unwrap();
            assert_eq!(returns(&value), [expected], "{input}");
        }
    }

    #[test]
    fn gen_false_commands_get_a_flattened_arguments_map() {
        for (flag, version) in [("false", (2, 4, 0)), ("'no'", (2, 0, 0))] {
            let input = format!(
                "{{ 'command': 'device_add', 'data': {{ 'driver': 'str' }}, 'gen': {flag} }}"
            );
            let value = process_command(command(&input, version), &HashMap::new()).unwrap();
            let [driver, arguments] = &value.fields[..] else {
                panic!("{flag}: {:?}", value.fields)
            };
            assert_eq!(driver.name, "driver");
            assert_eq!(arguments.name, "arguments");
            assert_eq!(arguments.r#type, ARGUMENTS_TYPE);
            assert!(arguments.optional && !arguments.array);
            assert_eq!(arguments.meta.attributes, [Attribute::new("flatten")]);
            assert!(arguments.meta.doc.is_some());
        }
        let value = process_command(
            command(
                "{ 'command': 'device_add', 'data': { 'driver': 'str' }, 'gen': 'yes' }",
                (2, 0, 0),
            ),
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(value.fields.len(), 1);
        // A declared `arguments` stays, apart from the flattened map.
        let value = process_command(
            command(
                "{ 'command': 'c', 'data': { 'arguments': 'str' }, 'gen': false }",
                (11, 1, 3),
            ),
            &HashMap::new(),
        )
        .unwrap();
        let flattened: Vec<_> = value
            .fields
            .iter()
            .map(|field| {
                (
                    field.name.as_str(),
                    field.meta.attributes.contains(&Attribute::new("flatten")),
                )
            })
            .collect();
        assert_eq!(flattened, [("arguments", false), ("arguments", true)]);
    }

    #[test]
    fn historical_success_response_strings_keep_their_wire_behavior() {
        for (flag, suppressed) in [("'no'", true), ("'yes'", false), ("false", true)] {
            let input = format!("{{'command':'test','success-response':{flag}}}");
            let (_, command) = QapiCommand::parse_v1(&input).unwrap();
            let ir = process_command(command, &HashMap::new()).unwrap();
            assert_eq!(
                ir.meta
                    .attributes
                    .contains(&Attribute::new("no_success_response")),
                suppressed,
                "{flag}",
            );
        }
    }

    #[test]
    fn test_process_command_with_flags() {
        let q = QapiCommand {
            name: "test-cmd".into(),
            success_response: Some("false".into()),
            allow_oob: Some("true"),
            allow_preconfig: Some("true"),
            ..Default::default()
        };
        let s = process_command(q, &HashMap::new()).unwrap();
        let attr_names: Vec<_> = s
            .meta
            .attributes
            .iter()
            .filter_map(|a| match a {
                Attribute::Unit(n) => Some(n.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            attr_names,
            [
                "command",
                "allow_oob",
                "allow_preconfig",
                "no_success_response"
            ]
        );
    }

    #[test]
    fn events_are_named_apart_from_commands_and_keep_their_wire_name() {
        let data = Struct {
            name: "ShutdownInfo".into(),
            fields: vec![StructField {
                name: "guest".into(),
                r#type: "bool".into(),
                optional: false,
                array: false,
                meta: Metadata::default(),
            }],
            meta: Metadata::default(),
        };
        let lookup = HashMap::from([("ShutdownInfo".to_owned(), data.clone())]);
        for (input, fields) in [
            (
                "{ 'event': 'SHUTDOWN', 'if': 'CONFIG_A', 'data': { 'guest': 'bool' } }",
                vec![StructField {
                    name: "guest".into(),
                    r#type: "bool".into(),
                    optional: false,
                    array: false,
                    meta: Metadata::default(),
                }],
            ),
            (
                "{ 'event': 'SHUTDOWN', 'if': 'CONFIG_A', 'data': 'ShutdownInfo' }",
                data.fields.clone(),
            ),
        ] {
            let QapiSchemaToken::Event(value) = token(input, (11, 1, 3)) else {
                panic!("event token")
            };
            let value = process_event(value, &lookup).unwrap();
            assert_eq!(value.name, "SHUTDOWNEvent");
            assert_eq!(value.meta.schema_item, Some(SchemaItem::Event));
            assert_eq!(
                value.meta.attributes,
                [
                    Attribute::new("event"),
                    Attribute::with_value("name", "SHUTDOWN"),
                    Attribute::with_value("condition", "CONFIG_A"),
                ]
            );
            assert_eq!(value.fields, fields, "{input}");
        }
    }

    #[test]
    fn test_process_alternate() {
        let q = QapiAlternate {
            name: "StrOrInt".into(),
            data: QapiAlternatives(vec![
                QapiAlternative {
                    malformed: Vec::new(),
                    name: "s".into(),
                    r#type: QapiTypeRef::Ref("str".into()),
                    r#if: None,
                    keys: None,
                },
                QapiAlternative {
                    malformed: Vec::new(),
                    name: "i".into(),
                    r#type: QapiTypeRef::Ref("int".into()),
                    r#if: None,
                    keys: None,
                },
            ]),
            r#if: None,
            features: None,
            doc: None,
            malformed: Vec::new(),
            keys: Vec::new(),
            text: "",
        };
        let e = process_alternate(q).unwrap();
        assert_eq!(e.name, "StrOrInt");
        assert_eq!(e.meta.schema_item, Some(SchemaItem::Alternate));
        assert_eq!(e.variants.len(), 2);
        assert_eq!(e.variants[0].kind, EnumVariantKind::Tuple("str".into()));
        assert_eq!(e.variants[1].kind, EnumVariantKind::Tuple("int".into()));
    }

    #[test]
    fn flat_union_synthesizes_only_known_implicit_empty_discriminator_branches() {
        let q = union(
            "{'union':'Cpu','base':{'target':'Target'},'discriminator':'target','data':{'s390x':'S390Details'}}",
            (11, 1, 3),
        );
        let mut tags = process_enum(QapiEnum {
            name: "Target".into(),
            data: vec![
                make_enum_value("x86_64"),
                make_enum_value("aarch64"),
                make_enum_value("s390x"),
            ],
            ..Default::default()
        });
        tags.variants[1]
            .meta
            .attributes
            .push(Attribute::with_value("condition", "TARGET_AARCH64"));
        let (value, branches, types) = process_union(
            q,
            &HashMap::new(),
            &HashMap::from([("Target".into(), tags.clone())]),
        )
        .unwrap();
        assert_eq!(types, None);
        assert!(
            !value
                .meta
                .attributes
                .contains(&Attribute::new("simple_union"))
        );
        let [target, branch] = &value.fields[..] else {
            panic!("{:?}", value.fields)
        };
        assert_eq!(target.name, "target");
        assert_eq!(target.meta.attributes, [Attribute::new("discriminator")]);
        assert_eq!(
            (branch.name.as_str(), branch.r#type.as_str()),
            ("u", "CpuBranch")
        );
        // The declared branch, then the other values with their metadata.
        assert_eq!(
            branches
                .variants
                .iter()
                .map(|v| (v.name.as_str(), &v.kind))
                .collect::<Vec<_>>(),
            [
                ("s390x", &EnumVariantKind::Tuple("S390Details".into())),
                ("x86_64", &EnumVariantKind::Unit),
                ("aarch64", &EnumVariantKind::Unit),
            ]
        );
        assert_eq!(branches.variants[2].meta, tags.variants[1].meta);
    }
}
