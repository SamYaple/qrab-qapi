//! Per-release extraction, parsing and lowering to the Rust IR.

use super::error::{SchemaResult, definition, require};
use super::merge::{field_rust_name, variant_rust_name};
use super::*;
use crate::symbols::synthesized;
use std::collections::HashSet;
use std::path::PathBuf;

/// Collect the symbols of one release, with every type that its era
/// predefines, and the predefined types that it uses.
fn plan_version(
    sources: &[(PathBuf, String)],
) -> SchemaResult<(crate::symbols::SchemaNames, BTreeSet<&'static str>)> {
    let schemas = crate::builtins::parse_sources(sources, &crate::builtins::PREDEFINED.into())?;
    let used = crate::builtins::used_predefined(&schemas);
    let mut symbols = crate::symbols::SchemaNames::default();
    for (_, schema) in &schemas {
        symbols.collect(schema);
    }
    Ok((symbols, used))
}

/// The name a definition's documentation block gives, with the definition's own.
fn documented_symbol<'a>(token: &'a QapiSchemaToken<'_>) -> Option<(&'a str, &'a str)> {
    let (name, doc) = match token {
        QapiSchemaToken::Enum(value) => (value.name.as_ref(), &value.doc),
        QapiSchemaToken::Struct(value) => (value.name.as_ref(), &value.doc),
        QapiSchemaToken::Union(value) => (value.name.as_ref(), &value.doc),
        QapiSchemaToken::Alternate(value) => (value.name.as_ref(), &value.doc),
        QapiSchemaToken::Command(value) => (value.name.as_ref(), &value.doc),
        QapiSchemaToken::Event(value) => (value.name.as_ref(), &value.doc),
        _ => return None,
    };
    Some((name, doc.as_ref()?.name))
}

/// Lower a struct and the anonymous structs of its members into `structs`.
fn add_struct(
    value: crate::QapiStruct<'_>,
    file: Option<&Path>,
    structs: &mut HashMap<String, Struct>,
    names: &mut HashSet<String>,
) -> SchemaResult {
    let label = format!("struct '{}'", value.name);
    let (processed, extracted) = process_struct(value, structs, names)
        .map_err(|error| SchemaError::from(error).located(Some(label), file))?;
    for value in std::iter::once(processed).chain(extracted) {
        structs.insert(value.name.clone(), value);
    }
    Ok(())
}

/// Lower one release with the predefined types of `predefined`, adding the
/// quirks it tolerates to `warnings`.
pub(super) fn process_version_schema_with_names(
    sources: &[(PathBuf, String)],
    symbols: &crate::symbols::SchemaNames,
    predefined: &BTreeSet<&str>,
    warnings: &mut Vec<WarningKind>,
) -> SchemaResult<(HashMap<String, Enum>, HashMap<String, Struct>)> {
    let mut enums_lookup = HashMap::new();
    let mut structs_lookup = HashMap::new();
    let mut unprocessed_structs = Vec::new();
    let mut unprocessed_unions = Vec::new();
    let mut unprocessed_commands = Vec::new();
    let mut unprocessed_events = Vec::new();
    let mut definitions = HashMap::new();
    let schemas = crate::builtins::parse_sources(sources, predefined)?;
    let declared = crate::symbols::SchemaNames::declared_names(&schemas);
    // Deferred definitions are lowered after the loop over their files.
    let locations: HashMap<_, _> = schemas
        .iter()
        .flat_map(|(path, schema)| {
            schema
                .0
                .iter()
                .filter_map(move |token| Some((definition(token)?, *path)))
        })
        .collect();
    let locate = |label: String| {
        let file = locations.get(&label).copied();
        move |error: SchemaError| error.located(Some(label.clone()), file)
    };
    let mut names = symbols.names.clone();
    let mut union_names = HashMap::new();
    let mut event_names = HashMap::new();
    let mut command_names = HashMap::new();
    for (_, schema) in &schemas {
        for token in &schema.0 {
            match token {
                QapiSchemaToken::Union(value) if !value.legacy_alternate => {
                    let root = symbols.type_name(&value.name);
                    let branch = synthesized(&symbols.role(&format!("{root}Branch")));
                    let discriminator =
                        synthesized(&symbols.type_name(&crate::symbols::implicit_tag(&value.name)));
                    union_names.insert(value.name.to_string(), (branch, discriminator));
                }
                QapiSchemaToken::Event(value) => {
                    let name = synthesized(&symbols.event_name(&value.name));
                    event_names.insert(value.name.to_string(), name);
                }
                QapiSchemaToken::Command(value) => {
                    let name =
                        symbols.role(&format!("{}Command", symbols.command_root(&value.name)));
                    command_names.insert(value.name.to_string(), name);
                }
                _ => {}
            }
        }
    }

    // Resolve exact QAPI names before assigning Rust identities for merging.
    for (path, schema) in schemas {
        for token in schema.0 {
            let label = definition(&token);
            let at = |error: SchemaError| error.located(label.clone(), Some(path));
            if let (Some((name, documented)), Some(label)) = (documented_symbol(&token), &label)
                && !documented.eq_ignore_ascii_case(name)
            {
                warnings.push(WarningKind::MislabeledDocumentation {
                    definition: label.clone(),
                    documented: documented.to_owned(),
                });
            }
            let namespace = match &token {
                QapiSchemaToken::Event(_) => "event",
                QapiSchemaToken::Command(_) => "command",
                _ => "type",
            };
            let mut defined = Vec::new();
            if let Some(name) = token.definition_name() {
                let symbol = if namespace == "type" {
                    name.to_owned()
                } else {
                    crate::schema_names::c_name(name)
                };
                defined.push((symbol, name));
                // A simple union defines its tag enum as well.
                if let QapiSchemaToken::Union(value) = &token
                    && !value.legacy_alternate
                    && value.discriminator.is_none()
                {
                    defined.push((crate::symbols::implicit_tag(name), name));
                }
            }
            for (symbol, name) in defined {
                if let Some(previous) =
                    definitions.insert((namespace, symbol.clone()), name.to_owned())
                {
                    return Err(at(SchemaErrorKind::SymbolClash {
                        namespace,
                        symbol,
                        previous,
                        name: name.to_owned(),
                    }
                    .into()));
                }
            }
            match token {
                QapiSchemaToken::Enum(v) => {
                    let processed = process_enum(v);
                    enums_lookup.insert(processed.name.clone(), processed);
                }
                QapiSchemaToken::Alternate(v) => {
                    let processed = process_alternate(v).map_err(|e| at(e.into()))?;
                    enums_lookup.insert(processed.name.clone(), processed);
                }
                QapiSchemaToken::Struct(v) => {
                    if let Some(base) = v.base.clone() {
                        unprocessed_structs.push((v, base));
                        continue;
                    }
                    add_struct(v, Some(path), &mut structs_lookup, &mut names)?;
                }
                QapiSchemaToken::Union(value) if value.legacy_alternate => {
                    let processed = process_legacy_alternate(value).map_err(|e| at(e.into()))?;
                    enums_lookup.insert(processed.name.clone(), processed);
                }
                QapiSchemaToken::Union(v) => {
                    unprocessed_unions.push(v);
                }
                QapiSchemaToken::Command(v) => {
                    unprocessed_commands.push(v);
                }
                QapiSchemaToken::Event(v) => {
                    unprocessed_events.push(v);
                }
                _ => continue,
            }
        }
    }

    // A derived struct may precede its base, including across include files.
    while !unprocessed_structs.is_empty() {
        let count = unprocessed_structs.len();
        let mut pending = Vec::new();
        for (value, base) in unprocessed_structs {
            if !structs_lookup.contains_key::<str>(base.as_ref()) {
                pending.push((value, base));
                continue;
            }
            let file = locations.get(&format!("struct '{}'", value.name)).copied();
            add_struct(value, file, &mut structs_lookup, &mut names)?;
        }
        require(pending.len() < count, || SchemaErrorKind::UnresolvedBases {
            pending: pending
                .iter()
                .map(|(value, base)| (value.name.to_string(), base.to_string()))
                .collect(),
        })
        .map_err(|error| locate(format!("struct '{}'", pending[0].0.name))(error))?;
        unprocessed_structs = pending;
    }
    for v in unprocessed_unions {
        let at = locate(format!("union '{}'", v.name));
        if let Some(discriminator) = v.discriminator.as_deref() {
            let (discriminator_type, optional, array) = match &v.base {
                crate::MembersOrRef::Ref(base) => structs_lookup
                    .get::<str>(base.as_ref())
                    .ok_or_else(|| LowerError::UnknownType {
                        definition: v.name.to_string(),
                        name: base.to_string(),
                    })
                    .map_err(|e| at(e.into()))?
                    .fields
                    .iter()
                    .find(|field| field.name == discriminator)
                    .map(|field| (field.r#type.clone(), field.optional, field.array)),
                crate::MembersOrRef::Members(members) => members
                    .iter()
                    .find(|field| field.name == discriminator)
                    .map(|field| {
                        let field = process_member(field.clone(), &v.name)?;
                        Ok::<_, LowerError>((field.r#type, field.optional, field.array))
                    })
                    .transpose()
                    .map_err(|e| at(e.into()))?,
                crate::MembersOrRef::Unset => None,
            }
            .ok_or_else(|| {
                at(LowerError::MissingDiscriminator {
                    union: v.name.to_string(),
                    discriminator: discriminator.to_owned(),
                }
                .into())
            })?;
            require(!optional && !array, || {
                SchemaErrorKind::DiscriminatorShape {
                    discriminator: discriminator.to_owned(),
                }
            })
            .map_err(&at)?;
            if let Some(tags) = enums_lookup.get(&discriminator_type) {
                require(!tags.variants.is_empty(), || SchemaErrorKind::NoBranches).map_err(&at)?;
                for branch in &v.data.0 {
                    require(
                        tags.variants.iter().any(|tag| tag.name == branch.name),
                        || SchemaErrorKind::UnknownBranch {
                            branch: branch.name.to_string(),
                            discriminator_type: discriminator_type.clone(),
                        },
                    )
                    .map_err(&at)?;
                }
            } else {
                require(crate::parser_version() < (6, 2, 0), || {
                    SchemaErrorKind::DiscriminatorNotEnum {
                        discriminator: discriminator.to_owned(),
                        discriminator_type: discriminator_type.clone(),
                    }
                })
                .map_err(&at)?;
                warnings.push(WarningKind::NonEnumDiscriminator {
                    union: v.name.to_string(),
                    discriminator: discriminator.to_owned(),
                    discriminator_type: discriminator_type.clone(),
                });
            }
        }
        let (branch_name, discriminator_name) = &union_names[v.name.as_ref()];
        let (mut s, mut e, disc_enum) =
            process_union(v, &structs_lookup, &enums_lookup).map_err(|e| at(e.into()))?;
        e.name.clone_from(branch_name);
        for field in &mut s.fields {
            if field.meta.attributes.contains(&Attribute::new("flatten")) {
                field.r#type.clone_from(branch_name);
            }
        }
        if let Some(mut de) = disc_enum {
            for field in &mut s.fields {
                if field
                    .meta
                    .attributes
                    .contains(&Attribute::new("discriminator"))
                {
                    field.r#type.clone_from(discriminator_name);
                }
            }
            de.name.clone_from(discriminator_name);
            enums_lookup.insert(de.name.clone(), de);
        }
        enums_lookup.insert(e.name.clone(), e);
        structs_lookup.insert(s.name.clone(), s);
    }
    for v in unprocessed_events {
        let at = locate(format!("event '{}'", v.name));
        let event_name = &event_names[v.name.as_ref()];
        let mut processed = process_event(v, &structs_lookup).map_err(|e| at(e.into()))?;
        processed.name.clone_from(event_name);
        structs_lookup.insert(processed.name.clone(), processed);
    }
    let symbol_name = |name: &str| {
        if let Some(name) = name.strip_prefix(crate::symbols::SYNTHESIZED) {
            name.to_owned()
        } else if declared.contains::<str>(name.as_ref()) {
            symbols.type_name(name.as_ref())
        } else {
            rustify_name(name)
        }
    };
    let mut occupied = symbols.data_symbols();
    occupied.extend(
        structs_lookup
            .values()
            .map(|value| symbol_name(&value.name)),
    );
    occupied.extend(enums_lookup.values().map(|value| symbol_name(&value.name)));
    for v in unprocessed_commands {
        let at = locate(format!("command '{}'", v.name));
        let split = symbols.separate_request(&v.name);
        let request_name = symbols
            .historical_command(&v.name)
            .unwrap_or_else(|| symbols.command_root(&v.name));
        // Check if this command references an existing struct via data
        let data_ref_name = match &v.data {
            Some(crate::MembersOrRef::Ref(name)) => Some(name.to_string()),
            _ => None,
        };

        let command_name = &command_names[v.name.as_ref()];
        let returns = v.returns.as_ref().and_then(|value| {
            let (name, array) = match value {
                crate::QapiTypeRef::Ref(name) => (name.as_ref(), false),
                crate::QapiTypeRef::ArrayRef(name) => (name.as_ref(), true),
                _ => return None,
            };
            let name = if declared.contains::<str>(name.as_ref()) {
                symbols.type_name(name.as_ref())
            } else {
                rustify_type(name)
            };
            Some(if array { format!("Vec<{name}>") } else { name })
        });
        let mut processed = process_command(v, &structs_lookup).map_err(|e| at(e.into()))?;
        if request_name != rustify_name(&processed.name) {
            processed
                .meta
                .attributes
                .retain(|attr| !matches!(attr, Attribute::Item { name, .. } if name == "name"));
            processed
                .meta
                .attributes
                .push(Attribute::with_value("name", processed.name.clone()));
            processed.name = request_name;
        }
        if let Some(returns) = returns {
            for attr in &mut processed.meta.attributes {
                if let Attribute::Item { name, value } = attr
                    && name == "returns"
                {
                    value.clone_from(&returns);
                }
            }
        }
        let mut cmd_key = rustify_name(&processed.name);

        if let Some(ref ref_name) = data_ref_name {
            let ref_key = rustify_name(ref_name);
            if !split && cmd_key == ref_key {
                // Command and its data ref share the same rustified name
                // (e.g. command `blockdev-snapshot-sync` → struct
                // `BlockdevSnapshotSync`). Merge command attributes onto the
                // existing data struct -- a single entry represents both.
                if let Some(existing) = structs_lookup.get_mut(ref_name) {
                    merge_command_attrs_onto(&processed, existing);
                    continue;
                }
            }
            // Different rustified names (e.g. `netdev_add` → `NetdevAdd`,
            // data ref `Netdev` → `Netdev`): the command gets its own entry
            // under its rustified key. The data struct stays under its key.
        }

        // A command and its *response* can rustify to the same identifier
        // (guest-file-read / GuestFileRead, guest-exec / GuestExec). Preserve
        // the data type and give the request an explicit Command suffix.
        // Otherwise Returns resolves to the request itself and decoding
        // silently loses response fields or fails on missing arguments.
        if occupied.contains(&cmd_key) {
            processed
                .meta
                .attributes
                .retain(|attr| !matches!(attr, Attribute::Item { name, .. } if name == "name"));
            processed
                .meta
                .attributes
                .push(Attribute::with_value("name", processed.name.clone()));
            processed.name.clone_from(command_name);
            cmd_key = rustify_name(&processed.name);
            // The plan gives that name to nothing else, nor do the extracted types take it.
            assert!(
                !occupied.contains(&cmd_key),
                "{cmd_key} is planned for the command alone"
            );
        }
        occupied.insert(cmd_key.clone());
        processed.name = synthesized(&cmd_key);
        structs_lookup.insert(processed.name.clone(), processed);
    }

    symbols.rewrite_types(&declared, &mut enums_lookup, &mut structs_lookup);
    Ok((
        enums_lookup
            .into_values()
            .map(|value| (rustify_name(&value.name), value))
            .collect(),
        structs_lookup
            .into_values()
            .map(|value| (rustify_name(&value.name), value))
            .collect(),
    ))
}

/// Result of extracting and parsing a single version's schema.
pub(super) struct VersionResult {
    pub(super) tag: String,
    pub(super) version: Version,
    /// The release's place in tag order, four-component tags included.
    pub(super) order: (u32, u32, u32, u32),
    /// Hash of the raw schema content (for dedup of identical patch releases).
    pub(super) schema_hash: u64,
    pub(super) enums: HashMap<String, Enum>,
    pub(super) structs: HashMap<String, Struct>,
    pub(super) warnings: Vec<Warning>,
}

pub(super) fn normalize_member_names(versions: &mut [VersionResult]) {
    #[derive(Default)]
    struct Group {
        identities: Vec<(String, bool)>,
        collision: bool,
    }
    #[derive(Default)]
    struct Groups {
        groups: BTreeMap<(String, String), Group>,
        reserved: BTreeMap<String, BTreeSet<String>>,
    }
    impl Groups {
        fn observe(&mut self, owner: &str, members: impl Iterator<Item = (String, String, bool)>) {
            let mut local: BTreeMap<String, Vec<_>> = BTreeMap::new();
            for (raw, preferred, virtual_field) in members {
                local
                    .entry(preferred)
                    .or_default()
                    .push((raw, virtual_field));
            }
            for (preferred, identities) in local {
                self.reserved
                    .entry(owner.to_owned())
                    .or_default()
                    .insert(preferred.clone());
                let group = self
                    .groups
                    .entry((owner.to_owned(), preferred))
                    .or_default();
                group.collision |= identities.len() > 1;
                for identity in identities {
                    if !group.identities.contains(&identity) {
                        group.identities.push(identity);
                    }
                }
            }
        }
        /// Give each member of a collision after the first a name of its
        /// own: the Rust name with `suffix` and a number from 2 if needed,
        /// whose Rust name, by `rust`, no member of the owner has.
        fn allocate(
            mut self,
            suffix: &str,
            rust: impl Fn(&str) -> String,
        ) -> BTreeMap<(String, (String, bool)), String> {
            let mut aliases = BTreeMap::new();
            for ((owner, preferred), mut group) in self.groups {
                if !group.collision {
                    continue;
                }
                // Declared wire fields keep priority over virtual flatten receivers.
                group
                    .identities
                    .sort_by_key(|(_, virtual_field)| *virtual_field);
                let used = self
                    .reserved
                    .get_mut(&owner)
                    .expect("observe reserves every owner it groups");
                let base = format!("{}{suffix}", preferred.trim_start_matches("r#"));
                for identity in group.identities.into_iter().skip(1) {
                    let mut name = base.clone();
                    let mut number = 2;
                    while !used.insert(rust(&name)) {
                        name = format!("{base}{number}");
                        number += 1;
                    }
                    aliases.insert((owner.clone(), identity), name);
                }
            }
            aliases
        }
    }
    let mut fields = Groups::default();
    let mut variants = Groups::default();
    let mut ordered: Vec<_> = versions.iter().collect();
    ordered.sort_by_key(|version| std::cmp::Reverse(version.order));
    for version in ordered {
        for (owner, value) in &version.structs {
            fields.observe(
                owner,
                value.fields.iter().map(|field| {
                    (
                        field.name.clone(),
                        field_rust_name(field),
                        field.meta.attributes.contains(&Attribute::new("flatten")),
                    )
                }),
            );
        }
        for (owner, value) in &version.enums {
            variants.observe(
                owner,
                value
                    .variants
                    .iter()
                    .map(|variant| (variant.name.clone(), variant_rust_name(variant), false)),
            );
        }
    }
    let fields = fields.allocate("_alt", crate::qapi_ir::rustify_field);
    let variants = variants.allocate("Alt", crate::qapi_ir::rustify_variant);
    if fields.is_empty() && variants.is_empty() {
        return;
    }
    let rename = |raw: &mut String, meta: &mut Metadata, assigned: &String| {
        if !meta
            .attributes
            .iter()
            .any(|attr| matches!(attr, Attribute::Item { name, .. } if name == "name"))
        {
            meta.attributes.push(Attribute::with_value("name", &*raw));
        }
        raw.clone_from(assigned);
    };
    for version in versions {
        for (owner, value) in &mut version.structs {
            for field in &mut value.fields {
                let identity = (
                    field.name.clone(),
                    field.meta.attributes.contains(&Attribute::new("flatten")),
                );
                if let Some(name) = fields.get(&(owner.clone(), identity)) {
                    rename(&mut field.name, &mut field.meta, name);
                }
            }
        }
        for (owner, value) in &mut version.enums {
            for variant in &mut value.variants {
                if let Some(name) = variants.get(&(owner.clone(), (variant.name.clone(), false))) {
                    rename(&mut variant.name, &mut variant.meta, name);
                }
            }
        }
    }
}

/// Bound concurrent git processes and parser workers even on hosts with large
/// CPU counts. Keep input order so errors and merges are reproducible.
pub(super) fn parse_versions<T: Send>(
    tags: &[&str],
    parse: impl Fn(&str) -> Result<T, RegistryError> + Sync,
) -> Result<Vec<T>, RegistryError> {
    let workers = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(8);
    let mut results = Vec::with_capacity(tags.len());
    for batch in tags.chunks(workers) {
        // Join every worker before returning: a worker left running when the
        // scope ends would make its panic the caller's.
        let batch: Vec<_> = std::thread::scope(|scope| {
            let handles: Vec<_> = batch
                .iter()
                .map(|tag| {
                    let parse = &parse;
                    (*tag, scope.spawn(move || parse(tag)))
                })
                .collect();
            handles
                .into_iter()
                .map(|(tag, handle)| {
                    handle.join().unwrap_or_else(|_| {
                        Err(RegistryError::WorkerPanicked {
                            tag: tag.to_owned(),
                        })
                    })
                })
                .collect()
        });
        for result in batch {
            results.push(result?);
        }
    }
    Ok(results)
}

/// The schema files of one release, as extracted from its tag.
pub(super) struct VersionSources {
    pub(super) tag: String,
    pub(super) version: Version,
    pub(super) order: (u32, u32, u32, u32),
    pub(super) schema_hash: u64,
    pub(super) sources: Vec<(std::path::PathBuf, String)>,
}

pub(super) fn extract_version_sources(
    qemu_repo: &Path,
    tag: &str,
    qga: bool,
) -> Result<Option<VersionSources>, RegistryError> {
    let order =
        parse_version_tag_full(tag).ok_or_else(|| RegistryError::Tag { tag: tag.into() })?;
    let version = (order.0, order.1, order.2);
    let tmpdir =
        extract_schema_from_tag(qemu_repo, tag).map_err(|source| RegistryError::Extract {
            tag: tag.into(),
            source,
        })?;
    let entry = if qga {
        find_qga_schema_entry(tmpdir.path())
    } else {
        find_schema_entry(tmpdir.path())
    };
    let first_schema = if qga { (0, 15, 0) } else { (1, 0, 0) };
    let schema_entry = match entry {
        Some(entry) => entry,
        None if version < first_schema => return Ok(None),
        None => {
            return Err(RegistryError::MissingEntry {
                tag: tag.into(),
                protocol: if qga { "QGA" } else { "QMP" },
            });
        }
    };
    let sources =
        crate::read_schema_with_version(&schema_entry, Some(version)).map_err(|source| {
            RegistryError::Read {
                tag: tag.into(),
                source,
            }
        })?;
    // merge_releases skips a release whose hash equals the previous one's.
    // Identical files lower differently across a change of the predefined
    // types or of the string grammar, so the hash covers both.
    let schema_hash = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        crate::builtins::Builtins::of(version).hash(&mut hasher);
        crate::parsers::common::string_model(version).hash(&mut hasher);
        if qga {
            hasher.write(b"qga");
        }
        for (path, content) in &sources {
            path.file_name().hash(&mut hasher);
            content.hash(&mut hasher);
        }
        hasher.finish()
    };
    Ok(Some(VersionSources {
        tag: tag.into(),
        version,
        order,
        schema_hash,
        sources,
    }))
}

pub(super) fn load_versions(
    qemu_repo: &Path,
    tags: &[&str],
    qga: bool,
) -> Result<Vec<VersionResult>, RegistryError> {
    let extracted = parse_versions(tags, |tag| extract_version_sources(qemu_repo, tag, qga))?;
    lower_versions(tags, extracted.into_iter().flatten().collect())
}

/// Plan the symbols of every release of `sources`, then lower each with
/// them, in the order of `tags`.
pub(super) fn lower_versions(
    tags: &[&str],
    sources: Vec<VersionSources>,
) -> Result<Vec<VersionResult>, RegistryError> {
    let by_tag: HashMap<_, _> = sources
        .iter()
        .map(|source| (source.tag.as_str(), source))
        .collect();
    let plans = parse_versions(tags, |tag| {
        let Some(source) = by_tag.get(tag) else {
            return Ok(None);
        };
        crate::set_parser_version(source.version);
        plan_version(&source.sources)
            .map(Some)
            .map_err(|source| RegistryError::Schema {
                tag: tag.into(),
                source,
            })
    })?;
    let mut symbols = crate::symbols::SchemaNames::default();
    let mut used = BTreeSet::new();
    for (plan, predefined) in plans.into_iter().flatten() {
        symbols.merge(plan);
        used.extend(predefined);
    }
    symbols.keep_predefined(&used);
    symbols.finish();
    let results = parse_versions(tags, |tag| {
        let Some(source) = by_tag.get(tag) else {
            return Ok(None);
        };
        crate::set_parser_version(source.version);
        let mut warnings = Vec::new();
        let (enums, structs) =
            process_version_schema_with_names(&source.sources, &symbols, &used, &mut warnings)
                .map_err(|source| RegistryError::Schema {
                    tag: tag.into(),
                    source,
                })?;
        Ok(Some(VersionResult {
            tag: source.tag.clone(),
            version: source.version,
            order: source.order,
            schema_hash: source.schema_hash,
            enums,
            structs,
            warnings: warnings
                .into_iter()
                .map(|kind| Warning {
                    tag: source.tag.clone(),
                    kind,
                })
                .collect(),
        }))
    })?;
    let mut results: Vec<_> = results.into_iter().flatten().collect();
    normalize_member_names(&mut results);
    Ok(results)
}
