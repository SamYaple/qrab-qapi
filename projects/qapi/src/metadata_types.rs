//! The types of the schema metadata tables, which `metadata` (QMP) and
//! `qga::metadata` (QGA) both re-export.
//!
//! The generated statics of those modules are the only values of these
//! types, so the fields are private to the crate and the methods are the
//! API. A type a table refers to is a reference into the `TYPE_REFS` of
//! the same module, so it always resolves against the right protocol's
//! table.

/// A QEMU release as `(major, minor, micro)`.
pub type Ver = (u32, u32, u32);

/// A run of releases: from [`since`](Self::since) up to, but not
/// including, [`until`](Self::until), or through the newest release that
/// the tables describe when `until` is `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Releases {
    pub(crate) since: Ver,
    pub(crate) until: Option<Ver>,
}

impl Releases {
    /// The first release of the run.
    pub fn since(&self) -> Ver {
        self.since
    }

    /// The first release after the run, `None` when the run continues
    /// through the newest release.
    pub fn until(&self) -> Option<Ver> {
        self.until
    }

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
#[non_exhaustive]
pub struct Table<T: 'static> {
    pub(crate) entries: &'static [(&'static str, &'static T)],
}

impl<T> Table<T> {
    /// The entry for the Rust type `name`.
    pub fn get(&self, name: &str) -> Option<&'static T> {
        let index =
            self.entries.binary_search_by(|(key, _)| (*key).cmp(name)).ok()?;
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
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = (&'static str, &'static T)> {
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
#[non_exhaustive]
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
#[non_exhaustive]
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
///
/// Each module's `TYPE_REFS` holds every type once, so two references of a
/// module are equal exactly when they name the same type.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TypeRef {
    pub(crate) kind: TypeKind,
    pub(crate) name: &'static str,
    pub(crate) prim: Option<Prim>,
    pub(crate) element: Option<&'static TypeRef>,
}

impl TypeRef {
    /// What it refers to.
    pub fn kind(&self) -> TypeKind {
        self.kind
    }

    /// A primitive's QAPI builtin, such as `str`, `int` or `int64`, as the
    /// newest release with the field, variant or return that refers to it
    /// spells it (`QomSet.value`, typed `visitor` in QEMU 1.1 to 2.3, `**`
    /// in 2.4 and `any` from 2.5, refers to `any`); the key of an enum or
    /// struct in `ENUMS` or `STRUCTS`; `List<element>` for a list.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The builtin of a primitive, `None` for any other kind.
    pub fn prim(&self) -> Option<Prim> {
        self.prim
    }

    /// A list's element, an entry of the same `TYPE_REFS`, `None` for any
    /// other kind.
    pub fn element(&self) -> Option<&'static TypeRef> {
        self.element
    }
}

/// A field of a struct, union, command or event.
#[derive(Debug)]
#[non_exhaustive]
pub struct FieldMeta {
    pub(crate) name: &'static str,
    pub(crate) wire_name: &'static str,
    pub(crate) ty: &'static TypeRef,
    pub(crate) optional: bool,
    pub(crate) discriminator: &'static [Releases],
    pub(crate) boxed: bool,
    pub(crate) since: Ver,
    pub(crate) until: Option<Ver>,
    pub(crate) deprecated: &'static [Releases],
    pub(crate) unstable: &'static [Releases],
}

impl FieldMeta {
    /// The Rust field name as the struct declares it, a raw identifier with
    /// its `r#` prefix, such as `r#in`.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The member's name on the wire.
    pub fn wire_name(&self) -> &'static str {
        self.wire_name
    }

    /// Its type, an entry of the same module's `TYPE_REFS`.
    pub fn type_ref(&self) -> &'static TypeRef {
        self.ty
    }

    /// Whether the active wire field may be absent, independently of Rust's
    /// version-merged `Option`.
    pub fn is_optional(&self) -> bool {
        self.optional
    }

    /// The releases in which it is its union's discriminator: a union may
    /// change which member discriminates while both exist.
    pub fn discriminator(&self) -> &'static [Releases] {
        self.discriminator
    }

    /// Whether the Rust field holds its value in a `Box`: a value of its
    /// type would otherwise contain the struct itself, other than through
    /// an array or another boxed field.
    pub fn is_boxed(&self) -> bool {
        self.boxed
    }

    /// The first release with the field.
    pub fn since(&self) -> Ver {
        self.since
    }

    /// The first release after [`since`](Self::since) without it, `None`
    /// while the newest has it.
    pub fn until(&self) -> Option<Ver> {
        self.until
    }

    /// The releases that deprecate the field.
    pub fn deprecated(&self) -> &'static [Releases] {
        self.deprecated
    }

    /// The releases that mark the field unstable.
    pub fn unstable(&self) -> &'static [Releases] {
        self.unstable
    }

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
#[non_exhaustive]
pub struct EnumVariantMeta {
    pub(crate) name: &'static str,
    pub(crate) wire_name: &'static str,
    pub(crate) payload_struct: Option<&'static str>,
    pub(crate) payload_type: Option<&'static TypeRef>,
    pub(crate) since: Ver,
    pub(crate) until: Option<Ver>,
    pub(crate) deprecated: &'static [Releases],
    pub(crate) unstable: &'static [Releases],
}

impl EnumVariantMeta {
    /// The Rust variant name.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The enum value, alternate branch or union tag on the wire.
    pub fn wire_name(&self) -> &'static str {
        self.wire_name
    }

    /// The key in `STRUCTS` of a direct struct payload, if any; scalar,
    /// enum and list payloads have only a
    /// [`payload_type`](Self::payload_type).
    pub fn payload_struct(&self) -> Option<&'static str> {
        self.payload_struct
    }

    /// The type of its payload, an entry of the same module's `TYPE_REFS`,
    /// also for a null payload; `None` for a unit variant.
    pub fn payload_type(&self) -> Option<&'static TypeRef> {
        self.payload_type
    }

    /// The first release with the variant.
    pub fn since(&self) -> Ver {
        self.since
    }

    /// The first release after [`since`](Self::since) without it, `None`
    /// while the newest has it.
    pub fn until(&self) -> Option<Ver> {
        self.until
    }

    /// The releases that deprecate the variant.
    pub fn deprecated(&self) -> &'static [Releases] {
        self.deprecated
    }

    /// The releases that mark the variant unstable.
    pub fn unstable(&self) -> &'static [Releases] {
        self.unstable
    }

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
#[non_exhaustive]
pub enum EnumKind {
    /// A QAPI enum: variants without payloads.
    Unit,
    /// A QAPI alternate: a value of one of the variants' types.
    Alternate,
    /// The branches of a union, the enum that its
    /// [`StructMeta::branch_enum`] names: a variant per tag, holding that
    /// branch's members. The union's [`StructMeta::simple_union`] gives the
    /// releases in which these travel as `"data"` beside the `"type"` tag
    /// (a simple union), and they are flat members of the union in the
    /// others.
    UnionBranch,
}

/// An enum, alternate or union branch enum.
#[derive(Debug)]
#[non_exhaustive]
pub struct EnumMeta {
    pub(crate) name: &'static str,
    pub(crate) wire_name: &'static str,
    pub(crate) variants: &'static [EnumVariantMeta],
    pub(crate) kind: EnumKind,
    pub(crate) since: Ver,
    pub(crate) until: Option<Ver>,
    pub(crate) deprecated: &'static [Releases],
    pub(crate) unstable: &'static [Releases],
}

impl EnumMeta {
    /// Rust type name, also used as the `ENUMS` lookup key.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The QAPI name; a type that the generator synthesizes, such as a
    /// union's branch enum, has its Rust name.
    pub fn wire_name(&self) -> &'static str {
        self.wire_name
    }

    /// The variants of every release.
    pub fn variants(&self) -> &'static [EnumVariantMeta] {
        self.variants
    }

    /// What kind of enum it is.
    pub fn kind(&self) -> EnumKind {
        self.kind
    }

    /// The first release with the type.
    pub fn since(&self) -> Ver {
        self.since
    }

    /// The first release after [`since`](Self::since) without it, `None`
    /// while the newest has it.
    pub fn until(&self) -> Option<Ver> {
        self.until
    }

    /// The releases that deprecate the type.
    pub fn deprecated(&self) -> &'static [Releases] {
        self.deprecated
    }

    /// The releases that mark the type unstable.
    pub fn unstable(&self) -> &'static [Releases] {
        self.unstable
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

/// A struct, union, or the arguments of a command or the data of an event.
#[derive(Debug)]
#[non_exhaustive]
pub struct StructMeta {
    pub(crate) name: &'static str,
    pub(crate) wire_name: &'static str,
    pub(crate) fields: &'static [FieldMeta],
    pub(crate) is_union: bool,
    pub(crate) discriminator_field: Option<&'static str>,
    pub(crate) branch_enum: Option<&'static str>,
    pub(crate) simple_union: &'static [Releases],
    pub(crate) free_form: &'static [Releases],
    pub(crate) since: Ver,
    pub(crate) until: Option<Ver>,
    pub(crate) deprecated: &'static [Releases],
    pub(crate) unstable: &'static [Releases],
}

impl StructMeta {
    /// Rust type name, also used as the `STRUCTS` lookup key.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The QAPI name, the wire name of a command or event, or for a type
    /// that the generator synthesizes, such as an anonymous struct, its Rust
    /// name.
    pub fn wire_name(&self) -> &'static str {
        self.wire_name
    }

    /// The fields that have their own wire members, in every release.
    pub fn fields(&self) -> &'static [FieldMeta] {
        self.fields
    }

    /// Whether it is a union, with a discriminator and branches.
    pub fn is_union(&self) -> bool {
        self.is_union
    }

    /// Rust field name of the newest observed discriminator representation;
    /// [`discriminator`](Self::discriminator) gives the field of one release.
    pub fn discriminator_field(&self) -> Option<&'static str> {
        self.discriminator_field
    }

    /// The key in `ENUMS` of the newest branch representation; earlier
    /// branch enums retain their own metadata.
    pub fn branch_enum(&self) -> Option<&'static str> {
        self.branch_enum
    }

    /// The releases in which a union is a simple union, with its branch in
    /// `"data"`.
    pub fn simple_union(&self) -> &'static [Releases] {
        self.simple_union
    }

    /// The releases in which a command takes arguments that its schema does
    /// not declare (`gen: false`), besides those of
    /// [`fields`](Self::fields).
    pub fn free_form(&self) -> &'static [Releases] {
        self.free_form
    }

    /// The first release with the type.
    pub fn since(&self) -> Ver {
        self.since
    }

    /// The first release after [`since`](Self::since) without it, `None`
    /// while the newest has it.
    pub fn until(&self) -> Option<Ver> {
        self.until
    }

    /// The releases that deprecate the type: the event, or for a command's
    /// arguments the command.
    pub fn deprecated(&self) -> &'static [Releases] {
        self.deprecated
    }

    /// The releases that mark the type, event or command unstable.
    pub fn unstable(&self) -> &'static [Releases] {
        self.unstable
    }

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
#[non_exhaustive]
pub struct CommandMeta {
    pub(crate) name: &'static str,
    pub(crate) wire_name: &'static str,
    pub(crate) args: &'static StructMeta,
    pub(crate) returns: Option<&'static TypeRef>,
    pub(crate) allow_oob: &'static [Releases],
    pub(crate) allow_preconfig: &'static [Releases],
    pub(crate) success_response: &'static [Releases],
    pub(crate) since: Ver,
    pub(crate) until: Option<Ver>,
    pub(crate) deprecated: &'static [Releases],
    pub(crate) unstable: &'static [Releases],
}

impl CommandMeta {
    /// The Rust name of the command's struct.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The command's name on the wire.
    pub fn wire_name(&self) -> &'static str {
        self.wire_name
    }

    /// The command's arguments, the entry of its struct in the same
    /// module's `STRUCTS`.
    pub fn args(&self) -> &'static StructMeta {
        self.args
    }

    /// What the newest release returns, an entry of the same module's
    /// `TYPE_REFS`; `None` for a command without a return type.
    pub fn returns(&self) -> Option<&'static TypeRef> {
        self.returns
    }

    /// The releases that allow out-of-band execution of the command.
    pub fn allow_oob(&self) -> &'static [Releases] {
        self.allow_oob
    }

    /// The releases that allow the command before the machine is configured.
    pub fn allow_preconfig(&self) -> &'static [Releases] {
        self.allow_preconfig
    }

    /// The releases that reply to the command when it succeeds.
    pub fn success_response(&self) -> &'static [Releases] {
        self.success_response
    }

    /// The first release with the command.
    pub fn since(&self) -> Ver {
        self.since
    }

    /// The first release after [`since`](Self::since) without it, `None`
    /// while the newest has it.
    pub fn until(&self) -> Option<Ver> {
        self.until
    }

    /// The releases that deprecate the command.
    pub fn deprecated(&self) -> &'static [Releases] {
        self.deprecated
    }

    /// The releases that mark the command unstable.
    pub fn unstable(&self) -> &'static [Releases] {
        self.unstable
    }

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
