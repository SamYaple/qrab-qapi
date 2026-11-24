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
pub(super) const TRACKED_BOOL_ATTRS: &[&str] = &[
    "allow_preconfig",
    "allow_oob",
    "no_success_response",
    "simple_union",
];

/// Extract version ranges for boolean command attributes present in metadata.
fn extract_bool_attr_versions(meta: &Metadata, version: Version) -> BTreeMap<String, VersionRange> {
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

/// Merge boolean command attributes from a new version's struct into an existing UnifiedStruct.
/// Adds newly-appearing attributes, marks removed ones with `until`.
fn merge_bool_attrs(existing: &mut UnifiedStruct, new_meta: &Metadata, version: Version) {
    for attr_name in TRACKED_BOOL_ATTRS {
        let present_in_new = new_meta
            .attributes
            .iter()
            .any(|a| matches!(a, Attribute::Unit(n) if n == *attr_name));

        if present_in_new {
            if let Some(range) = existing.attr_versions.get_mut(*attr_name) {
                if range.until.is_some() {
                    let old = std::mem::replace(range, VersionRange::new(version));
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
fn refresh_name(name: &mut String, meta: &mut Metadata, latest_name: &str, latest: &Metadata) {
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

/// Materialize old payloads only after merging, so synthetic names never look like schema renames.
pub(super) fn materialize_variant_history(registry: &mut UnifiedRegistry) {
    for value in registry.types.values_mut() {
        let UnifiedType::Enum(value) = value else {
            continue;
        };
        let mut used: BTreeSet<_> = value.ir.variants.iter().map(variant_rust_name).collect();
        for (historical, versions) in &value.variant_history {
            let mut variant = historical.clone();
            let wire = variant
                .meta
                .attributes
                .iter()
                .find_map(|attr| match attr {
                    Attribute::Item { name, value } if name == "name" => Some(value.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| variant.name.clone());
            let base = format!("{}Legacy", variant_rust_name(&variant));
            let mut name = base.clone();
            let mut suffix = 2;
            while !used.insert(name.clone()) {
                name = format!("{base}{suffix}");
                suffix += 1;
            }
            variant.name = name.clone();
            variant
                .meta
                .attributes
                .retain(|attr| !matches!(attr, Attribute::Item { name, .. } if name == "name"));
            variant
                .meta
                .attributes
                .push(Attribute::with_value("name", wire));
            value.variant_versions.insert(name, versions.clone());
            value.ir.variants.push(variant);
        }
    }
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

pub(super) fn merge_enum(registry: &mut UnifiedRegistry, name: &str, e: &Enum, version: Version) {
    if let Some(UnifiedType::Enum(existing)) = registry.types.get_mut(name) {
        existing.ir.meta.schema_item = e.meta.schema_item.or(existing.ir.meta.schema_item);
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
                    let mut old_versions = existing.variant_versions[&variant.name].clone();
                    old_versions.until = old_versions.until.or(Some(version));
                    existing.variant_history.push((old.clone(), old_versions));
                    existing
                        .variant_versions
                        .insert(variant.name.clone(), VersionRange::new(version));
                }
                // Variant still exists — clear its removal marker.
                let vr = existing.variant_versions.get_mut(&variant.name).unwrap();
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
                refresh_current_attributes(&mut existing_variant.meta, &variant.meta);
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
                // Check if this is a rename (same Rust name, different QAPI name)
                let rust_name = variant_rust_name(variant);
                let renamed_from = existing
                    .ir
                    .variants
                    .iter()
                    .position(|v| variant_rust_name(v) == rust_name && v.name != variant.name);

                if let Some(old_idx) = renamed_from {
                    let old = existing.ir.variants[old_idx].clone();
                    let mut old_versions = existing.variant_versions.remove(&old.name).unwrap();
                    old_versions.until = old_versions.until.or(Some(version));
                    existing.variant_history.push((old, old_versions));
                    let mut value = variant.clone();
                    clean_docs(&mut value.meta);
                    existing.ir.variants[old_idx] = value;
                    existing
                        .variant_versions
                        .insert(variant.name.clone(), VersionRange::new(version));
                } else {
                    // A new variant: the IR has exactly the variants with a range.
                    existing
                        .variant_versions
                        .insert(variant.name.clone(), VersionRange::new(version));
                    let mut v = variant.clone();
                    clean_docs(&mut v.meta);
                    existing.ir.variants.push(v);
                }
            }
        }

        // Mark variants NOT seen in this version
        for (vname, vr) in existing.variant_versions.iter_mut() {
            if !seen_variants.contains(vname) && vr.until.is_none() && vr.since < version {
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

fn same_field_shape(left: &StructField, right: &StructField) -> bool {
    rustify_type(&left.r#type) == rustify_type(&right.r#type)
        && left.array == right.array
        && left.optional == right.optional
        && (left.optional || condition(&left.meta).is_some() == condition(&right.meta).is_some())
}

pub(super) fn materialize_field_history(registry: &mut UnifiedRegistry) {
    for value in registry.types.values_mut() {
        let UnifiedType::Struct(value) = value else {
            continue;
        };
        let mut used: BTreeSet<_> = value.ir.fields.iter().map(field_rust_name).collect();
        for (historical, versions) in &value.field_history {
            let mut field = historical.clone();
            let wire = field
                .meta
                .attributes
                .iter()
                .find_map(|attr| match attr {
                    Attribute::Item { name, value } if name == "name" => Some(value.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| field.name.clone());
            let base = format!(
                "{}_legacy",
                field_rust_name(&field).trim_start_matches("r#")
            );
            let mut name = base.clone();
            let mut suffix = 2;
            while !used.insert(name.clone()) {
                name = format!("{base}{suffix}");
                suffix += 1;
            }
            field.name = name.clone();
            field
                .meta
                .attributes
                .retain(|attr| !matches!(attr, Attribute::Item { name, .. } if name == "name"));
            field
                .meta
                .attributes
                .push(Attribute::with_value("name", wire));
            value.field_versions.insert(name, versions.clone());
            value.ir.fields.push(field);
        }
    }
}

pub(super) fn merge_struct(
    registry: &mut UnifiedRegistry,
    name: &str,
    s: &Struct,
    version: Version,
) {
    if let Some(UnifiedType::Struct(existing)) = registry.types.get_mut(name) {
        existing.ir.meta.schema_item = s.meta.schema_item.or(existing.ir.meta.schema_item);
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
        // Merge version-tracked boolean command attributes (allow_preconfig, allow_oob, etc.)
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
                    let mut old_versions = existing.field_versions[&field.name].clone();
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
                existing_field.r#type = field.r#type.clone();
                existing_field.optional = field.optional;
                existing_field.array = field.array;
                merge_docs_from(&mut existing_field.meta, &field.meta);
                refresh_current_attributes(&mut existing_field.meta, &field.meta);
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
                let renamed_from = existing
                    .ir
                    .fields
                    .iter()
                    .position(|f| field_rust_name(f) == rust_name && f.name != field.name);

                if let Some(old_idx) = renamed_from {
                    let old = existing.ir.fields[old_idx].clone();
                    let mut old_versions = existing.field_versions.remove(&old.name).unwrap();
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
            if !seen_fields.contains(fname) && fr.until.is_none() && fr.since < version {
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
            field_versions.insert(field.name.clone(), VersionRange::new(version));
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
        // Only inline references affect Rust size; Vec and Box already break cycles.
        let mut deps: HashMap<String, BTreeSet<String>> = HashMap::new();
        for (name, ut) in &registry.types {
            let mut referenced = BTreeSet::new();
            match ut {
                UnifiedType::Struct(us) => {
                    for field in &us.ir.fields {
                        if !field.array && !field.meta.attributes.contains(&Attribute::new("boxed"))
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
                        && !field.meta.attributes.contains(&Attribute::new("boxed"))
                        && is_reachable(&referenced, name, &deps, &mut BTreeSet::new())
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
        if let Some(UnifiedType::Struct(us)) = registry.types.get_mut(type_name) {
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
