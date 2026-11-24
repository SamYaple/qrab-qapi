//! The Rust intermediate representation (IR) of QAPI definitions: structs
//! and enums with their fields and variants, and the attributes that keep
//! their wire names and schema metadata.
//!
//! Items, fields and variants are named as the schema names them. The
//! `rustify_*` functions derive the Rust identifiers from those names, and a
//! `name` attribute keeps a wire name that differs from its identifier.

use heck::{ToPascalCase, ToSnakeCase};

mod process_qapi;
pub use process_qapi::*;

mod rust_enum;
mod rust_struct;
pub use rust_enum::*;
pub use rust_struct::*;

/// The Rust type of a QAPI builtin type, `None` for any other name.
fn builtin_type(qapi_type: &str) -> Option<&'static str> {
    Some(match qapi_type {
        "str" => "String",
        "number" => "f64",
        "int" | "int64" => "i64",
        "int8" => "i8",
        "int16" => "i16",
        "int32" => "i32",
        "uint8" => "u8",
        "uint16" => "u16",
        "uint32" => "u32",
        "uint64" | "size" => "u64",
        "bool" => "bool",
        "any" => "serde_json::Value",
        "null" => "()",
        // The untyped values of schemas before QEMU 2.5. They are lowercase;
        // `Visitor` and `Dict` may be types that a schema defines.
        "visitor" | "dict" | "**" => "serde_json::Value",
        _ => return None,
    })
}

/// The type of the `arguments` member that the lowering gives a gen:false
/// command, for the arguments that its schema does not declare. It is a
/// Rust type, not a QAPI name.
const ARGUMENTS_TYPE: &str = "HashMap<String, serde_json::Value>";

/// The Rust type that a type name of the IR stands for: the Rust type of a
/// builtin, or the [`rustify_name`] of a defined or synthesized type.
pub(crate) fn rustify_type(qapi_type: &str) -> String {
    if let Some(rust_type) = builtin_type(qapi_type) {
        return rust_type.into();
    }
    // The `arguments` member of a gen:false command has a Rust type already.
    if qapi_type == ARGUMENTS_TYPE {
        return qapi_type.into();
    }
    rustify_name(qapi_type)
}

fn fix_leading_digit(input: &str) -> String {
    if input
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("_{}", input)
    } else {
        input.to_string()
    }
}

// QEMU's generator checks names only from 2.4, so a name of an older schema
// can contain Unicode outside Rust's identifier grammar. Escaping it as ASCII
// also keeps Rust's normalization from merging two distinct names.
fn ascii_identifier(input: &str) -> std::borrow::Cow<'_, str> {
    if input.is_ascii() {
        return input.into();
    }
    let mut name = String::with_capacity(input.len());
    for c in input.chars() {
        if c.is_ascii() {
            name.push(c);
        } else {
            use std::fmt::Write;
            write!(name, "_u{:x}_", c as u32).unwrap();
        }
    }
    name.into()
}

fn pascal_identifier(input: &str) -> String {
    let name = ascii_identifier(input).to_pascal_case();
    // Escapes can introduce an acronym boundary (E + U301); settle its case
    // before the resulting symbol is used again as a generated type reference.
    let name = if input.is_ascii() {
        name
    } else {
        name.to_pascal_case()
    };
    fix_leading_digit(&name)
}

/// The Rust type name of a QAPI name, in PascalCase and ASCII. A name that
/// would shadow `Self`, `HashMap` or a type, trait or variant of the Rust
/// prelude gets the prefix `Qapi`.
///
/// ```
/// use qapi_codegen::qapi_ir::rustify_name;
///
/// assert_eq!(rustify_name("query-status"), "QueryStatus");
/// assert_eq!(rustify_name("String"), "QapiString");
/// ```
pub fn rustify_name(input: &str) -> String {
    let name = pascal_identifier(input);
    match name.as_str() {
        "" => "QapiUnnamed".into(),
        "Self" | "String" | "Option" | "Vec" | "Box" | "Result" | "HashMap" | "Send" | "Sync"
        | "Sized" | "Drop" | "Fn" | "FnMut" | "FnOnce" | "Some" | "None" | "Ok" | "Err" => {
            format!("Qapi{name}")
        }
        _ => name,
    }
}

/// The Rust variant name of a QAPI name. Variants have a namespace of their
/// own, so only `Self` is renamed.
pub(crate) fn rustify_variant(input: &str) -> String {
    let name = pascal_identifier(input);
    if name.is_empty() {
        "QapiUnnamed".into()
    } else if name == "Self" {
        "QapiSelf".into()
    } else {
        name
    }
}

/// The Rust field name of a QAPI name, in snake_case and ASCII. A keyword
/// other than `self`, `super` and `crate` stays as it is.
fn rustify_field_name(input: &str) -> String {
    let name = fix_leading_digit(&ascii_identifier(input).to_snake_case());
    match name.as_str() {
        "" => "qapi_unnamed".into(),
        "self" | "super" | "crate" => format!("{name}_"),
        _ => name,
    }
}

/// The Rust field identifier of a QAPI name: its [`rustify_field_name`], as
/// a raw identifier where that is a keyword of Rust 2024.
pub(crate) fn rustify_field(input: &str) -> String {
    /// The strict, reserved and edition 2024 keywords that a snake_case name
    /// can spell and that can be raw: `self`, `super` and `crate` cannot, and
    /// [`rustify_field_name`] renames them, as it does an empty name.
    const KEYWORDS: &[&str] = &[
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do",
        "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in",
        "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
        "return", "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe",
        "unsized", "use", "virtual", "where", "while", "yield",
    ];
    let snake = rustify_field_name(input);
    if KEYWORDS.contains(&snake.as_str()) {
        format!("r#{snake}")
    } else {
        snake
    }
}

/// An item of an IR item's, field's or variant's metadata.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Attribute {
    /// A key with a value, such as `name = "query-status"`.
    Item { name: String, value: String },
    /// Keys that belong together: a feature and its own condition.
    List(Vec<Attribute>),
    /// A key alone, such as `command`.
    Unit(String),
}

impl Attribute {
    /// A key alone.
    pub fn new<N: ToString>(name: N) -> Self {
        Self::Unit(name.to_string())
    }

    /// A key with a value.
    pub fn with_value<N: ToString, V: ToString>(name: N, value: V) -> Self {
        Self::Item {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    /// Keys that belong together, each with its value if it has one.
    pub fn with_values<N: ToString, V: ToString>(values: Vec<(N, Option<V>)>) -> Self {
        let attrs = values
            .iter()
            .map(|i| {
                let name = i.0.to_string();
                if let Some(value) = &i.1 {
                    Self::Item {
                        name,
                        value: value.to_string(),
                    }
                } else {
                    Self::Unit(name)
                }
            })
            .collect();
        Self::List(attrs)
    }
}

/// The kind of QAPI definition that an item was lowered from.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SchemaItem {
    /// A command, lowered to the struct of its arguments.
    Command,
    /// An event, lowered to the struct of its data.
    Event,
    Struct,
    Enum,
    /// A union, lowered to a struct.
    Union,
    /// An alternate or legacy alternate, lowered to an enum.
    Alternate,
}

/// What the IR keeps about an item, field or variant besides its name and type.
#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct Metadata {
    /// Documentation that the lowering writes itself, not schema text.
    pub doc: Option<String>,
    /// The schema's own documentation of the item. It is QEMU's text, under
    /// GPL-2.0-or-later, and kept apart from `doc`.
    pub schema_doc: Option<String>,
    /// The kind of definition, for items that the schema declares; types
    /// that the lowering synthesizes have none.
    pub schema_item: Option<SchemaItem>,
    /// The lowered facts, in schema order:
    ///
    /// - `name = "..."`: the wire name, where its Rust identifier differs
    ///   from it (always, for events);
    /// - `condition = "..."`: the build condition, the schema's `if`;
    /// - `added = "X.Y.Z"`: the release of a `Since:` in the documentation;
    /// - a [`List`](Attribute::List) of `feature = "..."` and, for a feature
    ///   with a condition of its own, `condition = "..."`;
    /// - on items: `command`, `returns = "..."` (a Rust type), `allow_oob`,
    ///   `allow_preconfig`, `no_success_response`, `event`, `simple_union`
    ///   and `union_branch`;
    /// - on fields: `discriminator`, `union` and `flatten`.
    pub attributes: Vec<Attribute>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_unicode_names_have_stable_ascii_identifiers() {
        let identifier = |name: &str| {
            name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        };
        for raw in ["١Thing", "²", "É", "E\u{301}", "𐀀", "Ⅸ", "汉字", "Καλημέρα"]
        {
            let name = rustify_name(raw);
            assert!(identifier(&name), "{raw}: {name}");
            assert_eq!(rustify_name(&name), name);
            let field_name = rustify_field_name(raw);
            assert!(identifier(&field_name), "{raw}: {field_name}");
            assert_eq!(rustify_field_name(&field_name), field_name);
            let variant = rustify_variant(raw);
            assert!(identifier(&variant), "{raw}: {variant}");
            assert_eq!(rustify_variant(&variant), variant);
        }
        // Without Unicode normalization, the two spellings of É stay apart.
        assert_ne!(rustify_name("É"), rustify_name("E\u{301}"));
        assert_ne!(rustify_field_name("É"), rustify_field_name("E\u{301}"));
        assert_ne!(rustify_variant("É"), rustify_variant("E\u{301}"));
    }

    #[test]
    fn field_keywords_become_raw_identifiers_or_get_an_underscore() {
        for (qapi_name, field) in [
            ("type", "r#type"),
            ("gen", "r#gen"),
            ("async", "r#async"),
            ("try", "r#try"),
            ("Box", "r#box"),
            ("self", "self_"),
            ("Self", "self_"),
            ("super", "super_"),
            ("crate", "crate_"),
            ("_", "qapi_unnamed"),
            ("union", "union"),
            ("raw", "raw"),
            ("type-id", "type_id"),
        ] {
            assert_eq!(rustify_field(qapi_name), field, "{qapi_name}");
        }
    }

    #[test]
    fn builtin_types_map_to_rust_types_and_other_names_to_rust_names() {
        for (qapi_type, rust_type) in [
            ("str", "String"),
            ("number", "f64"),
            ("int", "i64"),
            ("int8", "i8"),
            ("uint16", "u16"),
            ("size", "u64"),
            ("bool", "bool"),
            ("any", "serde_json::Value"),
            ("null", "()"),
            ("dict", "serde_json::Value"),
            ("Dict", "Dict"),
            ("query-status", "QueryStatus"),
            ("1Thing", "_1thing"),
            (ARGUMENTS_TYPE, ARGUMENTS_TYPE),
        ] {
            assert_eq!(rustify_type(qapi_type), rust_type, "{qapi_type}");
        }
    }
}
