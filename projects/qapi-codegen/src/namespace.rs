//! Every Rust name of the generated modules, by the namespace it is
//! declared in, and the check that no two items of one namespace share one.
//!
//! The generator takes most names from QAPI, through `rustify_name`,
//! `rustify_field` and `rustify_variant`, and synthesizes the others. Each
//! [`Pattern`] is one way of making a name, with its escape. The symbol plan
//! (`crate::symbols`) names the types, commands and events of every
//! release, with their helper types, before the lowering, and the merge
//! names the historical members; `plan_returns_enums` then names the enums
//! of the return types that a command still has after coalescing, and
//! [`inventory`] lists every name and refuses a namespace in which two
//! items would share one, naming both, so that a clash is a generation error
//! and never a duplicate definition in the generated code.
//!
//! The namespaces are those of the Rust items, as the generated code
//! declares them:
//!
//! - the items of a protocol: its data types, command structs, event
//!   structs and `Returns` enums, which `mod.rs`, `commands.rs` and
//!   `events.rs` share through their glob imports, beside the modules
//!   `commands`, `events` and `metadata` and the root items of `qrab-qapi`,
//!   which its glob re-export of the QMP types meets;
//! - the fields of each struct and the variants of each enum and each
//!   `Returns` enum;
//! - the statics of `metadata.rs`.

use crate::{
    qapi_ir::{
        Attribute, EnumVariantKind, Metadata, RUNTIME_ITEMS, SchemaItem,
        rustify_field, rustify_name, rustify_variant,
    },
    unified::{RegistryError, UnifiedRegistry, UnifiedType, Version},
};
use heck::{ToPascalCase, ToSnakeCase};
use std::collections::{BTreeMap, HashMap};

/// The names that a command's enum of return types takes where
/// `{Command}Returns` is held by another symbol: `(protocol, command wire
/// name, enum name)`. No pinned release needs one, so the table is empty; a
/// [`RegistryError::ReturnsNameTaken`] asks for an entry.
const RETURNS_ENUM_NAMES: &[(&str, &str, &str)] = &[];

/// The modules beside the generated items of a protocol, in the type
/// namespace: QAPI names, in PascalCase, never spell them.
const MODULES: &[&str] = &["commands", "events", "metadata"];

/// How the generator makes a Rust name, with its escape. The patterns are
/// documented in `docs/codegen.md`, "Rust names".
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Pattern {
    /// A QAPI name as `rustify_name`, `rustify_field` or `rustify_variant`
    /// spell it.
    Schema,
    /// A QAPI name escaped: a reserved type name with the prefix `Qapi`,
    /// a variant `Self` as `QapiSelf`, a keyword field as a raw `r#`
    /// identifier or, for `self`, `super` and `crate`, with a trailing `_`,
    /// a leading digit after `_`, non-ASCII as `_u{hex}_`, and an empty
    /// name as `QapiUnnamed` or `qapi_unnamed`.
    Escaped,
    /// `{Name}Generated`, `{Name}Generated2`, ...: a later run of a type,
    /// command or event, or a claim whose plain name an earlier release's
    /// symbol holds.
    Generated,
    /// `{Union}Branch`: the enum of a union's branches.
    Branch,
    /// `{Union}Kind`: the implicit tag enum of a simple union, QEMU's name.
    ImplicitTag,
    /// `{Command}Command`: the request type of a command named like a type.
    Command,
    /// `{Event}Event`: an event's data.
    Event,
    /// `{Owner}{Member}`, then `Inline`, `Inline2`, ...: an anonymous
    /// struct of an old schema.
    Inline,
    /// `{Command}Returns`, or the name of the naming table: the enum of a
    /// command's return types.
    Returns,
    /// `V{major}_{minor}_{patch}`: a variant of a `Returns` enum, by the
    /// release that introduced its type.
    ReturnsVariant,
    /// `{member}_legacy`, `{member}_legacy2`, ... and
    /// `{member}_legacy_{major}_{minor}`, or `{Variant}Legacy...`: an earlier
    /// shape or wire spelling of a member.
    Legacy,
    /// `{member}_alt`, `{member}_alt2`, ... or `{Variant}Alt...`: a member
    /// whose Rust name another member of the same release has.
    Alt,
    /// A member that the lowering adds: a simple union's tag `r_type`, the
    /// flattened branches of a union and the catch-all `arguments` of a
    /// `gen: false` command.
    Lowering,
    /// A module or a root item of `qrab-qapi`.
    Reserved,
    /// A static of `metadata.rs`: `STRUCT_{Type}`, `FIELDS_{Type}`,
    /// `ENUM_{Type}`, `VARIANTS_{Type}` and the tables.
    Metadata,
}

/// A Rust name of the generated code.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    /// The namespace: `items`, `fields of {Type}`, `variants of {Type}` or
    /// `metadata`.
    pub namespace: String,
    /// The Rust name.
    pub name: String,
    /// How the generator made it.
    pub pattern: Pattern,
    /// What it stands for, as an error names it.
    pub source: String,
}

/// The value of `key = "..."` among `meta`'s attributes.
fn item<'a>(meta: &'a Metadata, key: &str) -> Option<&'a str> {
    meta.attributes.iter().find_map(|attr| match attr {
        Attribute::Item { name, value } if name == key => Some(value.as_str()),
        _ => None,
    })
}

/// A merged type's metadata and its QAPI name: the IR keeps the schema's
/// name where the plan gave the type the plain Rust spelling of it, and a
/// `name` attribute where it gave another.
fn meta(value: &UnifiedType) -> (&Metadata, &str) {
    let (meta, name) = match value {
        UnifiedType::Enum(value) => (&value.ir.meta, &value.ir.name),
        UnifiedType::Struct(value) => (&value.ir.meta, &value.ir.name),
    };
    (meta, item(meta, "name").unwrap_or(name))
}

/// The QAPI name of a merged type, command or event, and the kind of
/// definition, for a message: `struct 'Foo'`.
pub(crate) fn describe(key: &str, value: &UnifiedType) -> String {
    let (meta, wire) = meta(value);
    match meta.schema_item {
        Some(kind) => format!("{} '{wire}'", kind.as_str()),
        None if meta.attributes.contains(&Attribute::new("union_branch")) => {
            format!("the branch enum {key}")
        }
        None => format!("the generated type {key}"),
    }
}

/// The pattern of a merged type's Rust name.
fn type_pattern(key: &str, value: &UnifiedType) -> Pattern {
    let (meta, wire) = meta(value);
    match meta.schema_item {
        None => match value {
            UnifiedType::Enum(_)
                if meta
                    .attributes
                    .contains(&Attribute::new("union_branch")) =>
            {
                Pattern::Branch
            }
            UnifiedType::Enum(_) => Pattern::ImplicitTag,
            UnifiedType::Struct(_) => Pattern::Inline,
        },
        Some(SchemaItem::Event) => {
            if rustify_name(&format!("{wire}Event")) == key {
                Pattern::Event
            } else {
                Pattern::Generated
            }
        }
        Some(item) => {
            if rustify_name(wire) != key {
                if item == SchemaItem::Command && key.ends_with("Command") {
                    Pattern::Command
                } else {
                    Pattern::Generated
                }
            } else if wire.to_pascal_case() != key {
                Pattern::Escaped
            } else {
                Pattern::Schema
            }
        }
    }
}

/// The pattern of a member's Rust name: its IR name is the QAPI one unless
/// the merge or the member planning gave it another.
fn member_pattern(
    ir_name: &str,
    ident: &str,
    meta: &Metadata,
    plain: impl Fn(&str) -> String,
) -> Pattern {
    let wire = item(meta, "name").unwrap_or(ir_name);
    let legacy = ir_name.rfind("_legacy").max(ir_name.rfind("Legacy"));
    let alt = ir_name.rfind("_alt").max(ir_name.rfind("Alt"));
    if ir_name != wire && (legacy.is_some() || alt.is_some()) {
        if legacy > alt { Pattern::Legacy } else { Pattern::Alt }
    } else if meta.attributes.contains(&Attribute::new("flatten"))
        || (ir_name == "r-type" && wire == "type")
    {
        Pattern::Lowering
    } else if plain(wire) != ident {
        Pattern::Escaped
    } else {
        Pattern::Schema
    }
}

/// The names of one namespace, each with the first entry that took it.
#[derive(Default)]
struct Namespace {
    taken: HashMap<String, usize>,
}

/// The inventory being built, and the protocol, for its errors.
struct Inventory {
    protocol: &'static str,
    names: Vec<Name>,
    namespaces: BTreeMap<String, Namespace>,
}

impl Inventory {
    fn add(
        &mut self,
        namespace: &str,
        name: &str,
        pattern: Pattern,
        source: impl Into<String>,
    ) -> Result<(), RegistryError> {
        let source = source.into();
        let space = self.namespaces.entry(namespace.to_owned()).or_default();
        if let Some(&first) = space.taken.get(name) {
            return Err(RegistryError::NameCollision {
                protocol: self.protocol,
                namespace: namespace.to_owned(),
                name: name.to_owned(),
                first: self.names[first].source.clone(),
                second: source,
            });
        }
        space.taken.insert(name.to_owned(), self.names.len());
        self.names.push(Name {
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            pattern,
            source,
        });
        Ok(())
    }
}

/// The name of a command's enum of return types: the naming table's, or
/// `{Command}Returns`.
fn returns_enum_name(
    table: &[(&str, &str, &str)],
    protocol: &str,
    wire: &str,
    command: &str,
) -> String {
    table
        .iter()
        .find(|(table_protocol, table_wire, _)| {
            *table_protocol == protocol && *table_wire == wire
        })
        .map_or_else(
            || format!("{command}Returns"),
            |(.., name)| (*name).to_owned(),
        )
}

/// Name the enum of the return types of every command whose return type
/// changes across the merged releases, refusing a name that a type, a
/// command, an event, a reserved name or the enum of another command holds.
/// `tags` is the first merged tag of each release, which the error names.
pub(crate) fn plan_returns_enums(
    registry: &mut UnifiedRegistry,
    protocol: &'static str,
    tags: &BTreeMap<Version, String>,
) -> Result<(), RegistryError> {
    plan_returns_enums_with(registry, protocol, tags, RETURNS_ENUM_NAMES)
}

pub(crate) fn plan_returns_enums_with(
    registry: &mut UnifiedRegistry,
    protocol: &'static str,
    tags: &BTreeMap<Version, String>,
    table: &[(&str, &str, &str)],
) -> Result<(), RegistryError> {
    let mut planned: BTreeMap<String, String> = BTreeMap::new();
    let mut names = BTreeMap::new();
    for (key, value) in &registry.types {
        let UnifiedType::Struct(command) = value else {
            continue;
        };
        if command.returns_history.len() < 2 {
            continue;
        }
        let wire = item(&command.ir.meta, "name").unwrap_or(key);
        let name = returns_enum_name(table, protocol, wire, key);
        let holder = registry
            .types
            .get(&name)
            .map(|holder| describe(&name, holder))
            .or_else(|| {
                reserved(protocol)
                    .any(|(reserved, _)| reserved == name)
                    .then(|| format!("the reserved name {name}"))
            })
            .or_else(|| {
                planned.get(&name).map(|other| {
                    format!("the enum of return types of command '{other}'")
                })
            });
        if let Some(holder) = holder {
            let (changed, _) = command.returns_history[1];
            return Err(RegistryError::ReturnsNameTaken {
                protocol,
                command: wire.to_owned(),
                name,
                holder,
                tag: tags
                    .get(&changed)
                    .cloned()
                    .unwrap_or_else(|| crate::unified::version_str(changed)),
            });
        }
        planned.insert(name.clone(), wire.to_owned());
        names.insert(key.clone(), name);
    }
    for (key, name) in names {
        if let Some(UnifiedType::Struct(command)) = registry.types.get_mut(&key)
        {
            command.returns_enum = Some(name);
        }
    }
    Ok(())
}

/// The reserved names of a protocol's items, with what each is.
fn reserved(
    protocol: &str,
) -> impl Iterator<Item = (&'static str, String)> + '_ {
    MODULES
        .iter()
        .map(|module| (*module, format!("the module {module}")))
        .chain(
            RUNTIME_ITEMS
                .iter()
                .filter(move |_| protocol == "QMP")
                .map(|item| (*item, format!("the root item qapi::{item}"))),
        )
}

/// Every Rust name of the code that the registry of `protocol` generates,
/// in the order of the namespaces' declarations; two of one namespace are a
/// [`RegistryError::NameCollision`] naming both.
#[doc(hidden)]
pub fn inventory(
    registry: &UnifiedRegistry,
    protocol: &'static str,
) -> Result<Vec<Name>, RegistryError> {
    let mut inventory =
        Inventory { protocol, names: Vec::new(), namespaces: BTreeMap::new() };
    for (name, source) in reserved(protocol) {
        inventory.add("items", name, Pattern::Reserved, source)?;
    }
    for (key, value) in &registry.types {
        inventory.add(
            "items",
            key,
            type_pattern(key, value),
            describe(key, value),
        )?;
    }
    for (key, value) in &registry.types {
        match value {
            UnifiedType::Struct(value) => {
                let namespace = format!("fields of {key}");
                for field in &value.ir.fields {
                    let ident = rustify_field(&field.name);
                    let pattern = member_pattern(
                        &field.name,
                        &ident,
                        &field.meta,
                        |wire| wire.to_snake_case(),
                    );
                    let wire = item(&field.meta, "name").unwrap_or(&field.name);
                    let source = match value.field_versions.get(&field.name) {
                        Some(range) => format!(
                            "member '{wire}' from {}{}",
                            crate::unified::version_str(range.since),
                            range
                                .until
                                .map(|until| format!(
                                    " until {}",
                                    crate::unified::version_str(until)
                                ))
                                .unwrap_or_default()
                        ),
                        None => format!("member '{wire}'"),
                    };
                    inventory.add(&namespace, &ident, pattern, source)?;
                }
                if let Some(name) = &value.returns_enum {
                    let wire = item(&value.ir.meta, "name").unwrap_or(key);
                    inventory.add(
                        "items",
                        name,
                        Pattern::Returns,
                        format!("the enum of return types of command '{wire}'"),
                    )?;
                    let namespace = format!("variants of {name}");
                    for (added, returns) in &value.returns_history {
                        let variant =
                            format!("V{}_{}_{}", added.0, added.1, added.2);
                        inventory.add(
                            &namespace,
                            &variant,
                            Pattern::ReturnsVariant,
                            format!(
                                "the return type {returns} from {}",
                                crate::unified::version_str(*added)
                            ),
                        )?;
                    }
                }
            }
            UnifiedType::Enum(value) => {
                let namespace = format!("variants of {key}");
                for variant in &value.ir.variants {
                    let ident = rustify_variant(&variant.name);
                    let pattern = member_pattern(
                        &variant.name,
                        &ident,
                        &variant.meta,
                        |wire| wire.to_pascal_case(),
                    );
                    let wire =
                        item(&variant.meta, "name").unwrap_or(&variant.name);
                    let payload = match &variant.kind {
                        EnumVariantKind::Tuple(ty) => format!(" ({ty})"),
                        EnumVariantKind::Unit => String::new(),
                    };
                    inventory.add(
                        &namespace,
                        &ident,
                        pattern,
                        format!("value '{wire}'{payload}"),
                    )?;
                }
            }
        }
    }
    for table in ["TYPES", "TYPE_REFS", "COMMANDS", "STRUCTS", "ENUMS"] {
        inventory.add(
            "metadata",
            table,
            Pattern::Metadata,
            format!("the table {table}"),
        )?;
    }
    for (key, value) in &registry.types {
        let prefixes: &[&str] = match value {
            UnifiedType::Struct(_) => &["STRUCT", "FIELDS"],
            UnifiedType::Enum(_) => &["ENUM", "VARIANTS"],
        };
        for prefix in prefixes {
            inventory.add(
                "metadata",
                &format!("{prefix}_{key}"),
                Pattern::Metadata,
                format!("the {} of {key}", prefix.to_lowercase()),
            )?;
        }
    }
    Ok(inventory.names)
}
