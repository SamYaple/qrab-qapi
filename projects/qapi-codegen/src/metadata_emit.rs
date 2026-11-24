//! Emit compile-time schema metadata from the unified registry.
//!
//! Walks the `UnifiedRegistry` and produces the `metadata.rs` module: static
//! tables describing every struct (commands' arguments and events' data
//! included), enum and field, and the commands, so that a program can inspect
//! the schema, for example to build requests, without hard-coding QAPI type
//! information. `METADATA_TYPES` defines and documents the tables' types.
//!
//! - `TYPE_REFS`: a deduplicated pool of every type that a field, variant
//!   payload or command return refers to. `FieldMeta::ty`,
//!   `EnumVariantMeta::payload_type` and `CommandMeta::returns` are indices
//!   into it, and a `List` entry holds the index of its element.
//! - `STRUCTS` / `ENUMS`: `Table`s sorted and looked up by Rust type name.
//! - `COMMANDS`: the commands sorted by wire name, each pointing to the
//!   `StructMeta` of its arguments.

use crate::qapi_ir::{Attribute, EnumVariantKind, StructField, rustify_field, rustify_name};
use crate::unified::{
    TypeCategory, UnifiedEnum, UnifiedRegistry, UnifiedStruct, UnifiedType, Version, VersionRange,
    classify_type,
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

/// The end of a field's, variant's or flag's run within its type: the
/// registry leaves the run open when the type ends first.
fn until(range: &VersionRange, item: &VersionRange) -> Option<Version> {
    range.until.or(item.until)
}

/// A `&[Releases]` slice of the runs of `ranges` within `item`, in release
/// order.
fn releases<'a>(ranges: impl IntoIterator<Item = &'a VersionRange>, item: &VersionRange) -> String {
    runs(
        ranges.into_iter().map(|range| (range.since, range.until)),
        item.until,
    )
}

/// A `&[Releases]` slice of `runs`, in release order, those left open ending
/// with their item at `end`.
fn runs(
    runs: impl IntoIterator<Item = (Version, Option<Version>)>,
    end: Option<Version>,
) -> String {
    let mut ranges: Vec<_> = runs
        .into_iter()
        .map(|(since, until)| (since, until.or(end)))
        .collect();
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
fn feature_runs(ranges: &[(Version, Option<Version>)], item: &VersionRange) -> String {
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
fn flag_runs(value: &UnifiedStruct, name: &str) -> Vec<(Version, Option<Version>)> {
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
    attrs
        .iter()
        .any(|a| matches!(a, Attribute::Unit(n) if n == name))
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
    match rust
        .strip_prefix("Vec<")
        .and_then(|inner| inner.strip_suffix('>'))
    {
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
pub(crate) fn undefined_reference(registry: &UnifiedRegistry) -> Option<(String, String)> {
    let defined = |qapi_type: &str| {
        let element = qapi_type
            .strip_prefix('[')
            .and_then(|inner| inner.strip_suffix(']'))
            .unwrap_or(qapi_type);
        qapi_prim_for_type(element).is_some() || registry.types.contains_key(&rustify_name(element))
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
                        .filter(|field| !has_unit_attr(&field.meta.attributes, "flatten"))
                        .map(|field| (field.r#type.as_str(), format!("{name}.{}", field.name))),
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
                        EnumVariantKind::Tuple(ty) => {
                            Some((ty.as_str(), format!("{name}::{}", variant.name)))
                        }
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
        Self {
            registry,
            entries: Vec::new(),
            index: BTreeMap::new(),
        }
    }

    fn intern(&mut self, key: TypeKey) -> u32 {
        if let Some(&index) = self.index.get(&key) {
            return index;
        }
        let index = u32::try_from(self.entries.len()).expect("fewer than 2^32 types");
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
            None => panic!("the lowering left the type '{qapi_type}' unresolved"),
        };
        self.intern(TypeKey {
            kind,
            name,
            prim: None,
            inner: None,
        })
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
        s.push_str("pub static TYPE_REFS: &[TypeRef] = &[\n");
        for e in &self.entries {
            s.push_str(&format!(
                "    TypeRef {{ kind: TypeKind::{}, name: {}, prim: {}, inner: {} }},\n",
                e.kind,
                rs_str(&e.name),
                opt(e.prim.map(|prim| format!("Prim::{prim}"))),
                opt(e.inner),
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
            value
                .attr_history
                .iter()
                .filter_map(|(name, range)| (name == "simple_union").then_some(range)),
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
    let discriminator = newest(|field| has_unit_attr(&field.meta.attributes, "discriminator"))?;
    let branch = newest(|field| {
        has_unit_attr(&field.meta.attributes, "union")
            && has_unit_attr(&field.meta.attributes, "flatten")
    })?;
    Some((
        rustify_field(&discriminator.name),
        rustify_name(&branch.r#type),
    ))
}

fn field_range<'a>(value: &'a UnifiedStruct, field: &StructField) -> &'a VersionRange {
    value
        .field_versions
        .get(&field.name)
        .unwrap_or(&value.versions)
}

fn variant_range<'a>(value: &'a UnifiedEnum, variant: &str) -> &'a VersionRange {
    value
        .variant_versions
        .get(variant)
        .unwrap_or(&value.versions)
}

fn emit_fields(value: &UnifiedStruct, rust_name: &str, pool: &mut TypePool<'_>) -> String {
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
            pool.intern_type(&f.r#type, f.array),
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
fn emit_variants(value: &UnifiedEnum, rust_name: &str, pool: &mut TypePool<'_>) -> String {
    let mut s = format!("static VARIANTS_{rust_name}: &[EnumVariantMeta] = &[\n");
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
            opt(payload_type),
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
        opt(returns),
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
            out.push_str(&format!("        ({}, &{prefix}_{name}),\n", rs_str(name)));
        }
        out.push_str("    ],\n};\n");
        if table == "STRUCTS" {
            out.push('\n');
        }
    }
    out
}

/// The type definitions that start the generated metadata file: the public
/// API of the generated `metadata` modules.
const METADATA_TYPES: &str = r#"/// A QEMU release as `(major, minor, micro)`.
pub type Ver = (u32, u32, u32);

/// A run of releases: from `since` up to, but not including, `until`, or
/// through the newest release that the tables describe when `until` is
/// `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Releases {
    /// The first release of the run.
    pub since: Ver,
    /// The first release after the run, `None` when the run continues
    /// through the newest release.
    pub until: Option<Ver>,
}

impl Releases {
    /// Whether `version` is one of these releases.
    pub fn contains(&self, version: Ver) -> bool {
        version >= self.since && self.until.is_none_or(|until| version < until)
    }
}

/// Whether one of the runs `releases` has `version`.
fn any_contains(releases: &[Releases], version: Ver) -> bool {
    releases.iter().any(|releases| releases.contains(version))
}

/// Metadata entries sorted by Rust type name, looked up by binary search.
pub struct Table<T: 'static> {
    entries: &'static [(&'static str, &'static T)],
}

impl<T> Table<T> {
    /// The entry for the Rust type `name`.
    pub fn get(&self, name: &str) -> Option<&'static T> {
        let index = self.entries.binary_search_by(|(key, _)| (*key).cmp(name)).ok()?;
        Some(self.entries[index].1)
    }

    /// Whether the table has an entry for the Rust type `name`.
    pub fn contains_key(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Names and entries in name order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&'static str, &'static T)> {
        self.entries.iter().copied()
    }

    /// Names in order.
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &'static str> {
        self.entries.iter().map(|(name, _)| *name)
    }

    /// Entries in name order.
    pub fn values(&self) -> impl ExactSizeIterator<Item = &'static T> {
        self.entries.iter().map(|(_, entry)| *entry)
    }
}

impl<T> std::fmt::Debug for Table<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.keys()).finish()
    }
}

/// The value of a QAPI builtin type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prim {
    /// `str`.
    Str,
    /// `bool`.
    Bool,
    /// `int8`.
    I8,
    /// `int16`.
    I16,
    /// `int32`.
    I32,
    /// `int` and `int64`.
    I64,
    /// `uint8`.
    U8,
    /// `uint16`.
    U16,
    /// `uint32`.
    U32,
    /// `uint64` and `size`.
    U64,
    /// `number`.
    F64,
    /// `null`.
    Null,
    /// `any`, and the untyped values of schemas before QEMU 2.5.
    Any,
}

/// What a [`TypeRef`] refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    /// A QAPI builtin type.
    Primitive,
    /// An enum, alternate or union branch enum of `ENUMS`.
    Enum,
    /// A struct, union, or command or event struct of `STRUCTS`.
    Struct,
    /// An array.
    List,
}

/// A type that a field, variant payload or command return refers to, an
/// entry of `TYPE_REFS`.
#[derive(Debug)]
pub struct TypeRef {
    /// What it refers to.
    pub kind: TypeKind,
    /// A primitive's QAPI builtin, such as `str`, `int` or `int64`, as the
    /// newest release with the field, variant or return that refers to it
    /// spells it (`QomSet.value`, typed `visitor` in QEMU 1.1 to 2.3, `**`
    /// in 2.4 and `any` from 2.5, refers to `any`); the key of an enum or
    /// struct in `ENUMS` or `STRUCTS`; `List<element>` for a list.
    pub name: &'static str,
    /// The builtin of a primitive, `None` for any other kind.
    pub prim: Option<Prim>,
    /// The index in `TYPE_REFS` of a list's element, `None` for any other
    /// kind.
    pub inner: Option<u32>,
}

/// A field of a struct, union, command or event.
#[derive(Debug)]
pub struct FieldMeta {
    /// The Rust field name as the struct declares it, a raw identifier with
    /// its `r#` prefix, such as `r#in`.
    pub name: &'static str,
    /// The member's name on the wire.
    pub wire_name: &'static str,
    /// The index of its type in `TYPE_REFS`.
    pub ty: u32,
    /// Whether the active wire field may be absent, independently of Rust's version-merged Option.
    pub optional: bool,
    /// The releases in which it is its union's discriminator: a union may
    /// change which member discriminates while both exist.
    pub discriminator: &'static [Releases],
    /// Whether the Rust field holds its value in a `Box`: a value of its
    /// type would otherwise contain the struct itself, other than through
    /// an array or another boxed field.
    pub boxed: bool,
    /// The first release with the field.
    pub since: Ver,
    /// The first release after `since` without it, `None` while the newest has it.
    pub until: Option<Ver>,
    /// The releases that deprecate the field.
    pub deprecated: &'static [Releases],
    /// The releases that mark the field unstable.
    pub unstable: &'static [Releases],
}

impl FieldMeta {
    /// Whether release `version` deprecates the field.
    pub fn is_deprecated(&self, version: Ver) -> bool {
        any_contains(self.deprecated, version)
    }

    /// Whether release `version` marks the field unstable.
    pub fn is_unstable(&self, version: Ver) -> bool {
        any_contains(self.unstable, version)
    }

    /// Whether the field is its union's discriminator in release `version`.
    pub fn is_discriminator(&self, version: Ver) -> bool {
        any_contains(self.discriminator, version)
    }
}

/// A variant of an enum, alternate or union branch enum.
#[derive(Debug)]
pub struct EnumVariantMeta {
    /// The Rust variant name.
    pub name: &'static str,
    /// The enum value, alternate branch or union tag on the wire.
    pub wire_name: &'static str,
    /// Direct struct payload, if any; scalar, enum and list payloads use payload_type.
    pub payload_struct: Option<&'static str>,
    /// Index into TYPE_REFS, including null payloads; None means a unit variant.
    pub payload_type: Option<u32>,
    /// The first release with the variant.
    pub since: Ver,
    /// The first release after `since` without it, `None` while the newest has it.
    pub until: Option<Ver>,
    /// The releases that deprecate the variant.
    pub deprecated: &'static [Releases],
    /// The releases that mark the variant unstable.
    pub unstable: &'static [Releases],
}

impl EnumVariantMeta {
    /// Whether release `version` deprecates the variant.
    pub fn is_deprecated(&self, version: Ver) -> bool {
        any_contains(self.deprecated, version)
    }

    /// Whether release `version` marks the variant unstable.
    pub fn is_unstable(&self, version: Ver) -> bool {
        any_contains(self.unstable, version)
    }
}

/// The kind of an [`EnumMeta`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumKind {
    /// A QAPI enum: variants without payloads.
    Unit,
    /// A QAPI alternate: a value of one of the variants' types.
    Alternate,
    /// The branches of a union, the enum that its `StructMeta::branch_enum`
    /// names: a variant per tag, holding that branch's members. The union's
    /// `StructMeta::simple_union` gives the releases in which these travel
    /// as `"data"` beside the `"type"` tag (a simple union), and they are
    /// flat members of the union in the others.
    UnionBranch,
}

/// An enum, alternate or union branch enum.
#[derive(Debug)]
pub struct EnumMeta {
    /// Rust type name, also used as the ENUMS lookup key.
    pub name: &'static str,
    /// The QAPI name; a type that the generator synthesizes, such as a
    /// union's branch enum, has its Rust name.
    pub wire_name: &'static str,
    /// The variants of every release.
    pub variants: &'static [EnumVariantMeta],
    /// What kind of enum it is.
    pub kind: EnumKind,
    /// The first release with the type.
    pub since: Ver,
    /// The first release after `since` without it, `None` while the newest has it.
    pub until: Option<Ver>,
    /// The releases that deprecate the type.
    pub deprecated: &'static [Releases],
    /// The releases that mark the type unstable.
    pub unstable: &'static [Releases],
}

impl EnumMeta {
    /// Whether release `version` deprecates the type.
    pub fn is_deprecated(&self, version: Ver) -> bool {
        any_contains(self.deprecated, version)
    }

    /// Whether release `version` marks the type unstable.
    pub fn is_unstable(&self, version: Ver) -> bool {
        any_contains(self.unstable, version)
    }
}

/// A struct, union, or the arguments of a command or the data of an event.
#[derive(Debug)]
pub struct StructMeta {
    /// Rust type name, also used as the STRUCTS lookup key.
    pub name: &'static str,
    /// The QAPI name, the wire name of a command or event, or for a type
    /// that the generator synthesizes, such as an anonymous struct, its Rust
    /// name.
    pub wire_name: &'static str,
    /// The fields that have their own wire members, in every release.
    pub fields: &'static [FieldMeta],
    /// Whether it is a union, with a discriminator and branches.
    pub is_union: bool,
    /// Rust field name of the newest observed discriminator representation;
    /// [`discriminator`](Self::discriminator) gives the field of one release.
    pub discriminator_field: Option<&'static str>,
    /// Newest branch representation; earlier branch enums retain their own metadata.
    pub branch_enum: Option<&'static str>,
    /// The releases in which a union is a simple union, with its branch in
    /// `"data"`.
    pub simple_union: &'static [Releases],
    /// The releases in which a command takes arguments that its schema does
    /// not declare (`gen: false`), besides those of `fields`.
    pub free_form: &'static [Releases],
    /// The first release with the type.
    pub since: Ver,
    /// The first release after `since` without it, `None` while the newest has it.
    pub until: Option<Ver>,
    /// The releases that deprecate the type: the event, or for a command's
    /// arguments the command.
    pub deprecated: &'static [Releases],
    /// The releases that mark the type, event or command unstable.
    pub unstable: &'static [Releases],
}

impl StructMeta {
    /// Whether a union is a simple union in `version`.
    pub fn is_simple_union(&self, version: Ver) -> bool {
        any_contains(self.simple_union, version)
    }

    /// The field that is a union's discriminator in `version`.
    pub fn discriminator(&self, version: Ver) -> Option<&'static FieldMeta> {
        self.fields.iter().find(|field| field.is_discriminator(version))
    }

    /// Whether a command takes undeclared arguments in `version`.
    pub fn is_free_form(&self, version: Ver) -> bool {
        any_contains(self.free_form, version)
    }

    /// Whether release `version` deprecates the type.
    pub fn is_deprecated(&self, version: Ver) -> bool {
        any_contains(self.deprecated, version)
    }

    /// Whether release `version` marks the type unstable.
    pub fn is_unstable(&self, version: Ver) -> bool {
        any_contains(self.unstable, version)
    }
}

/// A command, an entry of `COMMANDS`.
#[derive(Debug)]
pub struct CommandMeta {
    /// The Rust name of the command's struct.
    pub name: &'static str,
    /// The command's name on the wire.
    pub wire_name: &'static str,
    /// The command's arguments, the entry of its struct in `STRUCTS`.
    pub args: &'static StructMeta,
    /// The index in `TYPE_REFS` of what the newest release returns, `None`
    /// for a command without a return type.
    pub returns: Option<u32>,
    /// The releases that allow out-of-band execution of the command.
    pub allow_oob: &'static [Releases],
    /// The releases that allow the command before the machine is configured.
    pub allow_preconfig: &'static [Releases],
    /// The releases that reply to the command when it succeeds.
    pub success_response: &'static [Releases],
    /// The first release with the command.
    pub since: Ver,
    /// The first release after `since` without it, `None` while the newest has it.
    pub until: Option<Ver>,
    /// The releases that deprecate the command.
    pub deprecated: &'static [Releases],
    /// The releases that mark the command unstable.
    pub unstable: &'static [Releases],
}

impl CommandMeta {
    /// Whether release `version` allows out-of-band execution of the command.
    pub fn allows_oob(&self, version: Ver) -> bool {
        any_contains(self.allow_oob, version)
    }

    /// Whether release `version` allows the command before the machine is
    /// configured.
    pub fn allows_preconfig(&self, version: Ver) -> bool {
        any_contains(self.allow_preconfig, version)
    }

    /// Whether release `version` replies to the command when it succeeds.
    pub fn sends_success_response(&self, version: Ver) -> bool {
        any_contains(self.success_response, version)
    }

    /// Whether release `version` deprecates the command.
    pub fn is_deprecated(&self, version: Ver) -> bool {
        any_contains(self.deprecated, version)
    }

    /// Whether release `version` marks the command unstable.
    pub fn is_unstable(&self, version: Ver) -> bool {
        any_contains(self.unstable, version)
    }
}
"#;
