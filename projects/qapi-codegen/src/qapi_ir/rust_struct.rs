use super::Metadata;

/// A field of a [`Struct`].
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct StructField {
    /// The QAPI name of the member, or for a member that the lowering
    /// synthesizes, a name of its own: `r-type` for a simple union's
    /// discriminator, and the flattened `u` for a union's branch and
    /// `arguments` for the undeclared arguments of a gen:false command. A
    /// declared member may have the name of a flattened one.
    pub name: String,
    /// The field's documentation and attributes.
    pub meta: Metadata,
    /// The QAPI name of the member's type, a builtin or a definition, or the
    /// Rust name of a type that the lowering synthesizes. The `arguments`
    /// member of a gen:false command has a Rust type,
    /// `serde_json::Map<String, serde_json::Value>`.
    pub r#type: String,
    /// The member may be absent (`'*name'`).
    pub optional: bool,
    /// The member holds a list of its type (`['T']`).
    pub array: bool,
}

/// A struct: a QAPI struct, union, command's arguments or event's data, or
/// a struct that the lowering extracts from an anonymous member type.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Struct {
    /// The QAPI name of the definition (for an event, `<name>Event`), or
    /// for an extracted struct, a Rust name (see [`process_struct`]).
    ///
    /// [`process_struct`]: super::process_struct
    pub name: String,
    /// The struct's documentation and attributes.
    pub meta: Metadata,
    /// The fields in schema order, those of a base first.
    pub fields: Vec<StructField>,
}
