#![doc = include_str!("../README.md")]
//!
//! ## Runtime
//!
//! The runtime of the QAPI types: types annotated with `#[qapi]` implement
//! [`QapiCommand`], [`QapiEvent`], [`PreconfigSafe`] or [`QapiFlatUnion`],
//! and their serialization checks each value against a peer's QEMU version:
//! [`with_version`] scopes that version to one operation, [`to_value`]
//! serializes a value for a version and reports a type, variant or field
//! missing from it as an [`EncodeError`], and [`set_qemu_version`] sets a
//! process-wide default for serialization outside any scope.
//! [`parse_qga_version`] reads a guest agent's version.
//!
//! ## Generated sources
//!
//! `src/generated` (QMP) and `src/qga` (QGA) are output of `qrab-qapi-codegen`,
//! and `contrib/check-qapi.sh` requires them to match a fresh generation byte
//! for byte. Change the generator, its IR or the macros and regenerate; never
//! edit the generated files by hand. The repository's `docs/codegen.md`
//! describes generation.
//! Its `docs/protocol.md` describes the version contexts.
#![forbid(unsafe_code)]

use std::{
    cell::{Cell, RefCell},
    sync::OnceLock,
};

// The `#[qapi]` expansion names this crate `::qapi`, also in the generated
// modules here.
extern crate self as qapi;

pub use qapi_macros::qapi;

/// What the `#[qapi]` expansion uses besides the public API. It is not
/// part of that API and may change in any release.
#[doc(hidden)]
pub mod __private {
    pub use serde;
    pub use serde_json;

    /// The serialization error of a type, variant (`Type::Variant`) or
    /// field (`Type.field`) outside its lifetime in the current version
    /// context. [`to_value`](crate::to_value) turns it into
    /// [`EncodeError::Unavailable`](crate::EncodeError::Unavailable), so a
    /// `Serialize` implementation must return it, or an error whose message
    /// carries its message, unchanged.
    pub fn unavailable_error<E: serde::ser::Error>(name: &str) -> E {
        let version = crate::qemu_version();
        let message = crate::unavailable_message(name, version);
        crate::record_unavailable(name.to_owned(), version, message.clone());
        E::custom(message)
    }

    /// Error text for a version-gated type, variant or field decoded
    /// outside its range.
    pub fn unavailable_type_message(type_name: &str) -> String {
        crate::unavailable_message(type_name, crate::qemu_version())
    }
}

/// Trait implemented by QAPI command types.
///
/// `#[qapi]` implements it for the generated commands, and a crate may
/// implement it by hand for a command of its own, such as one that its guest
/// agent adds. Such an implementation needs only [`Returns`](Self::Returns),
/// [`NAME`](Self::NAME) and [`AVAILABLE_SINCE`](Self::AVAILABLE_SINCE): the
/// flag constants default to a stable command that is still available, runs
/// neither out of band nor before the machine is configured and replies when
/// it succeeds, and the per-release methods default to those constants.
/// Items added to the trait later will have defaults too, so such an
/// implementation keeps compiling.
///
/// ```
/// use qapi::QapiCommand;
///
/// #[derive(serde::Serialize)]
/// struct QrabPing {}
///
/// impl QapiCommand for QrabPing {
///     type Returns = ();
///     const NAME: &'static str = "qrab-ping";
///     const AVAILABLE_SINCE: (u32, u32, u32) = (1, 0, 0);
/// }
///
/// assert!(QrabPing::SUCCESS_RESPONSE && !QrabPing::allows_oob((11, 1, 3)));
/// assert_eq!((QrabPing::REMOVED_IN, QrabPing::PRECONFIG_SINCE), (None, None));
/// ```
pub trait QapiCommand: serde::Serialize {
    /// The type of a successful reply. For a command whose return type
    /// changed across the releases of the build, an enum with a variant per
    /// return type, named by the release that introduced it, which decodes
    /// into the variant of the peer's release in a [`with_version`] scope.
    type Returns: serde::de::DeserializeOwned + 'static;
    /// The command's name on the wire.
    const NAME: &'static str;
    /// OOB flag in the newest represented schema; use `allows_oob` for a peer.
    /// `false` by default.
    const ALLOW_OOB: bool = false;
    /// Preconfig flag in the newest represented schema. `false` by default.
    const ALLOW_PRECONFIG: bool = false;
    /// Reply flag in the newest represented schema. `true` by default.
    const SUCCESS_RESPONSE: bool = true;
    /// Unstable marker in the newest represented schema. `false` by default.
    const UNSTABLE: bool = false;
    /// The first QEMU release with the command.
    const AVAILABLE_SINCE: (u32, u32, u32);
    /// The first QEMU release without the command, `None` while the newest
    /// has it, as by default.
    const REMOVED_IN: Option<(u32, u32, u32)> = None;
    /// The QEMU version when `allow-preconfig` was added for this command,
    /// in its first run of releases that the build supports. `None` if no
    /// release of the build supports preconfig for it, as by default.
    const PRECONFIG_SINCE: Option<(u32, u32, u32)> = None;

    /// Whether this command permits out-of-band execution on a peer release.
    fn allows_oob(_version: (u32, u32, u32)) -> bool {
        Self::ALLOW_OOB
    }

    /// Whether this command is safe during preconfig on a peer release.
    fn allows_preconfig(_version: (u32, u32, u32)) -> bool {
        Self::ALLOW_PRECONFIG
    }

    /// Whether a successful command produces a reply on a peer release.
    fn sends_success_response(_version: (u32, u32, u32)) -> bool {
        Self::SUCCESS_RESPONSE
    }

    /// Whether the command is marked unstable in a peer release.
    fn is_unstable(_version: (u32, u32, u32)) -> bool {
        Self::UNSTABLE
    }
}

/// Marker for commands with `allow-preconfig` in at least one release that
/// the build supports (`QEMU_MIN_VERSION` to `QEMU_MAX_VERSION`).
/// Check `QapiCommand::allows_preconfig` when selecting a peer release.
///
/// An API that issues commands while QEMU waits in its preconfig state,
/// before the machine is configured, can require `C: QapiCommand +
/// PreconfigSafe` and so refuse at compile time a command that no supported
/// release allows there.
///
/// The `--preconfig` flag was introduced in QEMU 3.0.0.
pub trait PreconfigSafe: QapiCommand {}

/// Trait implemented by QAPI event types.
pub trait QapiEvent: serde::de::DeserializeOwned {
    const NAME: &'static str;
}

/// Inspect and decode a QAPI union branch selected by its discriminator.
/// Flat union payloads are objects; historical simple unions can carry any JSON value.
pub trait QapiFlatUnion: Sized {
    /// The branch's wire tag, when known. Serializers use it to check the
    /// explicit discriminator; handwritten implementations may keep the default.
    fn wire_tag(&self) -> Option<&'static str> {
        None
    }

    fn deserialize_variant(
        discriminator: &str,
        map: serde_json::Map<String, serde_json::Value>,
    ) -> Result<Self, serde_json::Error>;

    /// Decode a branch payload, including historical scalar and array payloads.
    fn deserialize_variant_value(
        discriminator: &str,
        value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        use serde::de::Error as _;
        match value {
            serde_json::Value::Object(map) => Self::deserialize_variant(discriminator, map),
            _ => Err(serde_json::Error::custom(
                "expected an object for a flat union payload",
            )),
        }
    }
}

impl<T: QapiFlatUnion> QapiFlatUnion for Box<T> {
    fn wire_tag(&self) -> Option<&'static str> {
        T::wire_tag(self.as_ref())
    }

    fn deserialize_variant(
        discriminator: &str,
        map: serde_json::Map<String, serde_json::Value>,
    ) -> Result<Self, serde_json::Error> {
        T::deserialize_variant(discriminator, map).map(Box::new)
    }

    fn deserialize_variant_value(
        discriminator: &str,
        value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        T::deserialize_variant_value(discriminator, value).map(Box::new)
    }
}

static QEMU_VERSION: OnceLock<(u32, u32, u32)> = OnceLock::new();

thread_local! {
    static SERIALIZATION_VERSION: Cell<Option<(u32, u32, u32)>> = const { Cell::new(None) };
}

/// Scope QAPI serialization and historical wire layouts to a peer's version.
/// QMP and QGA clients use this for requests and replies, so different releases
/// can coexist.
///
/// The override is synchronous, thread-local, nestable and restored on unwind.
/// Perform the work inside the closure; do not return a future to await later.
pub fn with_version<T>(version: (u32, u32, u32), f: impl FnOnce() -> T) -> T {
    struct Restore(Option<(u32, u32, u32)>);
    impl Drop for Restore {
        fn drop(&mut self) {
            SERIALIZATION_VERSION.set(self.0);
        }
    }
    let _restore = Restore(SERIALIZATION_VERSION.replace(Some(version)));
    f()
}

/// [`set_qemu_version`] found a process default already set and kept it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
    "the process QEMU version is already {}.{}.{}, so {}.{}.{} was not set",
    .existing.0, .existing.1, .existing.2, .requested.0, .requested.1, .requested.2
)]
#[non_exhaustive]
pub struct VersionAlreadySet {
    /// The process default, which stays in effect.
    pub existing: (u32, u32, u32),
    /// The version the refused call asked for.
    pub requested: (u32, u32, u32),
}

/// Set the process-wide default version for serialization outside any
/// [`with_version`] scope. It never selects a decoded layout.
///
/// The default can be set once per process; every later call fails with
/// [`VersionAlreadySet`], even for an equal version. Protocol clients scope
/// serialization to their own peer, and code that talks to more than one
/// peer should do the same with [`with_version`] or [`to_value`].
pub fn set_qemu_version(requested: (u32, u32, u32)) -> Result<(), VersionAlreadySet> {
    let mut stored = false;
    let existing = *QEMU_VERSION.get_or_init(|| {
        stored = true;
        requested
    });
    if stored {
        Ok(())
    } else {
        Err(VersionAlreadySet {
            existing,
            requested,
        })
    }
}

/// Serialize a QAPI value for a particular peer version.
///
/// A value that uses a type, variant or field missing from that release
/// fails with [`EncodeError::Unavailable`], which names it, wherever it is
/// nested in the value; any other failure is [`EncodeError::Serde`].
///
/// Generated types gate fields and whole types on the peer's QEMU version,
/// so serializing outside a version context fails. Protocol clients scope
/// this for requests; use this helper for values built elsewhere.
///
/// ```
/// let props: qapi::CpuInstanceProperties =
///     serde_json::from_value(serde_json::json!({"core-id": 1})).unwrap();
/// let error = serde_json::to_value(&props).unwrap_err();
/// assert!(error.to_string().contains("qapi::with_version"));
/// let value = qapi::to_value((11, 1, 3), &props).unwrap();
/// assert_eq!(value["core-id"], 1);
/// let removed: qapi::TargetInfo =
///     serde_json::from_value(serde_json::json!({"arch": "x86_64"})).unwrap();
/// match qapi::to_value((11, 1, 3), &removed) {
///     Err(qapi::EncodeError::Unavailable { name, version }) => {
///         assert_eq!((name.as_str(), version), ("TargetInfo", (11, 1, 3)));
///     }
///     other => panic!("{other:?}"),
/// }
/// ```
pub fn to_value<T: serde::Serialize + ?Sized>(
    version: (u32, u32, u32),
    value: &T,
) -> Result<serde_json::Value, EncodeError> {
    struct Restore(Option<Vec<Unavailable>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            UNAVAILABLE.set(self.0.take());
        }
    }
    let restore = Restore(UNAVAILABLE.replace(Some(Vec::new())));
    let result = with_version(version, || serde_json::to_value(value));
    let recorded = UNAVAILABLE.take().unwrap_or_default();
    drop(restore);
    let error = match result {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };
    // A `Serialize` implementation may have recovered from an earlier
    // failure, so the failure is the newest one that the error carries.
    let text = error.to_string();
    match recorded
        .into_iter()
        .rev()
        .find(|unavailable| carries(&text, &unavailable.message))
    {
        Some(Unavailable { name, version, .. }) => {
            let error = EncodeError::Unavailable {
                name: name.clone(),
                version,
            };
            // An enclosing `to_value` reports it too, as the error that
            // carries it propagates there.
            record_unavailable(name, version, error.to_string());
            Err(error)
        }
        None => Err(EncodeError::Serde(error)),
    }
}

/// Whether the error text `text` carries `message` whole, not as a part of
/// a longer name or version.
fn carries(text: &str, message: &str) -> bool {
    text.match_indices(message).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + message.len()..].chars().next();
        !before.is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | ':' | '.'))
            && !after.is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

/// [`to_value`] could not serialize a value for a peer.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EncodeError {
    /// The value uses a type, variant or field that the peer's release does
    /// not have.
    #[error("{name} is not available in QEMU {}.{}.{}", .version.0, .version.1, .version.2)]
    Unavailable {
        /// The Rust name of the type (`TargetInfo`), the variant
        /// (`BlockdevDriver::Gluster`) or the field (`ImageInfo.limits`).
        name: String,
        /// The peer version the value was serialized for.
        version: (u32, u32, u32),
    },
    /// Any other failure, such as a missing required field.
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

/// A representation reported unavailable, with the text of its error.
struct Unavailable {
    name: String,
    version: (u32, u32, u32),
    message: String,
}

thread_local! {
    /// Inside [`to_value`] on this thread, the representations reported
    /// unavailable since it began, oldest first; `None` outside it, where
    /// nothing records.
    static UNAVAILABLE: RefCell<Option<Vec<Unavailable>>> = const { RefCell::new(None) };
}

fn record_unavailable(name: String, version: (u32, u32, u32), message: String) {
    UNAVAILABLE.with_borrow_mut(|slot| {
        if let Some(recorded) = slot {
            recorded.push(Unavailable {
                name,
                version,
                message,
            });
        }
    });
}

/// Without any version context the text names the fix rather than blaming
/// the peer.
fn unavailable_message(name: &str, version: (u32, u32, u32)) -> String {
    match version {
        (0, 0, 0) => format!(
            "{name} cannot be serialized without a QEMU version; \
             wrap serialization in qapi::with_version(peer_version, ..) or use qapi::to_value"
        ),
        (major, minor, patch) => format!("{name} is not available in QEMU {major}.{minor}.{patch}"),
    }
}

/// Get the explicit peer override, excluding the process default.
pub fn scoped_qemu_version() -> Option<(u32, u32, u32)> {
    SERIALIZATION_VERSION.get()
}

/// Get the scoped peer version, or the process default.
/// Returns (0, 0, 0) if neither has been set.
pub fn qemu_version() -> (u32, u32, u32) {
    SERIALIZATION_VERSION
        .get()
        .unwrap_or_else(|| *QEMU_VERSION.get().unwrap_or(&(0, 0, 0)))
}

/// Parse the version a guest agent reports in `guest-info`, for scoping its
/// requests and replies with [`with_version`].
///
/// The schema declares `GuestAgentInfo.version` as an opaque string; every
/// released `qemu-ga` reports `major.minor.patch`, which may carry a fourth
/// numeric component or a suffix after the patch number. Surrounding
/// whitespace and a leading `v` are ignored.
///
/// ```
/// assert_eq!(qapi::parse_qga_version("11.1.0"), Ok((11, 1, 0)));
/// assert_eq!(qapi::parse_qga_version("9.1.0-rc0"), Ok((9, 1, 0)));
/// assert!(qapi::parse_qga_version("9.1").is_err());
/// ```
pub fn parse_qga_version(version: &str) -> Result<(u32, u32, u32), VersionParseError> {
    let trimmed = version.trim().trim_start_matches(['v', 'V']);
    let parts: Vec<&str> = trimmed.split('.').collect();
    let error = |kind| VersionParseError {
        input: version.to_owned(),
        kind,
    };
    let component = |which, text: &str| {
        // `u32::from_str` takes a leading `+`, which no release has; parsed
        // alone, the sign is the invalid digit it is.
        let digits = if text.starts_with('+') { "+" } else { text };
        digits
            .parse::<u32>()
            .map_err(|source| error(VersionParseErrorKind::Component { which, source }))
    };
    fn leading_digits(text: &str) -> &str {
        let end = text
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(text.len());
        &text[..end]
    }
    let (major, minor, patch) = match parts[..] {
        [major, minor, patch] => (major, minor, leading_digits(patch)),
        [major, minor, patch, fourth] => {
            component(VersionComponent::Fourth, leading_digits(fourth))?;
            (major, minor, patch)
        }
        _ => return Err(error(VersionParseErrorKind::Shape)),
    };
    Ok((
        component(VersionComponent::Major, major)?,
        component(VersionComponent::Minor, minor)?,
        component(VersionComponent::Patch, patch)?,
    ))
}

/// [`parse_qga_version`] refused a version string.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid guest agent version {input:?}: {kind}")]
pub struct VersionParseError {
    input: String,
    kind: VersionParseErrorKind,
}

impl VersionParseError {
    /// The refused version string.
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Why it was refused.
    pub fn kind(&self) -> &VersionParseErrorKind {
        &self.kind
    }
}

/// Why [`parse_qga_version`] refused a version string.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum VersionParseErrorKind {
    /// Not three or four dot-separated components.
    #[error("expected major.minor.patch")]
    Shape,
    /// A component is not a decimal `u32`.
    #[error("the {which} component is not a 32-bit decimal number")]
    Component {
        which: VersionComponent,
        #[source]
        source: std::num::ParseIntError,
    },
}

/// A component of a version string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VersionComponent {
    Major,
    Minor,
    Patch,
    /// A stable branch's own release number after the patch.
    Fourth,
}

impl std::fmt::Display for VersionComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
            Self::Fourth => "fourth",
        })
    }
}

// The types of the generated `metadata` and `qga::metadata` tables, one set
// that both modules re-export.
mod metadata_types;

// The `schema-docs` feature swaps in modules regenerated by build.rs with
// QEMU's schema documentation; see docs/codegen.md.
#[cfg(not(feature = "schema-docs"))]
#[rustfmt::skip]
mod generated;
#[cfg(feature = "schema-docs")]
include!(concat!(env!("OUT_DIR"), "/schema_docs_modules.rs"));
/// QMP data types (structs, enums, unions) re-exported at the crate root.
pub use generated::*;

/// QMP command types — access via `qapi::commands::QueryStatus`, etc.
pub use generated::commands;

/// QMP event types — access via `qapi::events::ShutdownEvent`, etc.
pub use generated::events;

/// Compile-time schema metadata — access via `qapi::metadata::COMMANDS`,
/// `qapi::metadata::STRUCTS`, etc., for tooling that renders QAPI schema
/// information at run time.
pub use generated::metadata;

/// QEMU Guest Agent (QGA) types.
#[cfg(not(feature = "schema-docs"))]
#[rustfmt::skip]
pub mod qga;
