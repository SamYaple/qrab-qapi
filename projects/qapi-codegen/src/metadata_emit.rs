//! Emit compile-time schema metadata from the unified registry.
//!
//! Walks the `UnifiedRegistry` and produces the `metadata.rs` module: static
//! tables describing every struct (commands' arguments and events' data
//! included), enum and field, and the commands, so that a program can inspect
//! the schema, for example to build requests, without hard-coding QAPI type
//! information. The tables' types are defined and documented once, in the
//! `metadata_types` module of qrab-qapi, which the file re-exports
//! (`METADATA_TYPES`); the statics construct them, so they are in that crate.
//!
//! - `TYPE_REFS`: a deduplicated pool of every type that a field, variant
//!   payload or command return refers to. A field's type, a variant's
//!   payload type and a command's return are references to an entry of it,
//!   and a `List` entry refers to its element, so each resolves against the
//!   pool of its own module.
//! - `STRUCTS` / `ENUMS`: `Table`s sorted and looked up by Rust type name.
//! - `COMMANDS`: the commands sorted by wire name, each pointing to the
//!   `StructMeta` of its arguments.

use crate::{
    qapi_ir::{
        Attribute, EnumVariantKind, StructField, rustify_field, rustify_name,
    },
    unified::{
        TypeCategory, UnifiedEnum, UnifiedRegistry, UnifiedStruct, UnifiedType,
        Version, VersionRange, classify_type,
    },
};
use std::collections::BTreeMap;

fn ver_tuple(v: Version) -> String {
    format!("({}u32, {}u32, {}u32)", v.0, v.1, v.2)
}

fn ver_opt(v: Option<Version>) -> String {
    match v {
        Some(t) => format!("Some({})", ver_tuple(t)),
        None => "None".to_string(),
    }
}

fn opt(value: Option<impl std::fmt::Display>) -> String {
    value.map_or_else(|| "None".into(), |value| format!("Some({value})"))
}

/// A reference to the entry `index` of `TYPE_REFS`.
fn type_ref(index: u32) -> String {
    format!("&TYPES[{index}]")
}

/// The end of a field's, variant's or flag's run within its type: the
/// registry leaves the run open when the type ends first.
fn until(range: &VersionRange, item: &VersionRange) -> Option<Version> {
    range.until.or(item.until)
}

/// A `&[Releases]` slice of the runs of `ranges` within `item`, in release
/// order.
fn releases<'a>(
    ranges: impl IntoIterator<Item = &'a VersionRange>,
    item: &VersionRange,
) -> String {
    runs(ranges.into_iter().map(|range| (range.since, range.until)), item.until)
}

/// A `&[Releases]` slice of `runs`, in release order, those left open ending
/// with their item at `end`.
fn runs(
    runs: impl IntoIterator<Item = (Version, Option<Version>)>,
    end: Option<Version>,
) -> String {
    let mut ranges: Vec<_> =
        runs.into_iter().map(|(since, until)| (since, until.or(end))).collect();
    ranges.sort();
    let entries: Vec<_> = ranges
        .into_iter()
        .map(|(since, until)| {
            format!(
                "Releases {{ since: {}, until: {} }}",
                ver_tuple(since),
                ver_opt(until)
            )
        })
        .collect();
    format!("&[{}]", entries.join(", "))
}

/// The runs of a type's or command's `deprecated` or `unstable` feature,
/// ending with it at the latest.
fn feature_runs(
    ranges: &[(Version, Option<Version>)],
    item: &VersionRange,
) -> String {
    runs(ranges.iter().copied(), item.until)
}

/// The runs of the releases from `since` to `end` that `runs` leave out.
fn complement(
    mut runs: Vec<(Version, Option<Version>)>,
    since: Version,
    end: Option<Version>,
) -> Vec<(Version, Option<Version>)> {
    runs.sort();
    let mut gaps = Vec::new();
    let mut start = Some(since);
    for (run_since, run_until) in runs {
        let Some(from) = start else { break };
        if run_since > from {
            gaps.push((from, Some(run_since)));
        }
        start = match (run_until, start) {
            (Some(until), Some(from)) => Some(until.max(from)),
            _ => None,
        };
    }
    // A run ends before its item does: the registry leaves it open when
    // the item ends first.
    if let Some(from) = start {
        gaps.push((from, end));
    }
    gaps
}

/// The runs of a command's flag: its newest run and the earlier ones.
fn flag_runs(
    value: &UnifiedStruct,
    name: &str,
) -> Vec<(Version, Option<Version>)> {
    value
        .attr_versions
        .get(name)
        .into_iter()
        .chain(
            value
                .attr_history
                .iter()
                .filter_map(|(flag, range)| (flag == name).then_some(range)),
        )
        .map(|range| (range.since, range.until))
        .collect()
}

/// Escape a string for inclusion as a Rust string literal.
fn rs_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x00'..='\x1f' => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn has_unit_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| matches!(a, Attribute::Unit(n) if n == name))
}

/// The value of an `Item` attribute.
fn item_attr<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a str> {
    attrs.iter().find_map(|a| match a {
        Attribute::Item { name: n, value } if n == name => Some(value.as_str()),
        _ => None,
    })
}

/// The wire name of an item, field or variant: its `name` attribute, which
/// the lowering adds where the wire name differs from the IR name, or else
/// the IR name.
fn wire_name<'a>(name: &'a str, attrs: &'a [Attribute]) -> &'a str {
    item_attr(attrs, "name").unwrap_or(name)
}

/// The element type of a command's `returns`, a Rust type that the lowering
/// writes as `()`, `T` or `Vec<T>`, and whether it is an array.
pub(crate) fn returns_element(rust: &str) -> (&str, bool) {
    match rust.strip_prefix("Vec<").and_then(|inner| inner.strip_suffix('>')) {
        Some(inner) => (inner, true),
        None => (rust, false),
    }
}

/// The `Prim` variant of a QAPI builtin type, `None` for other names.
fn qapi_prim_for_type(t: &str) -> Option<&'static str> {
    Some(match t {
        "str" => "Str",
        "bool" => "Bool",
        "int" | "int64" => "I64",
        "int8" => "I8",
        "int16" => "I16",
        "int32" => "I32",
        "uint8" => "U8",
        "uint16" => "U16",
        "uint32" => "U32",
        "uint64" | "size" => "U64",
        "number" => "F64",
        "null" => "Null",
        "any" | "visitor" | "dict" | "**" => "Any",
        _ => return None,
    })
}

/// The first reference of the registry, in Rust name order, to a type that
/// it does not define: the QAPI name and where it is, such as `Foo.bar`.
/// The lowering refuses undefined types in every release, and the merge
/// keeps every type of every release, so any is a generator bug.
pub(crate) fn undefined_reference(
    registry: &UnifiedRegistry,
) -> Option<(String, String)> {
    let defined = |qapi_type: &str| {
        let element = qapi_type
            .strip_prefix('[')
            .and_then(|inner| inner.strip_suffix(']'))
            .unwrap_or(qapi_type);
        qapi_prim_for_type(element).is_some()
            || registry.types.contains_key(&rustify_name(element))
    };
    registry.types.iter().find_map(|(name, value)| {
        let mut references: Vec<(&str, String)> = Vec::new();
        match value {
            UnifiedType::Struct(value) => {
                references.extend(
                    value
                        .ir
                        .fields
                        .iter()
                        .filter(|field| {
                            !has_unit_attr(&field.meta.attributes, "flatten")
                        })
                        .map(|field| {
                            (
                                field.r#type.as_str(),
                                format!("{name}.{}", field.name),
                            )
                        }),
                );
                references.extend(
                    value
                        .ir
                        .meta
                        .qapi_returns
                        .as_deref()
                        .map(|returns| (returns, format!("{name} returns"))),
                );
            }
            UnifiedType::Enum(value) => {
                references.extend(value.ir.variants.iter().filter_map(
                    |variant| match &variant.kind {
                        EnumVariantKind::Tuple(ty) => Some((
                            ty.as_str(),
                            format!("{name}::{}", variant.name),
                        )),
                        EnumVariantKind::Unit => None,
                    },
                ));
            }
        }
        references
            .into_iter()
            .find(|(qapi_type, _)| !defined(qapi_type))
            .map(|(qapi_type, at)| (qapi_type.to_owned(), at))
    })
}

/// A `TypeRef` of the pool.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TypeKey {
    /// The `TypeKind` variant.
    kind: &'static str,
    /// The QAPI builtin, the Rust type name, or `List<element>`.
    name: String,
    /// The `Prim` variant of a primitive.
    prim: Option<&'static str>,
    /// The index of a list's element.
    inner: Option<u32>,
}

/// The deduplicated pool of every type that a field, variant payload or
/// command return refers to.
struct TypePool<'a> {
    registry: &'a UnifiedRegistry,
    entries: Vec<TypeKey>,
    index: BTreeMap<TypeKey, u32>,
}

impl<'a> TypePool<'a> {
    fn new(registry: &'a UnifiedRegistry) -> Self {
        Self { registry, entries: Vec::new(), index: BTreeMap::new() }
    }

    fn intern(&mut self, key: TypeKey) -> u32 {
        if let Some(&index) = self.index.get(&key) {
            return index;
        }
        let index =
            u32::try_from(self.entries.len()).expect("fewer than 2^32 types");
        self.entries.push(key.clone());
        self.index.insert(key, index);
        index
    }

    /// The index of the IR type `qapi_type`, or of a list of it.
    fn intern_type(&mut self, qapi_type: &str, array: bool) -> u32 {
        let element = self.intern_scalar(qapi_type);
        if !array {
            return element;
        }
        let name = format!("List<{}>", self.entries[element as usize].name);
        self.intern(TypeKey {
            kind: "List",
            name,
            prim: None,
            inner: Some(element),
        })
    }

    /// The index of a QAPI builtin or of a type of the registry. The emitter
    /// checks every reference first (`undefined_reference`), so any other
    /// name is a generator bug.
    fn intern_scalar(&mut self, qapi_type: &str) -> u32 {
        if let Some(prim) = qapi_prim_for_type(qapi_type) {
            return self.intern(TypeKey {
                kind: "Primitive",
                name: qapi_type.to_string(),
                prim: Some(prim),
                inner: None,
            });
        }
        let name = rustify_name(qapi_type);
        let kind = match self.registry.types.get(&name) {
            Some(UnifiedType::Enum(_)) => "Enum",
            Some(UnifiedType::Struct(_)) => "Struct",
            None => {
                panic!("the lowering left the type '{qapi_type}' unresolved")
            }
        };
        self.intern(TypeKey { kind, name, prim: None, inner: None })
    }

    /// The index of a command's newest return type, which the lowering
    /// keeps as the schema spells it (`Metadata::qapi_returns`), as fields
    /// and variants have theirs: a builtin such as `int`, a type, or
    /// `[element]`.
    fn intern_returns(&mut self, qapi_returns: &str) -> u32 {
        match qapi_returns
            .strip_prefix('[')
            .and_then(|inner| inner.strip_suffix(']'))
        {
            Some(element) => self.intern_type(element, true),
            None => self.intern_type(qapi_returns, false),
        }
    }

    fn emit(&self) -> String {
        let mut s = String::new();
        s.push_str("/// Every type that a field, variant payload or command return refers to.\n");
        s.push_str("pub static TYPE_REFS: &[TypeRef] = &TYPES;\n");
        // An array, so that its entries can refer to each other.
        s.push_str(&format!(
            "static TYPES: [TypeRef; {}] = [\n",
            self.entries.len()
        ));
        for e in &self.entries {
            s.push_str(&format!(
                "    TypeRef {{ kind: TypeKind::{}, name: {}, prim: {}, element: {} }},\n",
                e.kind,
                rs_str(&e.name),
                opt(e.prim.map(|prim| format!("Prim::{prim}"))),
                opt(e.inner.map(type_ref)),
            ));
        }
        s.push_str("];\n");
        s
    }
}

/// The releases in which a struct is free-form: those of the map that the
/// lowering gives a `gen: false` command for the arguments its schema does
/// not declare, the only flattened field that is not a union's branches.
fn free_form(value: &UnifiedStruct) -> String {
    releases(
        value
            .ir
            .fields
            .iter()
            .filter(|field| {
                has_unit_attr(&field.meta.attributes, "flatten")
                    && !has_unit_attr(&field.meta.attributes, "union")
            })
            .map(|field| field_range(value, field)),
        &value.versions,
    )
}

/// The releases in which a union has the simple `{"type": ..., "data": ...}`
/// layout rather than the flat one.
fn simple_union(value: &UnifiedStruct) -> String {
    releases(
        value.attr_versions.get("simple_union").into_iter().chain(
            value.attr_history.iter().filter_map(|(name, range)| {
                (name == "simple_union").then_some(range)
            }),
        ),
        &value.versions,
    )
}

/// The newest discriminator and branch fields of a union, by the Rust names
/// of the field and of the branch enum.
fn detect_union(value: &UnifiedStruct) -> Option<(String, String)> {
    let newest = |predicate: fn(&StructField) -> bool| {
        value
            .ir
            .fields
            .iter()
            .filter(|field| predicate(field))
            .max_by_key(|field| field_range(value, field).since)
    };
    let discriminator =
        newest(|field| has_unit_attr(&field.meta.attributes, "discriminator"))?;
    let branch = newest(|field| {
        has_unit_attr(&field.meta.attributes, "union")
            && has_unit_attr(&field.meta.attributes, "flatten")
    })?;
    Some((rustify_field(&discriminator.name), rustify_name(&branch.r#type)))
}

fn field_range<'a>(
    value: &'a UnifiedStruct,
    field: &StructField,
) -> &'a VersionRange {
    value.field_versions.get(&field.name).unwrap_or(&value.versions)
}

fn variant_range<'a>(
    value: &'a UnifiedEnum,
    variant: &str,
) -> &'a VersionRange {
    value.variant_versions.get(variant).unwrap_or(&value.versions)
}

fn emit_fields(
    value: &UnifiedStruct,
    rust_name: &str,
    pool: &mut TypePool<'_>,
) -> String {
    let mut s = format!("static FIELDS_{rust_name}: &[FieldMeta] = &[\n");
    for f in &value.ir.fields {
        // Flatten receivers are virtual Rust fields without their own wire key.
        if has_unit_attr(&f.meta.attributes, "flatten") {
            continue;
        }
        let range = field_range(value, f);
        let end = until(range, &value.versions);
        s.push_str(&format!(
            "    FieldMeta {{ name: {}, wire_name: {}, ty: {}, optional: {}, discriminator: {}, boxed: {}, since: {}, until: {}, deprecated: {}, unstable: {} }},\n",
            rs_str(&rustify_field(&f.name)),
            rs_str(wire_name(&f.name, &f.meta.attributes)),
            type_ref(pool.intern_type(&f.r#type, f.array)),
            f.optional,
            runs(range.discriminator_ranges.iter().copied(), end),
            has_unit_attr(&f.meta.attributes, "boxed"),
            ver_tuple(range.since),
            ver_opt(end),
            runs(range.deprecated_ranges.iter().copied(), end),
            runs(range.unstable_ranges.iter().copied(), end),
        ));
    }
    s.push_str("];\n");
    s
}

fn emit_struct(value: &UnifiedStruct, rust_name: &str) -> String {
    let (discriminator, branch) = detect_union(value).unzip();
    format!(
        "static STRUCT_{rust_name}: StructMeta = StructMeta {{\n\
         \x20   name: {},\n\
         \x20   wire_name: {},\n\
         \x20   fields: FIELDS_{rust_name},\n\
         \x20   is_union: {},\n\
         \x20   discriminator_field: {},\n\
         \x20   branch_enum: {},\n\
         \x20   simple_union: {},\n\
         \x20   free_form: {},\n\
         \x20   since: {},\n\
         \x20   until: {},\n\
         \x20   deprecated: {},\n\
         \x20   unstable: {},\n\
         }};\n",
        rs_str(rust_name),
        rs_str(wire_name(&value.ir.name, &value.ir.meta.attributes)),
        discriminator.is_some(),
        opt(discriminator.as_deref().map(rs_str)),
        opt(branch.as_deref().map(rs_str)),
        simple_union(value),
        free_form(value),
        ver_tuple(value.versions.since),
        ver_opt(value.versions.until),
        feature_runs(&value.versions.deprecated_ranges, &value.versions),
        feature_runs(&value.versions.unstable_ranges, &value.versions),
    )
}

/// Intern payloads with the same scalar/list rules as struct fields.
fn emit_variants(
    value: &UnifiedEnum,
    rust_name: &str,
    pool: &mut TypePool<'_>,
) -> String {
    let mut s =
        format!("static VARIANTS_{rust_name}: &[EnumVariantMeta] = &[\n");
    for v in &value.ir.variants {
        let payload_type = match &v.kind {
            EnumVariantKind::Tuple(ty) => Some(pool.intern_type(ty, v.array)),
            EnumVariantKind::Unit => None,
        };
        let payload_struct = payload_type.and_then(|index| {
            let ty = &pool.entries[index as usize];
            (ty.kind == "Struct").then(|| rs_str(&ty.name))
        });
        let range = variant_range(value, &v.name);
        let end = until(range, &value.versions);
        s.push_str(&format!(
            "    EnumVariantMeta {{ name: {}, wire_name: {}, payload_struct: {}, payload_type: {}, since: {}, until: {}, deprecated: {}, unstable: {} }},\n",
            rs_str(&crate::qapi_ir::rustify_variant(&v.name)),
            rs_str(wire_name(&v.name, &v.meta.attributes)),
            opt(payload_struct),
            opt(payload_type.map(type_ref)),
            ver_tuple(range.since),
            ver_opt(end),
            runs(range.deprecated_ranges.iter().copied(), end),
            runs(range.unstable_ranges.iter().copied(), end),
        ));
    }
    s.push_str("];\n");
    s
}

fn emit_enum(value: &UnifiedEnum, rust_name: &str) -> String {
    let kind = if has_unit_attr(&value.ir.meta.attributes, "union_branch") {
        "UnionBranch"
    } else if value
        .ir
        .variants
        .iter()
        .all(|v| matches!(v.kind, EnumVariantKind::Unit))
    {
        "Unit"
    } else {
        "Alternate"
    };
    format!(
        "static ENUM_{rust_name}: EnumMeta = EnumMeta {{\n\
         \x20   name: {},\n\
         \x20   wire_name: {},\n\
         \x20   variants: VARIANTS_{rust_name},\n\
         \x20   kind: EnumKind::{kind},\n\
         \x20   since: {},\n\
         \x20   until: {},\n\
         \x20   deprecated: {},\n\
         \x20   unstable: {},\n\
         }};\n",
        rs_str(rust_name),
        rs_str(wire_name(&value.ir.name, &value.ir.meta.attributes)),
        ver_tuple(value.versions.since),
        ver_opt(value.versions.until),
        feature_runs(&value.versions.deprecated_ranges, &value.versions),
        feature_runs(&value.versions.unstable_ranges, &value.versions),
    )
}

/// A command's `CommandMeta` line, with its wire name as the sort key.
fn emit_command(
    value: &UnifiedStruct,
    rust_name: &str,
    pool: &mut TypePool<'_>,
) -> (String, String) {
    let attrs = &value.ir.meta.attributes;
    let wire = wire_name(&value.ir.name, attrs).to_owned();
    let versions = &value.versions;
    let flag = |name| runs(flag_runs(value, name), versions.until);
    let success_response = complement(
        flag_runs(value, "no_success_response"),
        versions.since,
        versions.until,
    );
    let returns = value
        .ir
        .meta
        .qapi_returns
        .as_deref()
        .map(|returns| pool.intern_returns(returns));
    let line = format!(
        "    CommandMeta {{ name: {}, wire_name: {}, args: &STRUCT_{rust_name}, returns: {}, allow_oob: {}, allow_preconfig: {}, success_response: {}, since: {}, until: {}, deprecated: {}, unstable: {} }},\n",
        rs_str(rust_name),
        rs_str(&wire),
        opt(returns.map(type_ref)),
        flag("allow_oob"),
        flag("allow_preconfig"),
        runs(success_response, versions.until),
        ver_tuple(versions.since),
        ver_opt(versions.until),
        feature_runs(&versions.deprecated_ranges, versions),
        feature_runs(&versions.unstable_ranges, versions),
    );
    (wire, line)
}

/// Emit the source of `metadata.rs` from the unified registry.
pub(crate) fn emit(registry: &UnifiedRegistry) -> String {
    // Case-insensitively by Rust name, as the other modules are.
    let mut entries: Vec<_> = registry.types.iter().collect();
    entries.sort_by_cached_key(|(name, _)| name.to_lowercase());

    let mut pool = TypePool::new(registry);
    let mut bodies = String::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut commands = Vec::new();
    for (rust_name, value) in entries {
        match value {
            UnifiedType::Struct(us) => {
                bodies.push_str(&emit_fields(us, rust_name, &mut pool));
                bodies.push_str(&emit_struct(us, rust_name));
                structs.push(rust_name);
                if classify_type(value) == TypeCategory::Command {
                    commands.push(emit_command(us, rust_name, &mut pool));
                }
            }
            UnifiedType::Enum(ue) => {
                bodies.push_str(&emit_variants(ue, rust_name, &mut pool));
                bodies.push_str(&emit_enum(ue, rust_name));
                enums.push(rust_name);
            }
        }
    }

    let mut out = String::new();
    out.push_str("// AUTO-GENERATED by qapi-codegen — do not edit by hand.\n");
    out.push_str(
        "#![allow(\n    non_upper_case_globals,\n    reason = \"statics are named after Rust types, which can differ only in case\"\n)]\n\n",
    );
    out.push_str(METADATA_TYPES);
    out.push('\n');
    out.push_str(&pool.emit());
    out.push('\n');
    out.push_str(&bodies);
    out.push('\n');

    out.push_str("/// The commands, sorted by wire name.\n");
    out.push_str("pub static COMMANDS: &[CommandMeta] = &[\n");
    commands.sort();
    for (_, line) in &commands {
        out.push_str(line);
    }
    out.push_str("];\n\n");

    // `Table::get` searches by `str` order, so the tables sort by it.
    for (doc, table, meta, prefix, mut names) in [
        (
            "The structs, unions, and command and event structs",
            "STRUCTS",
            "StructMeta",
            "STRUCT",
            structs,
        ),
        (
            "The enums, alternates and union branch enums",
            "ENUMS",
            "EnumMeta",
            "ENUM",
            enums,
        ),
    ] {
        names.sort();
        out.push_str(&format!(
            "/// {doc}, by Rust name.\npub static {table}: Table<{meta}> = Table {{\n    entries: &[\n"
        ));
        for name in names {
            out.push_str(&format!(
                "        ({}, &{prefix}_{name}),\n",
                rs_str(name)
            ));
        }
        out.push_str("    ],\n};\n");
        if table == "STRUCTS" {
            out.push('\n');
        }
    }
    out
}

/// The types of the tables: qrab-qapi defines them once, in its
/// `metadata_types` module, for the metadata module of every protocol.
const METADATA_TYPES: &str = "pub use crate::metadata_types::{\n    CommandMeta, EnumKind, EnumMeta, EnumVariantMeta, FieldMeta, Prim, Releases, StructMeta, Table,\n    TypeKind, TypeRef, Ver,\n};\n";
