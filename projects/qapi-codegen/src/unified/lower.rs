//! Per-release extraction, parsing and lowering to the Rust IR.

use super::error::{SchemaResult, definition, require};
use super::merge::{field_rust_name, variant_rust_name};
use super::*;
use crate::symbols::synthesized;
use std::collections::HashSet;
use std::path::PathBuf;

/// Process a single version's schema through the IR pipeline, with symbols
/// planned for that version alone.
/// Returns maps of enum name → Enum IR and struct name → Struct IR.
#[cfg(test)]
pub(crate) fn process_version_schema(
    sources: &[(PathBuf, String)],
) -> SchemaResult<(HashMap<String, Enum>, HashMap<String, Struct>)> {
    let (mut symbols, used) = plan_version(sources)?;
    symbols.keep_predefined(&used);
    symbols.finish();
    process_version_schema_with_names(sources, &symbols, &used, &mut Vec::new())
}

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

/// The name of two members of one object, such as `'a'` and `'*a'`, which
/// the IR cannot hold: in a struct, the inline base of a union, the inline
/// arguments of a command or event, or an anonymous struct among them.
fn duplicate_member(token: &QapiSchemaToken<'_>) -> Option<String> {
    fn members(fields: &crate::QapiMembers<'_>) -> Option<String> {
        let mut names = HashSet::new();
        fields.iter().find_map(|field| {
            if !names.insert(field.name.as_ref()) {
                return Some(field.name.to_string());
            }
            match &field.r#type {
                crate::QapiTypeRef::InlineStruct(inner) => members(inner),
                _ => None,
            }
        })
    }
    let data = |data: Option<&crate::MembersOrRef<'_>>| match data {
        Some(crate::MembersOrRef::Members(fields)) => members(fields),
        _ => None,
    };
    match token {
        QapiSchemaToken::Struct(value) => members(&value.data),
        QapiSchemaToken::Union(value) => data(Some(&value.base)),
        QapiSchemaToken::Command(value) => data(value.data.as_ref()),
        QapiSchemaToken::Event(value) => data(value.data.as_ref()),
        _ => None,
    }
}

/// Require a union's base to name a struct, with the kind that QEMU's
/// checks report otherwise.
fn base_kind(
    base: &crate::MembersOrRef<'_>,
    kinds: &HashMap<String, &'static str>,
) -> Result<(), SchemaErrorKind> {
    match base {
        crate::MembersOrRef::Unset => Ok(()),
        crate::MembersOrRef::Members(_) => Err(SchemaErrorKind::AnonymousStruct),
        crate::MembersOrRef::Ref(name) => match kinds.get(name.as_ref()) {
            Some(&"struct") => Ok(()),
            Some(meta) => Err(SchemaErrorKind::TypeNotAllowed {
                meta,
                reference: format!("'{name}'"),
            }),
            None => Err(SchemaErrorKind::UnknownType {
                name: name.to_string(),
            }),
        },
    }
}

/// Where a definition is: its file, and the file's text with the
/// definition's.
type Location<'a> = (Option<&'a Path>, Option<(&'a str, &'a str)>);

/// The line and column at which the definition at `location` starts, found
/// only for an error.
fn position((_, text): Location<'_>) -> Option<(usize, usize)> {
    let (content, text) = text?;
    crate::locate(content, text)
}

/// Lower a struct and the anonymous structs of its members into `structs`.
fn add_struct(
    value: crate::QapiStruct<'_>,
    location: Location<'_>,
    structs: &mut HashMap<String, Struct>,
    names: &mut HashSet<String>,
) -> SchemaResult {
    let label = format!("struct '{}'", value.name);
    let (processed, extracted) = process_struct(value, structs, names).map_err(|error| {
        SchemaError::from(error)
            .located(Some(label), location.0)
            .at(position(location))
    })?;
    for value in std::iter::once(processed).chain(extracted) {
        structs.insert(value.name.clone(), value);
    }
    Ok(())
}

/// Before 2.4 QEMU checks no type reference but a union's base (from 2.0)
/// and event data (from 2.1), so a schema whose definitions use a type
/// that it does not define is valid; the IR cannot represent one. The
/// first such reference of a member, branch, argument or return type, in
/// the order of the files and their definitions, is
/// [`Construct::UndefinedType`]; bases that are no struct are
/// [`Construct::UndefinedBases`] and [`Construct::UnionBase`].
fn undefined_types(
    schemas: &[(&Path, crate::QapiSchema<'_>)],
    contents: &HashMap<&Path, &str>,
) -> SchemaResult {
    use crate::{MembersOrRef, QapiMembers, QapiTypeRef};
    let builtins = crate::builtins::Builtins::current();
    let mut types: HashSet<String> = HashSet::new();
    for token in schemas.iter().flat_map(|(_, schema)| &schema.0) {
        match token {
            QapiSchemaToken::Enum(_)
            | QapiSchemaToken::Struct(_)
            | QapiSchemaToken::Alternate(_) => {
                types.extend(token.definition_name().map(str::to_owned));
            }
            // QEMU defines a tag enum for each union.
            QapiSchemaToken::Union(value) => {
                types.insert(value.name.to_string());
                types.insert(crate::symbols::implicit_tag(&value.name));
            }
            _ => {}
        }
    }
    let defined = |name: &str| builtins.defines(name) || types.contains(name);
    fn members<'t>(fields: &'t QapiMembers<'_>, defined: &dyn Fn(&str) -> bool) -> Option<&'t str> {
        fields
            .0
            .iter()
            .find_map(|field| reference(&field.r#type, defined))
    }
    fn reference<'t>(
        value: &'t QapiTypeRef<'_>,
        defined: &dyn Fn(&str) -> bool,
    ) -> Option<&'t str> {
        match value {
            QapiTypeRef::Ref(name) | QapiTypeRef::ArrayRef(name) => {
                (!defined(name)).then_some(name.as_ref())
            }
            QapiTypeRef::InlineStruct(fields) => members(fields, defined),
            QapiTypeRef::Unset => None,
        }
    }
    fn object<'t>(
        value: Option<&'t MembersOrRef<'_>>,
        defined: &dyn Fn(&str) -> bool,
    ) -> Option<&'t str> {
        match value? {
            MembersOrRef::Ref(name) => {
                (!name.is_empty() && !defined(name)).then_some(name.as_ref())
            }
            MembersOrRef::Members(fields) => members(fields, defined),
            MembersOrRef::Unset => None,
        }
    }
    for (path, schema) in schemas {
        for token in &schema.0 {
            let undefined = match token {
                // Bases that are no struct are constructs of their own.
                QapiSchemaToken::Struct(value) => members(&value.data, &defined),
                QapiSchemaToken::Union(value) => match &value.base {
                    MembersOrRef::Members(fields) => members(fields, &defined),
                    _ => None,
                }
                .or_else(|| {
                    value
                        .data
                        .0
                        .iter()
                        .find_map(|branch| reference(&branch.r#type, &defined))
                }),
                QapiSchemaToken::Command(value) => {
                    object(value.data.as_ref(), &defined).or_else(|| {
                        value
                            .returns
                            .as_ref()
                            .and_then(|returns| reference(returns, &defined))
                    })
                }
                QapiSchemaToken::Event(value) => object(value.data.as_ref(), &defined),
                _ => None,
            };
            if let Some(name) = undefined {
                let location = contents
                    .get(path)
                    .copied()
                    .zip(token.definition_text())
                    .and_then(|(content, text)| crate::locate(content, text));
                return Err(SchemaError::from(SchemaErrorKind::Unsupported {
                    construct: Construct::UndefinedType {
                        name: name.to_owned(),
                    },
                })
                .located(definition(token), Some(path))
                .at(location));
            }
        }
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
    let (mut commands, mut events) = (HashSet::new(), HashSet::new());
    let schemas = crate::builtins::parse_sources(sources, predefined)?;
    crate::schema_validation::validate(&schemas, sources)?;
    let declared = crate::symbols::SchemaNames::declared_names(&schemas);
    // Deferred definitions are lowered after the loop over their files.
    let contents: &HashMap<_, _> = &sources
        .iter()
        .map(|(path, content)| (path.as_path(), content.as_str()))
        .collect();
    fn place<'a>(
        contents: &HashMap<&Path, &'a str>,
        path: &'a Path,
        token: &QapiSchemaToken<'a>,
    ) -> Location<'a> {
        let content = contents.get(path).copied();
        (Some(path), content.zip(token.definition_text()))
    }
    let locations: HashMap<_, _> = schemas
        .iter()
        .flat_map(|(path, schema)| {
            schema
                .0
                .iter()
                .filter_map(move |token| Some((definition(token)?, place(contents, path, token))))
        })
        .collect();
    if crate::parser_version() < (2, 4, 0) {
        undefined_types(&schemas, contents)?;
    }
    let location = |label: &String| locations.get(label).copied().unwrap_or_default();
    let locate = |label: String| {
        let location = location(&label);
        move |error: SchemaError| {
            error
                .located(Some(label.clone()), location.0)
                .at(position(location))
        }
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

    let alternates: HashSet<String> = schemas
        .iter()
        .flat_map(|(_, schema)| &schema.0)
        .filter_map(|token| match token {
            QapiSchemaToken::Alternate(value) => Some(value.name.to_string()),
            _ => None,
        })
        .collect();
    // What each definition is, for the bases of unions.
    let kinds: HashMap<String, &'static str> = schemas
        .iter()
        .flat_map(|(_, schema)| &schema.0)
        .filter_map(|token| {
            let word = match token {
                QapiSchemaToken::Struct(_) => "struct",
                QapiSchemaToken::Enum(_) => "enum",
                QapiSchemaToken::Union(value) if value.legacy_alternate => "alternate",
                QapiSchemaToken::Union(_) => "union",
                QapiSchemaToken::Alternate(_) => "alternate",
                _ => return None,
            };
            Some((token.definition_name()?.to_owned(), word))
        })
        .collect();
    // The tag enums that simple unions define.
    let implicit_tags: HashSet<String> = schemas
        .iter()
        .flat_map(|(_, schema)| &schema.0)
        .filter_map(|token| match token {
            QapiSchemaToken::Union(value)
                if !value.legacy_alternate && value.discriminator.is_none() =>
            {
                Some(crate::symbols::implicit_tag(&value.name))
            }
            _ => None,
        })
        .collect();
    // Resolve exact QAPI names before assigning Rust identities for merging.
    for (path, schema) in schemas {
        for token in schema.0 {
            let label = definition(&token);
            let location = place(contents, path, &token);
            let at = |error: SchemaError| {
                error
                    .located(label.clone(), Some(path))
                    .at(position(location))
            };
            if let (Some((name, documented)), Some(label)) = (documented_symbol(&token), &label)
                && !documented.eq_ignore_ascii_case(name)
            {
                warnings.push(WarningKind::MislabeledDocumentation {
                    definition: label.clone(),
                    documented: documented.to_owned(),
                });
            }
            // Commands and events whose C names clash are valid: the symbol
            // plan gives each its own Rust name. Two of one name are not, and
            // before 2.4 QEMU accepts them; from 2.4 the validation refuses
            // them.
            let shared = match &token {
                QapiSchemaToken::Command(value) => !commands.insert(value.name.to_string()),
                QapiSchemaToken::Event(value) => !events.insert(value.name.to_string()),
                _ => false,
            };
            if let Some(name) = token.definition_name().filter(|_| shared) {
                debug_assert!(
                    crate::parser_version() < (2, 4, 0),
                    "{name} is no duplicate"
                );
                return Err(at(SchemaErrorKind::Unsupported {
                    construct: Construct::SharedName {
                        name: name.to_owned(),
                    },
                }
                .into()));
            }
            let mut defined = Vec::new();
            if let Some(name) = token.definition_name()
                && !matches!(
                    &token,
                    QapiSchemaToken::Command(_) | QapiSchemaToken::Event(_)
                )
            {
                defined.push((name.to_owned(), name));
                // A simple union defines its tag enum as well.
                if let QapiSchemaToken::Union(value) = &token
                    && !value.legacy_alternate
                    && value.discriminator.is_none()
                {
                    defined.push((crate::symbols::implicit_tag(name), name));
                }
            }
            // Before 2.4 QEMU accepts a type of a builtin's name, which the
            // IR could not tell apart from the builtin.
            if let Some((name, _)) = defined.first()
                && crate::parser_version() < (2, 4, 0)
                && crate::builtins::Builtins::current().reserves(name)
            {
                let construct = Construct::BuiltinName { name: name.clone() };
                return Err(at(SchemaErrorKind::Unsupported { construct }.into()));
            }
            for (symbol, name) in defined {
                if let Some((previous, _)) =
                    definitions.insert(symbol.clone(), (name.to_owned(), path))
                {
                    // From 2.4 the validation refuses them in QEMU's one
                    // namespace.
                    debug_assert!(
                        crate::parser_version() < (2, 4, 0),
                        "{symbol} is no duplicate"
                    );
                    return Err(at(SchemaErrorKind::Unsupported {
                        construct: Construct::SharedTypeName {
                            symbol,
                            previous,
                            name: name.to_owned(),
                        },
                    }
                    .into()));
                }
            }
            // From 2.6 the validation refuses them as clashing members.
            if let Some(name) = duplicate_member(&token) {
                let version = crate::parser_version();
                debug_assert!(version < (2, 6, 0), "{name} is no clash");
                return Err(at(if version < (2, 5, 0) {
                    SchemaErrorKind::Unsupported {
                        construct: Construct::DuplicateMember { name },
                    }
                } else {
                    SchemaErrorKind::Crash {
                        crash: Crash::OptionalTwin { name },
                    }
                }
                .into()));
            }
            match token {
                // Before 2.4 the generators read an enum value written as an
                // object loosely.
                QapiSchemaToken::Enum(v)
                    if crate::parser_version() < (2, 4, 0)
                        && v.data.iter().any(|value| value.dictionary) =>
                {
                    let construct = Construct::Expression;
                    return Err(at(SchemaErrorKind::Unsupported { construct }.into()));
                }
                QapiSchemaToken::Enum(v) => {
                    let processed = process_enum(v);
                    enums_lookup.insert(processed.name.clone(), processed);
                }
                // 2.5 checks an alternate's branches against its implicit
                // tag member, `type`.
                QapiSchemaToken::Alternate(v)
                    if ((2, 5, 0)..(2, 6, 0)).contains(&crate::parser_version())
                        && v.data.0.iter().any(|branch| branch.name == "type") =>
                {
                    let crash = Crash::AlternateTypeBranch;
                    return Err(at(SchemaErrorKind::Crash { crash }.into()));
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
                    add_struct(v, location, &mut structs_lookup, &mut names)?;
                }
                QapiSchemaToken::Union(mut value) if value.legacy_alternate => {
                    // Its generators ignore its base, which QEMU checks
                    // from 2.0 like any union's.
                    let base = std::mem::take(&mut value.base);
                    if crate::parser_version() >= (2, 0, 0) {
                        base_kind(&base, &kinds).map_err(|kind| at(kind.into()))?;
                    }
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
                QapiSchemaToken::Unsupported(text) => {
                    let position = contents
                        .get(path)
                        .and_then(|content| crate::locate(content, text));
                    let error = SchemaError::from(SchemaErrorKind::Unsupported {
                        construct: Construct::Expression,
                    });
                    return Err(error.located(None, Some(path)).at(position));
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
            let location = location(&format!("struct '{}'", value.name));
            add_struct(value, location, &mut structs_lookup, &mut names)?;
        }
        if pending.len() == count {
            let bases: HashMap<&str, &str> = pending
                .iter()
                .map(|(value, base)| (value.name.as_ref(), base.as_ref()))
                .collect();
            // A chain of pending bases that comes back to where it started.
            let cyclic = pending.iter().find(|(value, _)| {
                let mut name: &str = value.name.as_ref();
                (0..bases.len()).any(|_| {
                    name = bases.get(name).copied().unwrap_or_default();
                    name == value.name.as_ref()
                })
            });
            // QEMU accepts undefined bases and cycles before 2.4; from 2.4
            // the validation refuses both.
            let (name, kind) = match cyclic {
                Some((value, _)) if crate::parser_version() < (2, 4, 0) => (
                    value.name.to_string(),
                    SchemaErrorKind::Unsupported {
                        construct: Construct::BaseCycle {
                            name: value.name.to_string(),
                        },
                    },
                ),
                _ if crate::parser_version() < (2, 4, 0) => (
                    pending[0].0.name.to_string(),
                    SchemaErrorKind::Unsupported {
                        construct: Construct::UndefinedBases {
                            pending: pending
                                .iter()
                                .map(|(value, base)| (value.name.to_string(), base.to_string()))
                                .collect(),
                        },
                    },
                ),
                _ => (
                    pending[0].0.name.to_string(),
                    SchemaErrorKind::Lower(LowerError::UnknownType {
                        definition: pending[0].0.name.to_string(),
                        name: pending[0].1.to_string(),
                    }),
                ),
            };
            return Err(locate(format!("struct '{name}'"))(kind.into()));
        }
        unprocessed_structs = pending;
    }
    for v in unprocessed_unions {
        let at = locate(format!("union '{}'", v.name));
        // Before 2.0 QEMU does not resolve a union's base, which the IR
        // takes only from a struct.
        if crate::parser_version() < (2, 0, 0)
            && base_kind(&v.base, &kinds).is_err()
            && let crate::MembersOrRef::Ref(base) = &v.base
        {
            return Err(at(SchemaErrorKind::Unsupported {
                construct: Construct::UnionBase {
                    base: base.to_string(),
                },
            }
            .into()));
        }
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
            // QEMU accepts them before 2.0; from 2.0 the validation refuses
            // them.
            if optional || array {
                debug_assert!(crate::parser_version() < (2, 0, 0), "{discriminator}");
                return Err(at(SchemaErrorKind::Unsupported {
                    construct: Construct::DiscriminatorShape {
                        discriminator: discriminator.to_owned(),
                    },
                }
                .into()));
            }
            // A simple union's tag enum, which the IR does not name for
            // other definitions: QEMU fails on it from 2.0 to 4.1 and
            // accepts it from 4.2.
            let version = crate::parser_version();
            if implicit_tags.contains(&discriminator_type)
                && !enums_lookup.contains_key(&discriminator_type)
                && version >= (2, 0, 0)
            {
                let discriminator = discriminator.to_owned();
                return Err(at(if version < (4, 2, 0) {
                    SchemaErrorKind::Crash {
                        crash: Crash::ImplicitTagDiscriminator { discriminator },
                    }
                } else {
                    SchemaErrorKind::Unsupported {
                        construct: Construct::ImplicitTagDiscriminator {
                            discriminator,
                            discriminator_type,
                        },
                    }
                }
                .into()));
            }
            if let Some(tags) = enums_lookup.get(&discriminator_type) {
                let tags: Vec<_> = tags.variants.iter().map(|tag| &tag.name).collect();
                for branch in &v.data.0 {
                    require(
                        tags.iter().any(|tag| **tag == branch.name)
                            || crate::parser_version() < (2, 0, 0),
                        || SchemaErrorKind::UnknownBranch {
                            branch: branch.name.to_string(),
                            discriminator_type: discriminator_type.clone(),
                        },
                    )
                    .map_err(&at)?;
                }
            } else {
                require(crate::parser_version() < (2, 0, 0), || {
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
    // QEMU takes an alternate as boxed arguments until 4.1 (boxed arrives
    // in 2.7); from 4.2 the validation refuses them.
    let boxed_alternate = |boxed: Option<&str>, data: &Option<crate::MembersOrRef<'_>>| match data {
        Some(crate::MembersOrRef::Ref(name))
            if boxed == Some("true") && alternates.contains(name.as_ref()) =>
        {
            debug_assert!(crate::parser_version() < (4, 2, 0), "{name} is boxed");
            Err(SchemaError::from(SchemaErrorKind::Unsupported {
                construct: Construct::BoxedAlternate {
                    name: name.to_string(),
                },
            }))
        }
        _ => Ok(()),
    };
    for v in unprocessed_events {
        let at = locate(format!("event '{}'", v.name));
        boxed_alternate(v.boxed, &v.data).map_err(&at)?;
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
        boxed_alternate(v.boxed, &v.data).map_err(&at)?;
        // 2.4 accepts an anonymous struct as the return type alone.
        if ((2, 4, 0)..(2, 5, 0)).contains(&crate::parser_version())
            && matches!(v.returns, Some(crate::QapiTypeRef::InlineStruct(_)))
        {
            return Err(at(SchemaErrorKind::Unsupported {
                construct: Construct::AnonymousStruct {
                    member: "returns".into(),
                },
            }
            .into()));
        }
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
    protocol: Protocol,
) -> Result<Option<VersionSources>, RegistryError> {
    let order =
        parse_version_tag_full(tag).ok_or_else(|| RegistryError::Tag { tag: tag.into() })?;
    let version = (order.0, order.1, order.2);
    let tmpdir =
        extract_schema_from_tag(qemu_repo, tag).map_err(|source| RegistryError::Extract {
            tag: tag.into(),
            source,
        })?;
    let entry = match protocol {
        Protocol::Qmp => find_schema_entry(tmpdir.path()),
        Protocol::Qga => find_qga_schema_entry(tmpdir.path()),
    };
    let schema_entry = match entry {
        Some(entry) => entry,
        None if version < protocol.first_release() => return Ok(None),
        None => {
            return Err(RegistryError::MissingEntry {
                tag: tag.into(),
                protocol: protocol.name(),
            });
        }
    };
    let sources =
        crate::read_schema_with_version(&schema_entry, Some(version)).map_err(|source| {
            RegistryError::Read {
                tag: tag.into(),
                source: tmpdir.label_error(source),
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
        if protocol == Protocol::Qga {
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
        sources: sources
            .into_iter()
            .map(|(path, content)| (tmpdir.label(&path), content))
            .collect(),
    }))
}

pub(super) fn load_versions(
    qemu_repo: &Path,
    tags: &[&str],
    protocol: Protocol,
) -> Result<Vec<VersionResult>, RegistryError> {
    let extracted = parse_versions(tags, |tag| {
        extract_version_sources(qemu_repo, tag, protocol)
    })?;
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
