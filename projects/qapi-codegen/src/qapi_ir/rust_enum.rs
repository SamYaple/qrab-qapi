use super::Metadata;

/// What a variant carries.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum EnumVariantKind {
    /// Nothing: an enum value, or a union branch without members.
    Unit,
    /// A value of a type, named as the `r#type` of a
    /// [`StructField`](super::StructField) is: a union branch or an
    /// alternative.
    Tuple(String),
}

/// A variant of an [`Enum`].
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct EnumVariant {
    /// The QAPI name of the enum value, branch or alternative.
    pub name: String,
    /// The variant's documentation and attributes.
    pub meta: Metadata,
    /// What the variant carries.
    pub kind: EnumVariantKind,
    /// The variant carries a list of its type (`['T']`).
    pub array: bool,
}

/// An enum: a QAPI enum or alternate, or the branches or discriminator
/// values of a union.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Enum {
    /// The QAPI name of the definition, or for an enum that the lowering
    /// synthesizes for a union, a Rust name (see [`process_union`]).
    ///
    /// [`process_union`]: super::process_union
    pub name: String,
    /// The enum's documentation and attributes.
    pub meta: Metadata,
    /// The values, branches or alternatives in schema order.
    pub variants: Vec<EnumVariant>,
}
