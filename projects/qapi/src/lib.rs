#![cfg_attr(
    qrab_qapi_schema_docs,
    doc = "<div class=\"warning\">This documentation contains QEMU's GPL-2.0-or-later \
           schema text, built with the <code>schema-docs</code> feature and \
           <code>QRAB_QAPI_SCHEMA_DOCS</code>. It is for local reading: do not publish \
           it.</div>\n"
)]
#![doc = include_str!("../README.md")]
//!
//! ## Runtime
//!
//! The runtime of the QAPI types is the crate `qrab-qapi-core`, which this
//! crate re-exports at the same paths. Types annotated with `#[qapi]`
//! implement [`QapiCommand`], [`QapiEvent`], [`PreconfigSafe`] or
//! [`QapiFlatUnion`], and their serialization checks each value against a
//! peer's QEMU version: [`with_version`] scopes that version to one
//! operation, [`to_value`] serializes a value for a version and reports a
//! type, variant or field missing from it as an [`EncodeError`], and
//! [`set_qemu_version`] sets a process-wide default for serialization
//! outside any scope. [`parse_qga_version`] reads a guest agent's version.
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

// The `#[qapi]` expansion names this crate `::qapi`, also in the generated
// modules here.
extern crate self as qapi;

pub use qapi_macros::qapi;

/// What the `#[qapi]` expansion uses besides the public API. It is not
/// part of that API and may change in any release.
#[doc(hidden)]
pub mod __private {
    pub use qapi_core::{unavailable_error, unavailable_type_message};
    pub use serde;
    pub use serde_json;
}

#[doc(inline)]
pub use qapi_core::{
    EncodeError, PreconfigSafe, QapiCommand, QapiEvent, VersionAlreadySet,
    VersionComponent, VersionParseError, VersionParseErrorKind,
    parse_qga_version, qemu_version, scoped_qemu_version, set_qemu_version,
    to_value, with_version,
};

/// Inspect and decode a QAPI union branch selected by its discriminator. Flat
/// union payloads are objects; historical simple unions can carry any JSON
/// value.
pub trait QapiFlatUnion: Sized {
    /// The branch's wire tag, when known. Serializers use it to check the
    /// explicit discriminator; handwritten implementations may keep the
    /// default.
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
            serde_json::Value::Object(map) => {
                Self::deserialize_variant(discriminator, map)
            }
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

// The types of the generated `metadata` and `qga::metadata` tables, one set
// that both modules re-export.
mod metadata_types;

// With the `schema-docs` feature and QRAB_QAPI_SCHEMA_DOCS, build.rs
// regenerates the modules with QEMU's schema documentation and sets this
// cfg, which swaps them in; see docs/codegen.md.
#[cfg(not(qrab_qapi_schema_docs))]
#[rustfmt::skip]
mod generated;
#[cfg(qrab_qapi_schema_docs)]
include!(concat!(env!("OUT_DIR"), "/schema_docs_modules.rs"));
/// QMP command types — access via `qapi::commands::QueryStatus`, etc.
pub use generated::commands;
/// QMP event types — access via `qapi::events::ShutdownEvent`, etc.
pub use generated::events;
/// Compile-time schema metadata — access via `qapi::metadata::COMMANDS`,
/// `qapi::metadata::STRUCTS`, etc., for tooling that renders QAPI schema
/// information at run time.
pub use generated::metadata;
/// QMP data types (structs, enums, unions) re-exported at the crate root.
pub use generated::*;

/// QEMU Guest Agent (QGA) types.
#[cfg(not(qrab_qapi_schema_docs))]
#[rustfmt::skip]
pub mod qga;
