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
/// [`definition`](Self::definition) and [`context`](Self::context) say where.
///
/// ```
/// use qapi_codegen::unified::{SchemaError, SchemaErrorKind};
///
/// let error = SchemaError::from(SchemaErrorKind::InvalidName { name: "q-x".into() });
/// assert_eq!(error.to_string(), "Invalid QAPI name 'q-x'");
/// assert_eq!(error.file(), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError(Box<Located>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Located {
    file: Option<PathBuf>,
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
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.definition(), self.file()) {
            (Some(definition), Some(file)) => {
                write!(f, "Invalid QAPI {definition} in {}: ", file.display())?
            }
            (Some(definition), None) => write!(f, "Invalid QAPI {definition}: ")?,
            (None, Some(file)) => write!(f, "Invalid QAPI schema {}: ", file.display())?,
            (None, None) => {}
        }
        for part in self.context() {
            write!(f, "{part}: ")?;
        }
        write!(f, "{}", self.kind())
    }
}

impl std::error::Error for SchemaError {}

impl From<SchemaErrorKind> for SchemaError {
    fn from(kind: SchemaErrorKind) -> Self {
        Self(Box::new(Located {
            file: None,
            definition: None,
            context: Vec::new(),
            kind,
        }))
    }
}

impl From<LowerError> for SchemaError {
    fn from(error: LowerError) -> Self {
        SchemaErrorKind::Lower(error).into()
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
    /// Two definitions whose names give the same C symbol in one namespace
    /// (`type`, `command` or `event`), such as `foo-bar` and `foo_bar`. A
    /// simple union's tag enum, `{Union}Kind`, is a type of the release.
    #[error(
        "Conflicting QAPI definitions {previous} and {name} share a {namespace} symbol {symbol}"
    )]
    SymbolClash {
        /// `type`, `command` or `event`.
        namespace: &'static str,
        /// The C symbol, or the type name.
        symbol: String,
        /// The definition that has the symbol first.
        previous: String,
        /// The definition that has it again.
        name: String,
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
    /// Structs whose bases are missing or inherit from each other. The error
    /// names the first of them as its definition.
    #[error(
        "Unresolved QAPI struct bases (missing definition or inheritance cycle): {}",
        arrows(.pending)
    )]
    UnresolvedBases {
        /// The structs that cannot be lowered, as `(struct, base)` pairs.
        pending: Vec<(String, String)>,
    },
    /// A definition that the IR cannot represent.
    #[error("{0}")]
    Lower(LowerError),
    /// A union discriminator that is optional or an array.
    #[error("Discriminator '{discriminator}' must be a required scalar enum")]
    DiscriminatorShape {
        /// The discriminator member.
        discriminator: String,
    },
    /// A union whose discriminator enum has no values.
    #[error("Union has no branches")]
    NoBranches,
    /// A union branch that is not a value of the discriminator enum.
    #[error("Branch '{branch}' is not a value of discriminator enum {discriminator_type}")]
    UnknownBranch {
        /// The branch.
        branch: String,
        /// The discriminator's enum.
        discriminator_type: String,
    },
    /// A union discriminator whose type is not an enum, from QEMU 6.2.
    #[error("Discriminator '{discriminator}' has non-enum type {discriminator_type}")]
    DiscriminatorNotEnum {
        /// The discriminator member.
        discriminator: String,
        /// Its type.
        discriminator_type: String,
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
    /// A union discriminator before QEMU 6.2 whose type is not an enum; the
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
