//! Emitting the merged registry as the generated Rust modules.

use super::{
    registry::{build_qga_registry, build_qmp_registry},
    *,
};

/// A QAPI protocol of QEMU, whose schema the generator reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Protocol {
    /// The QEMU Machine Protocol, `qapi/qapi-schema.json`, from QEMU 1.0.
    Qmp,
    /// The QEMU Guest Agent protocol, `qga/qapi-schema.json` (from QEMU
    /// 0.15, `qapi-schema-guest.json` before 1.1).
    Qga,
}

impl Protocol {
    /// The protocol's short name, `QMP` or `QGA`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Qmp => "QMP",
            Self::Qga => "QGA",
        }
    }

    /// The first QEMU release with the protocol's schema: 1.0 for QMP,
    /// 0.15 for QGA. [`generate`] leaves out the releases before it.
    pub fn first_release(self) -> Version {
        match self {
            Self::Qmp => (1, 0, 0),
            Self::Qga => (0, 15, 0),
        }
    }
}

/// The documentation of the generated items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Docs {
    /// A line for each item that the schema declares, naming it and
    /// pointing to QEMU's reference manual: the output carries none of the
    /// schema's GPL-2.0-or-later prose.
    #[default]
    Reference,
    /// QEMU's schema documentation as rustdoc, which is GPL-2.0-or-later,
    /// for local reading. An item without any keeps its reference line.
    Schema,
}

/// The QEMU releases to generate.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Releases<'a> {
    /// These release tags of the tree, in release order.
    Tags(&'a [&'a str]),
    /// The releases of a lockfile of `tag commit` lines, such as
    /// [`PINNED_VERSIONS`](crate::versions::PINNED_VERSIONS), once the tree
    /// is checked to have each tag at its commit.
    Pinned(&'a str),
}

/// What differs between the QMP and QGA output.
struct Flavor {
    /// The protocol as item documentation names it, e.g. "QMP".
    protocol: &'static str,
    /// The QEMU manual that documents the schema.
    reference: &'static str,
    /// Emit QAPI schema prose (GPL-2.0-or-later) instead of reference lines.
    schema_docs: bool,
}

/// Name a schema item and point at QEMU's manual. QAPI schema documentation
/// is GPL-2.0-or-later, so generated code carries none of its text. Types
/// that the lowering synthesizes have no such line.
fn reference_doc(
    meta: &Metadata,
    name: &str,
    flavor: &Flavor,
) -> Option<String> {
    let kind = meta.schema_item?;
    let wire_name = meta
        .attributes
        .iter()
        .find_map(|attr| match attr {
            Attribute::Item { name, value } if name == "name" => {
                Some(value.as_str())
            }
            _ => None,
        })
        .unwrap_or(name);
    Some(format!(
        "{} {} `{wire_name}`; see the {}.",
        flavor.protocol,
        kind.as_str(),
        flavor.reference
    ))
}

/// An item's documentation: its schema prose in schema-docs mode, or else,
/// and for an item without prose, the reference line.
fn item_docs(meta: &mut Metadata, name: &str, flavor: &Flavor) {
    clean_docs(meta);
    let schema_doc = meta.schema_doc.take();
    meta.doc = schema_doc
        .filter(|_| flavor.schema_docs)
        .or_else(|| reference_doc(meta, name, flavor));
}

/// A member's documentation: what the lowering writes, or its schema prose
/// in schema-docs mode.
fn member_docs(meta: &mut Metadata, flavor: &Flavor) {
    clean_docs(meta);
    let schema_doc = meta.schema_doc.take();
    if flavor.schema_docs {
        meta.doc = schema_doc.or(meta.doc.take());
    }
}

/// The types that a type's Rust code names: those of its fields, variant
/// payloads and, for a command, its return values in every release
/// (`returns_history`, whose last entry is the newest).
fn references(value: &UnifiedType) -> Vec<String> {
    match value {
        UnifiedType::Struct(value) => value
            .ir
            .fields
            .iter()
            .map(|field| rustify_type(&field.r#type))
            .chain(value.returns_history.iter().map(|(_, rust)| {
                crate::metadata_emit::returns_element(rust).0.to_owned()
            }))
            .collect(),
        UnifiedType::Enum(value) => value
            .ir
            .variants
            .iter()
            .filter_map(|variant| match &variant.kind {
                EnumVariantKind::Tuple(name) => Some(rustify_type(name)),
                EnumVariantKind::Unit => None,
            })
            .collect(),
    }
}

/// The glob imports of `mod.rs`. Each command struct is in `commands.rs`
/// and each event struct in `events.rs`, so that `commands` holds every
/// command and `events` every event. Those modules import `mod.rs` by glob,
/// and `mod.rs` imports a module by glob where a type outside it names one
/// of its structs, as a field, payload or return type.
fn glob_imports(registry: &UnifiedRegistry) -> String {
    let named_elsewhere = |category: TypeCategory| {
        registry.types.values().any(|value| {
            classify_type(value) != category
                && references(value).iter().any(|name| {
                    registry.types.get(name).is_some_and(|referenced| {
                        classify_type(referenced) == category
                    })
                })
        })
    };
    let mut imports = String::new();
    if named_elsewhere(TypeCategory::Command) {
        imports.push_str("use self::commands::*;\n");
    }
    if named_elsewhere(TypeCategory::Event) {
        imports.push_str("use self::events::*;\n");
    }
    imports
}

/// Emit the four modules of a registry, such as one that
/// [`build_qmp_registry`] or [`build_qga_registry`] builds, documented as
/// `docs` says. `protocol` is the protocol of the registry's schema, which
/// the documentation of every item names; [`generate`] builds the registry
/// of a protocol and emits it.
///
/// A registry that refers to a type it does not define, which the registry
/// builders never give, is a [`GenerateError::UndefinedType`].
pub fn emit_registry(
    registry: &UnifiedRegistry,
    protocol: Protocol,
    docs: Docs,
) -> Result<GeneratedOutput, GenerateError> {
    let schema_docs = docs == Docs::Schema;
    let flavor = match protocol {
        Protocol::Qga => Flavor {
            protocol: "QEMU Guest Agent",
            reference: "QEMU Guest Agent Protocol Reference",
            schema_docs,
        },
        Protocol::Qmp => Flavor {
            protocol: "QMP",
            reference: "QEMU QMP Reference Manual",
            schema_docs,
        },
    };
    let flavor = &flavor;
    if let Some((name, at)) =
        crate::metadata_emit::undefined_reference(registry)
    {
        return Err(GenerateError::UndefinedType { name, at });
    }
    // Sorted by Rust name, case-insensitively, for a stable, readable order.
    let mut entries: Vec<_> = registry.types.iter().collect();
    entries.sort_by_cached_key(|(name, _)| name.to_lowercase());

    let mut types_tokens = proc_macro2::TokenStream::new();
    let mut commands_tokens = proc_macro2::TokenStream::new();
    let mut events_tokens = proc_macro2::TokenStream::new();
    for (_, value) in entries {
        let tokens = match classify_type(value) {
            TypeCategory::DataType => &mut types_tokens,
            TypeCategory::Command => &mut commands_tokens,
            TypeCategory::Event => &mut events_tokens,
        };
        tokens.extend(match value {
            UnifiedType::Enum(value) => generate_unified_enum(value, flavor),
            UnifiedType::Struct(value) => {
                generate_unified_struct(value, flavor)
            }
        });
    }

    let types_tokens = quote::quote! {
        #types_tokens

        pub mod commands;
        pub mod events;
        pub mod metadata;
    };
    let metadata = crate::metadata_emit::emit(registry);
    parse_rust(&metadata, "metadata.rs")?;
    Ok(GeneratedOutput {
        types: format_rust_file(
            types_tokens,
            &glob_imports(registry),
            "mod.rs",
        )?,
        commands: format_rust_file(
            commands_tokens,
            "use super::*;\n",
            "commands.rs",
        )?,
        events: format_rust_file(
            events_tokens,
            "use super::*;\n",
            "events.rs",
        )?,
        metadata,
        warnings: registry.warnings.clone(),
    })
}

/// Build the `protocol` registry of `releases` of the QEMU git tree
/// `qemu_repo` and generate its four modules, documented as `docs` says.
/// The modes of `docs` differ only in `///` lines.
pub fn generate(
    qemu_repo: &Path,
    releases: Releases<'_>,
    protocol: Protocol,
    docs: Docs,
) -> Result<GeneratedOutput, GenerateError> {
    let pinned: Vec<String>;
    let pinned_tags: Vec<&str>;
    let tags = match releases {
        Releases::Tags(tags) => tags,
        Releases::Pinned(pins) => {
            let versions = crate::versions::parse_pinned_versions(pins)?;
            crate::versions::verify_pinned_versions(qemu_repo, &versions)?;
            pinned = versions.into_iter().map(|version| version.tag).collect();
            pinned_tags = pinned.iter().map(String::as_str).collect();
            &pinned_tags
        }
    };
    let registry = match protocol {
        Protocol::Qmp => build_qmp_registry(qemu_repo, tags)?,
        Protocol::Qga => build_qga_registry(qemu_repo, tags)?,
    };
    emit_registry(&registry, protocol, docs)
}

/// The four files produced by code generation.
#[derive(Debug)]
#[non_exhaustive]
pub struct GeneratedOutput {
    /// Data types (structs, enums, unions) — goes into `mod.rs`.
    pub types: String,
    /// Command types — goes into `commands.rs`.
    pub commands: String,
    /// Event types — goes into `events.rs`.
    pub events: String,
    /// Compile-time schema metadata — goes into `metadata.rs`.
    pub metadata: String,
    /// The tolerated quirks of the releases' schemas, in release order.
    pub warnings: Vec<Warning>,
}

impl GeneratedOutput {
    /// Write `mod.rs`, `commands.rs`, `events.rs` and `metadata.rs` into
    /// `output_dir`, creating it if needed and leaving byte-identical files
    /// untouched.
    ///
    /// Each changed file is first written in full to a temporary file in
    /// `output_dir`, named after it, the process and the write, and the
    /// temporary files replace the old ones only once all of them are
    /// written, so a failed write leaves every old file in place and removes
    /// the temporary files. The replacement is one rename per file: a
    /// failure between two renames can still leave some files new and the
    /// others old. A failure names the output file, with the temporary file
    /// it was written through, if any.
    pub fn write_to(&self, output_dir: &Path) -> Result<(), GenerateError> {
        self.write_with(
            output_dir,
            |path, content| std::fs::write(path, content),
            |from, to| std::fs::rename(from, to),
        )
    }

    /// [`write_to`](Self::write_to), writing each temporary file with
    /// `write_file` and moving it into place with `rename`.
    fn write_with(
        &self,
        output_dir: &Path,
        mut write_file: impl FnMut(&Path, &str) -> std::io::Result<()>,
        mut rename: impl FnMut(&Path, &Path) -> std::io::Result<()>,
    ) -> Result<(), GenerateError> {
        static WRITES: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        let write = WRITES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let failed = |path: &Path, temporary: Option<&Path>| {
            let path = path.to_path_buf();
            let temporary = temporary.map(Path::to_path_buf);
            move |source| GenerateError::Write { path, temporary, source }
        };
        std::fs::create_dir_all(output_dir)
            .map_err(failed(output_dir, None))?;
        let mut staged = Vec::new();
        let mut stage = || -> Result<(), GenerateError> {
            for (name, content) in [
                ("mod.rs", &self.types),
                ("commands.rs", &self.commands),
                ("events.rs", &self.events),
                ("metadata.rs", &self.metadata),
            ] {
                let path = output_dir.join(name);
                match std::fs::read(&path) {
                    Ok(existing) if existing == content.as_bytes() => continue,
                    Ok(_) => {}
                    Err(error)
                        if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(source) => {
                        return Err(GenerateError::Read { path, source });
                    }
                }
                let temporary = output_dir.join(format!(
                    ".{name}.{}-{write}.tmp",
                    std::process::id()
                ));
                // Staged first, so that a partly written file goes too.
                staged.push((temporary.clone(), path.clone()));
                write_file(&temporary, content)
                    .map_err(failed(&path, Some(&temporary)))?;
            }
            Ok(())
        };
        let mut result = stage();
        let mut renamed = 0;
        if result.is_ok() {
            for (temporary, path) in &staged {
                if let Err(error) = rename(temporary, path) {
                    result = Err(failed(path, Some(temporary))(error));
                    break;
                }
                renamed += 1;
            }
        }
        for (temporary, _) in &staged[renamed..] {
            let _ = std::fs::remove_file(temporary);
        }
        result
    }
}

/// Parse generated Rust, locating the first error in `source`.
fn parse_rust(
    source: &str,
    file: &'static str,
) -> Result<syn::File, GenerateError> {
    syn::parse_file(source).map_err(|error| {
        let message = error.to_string();
        let start = error.span().start();
        // syn reports the end of a group at its closing delimiter, but the
        // end of the file at the call site, its start; the file ends after
        // its last token.
        let call_site = proc_macro2::Span::call_site().start();
        let (line, column) = if message.starts_with("unexpected end of input")
            && (start.line, start.column) == (call_site.line, call_site.column)
        {
            crate::position(source, source.trim_end().len())
        } else {
            (start.line, start.column + 1)
        };
        GenerateError::InvalidRust {
            file,
            line,
            column,
            message,
            near: source
                .lines()
                .nth(line.saturating_sub(1))
                .unwrap_or_default()
                .chars()
                .skip(column - 1)
                .take(80)
                .collect(),
        }
    })
}

/// Format a token stream as a complete Rust file with the standard header.
/// An empty module has no imports.
fn format_rust_file(
    tokens: proc_macro2::TokenStream,
    extra_imports: &str,
    file: &'static str,
) -> Result<String, GenerateError> {
    let mut source = String::new();
    if !tokens.is_empty() {
        source.push_str("use qapi_macros::qapi;\n");
        source.push_str(extra_imports);
    }
    source.push('\n');
    source.push_str(&tokens.to_string());
    let syntax = parse_rust(&source, file)?;
    Ok(separate_items(&prettyplease::unparse(&syntax)))
}

/// Insert a blank line between top-level items for readability.
fn separate_items(code: &str) -> String {
    let mut result = String::with_capacity(code.len() + code.len() / 20);
    let mut prev_line = "";
    for line in code.lines() {
        // Insert blank line after a top-level item ends (line is or ends with `}`)
        let prev_ends_item = prev_line == "}" || prev_line.ends_with(" {}");
        if prev_ends_item && !line.is_empty() {
            result.push('\n');
        }
        result.push_str(line);
        result.push('\n');
        prev_line = line;
    }
    result
}

/// Drop the `added` of a `Since:` section, the one lifecycle attribute that
/// the lowering writes, so that the registry's releases are the single
/// source of truth.
fn strip_since(attrs: &mut Vec<Attribute>) {
    attrs.retain(
        |a| !matches!(a, Attribute::Item { name, .. } if name == "added"),
    );
}

/// A `deprecated(added, removed)` attribute per run of releases that
/// deprecate an item, `removed` where a later release that has the item no
/// longer deprecates it.
fn deprecation_attributes(
    versions: &VersionRange,
) -> impl Iterator<Item = Attribute> + '_ {
    versions.deprecated_ranges.iter().map(|(since, until)| {
        let mut attrs =
            vec![Attribute::with_value("added", version_str(*since))];
        if let Some(until) = until {
            attrs.push(Attribute::with_value("removed", version_str(*until)));
        }
        Attribute::nested("deprecated", attrs)
    })
}

/// A field's role as its union's discriminator: a bare `discriminator`
/// where it is one in every release that has it, and otherwise a
/// `discriminator(added = "...", removed = "...")` per run of releases,
/// with `added` where the run starts after the field and `removed` where it
/// ends before the field does.
fn discriminator_attributes(
    versions: &VersionRange,
) -> impl Iterator<Item = Attribute> + '_ {
    versions.discriminator_ranges.iter().map(|(since, until)| {
        let mut attrs = Vec::new();
        if *since > versions.since {
            attrs.push(Attribute::with_value("added", version_str(*since)));
        }
        if let Some(until) =
            until.filter(|until| Some(*until) != versions.until)
        {
            attrs.push(Attribute::with_value("removed", version_str(until)));
        }
        if attrs.is_empty() {
            Attribute::new("discriminator")
        } else {
            Attribute::nested("discriminator", attrs)
        }
    })
}

/// Keep only the QAPI features that the `#[qapi]` macro reads: `unstable` on
/// a field or variant, which documents it there. A type's instability and
/// every deprecation are emitted as version ranges instead. Other features,
/// and the build conditions of features, mean nothing to the expansion.
fn retain_expanded_features(attrs: &mut Vec<Attribute>, member: bool) {
    attrs.retain_mut(|attr| {
        let Attribute::List(items) = attr else {
            return true;
        };
        let keep = match items.first() {
            Some(Attribute::Item { name, value }) if name == "feature" => {
                member && value == "unstable"
            }
            _ => return true,
        };
        items.truncate(1);
        keep
    });
}

/// The lifecycle attributes of an item: `added`, its `deprecated(...)` and
/// `unstable(...)` runs and `removed`.
fn item_lifecycle(attrs: &mut Vec<Attribute>, versions: &VersionRange) {
    strip_since(attrs);
    attrs.push(Attribute::with_value("added", version_str(versions.since)));
    attrs.extend(deprecation_attributes(versions));
    retain_expanded_features(attrs, false);
    attrs.extend(unstable_attributes(versions));
    if let Some(until) = versions.until {
        attrs.push(Attribute::with_value("removed", version_str(until)));
    }
}

/// The lifecycle attributes of a field or variant: `added` when it is newer
/// than its item, its `deprecated(...)` runs and `removed`. Returns whether
/// it has a narrower lifetime than its item.
fn member_lifecycle(
    attrs: &mut Vec<Attribute>,
    versions: Option<&VersionRange>,
    item: &VersionRange,
) -> bool {
    strip_since(attrs);
    retain_expanded_features(attrs, true);
    let Some(versions) = versions else {
        return false;
    };
    let added = versions.since > item.since;
    if added {
        attrs.push(Attribute::with_value("added", version_str(versions.since)));
    }
    attrs.extend(deprecation_attributes(versions));
    if let Some(until) = versions.until {
        attrs.push(Attribute::with_value("removed", version_str(until)));
    }
    added || versions.until.is_some()
}

/// Generate code for a unified enum, with the releases of the registry.
fn generate_unified_enum(
    ue: &UnifiedEnum,
    flavor: &Flavor,
) -> proc_macro2::TokenStream {
    let mut ir = ue.ir.clone();
    item_lifecycle(&mut ir.meta.attributes, &ue.versions);
    for variant in &mut ir.variants {
        member_lifecycle(
            &mut variant.meta.attributes,
            ue.variant_versions.get(&variant.name),
            &ue.versions,
        );
        member_docs(&mut variant.meta, flavor);
    }
    item_docs(&mut ir.meta, &ir.name, flavor);
    ir.variants.sort_by_cached_key(|a| a.name.to_lowercase());
    ir.generate()
}

/// Generate code for a unified struct, with the releases of the registry,
/// making the fields that some releases lack optional. A type that
/// coalescing made a return type early starts its Rust lifetime there, with
/// the members it had from its first release, and its predecessor names it
/// as its alias.
fn generate_unified_struct(
    us: &UnifiedStruct,
    flavor: &Flavor,
) -> proc_macro2::TokenStream {
    let mut ir = us.ir.clone();
    let mut versions = us.versions.clone();
    let mut field_versions = us.field_versions.clone();
    if let Some(since) = us.rust_since {
        for range in field_versions.values_mut() {
            if range.since == versions.since {
                range.since = since;
            }
        }
        versions.since = since;
    }
    let versions = &versions;
    item_lifecycle(&mut ir.meta.attributes, versions);
    if let Some((since, target)) = &us.alias {
        ir.meta.attributes.push(Attribute::nested(
            "alias",
            vec![
                Attribute::with_value("added", version_str(*since)),
                Attribute::with_value("value", target),
            ],
        ));
    }

    // Version-tracked flags in nested syntax, one attribute per run:
    // #[qapi(allow_preconfig(added = "7.1.0", removed = "9.0.0"))]
    for (attr_name, vr) in &us.attr_versions {
        ir.meta
            .attributes
            .retain(|a| !matches!(a, Attribute::Unit(n) if n == attr_name));
        for range in us
            .attr_history
            .iter()
            .filter_map(|(name, range)| (name == attr_name).then_some(range))
            .chain(std::iter::once(vr))
        {
            let mut inner = Vec::new();
            if range.since > versions.since {
                inner.push(Attribute::with_value(
                    "added",
                    version_str(range.since),
                ));
            }
            if let Some(until) = range.until {
                inner
                    .push(Attribute::with_value("removed", version_str(until)));
            }
            if inner.is_empty() {
                ir.meta.attributes.push(Attribute::new(attr_name));
            } else {
                ir.meta.attributes.push(Attribute::nested(attr_name, inner));
            }
        }
    }

    if us.returns_history.len() > 1 {
        for (version, value) in &us.returns_history {
            ir.meta.attributes.push(Attribute::nested(
                "returns",
                vec![
                    Attribute::with_value("added", version_str(*version)),
                    Attribute::with_value("value", value),
                ],
            ));
        }
    }
    if let Some(name) = &us.returns_enum {
        ir.meta.attributes.push(Attribute::with_value("returns_enum", name));
    }

    for field in &mut ir.fields {
        let flatten =
            field.meta.attributes.contains(&Attribute::new("flatten"));
        if flatten {
            // A flattened field has no member of its own on the wire, so a
            // wire name, which the field planning gives a renamed one, means
            // nothing to it.
            field
                .meta
                .attributes
                .retain(|attr| !matches!(attr, Attribute::Item { name, .. } if name == "name"));
        }
        let discriminator = Attribute::new("discriminator");
        if let Some(at) =
            field.meta.attributes.iter().position(|attr| *attr == discriminator)
            && let Some(versions) = field_versions.get(&field.name)
        {
            field
                .meta
                .attributes
                .splice(at..=at, discriminator_attributes(versions));
        }
        let required = !field.optional;
        if member_lifecycle(
            &mut field.meta.attributes,
            field_versions.get(&field.name),
            versions,
        ) {
            field.optional = true;
        }
        let conditional = field
            .meta
            .attributes
            .iter()
            .any(|attr| matches!(attr, Attribute::Item { name, .. } if name == "condition"));
        if conditional {
            field.optional = true;
        }
        // Builds without its condition lack a conditional field, so no
        // release requires it in every build. A flattened field, the branch
        // of a union, is optional from the lowering on.
        if required && field.optional && !conditional {
            field.meta.attributes.push(Attribute::new("required"));
        }
        member_docs(&mut field.meta, flavor);
    }
    item_docs(&mut ir.meta, &ir.name, flavor);
    ir.fields.sort_by_cached_key(|a| a.name.to_lowercase());
    ir.generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_rust_is_located() {
        let tokens = quote::quote! { pub struct Valid {} pub struct ; };
        let error = format_rust_file(tokens, "use super::*;\n", "commands.rs")
            .unwrap_err();
        let GenerateError::InvalidRust { file, line, column, message, near } =
            &error
        else {
            panic!("{error:?}")
        };
        assert_eq!((*file, *line, *column), ("commands.rs", 4, 33), "{error}");
        assert_eq!(message, "expected identifier");
        assert_eq!(near, ";");

        let Err(error) =
            parse_rust("static A: u8 = 1;\nstatic B u8 = 2;\n", "metadata.rs")
        else {
            panic!("parsed")
        };
        assert!(
            matches!(
                &error,
                GenerateError::InvalidRust { file: "metadata.rs", line: 2, column: 10, near, .. }
                    if near == "u8 = 2;"
            ),
            "{error:?}"
        );
    }

    #[test]
    fn the_end_of_the_input_is_located_where_the_input_ends() {
        let tokens = quote::quote! { pub struct A };
        let error = format_rust_file(tokens, "", "mod.rs").unwrap_err();
        assert!(
            matches!(
                &error,
                GenerateError::InvalidRust { file: "mod.rs", line: 3, column: 13, message, near }
                    if message.starts_with("unexpected end of input") && near.is_empty()
            ),
            "{error:?}"
        );
        // Within a group, at its closing delimiter.
        let Err(error) =
            parse_rust("static A: u8 = 1;\nfn f() { let }\n\n", "metadata.rs")
        else {
            panic!("parsed")
        };
        assert!(
            matches!(
                &error,
                GenerateError::InvalidRust { line: 2, column: 14, message, near, .. }
                    if message.starts_with("unexpected end of input") && near == "}"
            ),
            "{error:?}"
        );
    }

    #[test]
    fn a_failed_rename_leaves_the_other_files_old() {
        let dir = tempfile::tempdir().unwrap();
        let output = |text: &str| GeneratedOutput {
            types: format!("{text} types"),
            commands: format!("{text} commands"),
            events: format!("{text} events"),
            metadata: format!("{text} metadata"),
            warnings: Vec::new(),
        };
        output("old").write_to(dir.path()).unwrap();
        let mut renames = 0;
        let error = output("new")
            .write_with(
                dir.path(),
                |path, content| std::fs::write(path, content),
                |from, to| {
                    renames += 1;
                    if renames == 2 {
                        return Err(std::io::Error::other("refused"));
                    }
                    std::fs::rename(from, to)
                },
            )
            .unwrap_err();
        assert!(
            matches!(&error, GenerateError::Write { path, .. } if path.ends_with("commands.rs")),
            "{error:?}"
        );
        let mut files: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                let text = std::fs::read_to_string(&path).unwrap();
                (path.file_name().unwrap().to_string_lossy().into_owned(), text)
            })
            .collect();
        files.sort();
        assert_eq!(
            files,
            [
                ("commands.rs".into(), "old commands".into()),
                ("events.rs".into(), "old events".into()),
                ("metadata.rs".into(), "old metadata".into()),
                ("mod.rs".into(), "new types".into()),
            ]
        );
    }

    #[test]
    fn a_partly_written_temporary_file_is_removed() {
        let dir = tempfile::tempdir().unwrap();
        let output = GeneratedOutput {
            types: "types".into(),
            commands: "commands".into(),
            events: "events".into(),
            metadata: "metadata".into(),
            warnings: Vec::new(),
        };
        let error = output
            .write_with(
                dir.path(),
                |path, content| {
                    if path.to_string_lossy().contains("events") {
                        std::fs::write(path, &content[..2])?;
                        return Err(std::io::Error::other("full"));
                    }
                    std::fs::write(path, content)
                },
                |_, _| panic!("nothing is renamed after a failed write"),
            )
            .unwrap_err();
        assert!(
            matches!(&error, GenerateError::Write { path, .. }
                if path.to_string_lossy().contains("events")),
            "{error:?}"
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    fn output(text: &str) -> GeneratedOutput {
        GeneratedOutput {
            types: format!("{text} types"),
            commands: format!("{text} commands"),
            events: format!("{text} events"),
            metadata: format!("{text} metadata"),
            warnings: Vec::new(),
        }
    }

    /// The files of `dir` and their contents.
    fn files(dir: &Path) -> Vec<(String, String)> {
        let mut files: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                (path.file_name().unwrap().to_string_lossy().into_owned(), text)
            })
            .collect();
        files.sort();
        files
    }

    #[cfg(unix)]
    #[test]
    fn writing_replaces_only_changed_files() {
        use std::os::unix::fs::MetadataExt;

        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out");
        output("old").write_to(&out).unwrap();
        assert_eq!(files(&out).len(), 4);
        let inode =
            |name: &str| std::fs::metadata(out.join(name)).unwrap().ino();
        let before = inode("events.rs");
        let mut changed = output("old");
        changed.types = "new types".into();
        changed.write_to(&out).unwrap();
        // A replaced file is a new one; an unchanged file is left as it was.
        assert_eq!(inode("events.rs"), before);
        assert!(files(&out).contains(&("mod.rs".into(), "new types".into())));
    }

    #[test]
    fn a_failed_write_keeps_the_previous_files() {
        let dir = tempfile::tempdir().unwrap();
        output("old").write_to(dir.path()).unwrap();
        let old = files(dir.path());

        // The temporary file of the second changed file cannot be written, so
        // the first one, already written, must go again.
        let error = output("new")
            .write_with(
                dir.path(),
                |path, content| {
                    if path.to_string_lossy().contains("commands") {
                        return Err(std::io::Error::other("full"));
                    }
                    std::fs::write(path, content)
                },
                |from, to| std::fs::rename(from, to),
            )
            .unwrap_err();
        let GenerateError::Write { path, temporary: Some(temporary), .. } =
            &error
        else {
            panic!("{error:?}")
        };
        assert_eq!(*path, dir.path().join("commands.rs"));
        assert_eq!(temporary.parent(), Some(dir.path()));
        let prefix = format!(".commands.rs.{}-", std::process::id());
        assert!(
            temporary
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&prefix),
            "{error:?}"
        );
        assert_eq!(
            error.to_string(),
            format!(
                "Failed to write {} through {}",
                path.display(),
                temporary.display()
            )
        );
        assert_eq!(files(dir.path()), old);

        output("new").write_to(dir.path()).unwrap();
        let new = files(dir.path());
        assert_eq!(new.len(), 4);
        assert!(
            new.iter().all(|(_, text)| text.starts_with("new ")),
            "{new:?}"
        );
    }

    #[test]
    fn an_uncreatable_output_directory_is_a_write_error() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("file");
        std::fs::write(&file, "").unwrap();
        let error = output("new").write_to(&file.join("out")).unwrap_err();
        assert!(
            matches!(&error, GenerateError::Write { path, temporary: None, .. }
                if *path == file.join("out")),
            "{error:?}"
        );
        assert_eq!(
            error.to_string(),
            format!("Failed to write {}", file.join("out").display())
        );
    }

    /// Every write has temporary files of its own, so writes of the same
    /// directory, from threads of one process, never use each other's.
    #[test]
    fn each_write_has_its_own_temporary_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut temporaries = Vec::new();
        for text in ["one", "two"] {
            output(text)
                .write_with(
                    dir.path(),
                    |path, content| {
                        temporaries.push(path.to_path_buf());
                        std::fs::write(path, content)
                    },
                    |from, to| std::fs::rename(from, to),
                )
                .unwrap();
        }
        assert_eq!(temporaries.len(), 8);
        let unique: BTreeSet<_> = temporaries.iter().collect();
        assert_eq!(unique.len(), 8, "{temporaries:?}");

        let written: Vec<_> = (0..8).map(|n| output(&n.to_string())).collect();
        std::thread::scope(|scope| {
            for output in &written {
                scope.spawn(|| {
                    for _ in 0..20 {
                        output.write_to(dir.path()).unwrap();
                    }
                });
            }
        });
        let files = files(dir.path());
        assert_eq!(files.len(), 4, "{files:?}");
        for (name, text) in files {
            let (_, kind) = text.split_once(' ').unwrap();
            assert_eq!(
                name,
                format!("{}.rs", if kind == "types" { "mod" } else { kind })
            );
        }
    }

    #[test]
    fn an_unreadable_output_file_is_a_read_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("events.rs")).unwrap();
        let error = output("new").write_to(dir.path()).unwrap_err();
        assert!(
            matches!(&error, GenerateError::Read { path, .. } if path.ends_with("events.rs")),
            "{error:?}"
        );
        assert_eq!(files(dir.path()), [("events.rs".into(), String::new())]);
    }

    /// The registry of synthetic QMP releases, each a `(tag, schema)` pair.
    fn registry(releases: &[(&str, &str)]) -> UnifiedRegistry {
        let tags: Vec<_> = releases.iter().map(|(tag, _)| *tag).collect();
        let sources = releases
            .iter()
            .zip(0..)
            .map(|((tag, text), hash)| {
                let order = parse_version_tag_full(tag).unwrap();
                super::super::lower::VersionSources {
                    tag: (*tag).to_owned(),
                    version: (order.0, order.1, order.2),
                    order,
                    schema_hash: hash,
                    sources: vec![("schema.json".into(), (*text).to_owned())],
                }
            })
            .collect();
        let lowered =
            super::super::lower::lower_versions(&tags, sources).unwrap();
        super::super::registry::merge_releases("QMP", lowered).unwrap()
    }

    /// The registry defines every type that it refers to, so a reference
    /// that it cannot resolve is a generator bug, which the emitter reports.
    #[test]
    fn an_unresolved_reference_is_an_error() {
        let mut registry = registry(&[(
            "v11.1.3",
            "{ 'struct': 'Payload', 'data': { 'v': 'int' } }\n\
             { 'struct': 'Holder', 'data': { 'p': 'Payload' } }\n\
             { 'struct': 'Answer', 'data': { 'v': 'int' } }\n\
             { 'command': 'ask', 'returns': 'Answer' }",
        )]);
        assert!(
            emit_registry(&registry, Protocol::Qmp, Docs::Reference).is_ok()
        );
        let complete = registry.types.clone();
        registry.types.remove("Payload");
        let error = emit_registry(&registry, Protocol::Qmp, Docs::Reference)
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Holder.p refers to the undefined type 'Payload'"
        );
        registry.types = complete;
        registry.types.remove("Answer");
        let error = emit_registry(&registry, Protocol::Qmp, Docs::Reference)
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Ask returns refers to the undefined type 'Answer'"
        );
    }

    /// Every command and event struct stays in its own module, and `mod.rs`
    /// imports `commands` or `events` by glob only where a type outside that
    /// module names one of its structs: a data type, or an event naming a
    /// command, which `events.rs` sees through its import of `mod.rs`.
    #[test]
    fn mod_rs_imports_the_command_and_event_structs_that_other_modules_name() {
        let mut registry = registry(&[(
            "v11.1.3",
            "{ 'struct': 'Args', 'data': { 'v': 'int' } }\n\
             { 'command': 'args', 'data': 'Args' }\n\
             { 'struct': 'Holder', 'data': { 'v': 'int' } }\n\
             { 'event': 'HAPPENED', 'data': { 'h': 'Holder' } }",
        )]);
        let mut name_from = |name: &str, field: usize, r#type: &str| {
            let Some(UnifiedType::Struct(value)) = registry.types.get_mut(name)
            else {
                panic!("{name}")
            };
            value.ir.fields[field].r#type = r#type.to_owned();
            let output =
                emit_registry(&registry, Protocol::Qmp, Docs::Reference)
                    .unwrap();
            assert!(
                output.commands.contains("\npub struct Args {"),
                "{}",
                output.commands
            );
            assert!(
                output.events.contains("\npub struct HappenedEvent {"),
                "{}",
                output.events
            );
            output
                .types
                .lines()
                .filter(|line| line.starts_with("use self::"))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        assert!(name_from("Holder", 0, "int").is_empty());
        assert_eq!(
            name_from("HappenedEvent", 0, "Args"),
            ["use self::commands::*;"]
        );
        assert_eq!(
            name_from("HappenedEvent", 0, "Holder"),
            Vec::<String>::new()
        );
        assert_eq!(
            name_from("Holder", 0, "HappenedEvent"),
            ["use self::events::*;"]
        );
        assert_eq!(
            name_from("Args", 0, "HappenedEvent"),
            ["use self::events::*;"],
            "a command naming an event"
        );
        assert_eq!(
            name_from("Holder", 0, "Args"),
            ["use self::commands::*;", "use self::events::*;"]
        );
    }

    /// A command whose arguments are a struct of the same Rust name is one
    /// entry, with the command's features: the struct's own describe the
    /// type.
    #[test]
    fn a_command_merged_with_its_argument_struct_has_its_own_features() {
        let registry = registry(&[
            (
                "v6.1.0",
                "{ 'struct': 'DriveBackup', 'data': { 'a': 'int' }, 'features': [ 'unstable' ] }\n\
                 { 'command': 'drive-backup', 'boxed': true, 'data': 'DriveBackup' }\n\
                 { 'struct': 'SnapSync', 'data': { 'a': 'int' }, 'features': [ 'unstable' ] }\n\
                 { 'command': 'snap-sync', 'data': 'SnapSync', 'features': [ 'deprecated' ] }",
            ),
            (
                "v6.2.0",
                "{ 'struct': 'DriveBackup', 'data': { 'a': 'int' } }\n\
                 { 'command': 'drive-backup', 'boxed': true, 'data': 'DriveBackup', 'features': [ 'deprecated' ] }\n\
                 { 'struct': 'SnapSync', 'data': { 'a': 'int' } }\n\
                 { 'command': 'snap-sync', 'data': 'SnapSync', 'features': [ 'deprecated', 'unstable' ] }",
            ),
        ]);
        let lifecycle = |name: &str| {
            let UnifiedType::Struct(value) = &registry.types[name] else {
                panic!("{name}")
            };
            assert_eq!(
                classify_type(&registry.types[name]),
                TypeCategory::Command
            );
            (
                value.versions.deprecated_ranges.clone(),
                value.versions.unstable_ranges.clone(),
            )
        };
        assert_eq!(
            lifecycle("DriveBackup"),
            (vec![((6, 2, 0), None)], Vec::new())
        );
        assert_eq!(
            lifecycle("SnapSync"),
            (vec![((6, 1, 0), None)], vec![((6, 2, 0), None)])
        );
        let output =
            emit_registry(&registry, Protocol::Qmp, Docs::Reference).unwrap();
        let attributes = |name: &str| -> Vec<&str> {
            let start =
                output.commands.find(&format!("pub struct {name} ")).unwrap();
            output.commands[..start]
                .lines()
                .rev()
                .take_while(|line| line.starts_with("#[qapi("))
                .collect()
        };
        assert!(
            attributes("DriveBackup")
                .contains(&"#[qapi(deprecated(added = \"6.2.0\"))]")
        );
        assert!(
            !attributes("DriveBackup")
                .iter()
                .any(|attr| attr.contains("unstable"))
        );
        assert!(
            attributes("SnapSync")
                .contains(&"#[qapi(unstable(added = \"6.2.0\"))]")
        );
        assert!(output.metadata.contains(
            "CommandMeta { name: \"DriveBackup\", wire_name: \"drive-backup\", args: &STRUCT_DriveBackup, returns: None, allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: None }], since: (6u32, 1u32, 0u32), until: None, deprecated: &[Releases { since: (6u32, 2u32, 0u32), until: None }], unstable: &[] }"
        ));
    }

    #[test]
    fn a_protocol_names_itself_and_its_first_release() {
        assert_eq!(
            [Protocol::Qmp, Protocol::Qga]
                .map(|protocol| (protocol.name(), protocol.first_release())),
            [("QMP", (1, 0, 0)), ("QGA", (0, 15, 0))]
        );
    }
}
