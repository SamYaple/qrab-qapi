//! Merging one release's IR into the registry and materializing history.

use super::*;

/// Check if an IR node has a named feature (e.g. `deprecated`, `unstable`)
/// in its QAPI `features` list.
pub(crate) fn has_feature(meta: &Metadata, feature_name: &str) -> bool {
    meta.attributes.iter().any(|a| match a {
        Attribute::List(items) => items.iter().any(|item| {
            matches!(item,
                Attribute::Item { name, value } if name == "feature" && value == feature_name
            )
        }),
        _ => false,
    })
}

/// Boolean schema properties whose presence changes between releases.
pub(super) const TRACKED_BOOL_ATTRS: &[&str] =
    &["allow_preconfig", "allow_oob", "no_success_response", "simple_union"];

/// Extract version ranges for boolean command attributes present in metadata.
fn extract_bool_attr_versions(
    meta: &Metadata,
    version: Version,
) -> BTreeMap<String, VersionRange> {
    let mut map = BTreeMap::new();
    for attr_name in TRACKED_BOOL_ATTRS {
        if meta
            .attributes
            .iter()
            .any(|a| matches!(a, Attribute::Unit(n) if n == *attr_name))
        {
            map.insert(attr_name.to_string(), VersionRange::new(version));
        }
    }
    map
}

/// Merge boolean command attributes from a new version's struct into an
/// existing UnifiedStruct. Adds newly-appearing attributes, marks removed ones
/// with `until`.
fn merge_bool_attrs(
    existing: &mut UnifiedStruct,
    new_meta: &Metadata,
    version: Version,
) {
    for attr_name in TRACKED_BOOL_ATTRS {
        let present_in_new = new_meta
            .attributes
            .iter()
            .any(|a| matches!(a, Attribute::Unit(n) if n == *attr_name));

        if present_in_new {
            if let Some(range) = existing.attr_versions.get_mut(*attr_name) {
                if range.until.is_some() {
                    let old =
                        std::mem::replace(range, VersionRange::new(version));
                    existing.attr_history.push((attr_name.to_string(), old));
                }
            } else {
                // Newly appeared in this version — add to IR and track
                existing
                    .attr_versions
                    .insert(attr_name.to_string(), VersionRange::new(version));
                let attr = Attribute::new(attr_name);
                if !existing.ir.meta.attributes.contains(&attr) {
                    existing.ir.meta.attributes.push(attr);
                }
            }
        } else if let Some(vr) = existing.attr_versions.get_mut(*attr_name) {
            // Was present before but not in this version — mark removed
            if vr.until.is_none() && vr.since < version {
                vr.until = Some(version);
            }
        }
    }
}

/// Spell a merged type as the newest release does: its QAPI name, and the
/// `name` attribute that the lowering gives it where that differs from the
/// Rust name. The registry merges releases by Rust name, so a renamed type
/// such as `SGXInfo`, which QEMU 10.1 spells `SgxInfo`, is one entry. A name
/// attribute that only the newest spelling needs joins with the release's
/// other new attributes.
fn refresh_name(
    name: &mut String,
    meta: &mut Metadata,
    latest_name: &str,
    latest: &Metadata,
) {
    let wire = |meta: &Metadata| {
        meta.attributes
            .iter()
            .position(|attr| matches!(attr, Attribute::Item { name, .. } if name == "name"))
    };
    latest_name.clone_into(name);
    if let Some(old) = wire(meta) {
        match wire(latest) {
            Some(new) => meta.attributes[old] = latest.attributes[new].clone(),
            None => {
                meta.attributes.remove(old);
            }
        }
    }
}

/// Map from QAPI variant name to its Rust name (for detecting renames).
pub(super) fn variant_rust_name(v: &EnumVariant) -> String {
    crate::qapi_ir::rustify_variant(&v.name)
}

fn same_variant_shape(left: &EnumVariant, right: &EnumVariant) -> bool {
    left.array == right.array
        && match (&left.kind, &right.kind) {
            (EnumVariantKind::Tuple(left), EnumVariantKind::Tuple(right)) => {
                rustify_type(left) == rustify_type(right)
            }
            (left, right) => left == right,
        }
}

/// Materialize old payloads only after merging, so synthetic names never look
/// like schema renames. Each earlier payload shape or wire spelling becomes a
/// variant of its own, named as [`legacy_names`] says from `{Variant}Legacy`.
pub(super) fn materialize_variant_history(
    registry: &mut UnifiedRegistry,
    protocol: &'static str,
) -> Result<(), RegistryError> {
    let releases = registry.versions.clone();
    for (key, value) in registry.types.iter_mut() {
        let UnifiedType::Enum(value) = value else {
            continue;
        };
        let wire = |variant: &EnumVariant| {
            variant
                .meta
                .attributes
                .iter()
                .find_map(|attr| match attr {
                    Attribute::Item { name, value } if name == "name" => {
                        Some(value.clone())
                    }
                    _ => None,
                })
                .unwrap_or_else(|| variant.name.clone())
        };
        let shape = |variant: &EnumVariant| {
            let payload = match &variant.kind {
                EnumVariantKind::Tuple(ty) => Some(rustify_type(ty)),
                EnumVariantKind::Unit => None,
            };
            (wire(variant), payload, variant.array)
        };
        let current = value.ir.variants.iter().map(|variant| {
            let range = &value.variant_versions[&variant.name];
            MemberRun {
                stem: variant_rust_name(variant),
                shape: shape(variant),
                since: range.since,
                until: range.until,
                history: None,
                source: format!("value '{}'", wire(variant)),
            }
        });
        let historical = value.variant_history.iter().enumerate().map(
            |(index, (variant, range))| MemberRun {
                stem: variant_rust_name(variant),
                shape: shape(variant),
                since: range.since,
                until: range.until,
                history: Some(index),
                source: format!("the earlier value '{}'", wire(variant)),
            },
        );
        let runs: Vec<_> = current.chain(historical).collect();
        let used = value
            .ir
            .variants
            .iter()
            .map(|variant| {
                (
                    variant_rust_name(variant),
                    format!("value '{}'", wire(variant)),
                )
            })
            .collect();
        let names = legacy_names(
            &runs,
            &releases,
            used,
            LegacyStyle::Variant,
            protocol,
            &format!("variants of {key}"),
        )?;
        for (index, name) in names {
            let (historical, versions) = &value.variant_history[index];
            let mut variant = historical.clone();
            let wire = wire(&variant);
            variant.name = name.clone();
            variant
                .meta
                .attributes
                .retain(|attr| !matches!(attr, Attribute::Item { name, .. } if name == "name"));
            variant.meta.attributes.push(Attribute::with_value("name", wire));
            value.variant_versions.insert(name, versions.clone());
            value.ir.variants.push(variant);
        }
    }
    Ok(())
}

/// A run of releases in which a member has one shape and wire spelling:
/// the member of the merged IR (`history` `None`), or an earlier run of the
/// type's history.
struct MemberRun<K> {
    /// The member's Rust name without the raw `r#` of a keyword.
    stem: String,
    shape: K,
    since: Version,
    until: Option<Version>,
    history: Option<usize>,
    /// What the run is, for a collision's error.
    source: String,
}

impl<K> MemberRun<K> {
    fn contains(&self, version: Version) -> bool {
        self.since <= version && self.until.is_none_or(|until| version < until)
    }
}

/// How the names of a kind of historical member are spelled.
#[derive(Clone, Copy)]
enum LegacyStyle {
    /// `type_legacy`, `type_legacy2`, `type_legacy_v8_2`.
    Field,
    /// `NoneLegacy`, `NoneLegacy2`, `NoneLegacyV8p2`.
    Variant,
}

impl LegacyStyle {
    fn base(self, stem: &str) -> String {
        match self {
            Self::Field => format!("{stem}_legacy"),
            Self::Variant => format!("{stem}Legacy"),
        }
    }

    /// The name of the `number`th run of a series. A variant's name stays
    /// as `rustify_variant` spells it, which joins the words of underscores
    /// and lowers the letters after a digit: `NoneLegacyV8p2`, `r2` after it.
    fn series(
        self,
        stem: &str,
        (major, minor): (u32, u32),
        number: usize,
    ) -> String {
        let base = self.base(stem);
        match (self, number) {
            (Self::Field, 1) => format!("{base}_v{major}_{minor}"),
            (Self::Field, _) => format!("{base}_v{major}_{minor}_{number}"),
            (Self::Variant, 1) => format!("{base}V{major}p{minor}"),
            (Self::Variant, _) => format!("{base}V{major}p{minor}r{number}"),
        }
    }

    /// The Rust identifier that the generated code spells `name` as.
    fn ident(self, name: &str) -> String {
        match self {
            Self::Field => crate::qapi_ir::rustify_field(name),
            Self::Variant => crate::qapi_ir::rustify_variant(name),
        }
    }
}

/// The first release of each series among `releases`, which opens it: a
/// patch release that a later pin adds to an older series never does. A
/// registry without releases, as unit tests build one, takes every release
/// at which one of `runs` starts or ends.
fn series_openers<K>(
    releases: &[Version],
    runs: &[&MemberRun<K>],
) -> Vec<Version> {
    if releases.is_empty() {
        let mut bounds: BTreeSet<Version> =
            runs.iter().map(|run| run.since).collect();
        bounds.extend(runs.iter().filter_map(|run| run.until));
        return bounds.into_iter().collect();
    }
    let mut openers: Vec<Version> = Vec::new();
    for release in releases {
        if openers
            .last()
            .is_none_or(|last| (last.0, last.1) != (release.0, release.1))
        {
            openers.push(*release);
        }
    }
    openers
}

/// Name the historical runs of members, keyed by their place in the type's
/// history and in its order, so that adding a patch release of an older
/// series to the selection renames none. `used` holds the Rust names of the
/// members of the merged IR, each with what it is.
///
/// The members that share a Rust name are named together. The series-level
/// history of that name is its shape in the release that opens each series
/// ([`series_openers`]); a change of shape there, or an absence, ends a run
/// of it. Those runs, but the one of the member that the IR has, are
/// numbered in release order, and the first run of releases of each takes
/// `{member}_legacy`, `{member}_legacy2` and so on. A run that opens no
/// series, which only patch releases have, and a later run of a
/// series-level run, which such a release interrupts, take the series of
/// their first release: `{member}_legacy_v{major}_{minor}`, then `_2`, `_3`
/// within one series (for variants `{Variant}LegacyV{major}p{minor}`, then
/// `r2`, `r3`). Where a member of the type holds one of the numbered
/// names, such as a QAPI member `foo-legacy` beside the earlier shapes of
/// `foo`, every run takes its series name; where one holds a series name
/// too, the clash is a [`RegistryError::NameCollision`].
fn legacy_names<K: PartialEq>(
    runs: &[MemberRun<K>],
    releases: &[Version],
    mut used: BTreeMap<String, String>,
    style: LegacyStyle,
    protocol: &'static str,
    namespace: &str,
) -> Result<Vec<(usize, String)>, RegistryError> {
    let mut stems: BTreeMap<&str, Vec<&MemberRun<K>>> = BTreeMap::new();
    for run in runs {
        stems.entry(&run.stem).or_default().push(run);
    }
    let mut names = Vec::new();
    for (stem, mut runs) in stems {
        if runs.iter().all(|run| run.history.is_none()) {
            continue;
        }
        runs.sort_by_key(|run| (run.since, run.history));
        // Each series-level run, as the runs that have its openers.
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut previous: Option<usize> = None;
        for opener in series_openers(releases, &runs) {
            let Some(index) = runs.iter().position(|run| run.contains(opener))
            else {
                previous = None;
                continue;
            };
            match (previous, groups.last_mut()) {
                (Some(before), Some(group))
                    if runs[before].shape == runs[index].shape =>
                {
                    if !group.contains(&index) {
                        group.push(index);
                    }
                }
                _ => groups.push(vec![index]),
            }
            previous = Some(index);
        }
        let mut numbered = BTreeMap::new();
        let historical = groups.iter().filter(|group| {
            group.iter().all(|index| runs[*index].history.is_some())
        });
        for (ordinal, group) in historical.enumerate() {
            let base = style.base(stem);
            let name = match ordinal {
                0 => base,
                _ => format!("{base}{}", ordinal + 1),
            };
            numbered.insert(group[0], name);
        }
        // A member of the type that holds one of the numbered names makes
        // every run of the name take its series name.
        if numbered.values().any(|name| used.contains_key(name)) {
            numbered.clear();
        }
        let mut series_numbers: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for (index, run) in runs.iter().enumerate() {
            let Some(history) = run.history else {
                continue;
            };
            let name = match numbered.remove(&index) {
                Some(name) => name,
                None => {
                    let series = (run.since.0, run.since.1);
                    let number = series_numbers.entry(series).or_default();
                    *number += 1;
                    let name = style.series(stem, series, *number);
                    if let Some(first) = used.get(&name) {
                        return Err(RegistryError::NameCollision {
                            protocol,
                            namespace: namespace.to_owned(),
                            name,
                            first: first.clone(),
                            second: run.source.clone(),
                        });
                    }
                    name
                }
            };
            debug_assert_eq!(style.ident(&name), name, "{namespace}");
            used.insert(name.clone(), run.source.clone());
            names.push((history, name));
        }
    }
    names.sort();
    Ok(names)
}

/// Refuse a release in which a type changes between enum and struct, or
/// returns after its removal: one entry with one version range can describe
/// neither. Symbol planning gives each such run of releases its own name.
/// `tag` is the release being merged, and `tags` the first merged tag of each
/// earlier release.
pub(super) fn check_lifetime(
    registry: &UnifiedRegistry,
    protocol: &'static str,
    name: &str,
    enumeration: bool,
    tag: &str,
    tags: &BTreeMap<Version, String>,
) -> Result<(), RegistryError> {
    let (previous, range) = match registry.types.get(name) {
        None => return Ok(()),
        Some(UnifiedType::Enum(value)) => ("enum", &value.versions),
        Some(UnifiedType::Struct(value)) => ("struct", &value.versions),
    };
    let kind = if enumeration { "enum" } else { "struct" };
    if previous != kind {
        return Err(RegistryError::KindChanged {
            protocol,
            name: name.to_owned(),
            previous,
            kind,
            tag: tag.to_owned(),
        });
    }
    match range.until {
        Some(removed) => Err(RegistryError::Reappeared {
            protocol,
            name: name.to_owned(),
            removed: tags[&removed].clone(),
            tag: tag.to_owned(),
        }),
        None => Ok(()),
    }
}

pub(super) fn merge_enum(
    registry: &mut UnifiedRegistry,
    name: &str,
    e: &Enum,
    version: Version,
) {
    if let Some(UnifiedType::Enum(existing)) = registry.types.get_mut(name) {
        existing.ir.meta.schema_item =
            e.meta.schema_item.or(existing.ir.meta.schema_item);
        // Update doc from latest version (latest has best docs)
        merge_doc_from(
            &mut existing.ir.meta.schema_doc,
            e.meta.schema_doc.as_deref(),
        );
        refresh_current_attributes(&mut existing.ir.meta, &e.meta);
        refresh_name(
            &mut existing.ir.name,
            &mut existing.ir.meta,
            &e.name,
            &e.meta,
        );
        // Union enum-level attributes across versions.
        for new_attr in &e.meta.attributes {
            if !existing
                .ir
                .meta
                .attributes
                .iter()
                .any(|a| attrs_equivalent(a, new_attr))
            {
                existing.ir.meta.attributes.push(new_attr.clone());
            }
        }

        let mut seen_variants = BTreeSet::new();
        for variant in &e.variants {
            seen_variants.insert(variant.name.clone());
            if existing.variant_versions.contains_key(&variant.name) {
                let old = existing
                    .ir
                    .variants
                    .iter()
                    .find(|old| old.name == variant.name)
                    .unwrap();
                if !same_variant_shape(old, variant)
                    || existing.variant_versions[&variant.name].until.is_some()
                {
                    let mut old_versions =
                        existing.variant_versions[&variant.name].clone();
                    old_versions.until = old_versions.until.or(Some(version));
                    existing.variant_history.push((old.clone(), old_versions));
                    existing.variant_versions.insert(
                        variant.name.clone(),
                        VersionRange::new(version),
                    );
                }
                // Variant still exists — clear its removal marker.
                let vr =
                    existing.variant_versions.get_mut(&variant.name).unwrap();
                vr.until = None;
                // Promote the variant to the latest version's definition:
                // tuple payload type and array marker track the newest
                // schema; attributes are unioned with the existing set;
                // doc keeps existing unless newer has one.
                let existing_variant = existing
                    .ir
                    .variants
                    .iter_mut()
                    .find(|v| v.name == variant.name)
                    .expect("every variant with a range is in the IR");
                existing_variant.kind = variant.kind.clone();
                existing_variant.array = variant.array;
                merge_docs_from(&mut existing_variant.meta, &variant.meta);
                refresh_current_attributes(
                    &mut existing_variant.meta,
                    &variant.meta,
                );
                for new_attr in &variant.meta.attributes {
                    if !existing_variant
                        .meta
                        .attributes
                        .iter()
                        .any(|a| attrs_equivalent(a, new_attr))
                    {
                        existing_variant.meta.attributes.push(new_attr.clone());
                    }
                }
            } else {
                // Check if this is a rename (same Rust name, different QAPI
                // name)
                let rust_name = variant_rust_name(variant);
                let renamed_from = existing.ir.variants.iter().position(|v| {
                    variant_rust_name(v) == rust_name && v.name != variant.name
                });

                if let Some(old_idx) = renamed_from {
                    let old = existing.ir.variants[old_idx].clone();
                    let mut old_versions =
                        existing.variant_versions.remove(&old.name).unwrap();
                    old_versions.until = old_versions.until.or(Some(version));
                    existing.variant_history.push((old, old_versions));
                    let mut value = variant.clone();
                    clean_docs(&mut value.meta);
                    existing.ir.variants[old_idx] = value;
                    existing.variant_versions.insert(
                        variant.name.clone(),
                        VersionRange::new(version),
                    );
                } else {
                    // A new variant: the IR has exactly the variants with a
                    // range.
                    existing.variant_versions.insert(
                        variant.name.clone(),
                        VersionRange::new(version),
                    );
                    let mut v = variant.clone();
                    clean_docs(&mut v.meta);
                    existing.ir.variants.push(v);
                }
            }
        }

        // Mark variants NOT seen in this version
        for (vname, vr) in existing.variant_versions.iter_mut() {
            if !seen_variants.contains(vname)
                && vr.until.is_none()
                && vr.since < version
            {
                vr.until = Some(version);
            }
        }
    } else {
        let variant_versions: BTreeMap<String, VersionRange> = e
            .variants
            .iter()
            .map(|v| (v.name.clone(), VersionRange::new(version)))
            .collect();
        let mut ir = e.clone();
        clean_docs(&mut ir.meta);
        for v in &mut ir.variants {
            clean_docs(&mut v.meta);
        }
        let versions = VersionRange::new(version);
        registry.types.insert(
            name.to_string(),
            UnifiedType::Enum(UnifiedEnum {
                ir,
                versions,
                variant_versions,
                variant_history: Vec::new(),
            }),
        );
    }
    let Some(UnifiedType::Enum(value)) = registry.types.get_mut(name) else {
        unreachable!("{name} has just been merged as an enum");
    };
    value.versions.track_features(&e.meta, version);
    for variant in &e.variants {
        value
            .variant_versions
            .get_mut(&variant.name)
            .expect("every merged variant has a range")
            .track_features(&variant.meta, version);
    }
}

pub(super) fn field_rust_name(f: &StructField) -> String {
    crate::qapi_ir::rustify_field(&f.name)
}

fn flattened(field: &StructField) -> bool {
    field.meta.attributes.contains(&Attribute::new("flatten"))
}

/// A flattened field, such as a gen:false command's catch-all `arguments`,
/// and a declared member of the same name are different fields: neither
/// takes the other's attributes or documentation.
fn same_field_kind(left: &StructField, right: &StructField) -> bool {
    flattened(left) == flattened(right)
}

fn same_field_shape(left: &StructField, right: &StructField) -> bool {
    same_field_kind(left, right)
        && rustify_type(&left.r#type) == rustify_type(&right.r#type)
        && left.array == right.array
        && left.optional == right.optional
        && (left.optional
            || condition(&left.meta).is_some()
                == condition(&right.meta).is_some())
}

/// Materialize each earlier shape or wire spelling of a field as a field of
/// its own, named as [`legacy_names`] says from `{field}_legacy`.
pub(super) fn materialize_field_history(
    registry: &mut UnifiedRegistry,
    protocol: &'static str,
) -> Result<(), RegistryError> {
    let releases = registry.versions.clone();
    for (key, value) in registry.types.iter_mut() {
        let UnifiedType::Struct(value) = value else {
            continue;
        };
        let wire = |field: &StructField| {
            field
                .meta
                .attributes
                .iter()
                .find_map(|attr| match attr {
                    Attribute::Item { name, value } if name == "name" => {
                        Some(value.clone())
                    }
                    _ => None,
                })
                .unwrap_or_else(|| field.name.clone())
        };
        let shape = |field: &StructField| {
            (
                wire(field),
                rustify_type(&field.r#type),
                field.optional,
                field.array,
                condition(&field.meta).is_some(),
                flattened(field),
            )
        };
        let stem = |field: &StructField| {
            field_rust_name(field).trim_start_matches("r#").to_owned()
        };
        let current = value.ir.fields.iter().map(|field| {
            let range = &value.field_versions[&field.name];
            MemberRun {
                stem: stem(field),
                shape: shape(field),
                since: range.since,
                until: range.until,
                history: None,
                source: format!("member '{}'", wire(field)),
            }
        });
        let historical = value.field_history.iter().enumerate().map(
            |(index, (field, range))| MemberRun {
                stem: stem(field),
                shape: shape(field),
                since: range.since,
                until: range.until,
                history: Some(index),
                source: format!("the earlier member '{}'", wire(field)),
            },
        );
        let runs: Vec<_> = current.chain(historical).collect();
        let used = value
            .ir
            .fields
            .iter()
            .map(|field| {
                (field_rust_name(field), format!("member '{}'", wire(field)))
            })
            .collect();
        let names = legacy_names(
            &runs,
            &releases,
            used,
            LegacyStyle::Field,
            protocol,
            &format!("fields of {key}"),
        )?;
        for (index, name) in names {
            let (historical, versions) = &value.field_history[index];
            let mut field = historical.clone();
            let wire = wire(&field);
            field.name = name.clone();
            field
                .meta
                .attributes
                .retain(|attr| !matches!(attr, Attribute::Item { name, .. } if name == "name"));
            field.meta.attributes.push(Attribute::with_value("name", wire));
            value.field_versions.insert(name, versions.clone());
            value.ir.fields.push(field);
        }
    }
    Ok(())
}

pub(super) fn merge_struct(
    registry: &mut UnifiedRegistry,
    name: &str,
    s: &Struct,
    version: Version,
) {
    if let Some(UnifiedType::Struct(existing)) = registry.types.get_mut(name) {
        existing.ir.meta.schema_item =
            s.meta.schema_item.or(existing.ir.meta.schema_item);
        // Update doc from latest version (latest has best docs)
        merge_doc_from(
            &mut existing.ir.meta.schema_doc,
            s.meta.schema_doc.as_deref(),
        );
        refresh_current_attributes(&mut existing.ir.meta, &s.meta);
        refresh_name(
            &mut existing.ir.name,
            &mut existing.ir.meta,
            &s.name,
            &s.meta,
        );
        // Merge version-tracked boolean command attributes (allow_preconfig,
        // allow_oob, etc.)
        merge_bool_attrs(existing, &s.meta, version);
        merge_returns(existing, &s.meta, version);
        // Union struct-level attributes across versions: keep the older
        // version's attributes and add any new attributes the newer version
        // introduced. Version-tracked attrs, bool attrs and the name are
        // handled separately above.
        for new_attr in &s.meta.attributes {
            if !existing
                .ir
                .meta
                .attributes
                .iter()
                .any(|a| attrs_equivalent(a, new_attr))
            {
                existing.ir.meta.attributes.push(new_attr.clone());
            }
        }

        let mut seen_fields = BTreeSet::new();
        for field in &s.fields {
            seen_fields.insert(field.name.clone());
            if existing.field_versions.contains_key(&field.name) {
                let old = existing
                    .ir
                    .fields
                    .iter()
                    .find(|old| old.name == field.name)
                    .unwrap();
                if !same_field_shape(old, field)
                    || existing.field_versions[&field.name].until.is_some()
                {
                    let mut old_versions =
                        existing.field_versions[&field.name].clone();
                    old_versions.until = old_versions.until.or(Some(version));
                    existing.field_history.push((old.clone(), old_versions));
                    existing
                        .field_versions
                        .insert(field.name.clone(), VersionRange::new(version));
                }
                // Field still exists — clear its removal marker.
                let fr = existing.field_versions.get_mut(&field.name).unwrap();
                fr.until = None;
                // Promote the field to the latest version's definition.
                // Type / optional / array always use the newer value (the
                // latest schema is authoritative for the field shape).
                // Attributes get unioned with the existing set so version-
                // specific directives (e.g. deprecated feature) survive.
                // Doc falls through: newer version wins if it has one,
                // otherwise the older doc is preserved.
                let existing_field = existing
                    .ir
                    .fields
                    .iter_mut()
                    .find(|f| f.name == field.name)
                    .expect("every field with a range is in the IR");
                if !same_field_kind(existing_field, field) {
                    // The other kind was retired into the history above.
                    let mut value = field.clone();
                    clean_docs(&mut value.meta);
                    *existing_field = value;
                    continue;
                }
                existing_field.r#type = field.r#type.clone();
                existing_field.optional = field.optional;
                existing_field.array = field.array;
                merge_docs_from(&mut existing_field.meta, &field.meta);
                refresh_current_attributes(
                    &mut existing_field.meta,
                    &field.meta,
                );
                for new_attr in &field.meta.attributes {
                    if !existing_field
                        .meta
                        .attributes
                        .iter()
                        .any(|a| attrs_equivalent(a, new_attr))
                    {
                        existing_field.meta.attributes.push(new_attr.clone());
                    }
                }
            } else {
                // Check for rename (same Rust field name, different QAPI name)
                let rust_name = field_rust_name(field);
                let renamed_from = existing.ir.fields.iter().position(|f| {
                    field_rust_name(f) == rust_name && f.name != field.name
                });

                if let Some(old_idx) = renamed_from {
                    let old = existing.ir.fields[old_idx].clone();
                    let mut old_versions =
                        existing.field_versions.remove(&old.name).unwrap();
                    old_versions.until = old_versions.until.or(Some(version));
                    existing.field_history.push((old, old_versions));
                    let mut value = field.clone();
                    clean_docs(&mut value.meta);
                    existing.ir.fields[old_idx] = value;
                    existing
                        .field_versions
                        .insert(field.name.clone(), VersionRange::new(version));
                } else {
                    // A new field: the IR has exactly the fields with a range.
                    existing
                        .field_versions
                        .insert(field.name.clone(), VersionRange::new(version));
                    let mut value = field.clone();
                    clean_docs(&mut value.meta);
                    existing.ir.fields.push(value);
                }
            }
        }

        for (fname, fr) in existing.field_versions.iter_mut() {
            if !seen_fields.contains(fname)
                && fr.until.is_none()
                && fr.since < version
            {
                fr.until = Some(version);
            }
        }
    } else {
        // Lowering gave every member of a release its own Rust name.
        let mut ir = s.clone();
        clean_docs(&mut ir.meta);
        let mut field_versions = BTreeMap::new();
        for field in &mut ir.fields {
            clean_docs(&mut field.meta);
            field_versions
                .insert(field.name.clone(), VersionRange::new(version));
        }
        // Track version-sensitive boolean attributes from initial version
        let attr_versions = extract_bool_attr_versions(&ir.meta, version);
        let returns_history = returns_attr(&ir.meta)
            .map(|r| vec![(version, r.to_owned())])
            .unwrap_or_default();
        let versions = VersionRange::new(version);
        registry.types.insert(
            name.to_string(),
            UnifiedType::Struct(UnifiedStruct {
                ir,
                versions,
                field_versions,
                field_history: Vec::new(),
                attr_versions,
                attr_history: Vec::new(),
                returns_history,
                returns_enum: None,
                rust_since: None,
                alias: None,
            }),
        );
    }
    let Some(UnifiedType::Struct(value)) = registry.types.get_mut(name) else {
        unreachable!("{name} has just been merged as a struct");
    };
    value.versions.track_features(&s.meta, version);
    for field in &s.fields {
        value
            .field_versions
            .get_mut(&field.name)
            .expect("every merged field has a range")
            .track_features(&field.meta, version);
    }
}

/// Mark every inline field on a cycle of inline references `boxed`: a value of
/// such a type would contain itself. A cycle can run through struct fields and
/// the payloads of enum variants, that is union branches and alternates;
/// arrays and boxed fields break it. Every inline field of a cycle is marked,
/// not only one per cycle.
pub(super) fn fix_recursive_types(registry: &mut UnifiedRegistry) {
    // Collect (type_name, field_name) pairs that need Box wrapping, then
    // apply mutations after releasing the immutable borrow on registry.
    let needs_box: BTreeSet<(String, String)> = {
        // Only inline references affect Rust size; Vec and Box already break
        // cycles.
        let mut deps: HashMap<String, BTreeSet<String>> = HashMap::new();
        for (name, ut) in &registry.types {
            let mut referenced = BTreeSet::new();
            match ut {
                UnifiedType::Struct(us) => {
                    for field in &us.ir.fields {
                        if !field.array
                            && !field
                                .meta
                                .attributes
                                .contains(&Attribute::new("boxed"))
                        {
                            referenced.insert(rustify_type(&field.r#type));
                        }
                    }
                }
                UnifiedType::Enum(ue) => {
                    for variant in &ue.ir.variants {
                        if !variant.array
                            && let EnumVariantKind::Tuple(t) = &variant.kind
                        {
                            referenced.insert(rustify_type(t));
                        }
                    }
                }
            }
            deps.insert(name.clone(), referenced);
        }

        // Find types that are part of a cycle
        let mut result: BTreeSet<(String, String)> = BTreeSet::new();
        for (name, ut) in &registry.types {
            if let UnifiedType::Struct(us) = ut {
                for field in &us.ir.fields {
                    let referenced = rustify_type(&field.r#type);
                    if !field.array
                        && !field
                            .meta
                            .attributes
                            .contains(&Attribute::new("boxed"))
                        && is_reachable(
                            &referenced,
                            name,
                            &deps,
                            &mut BTreeSet::new(),
                        )
                    {
                        result.insert((name.clone(), field.name.clone()));
                    }
                }
            }
        }
        result
    };

    // Apply Box wrapping by setting a flag on the field
    for (type_name, field_name) in &needs_box {
        if let Some(UnifiedType::Struct(us)) = registry.types.get_mut(type_name)
        {
            for field in &mut us.ir.fields {
                if field.name == *field_name {
                    field.meta.attributes.push(Attribute::new("boxed"));
                }
            }
        }
    }
}

/// Check if `target` is reachable from `start` through the dependency graph.
fn is_reachable<'a>(
    start: &'a str,
    target: &str,
    deps: &'a HashMap<String, BTreeSet<String>>,
    visited: &mut BTreeSet<&'a str>,
) -> bool {
    if start == target {
        return true;
    }
    if !visited.insert(start) {
        return false;
    }
    if let Some(refs) = deps.get(start) {
        for r in refs {
            if is_reachable(r, target, deps, visited) {
                return true;
            }
        }
    }
    false
}

/// Whether a type holds `version`: it is in its lifetime.
fn has_release(value: &UnifiedType, version: Version) -> bool {
    let range = match value {
        UnifiedType::Enum(value) => &value.versions,
        UnifiedType::Struct(value) => &value.versions,
    };
    range.since <= version && range.until.is_none_or(|until| version < until)
}

/// The deepest nesting of struct members that [`same_on_the_wire`]
/// compares; deeper types must have one Rust name.
const WIRE_DEPTH: usize = 8;

/// Whether the type `left` in release `left_at` and `right` in `right_at`
/// are the same on the wire: one Rust type, or plain structs whose members
/// in those releases have the same wire names, optionality, arrays and
/// build conditions, and types that are the same on the wire in turn.
fn same_on_the_wire(
    registry: &UnifiedRegistry,
    (left, left_at): (&str, Version),
    (right, right_at): (&str, Version),
    depth: usize,
) -> bool {
    if left == right {
        return true;
    }
    let (
        Some(UnifiedType::Struct(left_type)),
        Some(UnifiedType::Struct(right_type)),
    ) = (registry.types.get(left), registry.types.get(right))
    else {
        return false;
    };
    let plain = |value: &UnifiedStruct| {
        classify_type(&UnifiedType::Struct(value.clone()))
            == TypeCategory::DataType
            && value.attr_versions.is_empty()
            && !value.ir.fields.iter().any(flattened)
    };
    if depth == 0
        || !plain(left_type)
        || !plain(right_type)
        || !has_release(&registry.types[left], left_at)
        || !has_release(&registry.types[right], right_at)
    {
        return false;
    }
    fn members(
        value: &UnifiedStruct,
        at: Version,
    ) -> BTreeMap<String, &StructField> {
        value
            .ir
            .fields
            .iter()
            .filter(|field| {
                value.field_versions.get(&field.name).is_some_and(|range| {
                    range.since <= at
                        && range.until.is_none_or(|until| at < until)
                })
            })
            .map(|field| {
                let wire = field
                    .meta
                    .attributes
                    .iter()
                    .find_map(|attr| match attr {
                        Attribute::Item { name, value } if name == "name" => {
                            Some(value.clone())
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| field.name.clone());
                (wire, field)
            })
            .collect()
    }
    let (left_members, right_members) =
        (members(left_type, left_at), members(right_type, right_at));
    left_members.len() == right_members.len()
        && left_members.iter().all(|(wire, left)| {
            right_members.get(wire).is_some_and(|right| {
                left.optional == right.optional
                    && left.array == right.array
                    && condition(&left.meta).is_some()
                        == condition(&right.meta).is_some()
                    && same_on_the_wire(
                        registry,
                        (&rustify_type(&left.r#type), left_at),
                        (&rustify_type(&right.r#type), right_at),
                        depth - 1,
                    )
            })
        })
}

/// Coalesce the wire-identical successors of a command's return types.
///
/// Take consecutive return types `A`, from release `a`, and `B`, from `b`,
/// where `B` is a struct that `b` introduces. Let `r` be the earliest release
/// at or after `a` such that `A` in every merged release of `[r, b)` is the
/// same on the wire as `B` in `b`, and `A` from `b` until its removal as
/// `B` in each of those releases. Where `r` is before `b`, `B` returns
/// from `r` and its Rust lifetime starts there, while its metadata keeps
/// QAPI's; `A`'s return type ends there, gone if `r` is `a`; and `A`'s
/// documentation says that it is the same on the wire as `B` from `r`.
/// `query-target` returns `TargetInfo` until 10.1 and `QemuTargetInfo`
/// from then, which is `TargetInfo` from 3.0, when its `arch` became a
/// `SysEmuTarget`.
///
/// A type that two coalescings would move or alias in two ways is a
/// [`RegistryError::CoalesceAmbiguous`].
pub(super) fn coalesce_returns(
    registry: &mut UnifiedRegistry,
    protocol: &'static str,
) -> Result<(), RegistryError> {
    let mut releases = registry.versions.clone();
    releases.sort();
    releases.dedup();
    // (command, index of B's entry, A, B, r)
    let mut plans: Vec<(String, usize, String, String, Version)> = Vec::new();
    for (key, value) in &registry.types {
        let UnifiedType::Struct(command) = value else {
            continue;
        };
        for (index, pair) in command.returns_history.windows(2).enumerate() {
            let [(a, before), (b, after)] = pair else {
                unreachable!("windows of two")
            };
            let (before, before_array) =
                crate::metadata_emit::returns_element(before);
            let (after, after_array) =
                crate::metadata_emit::returns_element(after);
            let Some(UnifiedType::Struct(successor)) =
                registry.types.get(after)
            else {
                continue;
            };
            let Some(UnifiedType::Struct(predecessor)) =
                registry.types.get(before)
            else {
                continue;
            };
            if before_array != after_array || successor.versions.since != *b {
                continue;
            }
            let same = |at: Version, other: Version| {
                same_on_the_wire(
                    registry,
                    (before, at),
                    (after, other),
                    WIRE_DEPTH,
                )
            };
            // After `b`, the alias stands for `B` until `A`'s removal.
            let later = releases.iter().filter(|release| {
                **release >= *b
                    && predecessor
                        .versions
                        .until
                        .is_none_or(|until| **release < until)
            });
            if !later.clone().all(|release| same(*release, *release)) {
                continue;
            }
            let mut from = *b;
            for release in releases
                .iter()
                .rev()
                .filter(|release| (*a..*b).contains(*release))
            {
                if !same(*release, *b) {
                    break;
                }
                from = *release;
            }
            if from < *b {
                plans.push((
                    key.clone(),
                    index + 1,
                    before.to_owned(),
                    after.to_owned(),
                    from,
                ));
            }
        }
    }
    let describe = |(_, _, before, after, from): &(
        String,
        usize,
        String,
        String,
        Version,
    )| {
        format!("{before} -> {after} from {}", version_str(*from))
    };
    for (index, plan) in plans.iter().enumerate() {
        for other in &plans[..index] {
            let (moves, aliases) = (plan.3 == other.3, plan.2 == other.2);
            if (moves || aliases)
                && (&plan.2, &plan.3, plan.4) != (&other.2, &other.3, other.4)
            {
                return Err(RegistryError::CoalesceAmbiguous {
                    protocol,
                    name: if moves { plan.3.clone() } else { plan.2.clone() },
                    first: describe(other),
                    second: describe(plan),
                });
            }
        }
    }
    for (command, index, before, after, from) in plans {
        if let Some(UnifiedType::Struct(value)) =
            registry.types.get_mut(&command)
        {
            value.returns_history[index].0 = from;
            if value.returns_history[index - 1].0 == from {
                value.returns_history.remove(index - 1);
            }
            value
                .returns_history
                .dedup_by(|later, earlier| later.1 == earlier.1);
        }
        if let Some(UnifiedType::Struct(value)) = registry.types.get_mut(&after)
        {
            value.rust_since = Some(from);
        }
        if let Some(UnifiedType::Struct(value)) =
            registry.types.get_mut(&before)
        {
            value.alias = Some((from, after.clone()));
        }
        registry.coalesced.push((command, before, after, from));
    }
    Ok(())
}
