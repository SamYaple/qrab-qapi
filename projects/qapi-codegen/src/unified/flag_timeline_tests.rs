use super::{
    lower::{
        extract_version_sources, load_versions, lower_versions, parse_versions,
    },
    merge::{TRACKED_BOOL_ATTRS, has_feature},
    registry::{build_qga_registry, build_qmp_registry, merge_releases},
    *,
};

fn deprecation_recorded<'a>(
    ranges: impl Iterator<Item = &'a VersionRange>,
    version: Version,
) -> bool {
    ranges
        .filter(|range| {
            version >= range.since
                && range.until.is_none_or(|until| version < until)
        })
        .any(|range| range.deprecated.is_some_and(|since| since <= version))
}

#[test]
#[ignore = "compares every pinned QMP/QGA flag against its source release"]
fn boolean_flag_ranges_match_each_source_release() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/qemu");
    let pins = crate::versions::parse_pinned_versions(
        crate::versions::PINNED_VERSIONS,
    )
    .unwrap();
    let tags: Vec<_> = pins.iter().map(|pin| pin.tag.as_str()).collect();
    let mut mismatches = Vec::new();
    for qga in [false, true] {
        let sources = load_versions(
            &repo,
            &tags,
            if qga { Protocol::Qga } else { Protocol::Qmp },
        )
        .unwrap();
        let registry = if qga {
            build_qga_registry(&repo, &tags).unwrap()
        } else {
            build_qmp_registry(&repo, &tags).unwrap()
        };
        for source in sources {
            for (name, value) in &registry.types {
                let present = source.structs.contains_key(name)
                    || source.enums.contains_key(name);
                let range = match value {
                    UnifiedType::Enum(value) => &value.versions,
                    UnifiedType::Struct(value) => &value.versions,
                };
                let active = source.version >= range.since
                    && range.until.is_none_or(|until| source.version < until);
                if present != active {
                    mismatches.push(format!(
                        "{} {name}: definition present={present}, merged availability={active}",
                        version_str(source.version)
                    ));
                }
                if let UnifiedType::Struct(value) = value {
                    let command = Attribute::new("command");
                    let expected =
                        source.structs.get(name).is_some_and(|value| {
                            value.meta.attributes.contains(&command)
                        });
                    let actual =
                        active && value.ir.meta.attributes.contains(&command);
                    if expected != actual {
                        mismatches.push(format!(
                            "{} {name}: command binding present={expected}, merged={actual}",
                            version_str(source.version)
                        ));
                    }
                }
            }
            for (name, value) in source.structs {
                let UnifiedType::Struct(merged) = &registry.types[&name] else {
                    panic!()
                };
                if has_feature(&value.meta, "unstable")
                    != merged.versions.is_unstable(source.version)
                {
                    mismatches.push(format!(
                        "{} {name}: unstable feature mismatch",
                        version_str(source.version)
                    ));
                }
                for flag in TRACKED_BOOL_ATTRS {
                    let expected =
                        value.meta.attributes.contains(&Attribute::new(flag));
                    let actual = merged.flag_enabled(flag, source.version);
                    if expected != actual {
                        mismatches.push(format!(
                            "{} {name}.{flag}: source={expected}, merged={actual}",
                            version_str(source.version)
                        ));
                    }
                }
                for field in &value.fields {
                    if has_feature(&field.meta, "deprecated") {
                        let ranges = merged
                            .field_versions
                            .get(&field.name)
                            .into_iter()
                            .chain(merged.field_history.iter().filter_map(
                                |(old, range)| {
                                    (old.name == field.name).then_some(range)
                                },
                            ));
                        if !deprecation_recorded(ranges, source.version) {
                            mismatches.push(format!(
                                "{} {name}.{}: missing or late deprecation date",
                                version_str(source.version),
                                field.name
                            ));
                        }
                    }
                }
            }
            for (name, value) in source.enums {
                let UnifiedType::Enum(merged) = &registry.types[&name] else {
                    panic!()
                };
                if has_feature(&value.meta, "unstable")
                    != merged.versions.is_unstable(source.version)
                {
                    mismatches.push(format!(
                        "{} {name}: unstable feature mismatch",
                        version_str(source.version)
                    ));
                }
                for variant in &value.variants {
                    if has_feature(&variant.meta, "deprecated") {
                        let ranges = merged
                            .variant_versions
                            .get(&variant.name)
                            .into_iter()
                            .chain(merged.variant_history.iter().filter_map(
                                |(old, range)| {
                                    (old.name == variant.name).then_some(range)
                                },
                            ));
                        if !deprecation_recorded(ranges, source.version) {
                            mismatches.push(format!(
                                "{} {name}.{}: missing or late deprecation date",
                                version_str(source.version),
                                variant.name
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// The commands and events that a release's schema, as the parser reads it
/// before any lowering, marks with `feature`, by wire name.
fn featured(
    sources: &[(std::path::PathBuf, String)],
    version: Version,
    feature: &str,
) -> BTreeSet<String> {
    let mut featured = BTreeSet::new();
    for (path, text) in sources {
        let schema = crate::parse_schema_with_version(text, version)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        for token in &schema.0 {
            let (name, features) = match token {
                QapiSchemaToken::Command(value) => {
                    (&value.name, &value.features)
                }
                QapiSchemaToken::Event(value) => (&value.name, &value.features),
                _ => continue,
            };
            if features
                .iter()
                .flat_map(|features| &features.0)
                .any(|value| value.name.as_ref() == feature)
            {
                featured.insert(name.as_ref().to_owned());
            }
        }
    }
    featured
}

/// The discriminator of each union of a release's schema, as the parser
/// reads it, by wire name: `type` for a simple union.
fn discriminators(
    sources: &[(std::path::PathBuf, String)],
    version: Version,
) -> BTreeMap<String, String> {
    let mut discriminators = BTreeMap::new();
    for (path, text) in sources {
        let schema = crate::parse_schema_with_version(text, version)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        for token in &schema.0 {
            if let QapiSchemaToken::Union(value) = token
                && !value.legacy_alternate
            {
                let discriminator =
                    value.discriminator.as_ref().map_or("type", AsRef::as_ref);
                discriminators.insert(
                    value.name.as_ref().to_owned(),
                    discriminator.to_owned(),
                );
            }
        }
    }
    discriminators
}

/// The wire name of an item or member of the registry.
fn wire_name<'a>(name: &'a str, meta: &'a Metadata) -> &'a str {
    meta.attributes
        .iter()
        .find_map(|attr| match attr {
            Attribute::Item { name, value } if name == "name" => {
                Some(value.as_str())
            }
            _ => None,
        })
        .unwrap_or(name)
}

/// The registry's structs of `category`, by wire name.
fn structs_of(
    registry: &UnifiedRegistry,
    category: TypeCategory,
) -> BTreeMap<&str, &UnifiedStruct> {
    registry
        .types
        .values()
        .filter(|value| classify_type(value) == category)
        .filter_map(|value| match value {
            UnifiedType::Struct(value) => {
                Some((wire_name(&value.ir.name, &value.ir.meta), value))
            }
            UnifiedType::Enum(_) => None,
        })
        .collect()
}

/// The deprecation and instability of every command and event, and the
/// discriminator of every union, in every pinned release, are those of its
/// definition in the release's schema, which the lowering does not touch: a
/// command that shares its Rust name with its argument struct is one entry
/// with the struct, and a union may change its discriminator while both
/// members exist, which only this comparison sees.
#[test]
#[ignore = "compares every pinned QMP/QGA command, event and union with its source schema"]
fn commands_events_and_unions_match_each_source_schema() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/qemu");
    let pins = crate::versions::parse_pinned_versions(
        crate::versions::PINNED_VERSIONS,
    )
    .unwrap();
    let tags: Vec<_> = pins.iter().map(|pin| pin.tag.as_str()).collect();
    let mut mismatches = Vec::new();
    let (mut checked, mut unions) = (0, 0);
    let live = |range: &VersionRange, version: Version| {
        version >= range.since
            && range.until.is_none_or(|until| version < until)
    };
    for protocol in [Protocol::Qmp, Protocol::Qga] {
        let name = protocol.name();
        let sources: Vec<_> = parse_versions(&tags, |tag| {
            extract_version_sources(&repo, tag, protocol)
        })
        .unwrap()
        .into_iter()
        .flatten()
        .collect();
        let expected: Vec<_> = sources
            .iter()
            .map(|source| {
                let feature =
                    |name| featured(&source.sources, source.version, name);
                (
                    source.version,
                    feature("deprecated"),
                    feature("unstable"),
                    discriminators(&source.sources, source.version),
                )
            })
            .collect();
        let registry =
            merge_releases(name, lower_versions(&tags, sources).unwrap())
                .unwrap();
        let mut merged = structs_of(&registry, TypeCategory::Command);
        merged.extend(structs_of(&registry, TypeCategory::Event));
        let data = structs_of(&registry, TypeCategory::DataType);
        for (version, deprecated, unstable, discriminators) in &expected {
            let release = version_str(*version);
            for (wire, value) in &merged {
                if !live(&value.versions, *version) {
                    continue;
                }
                checked += 1;
                for (feature, source, merged) in [
                    (
                        "deprecated",
                        deprecated,
                        value.versions.is_deprecated(*version),
                    ),
                    (
                        "unstable",
                        unstable,
                        value.versions.is_unstable(*version),
                    ),
                ] {
                    if source.contains(*wire) != merged {
                        mismatches.push(format!(
                            "{name} {release} {wire}: {feature} in the schema={}, merged={merged}",
                            source.contains(*wire),
                        ));
                    }
                }
            }
            for wire in deprecated.iter().chain(unstable) {
                if !merged.contains_key(wire.as_str()) {
                    mismatches.push(format!(
                        "{name} {release}: no command or event {wire}"
                    ));
                }
            }
            for (union, discriminator) in discriminators {
                let Some(value) = data.get(union.as_str()) else {
                    mismatches
                        .push(format!("{name} {release}: no union {union}"));
                    continue;
                };
                unions += 1;
                let merged: Vec<_> = value
                    .ir
                    .fields
                    .iter()
                    .filter(|field| {
                        value.field_versions.get(&field.name).is_some_and(
                            |range| {
                                live(range, *version)
                                    && range.is_discriminator(*version)
                            },
                        )
                    })
                    .map(|field| wire_name(&field.name, &field.meta))
                    .collect();
                if merged != [discriminator.as_str()] {
                    mismatches.push(format!(
                        "{name} {release} {union}: discriminator {discriminator} in the schema, \
                         merged {merged:?}"
                    ));
                }
            }
        }
    }
    assert!(checked > 0 && unions > 0, "nothing was compared");
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
