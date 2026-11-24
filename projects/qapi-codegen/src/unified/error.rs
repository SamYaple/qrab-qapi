//! Errors of checking release schemas and building registries from them.

use crate::qapi_ir::LowerError;
use crate::versions::TreeError;
use crate::{ParseError, QapiSchemaToken, ReadError};
use std::fmt;
use std::path::{Path, PathBuf};

/// A release schema that QEMU's own QAPI generator rejects, or that cannot be
/// lowered to the IR.
///
/// [`kind`](Self::kind) says what is wrong; [`file`](Self::file),
/// [`line`](Self::line), [`column`](Self::column),
/// [`definition`](Self::definition) and [`context`](Self::context) say where.
///
/// ```
/// use qapi_codegen::unified::{SchemaError, SchemaErrorKind};
///
/// let error = SchemaError::from(SchemaErrorKind::InvalidName { name: "q-x".into() });
/// assert_eq!(error.to_string(), "Invalid QAPI name 'q-x'");
/// assert_eq!(error.file(), None);
/// assert_eq!(error.line(), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError(Box<Located>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Located {
    file: Option<PathBuf>,
    position: Option<(usize, usize)>,
    definition: Option<String>,
    context: Vec<String>,
    kind: SchemaErrorKind,
}

impl SchemaError {
    /// What is wrong.
    pub fn kind(&self) -> &SchemaErrorKind {
        &self.0.kind
    }

    /// The schema file, when the failure belongs to one.
    pub fn file(&self) -> Option<&Path> {
        self.0.file.as_deref()
    }

    /// The 1-based line in [`file`](Self::file) that QEMU's generator names
    /// for the failure: the start of the definition, or the line of the
    /// documentation comment at fault.
    pub fn line(&self) -> Option<usize> {
        self.0.position.map(|(line, _)| line)
    }

    /// The 1-based column, in characters, on [`line`](Self::line).
    pub fn column(&self) -> Option<usize> {
        self.0.position.map(|(_, column)| column)
    }

    /// The definition the failure belongs to, as its kind and name, such as
    /// `struct 'Foo'`.
    pub fn definition(&self) -> Option<&str> {
        self.0.definition.as_deref()
    }

    /// The parts of the definition that lead to the failure, outermost first,
    /// such as `member 'foo'`.
    pub fn context(&self) -> &[String] {
        &self.0.context
    }

    /// Name the definition and file, unless inner ones are named already.
    pub(crate) fn located(mut self, definition: Option<String>, file: Option<&Path>) -> Self {
        if self.0.definition.is_none() {
            self.0.definition = definition;
        }
        if self.0.file.is_none() {
            self.0.file = file.map(Path::to_path_buf);
        }
        self
    }

    /// Name the line and column, unless an inner one is named already.
    pub(crate) fn at(mut self, position: Option<(usize, usize)>) -> Self {
        if self.0.position.is_none() {
            self.0.position = position;
        }
        self
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let position = match self.0.position {
            Some((line, column)) => format!(":{line}:{column}"),
            None => String::new(),
        };
        match (self.definition(), self.file()) {
            (Some(definition), Some(file)) => write!(
                f,
                "Invalid QAPI {definition} in {}{position}: ",
                file.display()
            )?,
            (Some(definition), None) => write!(f, "Invalid QAPI {definition}: ")?,
            (None, Some(file)) => write!(f, "Invalid QAPI schema {}{position}: ", file.display())?,
            (None, None) => {}
        }
        for part in self.context() {
            write!(f, "{part}: ")?;
        }
        match (self.kind(), self.0.position) {
            (SchemaErrorKind::Parse(error), Some(_)) => write!(f, "{}", error.fault()),
            (kind, _) => write!(f, "{kind}"),
        }
    }
}

impl std::error::Error for SchemaError {}

impl From<SchemaErrorKind> for SchemaError {
    fn from(kind: SchemaErrorKind) -> Self {
        Self(Box::new(Located {
            file: None,
            position: None,
            definition: None,
            context: Vec::new(),
            kind,
        }))
    }
}

impl From<LowerError> for SchemaError {
    /// What the IR cannot lower in the releases whose QEMU accepts it is
    /// [`SchemaErrorKind::Unsupported`].
    fn from(error: LowerError) -> Self {
        let version = crate::parser_version();
        let construct = match error {
            LowerError::AnonymousStruct { member, .. } if version < (2, 4, 0) => {
                Construct::AnonymousStruct { member }
            }
            LowerError::UnknownType { name, .. } if version < (2, 4, 0) => {
                Construct::UndefinedType { name }
            }
            LowerError::MissingDiscriminator { discriminator, .. } if version < (2, 0, 0) => {
                Construct::UndefinedDiscriminator { discriminator }
            }
            error => return SchemaErrorKind::Lower(error).into(),
        };
        SchemaErrorKind::Unsupported { construct }.into()
    }
}

/// What makes a release schema invalid.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaErrorKind {
    /// A schema file that does not parse under its release's grammar.
    #[error("{0}")]
    Parse(ParseError),
    /// A definition of a type that QEMU predefines in the release.
    #[error("Predefined QAPI type '{name}' cannot be redefined")]
    Predefined {
        /// The definition's name.
        name: String,
    },
    /// A name that the release's QAPI grammar does not allow.
    #[error("Invalid QAPI name '{name}'")]
    InvalidName {
        /// The name.
        name: String,
    },
    /// A definition of a name that the release defines already, in the same
    /// file or another.
    #[error("Conflicting QAPI definitions: '{name}' is already defined in {}", previous.display())]
    Duplicate {
        /// The name.
        name: String,
        /// The file of its first definition.
        previous: PathBuf,
    },
    /// A definition name with a suffix that the release reserves for types
    /// its generator names itself: `Kind` for the tag enums of simple unions
    /// (QEMU 2.4 to 6.1), `List` for arrays (from 2.5).
    #[error("QAPI definition '{name}' uses reserved {suffix} suffix")]
    ReservedSuffix {
        /// The definition's name.
        name: String,
        /// `Kind` or `List`.
        suffix: &'static str,
    },
    /// An array of arrays, from QEMU 4.2, which accepts `ElementList` names.
    #[error("Nested QAPI arrays are not permitted: '{name}'")]
    NestedArray {
        /// The array's `ElementList` name.
        name: String,
    },
    /// An object type that contains itself: a struct whose bases lead back
    /// to it or, from QEMU 8.1, a union among its own branches. The error
    /// names the first definition on the cycle.
    #[error("Object {name} contains itself")]
    InheritanceCycle {
        /// That definition's name.
        name: String,
    },
    /// A schema on which the release's own QAPI generator fails with an
    /// exception instead of reporting an error, which counts as refusing it.
    #[error("QEMU's QAPI generator fails on {crash}")]
    Crash {
        /// What it fails on.
        crash: Crash,
    },
    /// A definition that the IR cannot represent.
    #[error("{0}")]
    Lower(LowerError),
    /// A construct that the release's QEMU generator accepts but the IR
    /// cannot represent, so the schema is not invalid: the known limits of
    /// the IR, which no mirrored release reaches.
    #[error("{construct} cannot be represented")]
    Unsupported {
        /// What the IR cannot represent.
        construct: Construct,
    },
    /// A union without branches, from QEMU 2.6: until 4.1 one whose `data`
    /// is empty, from 4.2 one whose discriminator enum has no values either.
    #[error("Union has no branches")]
    NoBranches,
    /// A union branch that is not a value of the discriminator enum, from
    /// QEMU 2.0.
    #[error("Branch '{branch}' is not a value of discriminator enum {discriminator_type}")]
    UnknownBranch {
        /// The branch.
        branch: String,
        /// The discriminator's enum.
        discriminator_type: String,
    },
    /// A union discriminator whose type is not an enum, from QEMU 2.0.
    #[error("Discriminator '{discriminator}' has non-enum type {discriminator_type}")]
    DiscriminatorNotEnum {
        /// The discriminator member.
        discriminator: String,
        /// Its type.
        discriminator_type: String,
    },
    /// A key that the release does not know in a definition, or in one of
    /// its members, branches, enum values or features written as a
    /// dictionary, from QEMU 2.4: any key of no release, and those of later
    /// ones. `prefix` arrives in 2.5, `boxed` in 2.7, `allow-oob` in 2.12,
    /// `if` and `allow-preconfig` in 3.0, members, branches and enum values
    /// as dictionaries in 4.0, features of structs in 4.1, of commands in
    /// 4.2, of other definitions and members in 5.0, of enum values in 6.2,
    /// and `coroutine` in 5.2.
    #[error("Unknown key '{key}'")]
    UnknownKey {
        /// The key.
        key: String,
    },
    /// A key that a definition, or a member, branch, enum value or feature
    /// written as a dictionary, must have, from QEMU 2.4: the `data` of an
    /// enum, struct, union or alternate (from 6.2 a union's `base` and
    /// `discriminator` too), the `type` of a member or branch, the `name`
    /// of an enum value or feature.
    #[error("Key '{key}' is missing")]
    MissingKey {
        /// The key.
        key: String,
    },
    /// An expression with the keys of two kinds of definition, such as
    /// `enum` and `struct`, from QEMU 6.2; before, the kind that QEMU looks
    /// for first has the other key as an unknown one.
    #[error(
        "Expression must have exactly one key 'enum', 'struct', 'union', 'alternate', 'command', 'event'"
    )]
    Metatypes,
    /// An expression without a key that names a kind of definition or
    /// directive, from QEMU 2.4 to 6.1; from 6.2 it is [`Self::Metatypes`].
    #[error("Expression is missing metatype")]
    MissingMetatype,
    /// An `if` condition without a configuration name: `''` or `[]` from
    /// QEMU 3.0 (conditions of enum values from 4.0, of members, branches
    /// and features from 4.2, of struct features in 4.1), a blank string
    /// from 4.2, until 6.1; from 6.2 `[]` as the operands of `all` or
    /// `any`, and a string that is no identifier is
    /// [`Self::InvalidCondition`].
    #[error("'if' condition {condition} {}", if condition == "[]" { "is useless" } else { "makes no sense" })]
    EmptyCondition {
        /// The condition in QAPI syntax, `[]` or a quoted string.
        condition: String,
    },
    /// A name that is not a string, from QEMU 2.4: a definition's, an enum
    /// value's, a feature's or a discriminator. From 2.4 to 3.1 an enum
    /// value written as an object, `{ 'name': ... }`, is none either.
    #[error("Name must be a string")]
    NameNotString,
    /// A name that starts with `*` where QEMU 2.4 to 4.1 allow no optional
    /// name: a definition's, an enum value's, a branch's, a feature's or a
    /// discriminator. From 4.2 it is [`Self::InvalidName`].
    #[error("Name '{name}' cannot be optional")]
    OptionalName {
        /// The name, as written.
        name: String,
    },
    /// A value of a shape that its place does not take, from QEMU 2.4.
    #[error("{expected}")]
    ValueShape {
        /// What the place takes.
        expected: Shape,
    },
    /// A condition's configuration name that is not an identifier in
    /// uppercase, from QEMU 6.2.
    #[error("'if' condition {condition} is not a valid identifier")]
    InvalidCondition {
        /// The name, quoted.
        condition: String,
    },
    /// A condition object with other than one of `all`, `any` and `not`,
    /// from QEMU 6.2.
    #[error("'if' condition has conflicting keys")]
    ConditionKeys,
    /// A flat union whose base is not a type name, in QEMU 2.4: an object
    /// written out in place, or none.
    #[error("Flat union must have a string base")]
    StringBase,
    /// A name that the release reserves for its generated C: a member `u`
    /// or `has_*` of an object written out in place, from QEMU 2.5, or the
    /// event `MAX` in 2.4 and 2.5.
    #[error("Name '{name}' is reserved")]
    ReservedName {
        /// The name, as written.
        name: String,
    },
    /// A name in a case that the release refuses: uppercase in member
    /// names from QEMU 2.6, and from 6.0 also `_` in member and command
    /// names, lowercase in event names and anything but CamelCase in type
    /// names, unless a pragma exempts it.
    #[error("Name '{name}' {rule}")]
    NameCase {
        /// The name, as written.
        name: String,
        /// The rule it breaks.
        rule: NameRule,
    },
    /// Two members, enum values or branches of one definition whose names
    /// clash: in their C names from QEMU 2.6, before that in their
    /// uppercase C enumeration constants.
    #[error("'{name}' collides with '{previous}'")]
    MemberClash {
        /// The member that clashes.
        name: String,
        /// The member it clashes with, or `(automatic)` for a name that the
        /// generator takes itself: `MAX` before 2.6, and the `KIND` and
        /// `TYPE` of a simple union in 2.5.
        previous: String,
    },
    /// A member of a struct or of a flat union's branch that a base, or a
    /// base of the base, declares as written, in QEMU 2.4 and 2.5.
    #[error("Member '{name}' clashes with base '{base}'")]
    BaseClash {
        /// The member.
        name: String,
        /// The base that declares it.
        base: String,
    },
    /// Two features of one definition or member whose C names clash, from
    /// QEMU 4.1.
    #[error("Feature '{name}' collides with '{previous}'")]
    FeatureClash {
        /// The feature that clashes.
        name: String,
        /// The feature it clashes with.
        previous: String,
    },
    /// A feature that the release gives a meaning of its own, which types
    /// may not have: `deprecated` from QEMU 5.0, `unstable` from 6.2.
    #[error("Feature '{name}' is not supported for types")]
    SpecialFeature {
        /// The feature.
        name: String,
    },
    /// The 65th distinct feature name of a schema, from QEMU 10.0; the
    /// error names the definition that first uses it.
    #[error("Maximum of 64 schema features is permitted")]
    TooManyFeatures,
    /// A reference to a type that the release does not define.
    #[error("Unknown type '{name}'")]
    UnknownType {
        /// The type, as referenced.
        name: String,
    },
    /// A reference to a type of a kind that its place does not allow, such
    /// as an alternate branch of type `any`, a union base that is not a
    /// struct or a command returning an enum.
    #[error("Cannot use {meta} type {reference}")]
    TypeNotAllowed {
        /// The kind of the type, in QEMU's words: `built-in`, `enum`,
        /// `struct`, `union`, `alternate`, `array`, `command` or `event`,
        /// or `null` for an alternate branch of QEMU 2.6 to 4.1 that has
        /// none.
        meta: &'static str,
        /// The reference, in QAPI syntax: `'name'`, or `['name']` for an
        /// array.
        reference: String,
    },
    /// An array where the release allows only a type name: a flat union
    /// branch, or an alternate branch before QEMU 7.1.
    #[error("Cannot be an array")]
    ArrayNotAllowed,
    /// The untyped `**` outside the arguments and return type of a command
    /// whose code QEMU does not generate (`'gen': false`), in QEMU 2.4.
    #[error("'**' requires 'gen': false")]
    UntypedWithoutGen,
    /// An object written out in place where the release requires a type
    /// name: a union base from QEMU 2.0 to 2.5 (in 2.4 a flat union's is
    /// [`Self::StringBase`]), the type of a member or branch from 2.4 to
    /// 3.1 (from 4.0 an object there spells the type with its keys, whose
    /// `type` must be a type name), a struct's base and a command's return
    /// type from 2.5, and anything but the arguments of a command or
    /// event, the members of a struct and the base of a flat union from
    /// 4.2. Where QEMU 8.1 and later also take an array, it is
    /// [`Shape::TypeNameOrArray`] there.
    #[error("Should be a type name")]
    AnonymousStruct,
    /// A command or event flag with a value that QEMU does not allow for
    /// it: `gen` and `success-response` may only be false from QEMU 2.4,
    /// `boxed`, `allow-oob`, `allow-preconfig` and `coroutine` only true.
    #[error("Flag '{flag}' may only use {value} value")]
    FlagValue {
        /// The flag.
        flag: &'static str,
        /// The value it allows.
        value: bool,
    },
    /// A command with both `allow-oob` and `coroutine`.
    #[error("Flags 'allow-oob' and 'coroutine' are incompatible")]
    IncompatibleFlags,
    /// Boxed arguments without `data`, from QEMU 2.7. (Boxed arguments
    /// written out in place are no type name.)
    #[error("'boxed': true requires 'data' of a named type")]
    BoxedWithoutType,
    /// Boxed arguments of a type without members, from QEMU 2.7 to 4.1.
    #[error("Cannot use 'boxed' with empty type '{name}'")]
    BoxedEmpty {
        /// The argument type.
        name: String,
    },
    /// Union arguments that are not boxed, from QEMU 4.2; before, a type
    /// that its place does not allow.
    #[error("Union argument type '{name}' requires 'boxed': true")]
    UnboxedUnion {
        /// The argument type.
        name: String,
    },
    /// Conditional members in arguments that are not boxed, from QEMU 8.1.
    #[error("Conditional arguments require 'boxed': true")]
    ConditionalArguments,
    /// Two alternate branches that the same JSON value could match.
    #[error("Alternate branch '{branch}' cannot be distinguished from '{previous}'")]
    AmbiguousAlternate {
        /// The later branch.
        branch: String,
        /// The branch it cannot be told apart from.
        previous: String,
    },
    /// An alternate with fewer branches than the release requires: two
    /// from QEMU 2.6 to 4.1, one from 4.2.
    #[error("{}", if *minimum == 1 {
        "'data' must not be empty".to_owned()
    } else {
        format!("Alternate should have at least {minimum} branches in 'data'")
    })]
    AlternateBranches {
        /// The fewest branches allowed.
        minimum: usize,
    },
    /// A union base without members: a struct named as the base of any
    /// union from QEMU 2.0 to 2.3, of a flat union until 2.8 (2.5 to 2.8
    /// fail an assertion on it). From 2.9 the discriminator is no member.
    #[error("Base '{name}' has no members")]
    EmptyBase {
        /// The base.
        name: String,
    },
    /// A member of an event's data, or of the struct that the data names,
    /// that is an object written out in place, from QEMU 2.1 to 2.3.
    #[error("Nested structure in event data is not supported")]
    NestedEventData,
    /// A union with a discriminator but no base, from QEMU 2.0; from 2.6
    /// to 6.1 an empty object written out in place is none either.
    #[error("'discriminator' requires 'base'")]
    MissingBase,
    /// A union with a base but no discriminator, from QEMU 2.4.
    #[error("'base' requires 'discriminator'")]
    BaseWithoutDiscriminator,
    /// A union discriminator that is not a member of the base: from QEMU 2.0
    /// to 4.1 a required member that the base declares itself, from 4.2 any
    /// member, inherited ones too.
    #[error("Discriminator '{discriminator}' is not a member of 'base'")]
    DiscriminatorNotMember {
        /// The discriminator.
        discriminator: String,
    },
    /// A flat union discriminator that is also a value of its enum, in
    /// QEMU 2.5.
    #[error("Discriminator '{discriminator}' collides with a value of enum '{enumeration}'")]
    DiscriminatorClash {
        /// The discriminator.
        discriminator: String,
        /// Its enum.
        enumeration: String,
    },
    /// An optional union discriminator, from QEMU 4.2; before, the base
    /// declares no member of that name.
    #[error("Discriminator '{discriminator}' must not be optional")]
    OptionalDiscriminator {
        /// The discriminator.
        discriminator: String,
    },
    /// A union discriminator with a build condition, from QEMU 4.0.
    #[error("Discriminator '{discriminator}' must not be conditional")]
    ConditionalDiscriminator {
        /// The discriminator.
        discriminator: String,
    },
    /// A value of a flat union's discriminator enum without a branch, from
    /// QEMU 2.7 to 2.12.
    #[error("Union misses branch '{branch}'")]
    MissingBranch {
        /// The enum value.
        branch: String,
    },
    /// Documentation that names another symbol than its definition, from QEMU 2.9.
    #[error("documentation comment is for '{documented}'")]
    DocumentationSymbol {
        /// The definition the documentation names.
        documented: String,
        /// The definition it precedes.
        name: String,
    },
    /// A definition without documentation while the `doc-required` pragma
    /// holds, from QEMU 2.9.
    #[error("documentation comment required")]
    DocumentationRequired,
    /// A Returns section for a command without a return type, from QEMU 9.0.
    #[error("'Returns' section, but command doesn't return anything")]
    ReturnsWithoutType,
    /// A Returns section outside a command, from QEMU 2.9.
    #[error("'Returns' section is only valid for commands")]
    ReturnsSection,
    /// An Errors section outside a command, from QEMU 9.1. (9.0 reports a
    /// Returns section there first and fails without one.)
    #[error("'Errors' section is only valid for commands")]
    ErrorsSection,
    /// An Errors section outside a command without a Returns section, on
    /// which QEMU 9.0 fails as it looks for the Returns section to report.
    #[error("'Errors' section outside a command, without a 'Returns' section")]
    ErrorsWithoutReturns,
    /// Descriptions of members that the definition does not have: from
    /// QEMU 2.9 for structs, unions, alternates and inline, non-empty
    /// arguments, from 4.2 for every definition.
    #[error("{}", documented("member", .names))]
    UnknownDocumentedMember {
        /// The described names, in the documentation's order.
        names: Vec<String>,
    },
    /// Descriptions of features that the definition does not have, from
    /// QEMU 4.2.
    #[error("{}", documented(word, .names))]
    UnknownDocumentedFeature {
        /// The described names, in the documentation's order.
        names: Vec<String>,
        /// What QEMU's message calls them: `member` until 6.1, `feature`
        /// from 6.2.
        word: &'static str,
    },
    /// A feature without a description, from QEMU 4.2: the first in the
    /// definition's order, its own features before its members'.
    #[error("feature '{name}' lacks documentation")]
    UndocumentedFeature {
        /// The feature.
        name: String,
    },
    /// A member without a description of a definition that the
    /// `documentation-exceptions` pragma exempts, while every section of
    /// the documentation after its introduction describes a member: QEMU
    /// 10.0 to 11.0 fail as they add a section for it after those.
    #[error(
        "{role} '{name}' lacks documentation, and only member descriptions follow the introduction"
    )]
    ExemptMemberAfterMembers {
        /// `member`, `value` or `branch`.
        role: &'static str,
        /// Its name.
        name: String,
    },
    /// A member, enum value or alternate branch without a description,
    /// from QEMU 9.0, unless the `documentation-exceptions` pragma names
    /// the definition.
    #[error("{role} '{name}' lacks documentation")]
    UndocumentedMember {
        /// `member`, `value` or `branch`.
        role: &'static str,
        /// Its name.
        name: String,
    },
}

/// The shapes that a place takes, as [`SchemaErrorKind::ValueShape`]
/// reports them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Shape {
    /// A type name: a base, the type of a member or branch, or from 2.5 a
    /// command's return type. (An object there is
    /// [`SchemaErrorKind::AnonymousStruct`].)
    #[error("Should be a type name")]
    TypeName,
    /// From QEMU 8.1, a type name or an array of one: a return type, the
    /// type of a member, or an alternate's branch.
    #[error("Should be a type name or array")]
    TypeNameOrArray,
    /// A type name or an object of members: a struct's members, the
    /// arguments of a command or event, a flat union's base.
    #[error("Should be an object or type name")]
    ObjectOrTypeName,
    /// An object: in QEMU 2.4 any place of a type, from 6.1 the branches
    /// of a union or alternate.
    #[error("Should be an object")]
    Object,
    /// An array: an enum's values, the features, from 6.2 the operands of
    /// `all` and `any`.
    #[error("Should be an array")]
    Array,
    /// A string: an enum's prefix.
    #[error("Should be a string")]
    String,
    /// An array type of one type name.
    #[error("Array type must contain single type name")]
    ArrayOfOneTypeName,
    /// A condition: a string or a list of strings from QEMU 3.0 to 6.1, a
    /// string or an object from 6.2.
    #[error("'if' condition must be a string, until 6.1 a list of strings, from 6.2 an object")]
    Condition,
}

/// A case rule of QAPI names, as [`SchemaErrorKind::NameCase`] reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NameRule {
    /// No uppercase letters: names of members, enum values and branches
    /// from QEMU 2.6 to 5.2, and of features from 4.2 to 5.2.
    #[error("must not use uppercase")]
    NoUppercase,
    /// No uppercase letters and no `_`: member, feature and command names
    /// from QEMU 6.0.
    #[error("must not use uppercase or '_'")]
    Lowercase,
    /// No lowercase letters and no `-`: event names from QEMU 6.0.
    #[error("must not use lowercase or '-'")]
    Uppercase,
    /// CamelCase: type names from QEMU 6.0.
    #[error("must use CamelCase")]
    CamelCase,
}

fn documented(what: &str, names: &[String]) -> String {
    let quoted = names.join("', '");
    match names {
        [_] => format!("documented {what} '{quoted}' does not exist"),
        _ => format!("documented {what}s '{quoted}' do not exist"),
    }
}

/// A construct of [`SchemaErrorKind::Unsupported`]: valid for the release's
/// QEMU, but beyond the IR.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Construct {
    /// An anonymous struct outside struct members and event data, where
    /// QEMU accepts it: before 2.4 as a branch, an argument or a command's
    /// return type, and in 2.4 as a command's return type.
    #[error("The anonymous struct of '{member}'")]
    AnonymousStruct {
        /// The member, branch or `returns` that it types.
        member: String,
    },
    /// A reference to a type that the schema does not define before 2.4,
    /// when QEMU checks only union bases (from 2.0).
    #[error("The undefined type '{name}'")]
    UndefinedType {
        /// The type name.
        name: String,
    },
    /// Structs whose bases are not defined as structs, before 2.4. The
    /// error names the first of them as its definition.
    #[error("The struct bases {}", arrows(.pending))]
    UndefinedBases {
        /// The structs, as `(struct, base)` pairs.
        pending: Vec<(String, String)>,
    },
    /// Struct bases that lead back to the struct before 2.4, when QEMU
    /// accepts them.
    #[error("The inheritance cycle of '{name}'")]
    BaseCycle {
        /// The first struct on the cycle.
        name: String,
    },
    /// Two commands or two events of one name before 2.4, when QEMU
    /// accepts them; the IR has one command or event of each name.
    #[error("The second definition of '{name}'")]
    SharedName {
        /// The name.
        name: String,
    },
    /// Two definitions of one type name before 2.4, when QEMU had no
    /// namespace of definitions: the name defined twice, or a simple union
    /// `Foo` and a definition of its tag enum `FooKind`. (Commands and
    /// events whose C names clash each get a Rust name of their own.)
    #[error("The definitions {previous} and {name} of the type name {symbol}")]
    SharedTypeName {
        /// The type name.
        symbol: String,
        /// The definition that has the name first.
        previous: String,
        /// The definition that has it again.
        name: String,
    },
    /// A flat union's discriminator whose type is the tag enum of a simple
    /// union, `{Union}Kind`, from 4.2 to 6.1, when QEMU accepts it.
    #[error(
        "The discriminator '{discriminator}' of the simple union tag enum {discriminator_type}"
    )]
    ImplicitTagDiscriminator {
        /// The discriminator.
        discriminator: String,
        /// The tag enum.
        discriminator_type: String,
    },
    /// A union's base that is no struct before 2.0, when QEMU does not
    /// resolve it.
    #[error("The base '{base}' of a union, which is no struct")]
    UnionBase {
        /// The base.
        base: String,
    },
    /// A union discriminator that its base does not declare, before 2.0.
    #[error("The undeclared discriminator '{discriminator}'")]
    UndefinedDiscriminator {
        /// The discriminator.
        discriminator: String,
    },
    /// An optional or array union discriminator, before 2.0.
    #[error("The optional or array discriminator '{discriminator}'")]
    DiscriminatorShape {
        /// The discriminator.
        discriminator: String,
    },
    /// Before 2.4, an expression with values that the release's
    /// generators read loosely, beyond the grammar of its schemas: for
    /// example an enum value written as an object.
    #[error("The expression beyond the release's grammar")]
    Expression,
    /// An alternate as the boxed arguments of a command or event, from 2.7
    /// to 4.1.
    #[error("The boxed alternate arguments '{name}'")]
    BoxedAlternate {
        /// The alternate.
        name: String,
    },
    /// A type named like a builtin type before 2.4, which the IR cannot
    /// tell apart from the builtin.
    #[error("The type named like the builtin '{name}'")]
    BuiltinName {
        /// The name.
        name: String,
    },
    /// From 2.4, a value of a shape that the release's QEMU takes without
    /// checking it: a member's condition in 4.0 and 4.1, a struct's
    /// features that are an empty object in 4.1, or in 2.4 `null` as a
    /// struct's members or a member's or a branch's type.
    #[error("The value of '{key}'")]
    Value {
        /// The key of the value.
        key: String,
    },
    /// Two members of one object whose names differ only in the optional
    /// marker, such as `'a'` and `'*a'`, before 2.5, when QEMU accepts them
    /// (2.5 fails an assertion, and from 2.6 they clash).
    #[error("The two members named '{name}'")]
    DuplicateMember {
        /// Their name.
        name: String,
    },
}

/// What a release's QAPI generator fails on, in
/// [`SchemaErrorKind::Crash`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Crash {
    /// A cycle of struct bases in 2.4 and 2.5, whose member checks follow
    /// the bases until Python's recursion limit.
    #[error("the inheritance cycle of '{name}'")]
    BaseCycle {
        /// The base at which the cycle closes.
        name: String,
    },
    /// Two members whose names differ only in the optional marker, such as
    /// `'a'` and `'*a'`, in 2.5, where an assertion fails.
    #[error("the two members named '{name}'")]
    OptionalTwin {
        /// Their name.
        name: String,
    },
    /// A branch named `type` of an alternate in 2.5, where an assertion
    /// fails.
    #[error("the alternate branch 'type'")]
    AlternateTypeBranch,
    /// From 2.0 to 4.1, a flat union's discriminator whose type is the
    /// tag enum of a simple union, `{Union}Kind`, whose values QEMU looks
    /// for in vain.
    #[error("the discriminator '{discriminator}' of a simple union's tag enum")]
    ImplicitTagDiscriminator {
        /// The discriminator.
        discriminator: String,
    },
    /// From 2.0 to 2.3, an enum or union without `data`, or a union whose
    /// base is a struct without it, whose `data` QEMU looks up.
    #[error("the missing 'data' of '{name}'")]
    MissingData {
        /// The definition without `data`.
        name: String,
    },
    /// In 2.10, an alternate's string branch after a number or boolean
    /// branch that it cannot be told apart from: QEMU names the earlier
    /// branch by the string type, which no earlier branch has.
    #[error("the alternate branch '{branch}'")]
    AlternateConflict {
        /// The string branch.
        branch: String,
    },
    /// In 2.5, an alternate branch of type `any`, which has no JSON type
    /// for QEMU to tell the branches apart by.
    #[error("the alternate branch '{branch}' of type 'any'")]
    AlternateAny {
        /// The branch.
        branch: String,
    },
    /// From 2.6 to 4.1, the type `{Alternate}Kind`, the tag enum of an
    /// alternate, which the expressions take for an enum but the schema
    /// does not define.
    #[error("the type '{name}'")]
    AlternateTag {
        /// The tag enum's name.
        name: String,
    },
    /// From 2.9 to 4.1, a flat union's discriminator that its base
    /// declares as an array, or until 3.1 as an object written out in
    /// place, whose type QEMU looks up as a key; from 2.4 to 4.1 a
    /// discriminator that is an array or an object itself, and from 2.9
    /// such a base.
    #[error("the type of the discriminator '{discriminator}'")]
    UnhashableDiscriminator {
        /// The discriminator, or its value as written.
        discriminator: String,
    },
    /// From 2.1 to 2.3, event data that names no struct, which QEMU
    /// asserts it finds.
    #[error("the event data '{name}'")]
    EventData {
        /// The name.
        name: String,
    },
    /// From 4.2 to 6.1, a simple union's branch of an undefined type: QEMU
    /// fails as it describes the member of the object type that wraps it.
    #[error("the branch '{branch}' of an undefined type")]
    SimpleBranchType {
        /// The branch.
        branch: String,
    },
    /// A value of a shape that the release uses without checking it: the
    /// `data` of a union or alternate that is no object (2.4 to 6.0), a
    /// struct's members as a type name (4.2 to 8.0), from 2.5 to 4.1 `null`
    /// as a struct's members or a member's or a branch's type, in 4.1
    /// `null` or `false` as the features, the empty name of an enum value
    /// from 2.6, and in 6.0 an enum value's object without `name`.
    #[error("the value of '{key}'")]
    UncheckedValue {
        /// The key of the value.
        key: String,
    },
}

fn arrows(pending: &[(String, String)]) -> String {
    pending
        .iter()
        .map(|(name, base)| format!("{name} -> {base}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) type SchemaResult<T = ()> = Result<T, SchemaError>;

/// Fail with `kind` unless `condition` holds.
pub(crate) fn require(condition: bool, kind: impl FnOnce() -> SchemaErrorKind) -> SchemaResult {
    if condition {
        Ok(())
    } else {
        Err(kind().into())
    }
}

/// The kind and name of a definition, as [`SchemaError::definition`] names it.
pub(crate) fn definition(token: &QapiSchemaToken<'_>) -> Option<String> {
    let (kind, name) = match token {
        QapiSchemaToken::Enum(value) => ("enum", value.name.as_ref()),
        QapiSchemaToken::Struct(value) => ("struct", value.name.as_ref()),
        QapiSchemaToken::Union(value) => ("union", value.name.as_ref()),
        QapiSchemaToken::Alternate(value) => ("alternate", value.name.as_ref()),
        QapiSchemaToken::Command(value) => ("command", value.name.as_ref()),
        QapiSchemaToken::Event(value) => ("event", value.name.as_ref()),
        _ => return None,
    };
    Some(format!("{kind} '{name}'"))
}

/// Locate schema errors as they propagate out of a definition.
pub(crate) trait SchemaContext<T> {
    /// Name the part of the definition that the error occurred in.
    fn within(self, part: impl FnOnce() -> String) -> SchemaResult<T>;
    /// Name the definition, unless an inner one is named already.
    fn in_definition(self, definition: impl FnOnce() -> Option<String>) -> SchemaResult<T>;
    /// Name the file, unless an inner one is named already.
    fn in_file(self, file: &Path) -> SchemaResult<T>;
    /// Name the line and column, unless inner ones are named already.
    fn at(self, position: impl FnOnce() -> Option<(usize, usize)>) -> SchemaResult<T>;
}

impl<T, E: Into<SchemaError>> SchemaContext<T> for Result<T, E> {
    fn within(self, part: impl FnOnce() -> String) -> SchemaResult<T> {
        self.map_err(|error| {
            let mut error = error.into();
            error.0.context.insert(0, part());
            error
        })
    }

    fn in_definition(self, definition: impl FnOnce() -> Option<String>) -> SchemaResult<T> {
        self.map_err(|error| error.into().located(definition(), None))
    }

    fn in_file(self, file: &Path) -> SchemaResult<T> {
        self.map_err(|error| error.into().located(None, Some(file)))
    }

    fn at(self, position: impl FnOnce() -> Option<(usize, usize)>) -> SchemaResult<T> {
        self.map_err(|error| error.into().at(position()))
    }
}

/// A quirk of an old schema that the generator tolerates.
///
/// The registry builders collect them in
/// [`UnifiedRegistry::warnings`](super::UnifiedRegistry::warnings); the
/// library never prints them.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{tag}: {kind}")]
#[non_exhaustive]
pub struct Warning {
    /// The release tag whose schema has the quirk.
    pub tag: String,
    /// The quirk.
    pub kind: WarningKind,
}

/// The quirks of old schemas that the generator tolerates.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum WarningKind {
    /// Documentation before QEMU 2.9 that names another symbol than its
    /// definition; the text stays attached to the definition.
    #[error("documentation symbol '{documented}' does not match {definition}")]
    MislabeledDocumentation {
        /// The definition, as its kind and name.
        definition: String,
        /// The symbol that the documentation names.
        documented: String,
    },
    /// A union discriminator before QEMU 2.0 whose type is not an enum; the
    /// discriminator stays a field of that type.
    #[error(
        "union '{union}' discriminator '{discriminator}' has non-enum type {discriminator_type}"
    )]
    NonEnumDiscriminator {
        /// The union.
        union: String,
        /// The discriminator member.
        discriminator: String,
        /// Its type.
        discriminator_type: String,
    },
}

/// Why building a registry from a QEMU git tree failed.
///
/// The errors name a schema file of a release as git names a file of its tag,
/// such as `v11.1.3:qga/qapi-schema.json`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RegistryError {
    /// A tag that is not a stable release tag.
    #[error("Invalid version tag: {tag}")]
    Tag {
        /// The tag.
        tag: String,
    },
    /// A release's schema files could not be extracted from the tree.
    #[error("Failed to extract the schema of {tag}")]
    Extract {
        /// The release tag.
        tag: String,
        #[source]
        source: TreeError,
    },
    /// A release that should ship the protocol's schema has no entry point:
    /// a QMP release from 1.0, a QGA release from 0.15.
    #[error("No {protocol} schema entry point found for {tag}")]
    MissingEntry {
        /// The release tag.
        tag: String,
        /// `QMP` or `QGA`.
        protocol: &'static str,
    },
    /// A release's schema files could not be read or parsed.
    #[error("Failed to read the schema of {tag}")]
    Read {
        /// The release tag.
        tag: String,
        #[source]
        source: ReadError,
    },
    /// A release's schema is invalid.
    #[error("Failed to process {tag}")]
    Schema {
        /// The release tag.
        tag: String,
        #[source]
        source: SchemaError,
    },
    /// A worker thread panicked while processing a release.
    #[error("Parser worker panicked for {tag}")]
    WorkerPanicked {
        /// The release tag.
        tag: String,
    },
    /// A type that a later release declares again after its removal, which
    /// the symbol planning failed to give a name of its own.
    #[error("{protocol} type {name} returns in {tag} after its removal in {removed}")]
    Reappeared {
        /// `QMP` or `QGA`.
        protocol: &'static str,
        name: String,
        /// The tag of the first release without the type.
        removed: String,
        /// The tag of the release that declares it again.
        tag: String,
    },
    /// A type that is an enum in one release and a struct in a later one,
    /// which the symbol planning failed to give a name of its own.
    #[error("{protocol} type {name} changes from {previous} to {kind} in {tag}")]
    KindChanged {
        /// `QMP` or `QGA`.
        protocol: &'static str,
        name: String,
        /// `enum` or `struct`: the type in the earlier releases.
        previous: &'static str,
        /// `enum` or `struct`: the type in `tag`.
        kind: &'static str,
        /// The tag of the release that changes it.
        tag: String,
    },
}
