//! Multi-version QAPI schema merge.
//!
//! [`generate`] parses the schemas of multiple QEMU release tags, merges them
//! into a single registry in which each type, field and variant has the
//! range of releases that have it, and emits the Rust modules of that
//! registry as a [`GeneratedOutput`].

use crate::{
    QapiSchemaToken,
    qapi_ir::*,
    versions::{
        extract_schema_from_tag, find_qga_schema_entry, find_schema_entry,
        parse_version_tag_full,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::Path,
};

mod emit;
mod error;
mod lower;
#[cfg(test)]
pub(crate) use lower::process_version_schema;
mod merge;
mod registry;

pub use crate::qapi_ir::LowerError;
#[doc(hidden)]
pub use emit::emit_registry;
pub use emit::{Docs, GeneratedOutput, Protocol, Releases, generate};
pub use error::{
    Construct, Crash, GenerateError, NameRule, RegistryError, SchemaError,
    SchemaErrorKind, Shape, Warning, WarningKind,
};
pub(crate) use error::{SchemaContext, SchemaResult, definition, require};
pub(crate) use merge::has_feature;
#[doc(hidden)]
pub use registry::{build_qga_registry, build_qmp_registry};

/// A QEMU release as `(major, minor, micro)`. A four-component tag has the
/// tuple of its first three components.
pub type Version = (u32, u32, u32);

pub(crate) fn version_str(v: Version) -> String {
    format!("{}.{}.{}", v.0, v.1, v.2)
}

/// Classify a unified type as a command, event, or plain data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypeCategory {
    Command,
    Event,
    DataType,
}

/// Determine the category of a unified type by inspecting its IR attributes.
pub(crate) fn classify_type(ut: &UnifiedType) -> TypeCategory {
    let attrs = match ut {
        UnifiedType::Struct(us) => &us.ir.meta.attributes,
        UnifiedType::Enum(_) => return TypeCategory::DataType,
    };
    for attr in attrs {
        match attr {
            Attribute::Unit(n) if n == "command" => {
                return TypeCategory::Command;
            }
            Attribute::Unit(n) if n == "event" => return TypeCategory::Event,
            _ => {}
        }
    }
    TypeCategory::DataType
}

fn clean_doc(doc: &str) -> Option<String> {
    let doc = doc.trim_matches('\n');
    if doc.trim().is_empty() {
        return None;
    }
    Some(doc.to_string())
}

/// Clean a doc string field in place. If `doc` is `Some`, replaces it with
/// the cleaned value (which may become `None` when only whitespace remained).
/// `None` inputs stay `None`.
fn clean_doc_in_place(doc: &mut Option<String>) {
    *doc = doc.as_deref().and_then(clean_doc);
}

/// Update `dst` from `src` when `src` has a doc, applying [`clean_doc`].
/// If `src` is `None`, leaves `dst` unchanged (lets older docs survive
/// version merges where the newer schema dropped the documentation).
fn merge_doc_from(dst: &mut Option<String>, src: Option<&str>) {
    if let Some(s) = src {
        *dst = clean_doc(s);
    }
}

fn clean_docs(meta: &mut Metadata) {
    clean_doc_in_place(&mut meta.doc);
    clean_doc_in_place(&mut meta.schema_doc);
}

fn merge_docs_from(dst: &mut Metadata, src: &Metadata) {
    merge_doc_from(&mut dst.doc, src.doc.as_deref());
    merge_doc_from(&mut dst.schema_doc, src.schema_doc.as_deref());
}

/// The merged releases that have a type, field, variant or flag, and those
/// that mark it deprecated or unstable. A range is a first release and an
/// exclusive end, the first release without it.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct VersionRange {
    /// The first merged release that has the item.
    pub since: Version,
    /// The first merged release after `since` without the item, or `None`
    /// while the newest merged release has it.
    pub until: Option<Version>,
    /// The first release that marks the item deprecated;
    /// `deprecated_ranges` has each deprecation.
    pub deprecated: Option<Version>,
    /// Each run of releases that marks the item deprecated, as `since` and
    /// exclusive `until`.
    pub deprecated_ranges: Vec<(Version, Option<Version>)>,
    /// The first release that marks the item unstable; `unstable_ranges`
    /// has each instability, stabilization gaps included.
    pub unstable: Option<Version>,
    /// Each run of releases that marks the item unstable, as `since` and
    /// exclusive `until`.
    pub unstable_ranges: Vec<(Version, Option<Version>)>,
    /// Each run of releases in which a field is its union's discriminator,
    /// as `since` and exclusive `until`: `CpuInfoFast.arch` is in 2.12, and
    /// `target` from 3.0, while `arch` lasts until 6.0.
    pub discriminator_ranges: Vec<(Version, Option<Version>)>,
}

/// Whether `ranges` cover `version`.
#[cfg(test)]
fn in_ranges(ranges: &[(Version, Option<Version>)], version: Version) -> bool {
    ranges.iter().any(|(since, until)| {
        version >= *since && until.is_none_or(|end| version < end)
    })
}

/// Record whether a feature applies in `version`, the next merged release.
fn track_feature(
    first: &mut Option<Version>,
    ranges: &mut Vec<(Version, Option<Version>)>,
    present: bool,
    version: Version,
) {
    if present {
        first.get_or_insert(version);
        if ranges.last().is_none_or(|(_, until)| until.is_some()) {
            ranges.push((version, None));
        }
    } else if let Some((_, until)) = ranges.last_mut() {
        until.get_or_insert(version);
    }
}

impl VersionRange {
    pub(crate) fn new(since: Version) -> Self {
        Self {
            since,
            until: None,
            deprecated: None,
            deprecated_ranges: Vec::new(),
            unstable: None,
            unstable_ranges: Vec::new(),
            discriminator_ranges: Vec::new(),
        }
    }

    /// Whether the deprecated feature applies in a particular release.
    #[cfg(test)]
    pub(crate) fn is_deprecated(&self, version: Version) -> bool {
        in_ranges(&self.deprecated_ranges, version)
    }

    /// Whether the unstable feature applies in a particular release.
    #[cfg(test)]
    pub(crate) fn is_unstable(&self, version: Version) -> bool {
        in_ranges(&self.unstable_ranges, version)
    }

    /// Whether a field is its union's discriminator in a particular release.
    #[cfg(test)]
    pub(crate) fn is_discriminator(&self, version: Version) -> bool {
        in_ranges(&self.discriminator_ranges, version)
    }

    /// Record the features of `version`, the next merged release, and
    /// whether it makes the field its union's discriminator.
    fn track_features(&mut self, meta: &Metadata, version: Version) {
        let discriminator =
            meta.attributes.contains(&Attribute::new("discriminator"));
        track_feature(
            &mut None,
            &mut self.discriminator_ranges,
            discriminator,
            version,
        );
        let deprecated = has_feature(meta, "deprecated");
        track_feature(
            &mut self.deprecated,
            &mut self.deprecated_ranges,
            deprecated,
            version,
        );
        let unstable = has_feature(meta, "unstable");
        track_feature(
            &mut self.unstable,
            &mut self.unstable_ranges,
            unstable,
            version,
        );
    }
}

/// An enum, alternate or union branch enum, with the releases of each of its
/// variants.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct UnifiedEnum {
    /// The enum as the newest merged release declares it, with the variants
    /// of the earlier releases too, and each earlier payload shape of a
    /// variant as a variant of its own named `{Variant}Legacy`.
    pub ir: Enum,
    /// The releases that have the enum.
    pub versions: VersionRange,
    /// The releases of each variant of `ir`, by its IR name, `…Legacy`
    /// variants included.
    pub variant_versions: BTreeMap<String, VersionRange>,
    /// The earlier payload shapes and wire spellings of variants, with their
    /// releases, from which the `…Legacy` variants of `ir` are made.
    pub variant_history: Vec<(EnumVariant, VersionRange)>,
}

/// A struct, union, command or event, with the releases of each of its
/// fields and flags.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct UnifiedStruct {
    /// The struct as the newest merged release declares it, with the fields
    /// of the earlier releases too, and each earlier shape of a field as a
    /// field of its own named `{field}_legacy`.
    pub ir: Struct,
    /// The releases that have the struct.
    pub versions: VersionRange,
    /// The releases of each field of `ir`, by its IR name, `…_legacy` fields
    /// included.
    pub field_versions: BTreeMap<String, VersionRange>,
    /// The earlier types, optionality and wire spellings of fields, with
    /// their releases, from which the `…_legacy` fields of `ir` are made.
    pub field_history: Vec<(StructField, VersionRange)>,
    /// The newest run of releases of each flag that the struct has had:
    /// `allow_oob`, `allow_preconfig`, `no_success_response` and
    /// `simple_union`.
    pub attr_versions: BTreeMap<String, VersionRange>,
    /// The earlier runs of flags that a later release sets again.
    pub attr_history: Vec<(String, VersionRange)>,
    /// A command's return types in release order, each with the first merged
    /// release that returns that Rust type, or for a type that coalescing
    /// moved, the first release from which the type is its predecessor on
    /// the wire. The newest is the `returns` attribute of `ir`.
    pub returns_history: Vec<(Version, String)>,
    /// The name of the enum of the return types, which a command with two
    /// or more of them in `returns_history` needs: planned after the merge,
    /// as `{Command}Returns` unless the generator's naming table names it.
    pub returns_enum: Option<String>,
    /// For a type that coalescing made a command's return type before QAPI
    /// does, the release from which its Rust lifetime starts: the
    /// predecessor is the same on the wire from then. `versions` keeps
    /// QAPI's.
    pub rust_since: Option<Version>,
    /// For the predecessor of such a type, the release from which it is
    /// the same on the wire as that type, and the type,
    /// `alias(added = "...", value = "...")`, which its documentation states.
    pub alias: Option<(Version, String)>,
}

impl UnifiedStruct {
    /// Whether a flag of `attr_versions` is set in `version`, in its newest
    /// run or an earlier one.
    #[cfg(test)]
    pub(crate) fn flag_enabled(&self, name: &str, version: Version) -> bool {
        self.attr_versions
            .get(name)
            .into_iter()
            .chain(
                self.attr_history.iter().filter_map(|(flag, range)| {
                    (flag == name).then_some(range)
                }),
            )
            .any(|range| {
                version >= range.since
                    && range.until.is_none_or(|until| version < until)
            })
    }

    /// The Rust type that a command returns in the newest merged release.
    pub fn returns(&self) -> Option<&str> {
        returns_attr(&self.ir.meta)
    }
}

fn returns_attr(meta: &Metadata) -> Option<&str> {
    meta.attributes.iter().find_map(|a| match a {
        Attribute::Item { name, value } if name == "returns" => {
            Some(value.as_str())
        }
        _ => None,
    })
}

/// A command's return type changed: the newest version is authoritative,
/// and the change is recorded in `returns_history`.
fn merge_returns(
    existing: &mut UnifiedStruct,
    new_meta: &Metadata,
    version: Version,
) {
    let Some(new) = returns_attr(new_meta) else {
        return;
    };
    if existing.returns_history.last().map(|(_, r)| r.as_str()) != Some(new) {
        existing.returns_history.push((version, new.to_owned()));
    }
    existing.ir.meta.qapi_returns.clone_from(&new_meta.qapi_returns);
    // A struct first seen as plain data gains `returns` through the
    // ordinary attribute union, keeping attribute order stable.
    for attr in &mut existing.ir.meta.attributes {
        if let Attribute::Item { name, value } = attr
            && name == "returns"
        {
            *value = new.to_owned();
        }
    }
}

/// A merged type.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum UnifiedType {
    /// An enum, alternate or union branch enum.
    Enum(UnifiedEnum),
    /// A struct, union, command or event.
    Struct(UnifiedStruct),
}

/// The types of every merged release.
#[doc(hidden)]
#[derive(Debug, Default)]
pub struct UnifiedRegistry {
    /// Every type, by its Rust name.
    pub types: BTreeMap<String, UnifiedType>,
    /// Every release of the selection in tag order, those skipped as
    /// identical to the release before included. Four-component tags repeat
    /// the tuple of their release.
    pub versions: Vec<Version>,
    /// The tolerated quirks of the releases' schemas, in release order.
    pub warnings: Vec<Warning>,
    /// The return types that coalescing moved: `(command, predecessor,
    /// type, release)`, by command, for the test that pins them.
    pub coalesced: Vec<(String, String, String, Version)>,
}

/// Return true when two attributes represent the "same" directive. For
/// directive attributes that can only appear once per item (all but `List`)
/// same means "same name/key". For `List` attributes we compare by
/// structural equality since a list may carry multiple semantically distinct
/// bundles (e.g. multiple feature markers). Used to dedupe attribute sets
/// while merging struct metadata across versions.
fn attrs_equivalent(a: &Attribute, b: &Attribute) -> bool {
    match (a, b) {
        (Attribute::Unit(a), Attribute::Unit(b)) => a == b,
        (Attribute::Item { name: a, .. }, Attribute::Item { name: b, .. }) => {
            a == b
        }
        (Attribute::List(a), Attribute::List(b)) => a == b,
        _ => false,
    }
}

fn condition(meta: &Metadata) -> Option<&str> {
    meta.attributes.iter().find_map(|attr| match attr {
        Attribute::Item { name, value } if name == "condition" => {
            Some(value.as_str())
        }
        _ => None,
    })
}

// Conditions, deprecation and instability describe the newest declaration,
// not their historical union.
fn refresh_current_attributes(existing: &mut Metadata, latest: &Metadata) {
    existing.attributes.retain(|attr| !lifecycle_feature(attr));
    existing.attributes.extend(
        latest
            .attributes
            .iter()
            .filter(|attr| lifecycle_feature(attr))
            .cloned(),
    );
    if condition(existing) == condition(latest) {
        return;
    }
    existing
        .attributes
        .retain(|attr| !matches!(attr, Attribute::Item { name, .. } if name == "condition"));
    if let Some(value) = condition(latest) {
        existing.attributes.push(Attribute::with_value("condition", value));
    }
}

/// A QAPI feature, as a list of `feature` and its own `condition`.
fn feature(attr: &Attribute) -> bool {
    matches!(attr, Attribute::List(items)
        if matches!(items.first(), Some(Attribute::Item { name, .. }) if name == "feature"))
}

/// A `deprecated` or `unstable` feature, whose ranges the merge tracks.
fn lifecycle_feature(attr: &Attribute) -> bool {
    match attr {
        Attribute::Item { name, value } => {
            name == "feature" && (value == "deprecated" || value == "unstable")
        }
        Attribute::List(values) => values.iter().any(lifecycle_feature),
        _ => false,
    }
}

fn unstable_attributes(
    versions: &VersionRange,
) -> impl Iterator<Item = Attribute> + '_ {
    versions.unstable_ranges.iter().map(|(since, until)| {
        let mut attrs =
            vec![Attribute::with_value("added", version_str(*since))];
        if let Some(until) = until {
            attrs.push(Attribute::with_value("removed", version_str(*until)));
        }
        Attribute::nested("unstable", attrs)
    })
}

/// Merge command-specific attributes (command, returns, allow_oob, etc.) from
/// a processed command struct onto an existing data struct. The single
/// entry is the command's, so its features are the command's: those of the
/// data struct, which describe the type, do not carry over.
fn merge_command_attrs_onto(source: &Struct, target: &mut Struct) {
    target.meta.attributes.retain(|attr| !feature(attr));
    target.meta.attributes.extend(
        source.meta.attributes.iter().filter(|attr| feature(attr)).cloned(),
    );
    // The argument struct's QAPI name is not the command's wire name.
    // Preserve the command spelling when the two share a Rust identifier.
    target.meta.attributes.retain(
        |attr| !matches!(attr, Attribute::Item { name, .. } if name == "name"),
    );
    target
        .meta
        .attributes
        .push(Attribute::with_value("name", source.name.clone()));
    target.meta.schema_item =
        source.meta.schema_item.or(target.meta.schema_item);
    merge_doc_from(
        &mut target.meta.schema_doc,
        source.meta.schema_doc.as_deref(),
    );
    for attr in &source.meta.attributes {
        match attr {
            Attribute::Unit(n)
                if n == "command"
                    || n == "allow_oob"
                    || n == "allow_preconfig"
                    || n == "no_success_response" =>
            {
                if !target.meta.attributes.contains(attr) {
                    target.meta.attributes.push(attr.clone());
                }
            }
            Attribute::Item { name, .. }
                if name == "returns"
                    && !target.meta.attributes.iter().any(
                        |a| matches!(a, Attribute::Item { name: n, .. } if n == "returns"),
                    ) =>
            {
                target.meta.attributes.push(attr.clone());
                target
                    .meta
                    .qapi_returns
                    .clone_from(&source.meta.qapi_returns);
            }
            _ => {}
        }
    }
    if !target
        .meta
        .attributes
        .iter()
        .any(|a| matches!(a, Attribute::Unit(n) if n == "command"))
    {
        target.meta.attributes.push(Attribute::new("command"));
    }
}

#[cfg(test)]
mod enum_history_tests;
#[cfg(test)]
mod field_history_tests;
#[cfg(test)]
mod flag_timeline_tests;
#[cfg(test)]
mod naming_tests;
#[cfg(test)]
mod registry_tests;
#[cfg(test)]
mod schema_resolution_tests;
