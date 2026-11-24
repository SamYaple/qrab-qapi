//! Check a release's definitions as that release's QAPI generator does, in
//! the order in which it checks them.
//!
//! QEMU checks nothing before 2.0, and until 2.3 only unions with a base
//! and, from 2.1, the members of event data. From 2.4 to 4.1 its generator
//! first learns every definition, checking its keys, its name and that it
//! is new, then checks each expression, then (from 2.6) the schema of
//! types that it builds. From 4.2 it checks every expression first, then
//! defines the schema's entities, which finds duplicates, then checks each
//! entity with the types it uses. Every error names the definition that
//! QEMU's message names, at the line and column where it starts.

use crate::builtins::Builtins;
use crate::parsers::common::{Malformed, Raw, RawValue};
use crate::schema_docs::{self, Pragmas};
use crate::schema_shapes::bad;
use crate::unified::{
    NameRule, SchemaContext, SchemaError, SchemaErrorKind, SchemaResult, Version, definition,
    require,
};
use crate::union_layout::{Member, Objects};
use crate::{
    MembersOrRef, QapiCond, QapiFeatures, QapiMember, QapiMembers, QapiPragma, QapiSchema,
    QapiSchemaToken, QapiTypeRef, QapiUnion, QapiVariant,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Whether the release being checked is `version` or later.
pub(crate) fn since(version: Version) -> bool {
    crate::parser_version() >= version
}

/// What a name defines, in the words of QEMU's messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Meta {
    Builtin,
    Enum,
    Struct,
    Union,
    Alternate,
    Command,
    Event,
}

impl Meta {
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Builtin => "built-in",
            Self::Enum => "enum",
            Self::Struct => "struct",
            Self::Union => "union",
            Self::Alternate => "alternate",
            Self::Command => "command",
            Self::Event => "event",
        }
    }

    fn of(token: &QapiSchemaToken<'_>) -> Option<Self> {
        Some(match token {
            QapiSchemaToken::Enum(_) => Self::Enum,
            QapiSchemaToken::Struct(_) => Self::Struct,
            QapiSchemaToken::Union(value) if value.legacy_alternate => Self::Alternate,
            QapiSchemaToken::Union(_) => Self::Union,
            QapiSchemaToken::Alternate(_) => Self::Alternate,
            QapiSchemaToken::Command(_) => Self::Command,
            QapiSchemaToken::Event(_) => Self::Event,
            _ => return None,
        })
    }
}

/// A type reference in QAPI syntax: `'name'`, or `['name']` for an array.
pub(crate) fn spelled(value: &QapiTypeRef<'_>) -> String {
    match value {
        QapiTypeRef::Ref(name) => format!("'{name}'"),
        QapiTypeRef::ArrayRef(name) => format!("['{name}']"),
        QapiTypeRef::InlineStruct(_) | QapiTypeRef::Unset => "{}".into(),
    }
}

/// The hard-coded lists of commands that may return any type, before the
/// `returns-whitelist` pragma of QEMU 2.9: 2.4's, then that of 2.5 to 2.8.
const RETURNS: [&[&str]; 2] = [
    &[
        "human-monitor-command",
        "query-migrate-cache-size",
        "query-tpm-models",
        "query-tpm-types",
        "ringbuf-read",
        "guest-file-open",
        "guest-fsfreeze-freeze",
        "guest-fsfreeze-freeze-list",
        "guest-fsfreeze-status",
        "guest-fsfreeze-thaw",
        "guest-get-time",
        "guest-set-vcpus",
        "guest-sync",
        "guest-sync-delimited",
        "user_def_cmd3",
    ],
    &[
        "human-monitor-command",
        "qom-get",
        "query-migrate-cache-size",
        "query-tpm-models",
        "query-tpm-types",
        "ringbuf-read",
        "guest-file-open",
        "guest-fsfreeze-freeze",
        "guest-fsfreeze-freeze-list",
        "guest-fsfreeze-status",
        "guest-fsfreeze-thaw",
        "guest-get-time",
        "guest-set-vcpus",
        "guest-sync",
        "guest-sync-delimited",
    ],
];

/// The hard-coded owners whose members may use uppercase, from QEMU 2.6 to
/// 2.8, before the `name-case-whitelist` pragma.
const CASE_2_6: &[&str] = &[
    "ACPISlotType",
    "CpuInfoMIPS",
    "CpuInfoTricore",
    "QapiErrorClass",
    "UuidInfo",
    "X86CPURegister32",
    "q_obj_CpuInfo-base",
];

/// The exceptions that the schema's pragmas grant: the last list that the
/// schema sets for each, as in QEMU.
#[derive(Default)]
struct Pragma<'a> {
    returns: Vec<&'a str>,
    member_names: Vec<&'a str>,
    command_names: Vec<&'a str>,
}

/// Arguments that a command or event does not have.
const NONE: MembersOrRef<'static> = MembersOrRef::Unset;

/// The JSON types that tell an alternate's branches apart.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Json {
    Null,
    Bool,
    Integer,
    Float,
    Number,
    String,
    Object,
    Array,
}

/// A definition and where it starts.
struct Definition<'a, 's> {
    path: &'a Path,
    /// The text of its file.
    content: &'a str,
    token: &'a QapiSchemaToken<'s>,
    /// Its expression in `content`, which an error locates.
    text: &'a str,
    /// A definition that QEMU makes itself, which is never checked.
    predefined: bool,
}

/// An entity of the schema that QEMU's generator builds from 2.6: a
/// definition, or a type that QEMU makes for one, whose errors belong to
/// that definition.
enum Entity<'a, 's> {
    /// The array type of `element`, made where a definition first uses it.
    Array {
        element: &'a str,
        owner: usize,
    },
    /// A command's or event's arguments or a union's base written out in
    /// place, the object type `q_obj_{name}-arg` or `q_obj_{name}-base`.
    Implicit(String),
    /// The object type that wraps a simple union branch's type, made where
    /// a branch first uses that type.
    Wrapper {
        branch: &'a QapiVariant<'s>,
        owner: usize,
    },
    /// A simple union's tag enum.
    Kind(usize),
    Definition(usize),
}

/// Before 2.4, an expression that the generators read loosely, which the
/// grammar here does not describe.
struct Loose<'a> {
    /// The number of definitions before it.
    at: usize,
    path: &'a Path,
    content: &'a str,
    text: &'a str,
    /// Its kind in the order QEMU 2.0 to 2.3 look for one, its name and
    /// whether it has `data`.
    meta: Option<&'static str>,
    name: Option<String>,
    data: bool,
    /// A union's base and discriminator, as strings.
    base: Option<String>,
    discriminator: Option<String>,
}

impl<'a> Loose<'a> {
    fn new(path: &'a Path, content: &'a str, text: &'a str, at: usize) -> Self {
        let entries = match crate::parsers::common::expression_value(text).map(|value| value.value)
        {
            Some(crate::parsers::common::Raw::Object(entries)) => entries,
            _ => Vec::new(),
        };
        let string = |key: &str| {
            entries.iter().find_map(|(name, value)| match &value.value {
                crate::parsers::common::Raw::String(value) if name == key => {
                    Some(value.to_string())
                }
                _ => None,
            })
        };
        let has = |key: &str| entries.iter().any(|(name, _)| name == key);
        let meta = ["enum", "union", "type"].into_iter().find(|meta| has(meta));
        Self {
            at,
            path,
            content,
            text,
            name: meta.and_then(string),
            meta,
            data: has("data"),
            base: string("base"),
            discriminator: string("discriminator"),
        }
    }

    /// QEMU's failure on its missing `data`, at it.
    fn crash<T>(&self, name: &str) -> SchemaResult<T> {
        Err(SchemaError::from(SchemaErrorKind::Crash {
            crash: crate::unified::Crash::MissingData {
                name: name.to_owned(),
            },
        }))
        .in_file(self.path)
        .at(|| crate::locate(self.content, self.text))
    }
}

/// The definitions of one release, in QEMU's order.
pub(crate) struct Schema<'a, 's> {
    definitions: Vec<Definition<'a, 's>>,
    names: HashMap<&'a str, usize>,
    /// The tag enums that QEMU defines for simple unions, `{Union}Kind`,
    /// and until 4.1 for alternates.
    implicit_enums: HashSet<String>,
    /// Before 2.4, the expressions that the generators read loosely.
    loose: Vec<Loose<'a>>,
    /// The object types that QEMU makes of members written out in place,
    /// a command's or event's arguments and a union's base, by name: the
    /// definition and the members.
    implicit: HashMap<String, (usize, &'a QapiMembers<'s>)>,
    pragma: Pragma<'a>,
    /// The pragmas of the documentation rules.
    docs: Pragmas,
}

/// Check the schema of the release being lowered.
pub(crate) fn validate(
    schemas: &[(&Path, QapiSchema<'_>)],
    sources: &[(PathBuf, String)],
) -> SchemaResult {
    Schema::new(schemas, sources).check()
}

/// The schema's tokens in the order QEMU reads them: each file in place of
/// the include that names it first.
fn in_include_order<'a, 's>(
    schemas: &'a [(&'a Path, QapiSchema<'s>)],
) -> Vec<(&'a Path, &'a QapiSchemaToken<'s>)> {
    let files: HashMap<&Path, &QapiSchema<'_>> = schemas
        .iter()
        .map(|(path, schema)| (*path, schema))
        .collect();
    let include = |from: &Path, name: &str| -> Option<&'a Path> {
        let target = sibling(from, name)?;
        let canonical = target.canonicalize().ok();
        schemas.iter().map(|(path, _)| *path).find(|path| {
            *path == target || (canonical.is_some() && path.canonicalize().ok() == canonical)
        })
    };
    let included: HashSet<&Path> = schemas
        .iter()
        .flat_map(|(path, schema)| {
            schema.0.iter().filter_map(move |token| match token {
                QapiSchemaToken::Include(name) => include(path, &name.0),
                _ => None,
            })
        })
        .collect();
    let mut order = Vec::new();
    let mut seen = HashSet::new();
    let mut stack: Vec<_> = schemas
        .iter()
        .rev()
        .map(|(path, _)| *path)
        .filter(|path| !included.contains(path))
        .map(|path| (path, 0))
        .collect();
    // Depth first, each file in its order.
    while let Some((path, next)) = stack.pop() {
        if next == 0 && !seen.insert(path) {
            continue;
        }
        for (position, token) in files[path].0.iter().enumerate().skip(next) {
            order.push((path, token));
            if let QapiSchemaToken::Include(name) = token
                && let Some(target) = include(path, &name.0)
            {
                stack.push((path, position + 1));
                stack.push((target, 0));
                break;
            }
        }
    }
    order
}

/// The file `name` next to `from`. A release's files are labelled with its
/// tag ([`crate::versions::ExtractedSchemas::label`]), so a file at the top
/// of the tree, `v2.9.0:qapi-schema.json`, has its siblings under that label.
fn sibling(from: &Path, name: &str) -> Option<PathBuf> {
    let parent = from.parent()?;
    if parent.as_os_str().is_empty()
        && let Some((label, _)) = from.to_str().and_then(|from| from.split_once(':'))
    {
        return Some(PathBuf::from(format!("{label}:{name}")));
    }
    Some(parent.join(name))
}

/// The schema's pragma directives in the order QEMU reads them, so the
/// value set last wins.
fn pragmas<'a, 's>(schemas: &'a [(&'a Path, QapiSchema<'s>)]) -> Vec<&'a QapiPragma<'s>> {
    in_include_order(schemas)
        .into_iter()
        .filter_map(|(_, token)| match token {
            QapiSchemaToken::Pragma(pragma) => Some(pragma),
            _ => None,
        })
        .collect()
}

impl<'a, 's> Schema<'a, 's> {
    fn new(schemas: &'a [(&'a Path, QapiSchema<'s>)], sources: &'a [(PathBuf, String)]) -> Self {
        let contents: HashMap<_, _> = sources
            .iter()
            .map(|(path, content)| (path.as_path(), content.as_str()))
            .collect();
        let pragmas = pragmas(schemas);
        let mut pragma = Pragma::default();
        for value in &pragmas {
            for (list, target) in [
                (&value.command_returns_exceptions, &mut pragma.returns),
                (&value.member_name_exceptions, &mut pragma.member_names),
                (&value.command_name_exceptions, &mut pragma.command_names),
            ] {
                if let Some(list) = list {
                    *target = list.iter().map(|name| name.as_ref()).collect();
                }
            }
        }
        let mut definitions = Vec::new();
        let mut loose = Vec::new();
        for (path, token) in in_include_order(schemas) {
            if let QapiSchemaToken::Unsupported(text) = token
                && since((2, 0, 0))
            {
                let content = contents.get(path).copied().unwrap_or_default();
                loose.push(Loose::new(path, content, text, definitions.len()));
            }
            let Some(text) = token.definition_text() else {
                continue;
            };
            let content = contents.get(path).copied();
            definitions.push(Definition {
                path,
                content: content.unwrap_or_default(),
                token,
                text,
                predefined: content.is_none(),
            });
        }
        let mut names = HashMap::new();
        let mut implicit_enums = HashSet::new();
        let mut implicit = HashMap::new();
        for (index, item) in definitions.iter().enumerate() {
            let name = item.token.definition_name().unwrap_or_default();
            names.entry(name).or_insert(index);
            let (role, fields) = match item.token {
                QapiSchemaToken::Union(value) => {
                    // 2.4 and 2.5 define the tag enum of an alternate too.
                    if value.discriminator.is_none() && !value.legacy_alternate {
                        implicit_enums.insert(crate::symbols::implicit_tag(name));
                    }
                    ("base", &value.base)
                }
                // Until 4.1 the expressions know the tag enum of an
                // alternate, which only 2.4 and 2.5 define.
                QapiSchemaToken::Alternate(_) => {
                    if !since((4, 2, 0)) {
                        implicit_enums.insert(crate::symbols::implicit_tag(name));
                    }
                    continue;
                }
                QapiSchemaToken::Command(value) => ("arg", value.data.as_ref().unwrap_or(&NONE)),
                QapiSchemaToken::Event(value) => ("arg", value.data.as_ref().unwrap_or(&NONE)),
                _ => continue,
            };
            if let MembersOrRef::Members(fields) = fields
                && !fields.0.is_empty()
            {
                implicit.insert(format!("q_obj_{name}-{role}"), (index, fields));
            }
        }
        Self {
            definitions,
            names,
            implicit_enums,
            loose,
            implicit,
            pragma,
            docs: Pragmas::of(&pragmas),
        }
    }

    /// Name the definition at `index` as the place of an error that does
    /// not name one already.
    pub(crate) fn located<T>(&self, index: usize, result: SchemaResult<T>) -> SchemaResult<T> {
        let item = &self.definitions[index];
        result
            .in_definition(|| definition(item.token))
            .in_file(item.path)
            .at(|| crate::locate(item.content, item.text))
    }

    pub(crate) fn index(&self, name: &str) -> Option<usize> {
        self.names.get(name).copied()
    }

    /// The implicit object type `name`: the definition that QEMU makes it
    /// for, and its members.
    pub(crate) fn implicit(&self, name: &str) -> Option<(usize, &'a QapiMembers<'s>)> {
        self.implicit.get(name).copied()
    }

    pub(crate) fn token(&self, name: &str) -> Option<&'a QapiSchemaToken<'s>> {
        self.index(name).map(|index| self.definitions[index].token)
    }

    /// What `name` defines, if anything.
    pub(crate) fn meta(&self, name: &str) -> Option<Meta> {
        if Builtins::current().defines(name) {
            return Some(Meta::Builtin);
        }
        // Until 4.1 the empty struct is no type that a definition can use,
        // and from 2.6 the expressions take `QType` for a builtin type.
        if name == "q_empty" && !since((4, 2, 0)) {
            return None;
        }
        if name == "QType" && since((2, 6, 0)) && !since((4, 2, 0)) {
            return Some(Meta::Builtin);
        }
        if let Some(token) = self.token(name) {
            return Meta::of(token);
        }
        self.implicit_enums.contains(name).then_some(Meta::Enum)
    }

    /// Refuse the tag enum of an alternate as a type, which QEMU 2.6 to 4.1
    /// fail on.
    pub(crate) fn alternate_tag(&self, name: &str) -> SchemaResult {
        let tag = since((2, 6, 0))
            && !since((4, 2, 0))
            && self.token(name).is_none()
            && name
                .strip_suffix("Kind")
                .and_then(|alternate| self.token(alternate))
                .is_some_and(|token| matches!(token, QapiSchemaToken::Alternate(_)));
        require(!tag, || SchemaErrorKind::Crash {
            crash: crate::unified::Crash::AlternateTag {
                name: name.to_owned(),
            },
        })
    }

    /// The values of the enum `name`, if it is one.
    pub(crate) fn enum_values(&self, name: &str) -> Option<Vec<&'a str>> {
        if let Some(QapiSchemaToken::Enum(value)) = self.token(name) {
            return Some(value.data.iter().map(|value| value.name.as_ref()).collect());
        }
        let union = name.strip_suffix("Kind")?;
        match self.token(union)? {
            QapiSchemaToken::Union(value) if self.implicit_enums.contains(name) => Some(
                value
                    .data
                    .0
                    .iter()
                    .map(|branch| branch.name.as_ref())
                    .collect(),
            ),
            _ => None,
        }
    }

    /// Whether the release lets the members of `owner` use uppercase (until
    /// 5.2) or uppercase and `_` (from 6.0): a hard-coded list until 2.8,
    /// then the `name-case-whitelist` or `member-name-exceptions` pragma.
    pub(crate) fn case_exempt(&self, owner: &str) -> bool {
        if since((2, 9, 0)) {
            self.pragma.member_names.contains(&owner)
        } else {
            CASE_2_6.contains(&owner)
        }
    }

    fn returns_exempt(&self, command: &str) -> bool {
        if since((2, 9, 0)) {
            self.pragma.returns.contains(&command)
        } else {
            RETURNS[usize::from(since((2, 5, 0)))].contains(&command)
        }
    }

    fn check(&self) -> SchemaResult {
        let all: Vec<_> = (0..self.definitions.len())
            .filter(|index| !self.definitions[*index].predefined)
            .collect();
        if !since((2, 4, 0)) {
            self.missing_data()?;
            // The checks of unions and events, and the loose unions in
            // their order.
            let mut loose = self.loose.iter().peekable();
            for &index in &all {
                while let Some(item) = loose.next_if(|item| item.at <= index) {
                    self.loose_union(item)?;
                }
                self.located(index, self.early(index))?;
            }
            for item in loose {
                self.loose_union(item)?;
            }
            return Ok(());
        }
        let mut objects = Objects::default();
        let mut defined = HashMap::new();
        if !since((4, 2, 0)) {
            for &index in &all {
                self.located(index, self.learn(index, &mut defined))?;
            }
            for &index in &all {
                self.located(index, self.discriminator_key(index))?;
            }
            for &index in &all {
                let item = &self.definitions[index];
                let check = self
                    .expression_classic(index)
                    .and_then(|()| schema_docs::check_sections(item.token, item.content));
                self.located(index, check)?;
            }
            // QEMU then builds the schema, from 2.5.
            if since((2, 5, 0)) {
                for &index in &all {
                    self.located(index, self.build(index))?;
                }
            }
            if since((2, 6, 0)) {
                for entity in self.entities(&all) {
                    self.entity(&entity, &mut objects)?;
                    // The documentation's descriptions are checked with
                    // each type, where QEMU checks it.
                    if let Entity::Definition(index) = entity
                        && !matches!(
                            self.definitions[index].token,
                            QapiSchemaToken::Struct(_) | QapiSchemaToken::Union(_)
                        )
                    {
                        self.described(index)?;
                    }
                }
            }
            return self.unchecked(&all);
        }
        for &index in &all {
            self.located(index, self.expression(index))?;
        }
        for &index in &all {
            self.located(index, self.define(index, &mut defined))?;
        }
        // QEMU connects each definition to its documentation after its
        // types, and checks the descriptions right after until 8.2, after
        // every definition from 9.0.
        let interleaved = !since((9, 0, 0));
        for entity in self.entities(&all) {
            self.entity(&entity, &mut objects)?;
            let Entity::Definition(index) = entity else {
                continue;
            };
            let token = self.definitions[index].token;
            self.located(index, schema_docs::connect(token, &self.docs))?;
            if interleaved {
                self.described(index)?;
            }
        }
        if !interleaved {
            for &index in &all {
                self.described(index)?;
            }
        }
        self.feature_limit(&all)?;
        self.unchecked(&all)
    }

    /// The values of a shape that QEMU's generator uses without checking
    /// it, where building the schema fails: from 2.5 to 4.1 `null` as a
    /// struct's members, as the type of a member or branch, or in 4.1 as
    /// the features of a struct; from 4.2 to 8.0 a type name as the
    /// members of a struct.
    fn build(&self, index: usize) -> SchemaResult {
        let token = self.definitions[index].token;
        let malformed = malformed_of(token);
        let null = |raw: Option<&RawValue<'_>>| raw.is_some_and(|raw| raw.value == Raw::Null);
        let members = |fields: &QapiMembers<'_>| {
            require(
                !fields
                    .0
                    .iter()
                    .any(|field| null(bad(&field.malformed, "type"))),
                || SchemaErrorKind::Crash {
                    crash: crate::unified::Crash::UncheckedValue { key: "type".into() },
                },
            )
        };
        if since((4, 2, 0)) {
            let named = matches!(
                bad(malformed, "data").map(|raw| &raw.value),
                Some(Raw::String(_))
            );
            if matches!(token, QapiSchemaToken::Struct(_)) && named && !since((8, 1, 0)) {
                return Err(crate::schema_shapes::unchecked("data"));
            }
            return Ok(());
        }
        match token {
            QapiSchemaToken::Struct(value) => {
                if null(bad(malformed, "data")) {
                    return Err(crate::schema_shapes::unchecked("data"));
                }
                members(&value.data)?;
                // 4.1 iterates the features that its checks skip as false.
                let iterable = |raw: &RawValue<'_>| !matches!(raw.value, Raw::Null | Raw::Bool(_));
                if since((4, 1, 0)) && bad(malformed, "features").is_some_and(|raw| !iterable(raw))
                {
                    return Err(crate::schema_shapes::unchecked("features"));
                }
            }
            QapiSchemaToken::Union(value) => {
                if let MembersOrRef::Members(fields) = &value.base {
                    members(fields)?;
                }
                if value
                    .data
                    .0
                    .iter()
                    .any(|branch| null(bad(&branch.malformed, "type")))
                {
                    return Err(crate::schema_shapes::unchecked("type"));
                }
            }
            QapiSchemaToken::Command(crate::QapiCommand {
                data: Some(MembersOrRef::Members(fields)),
                ..
            })
            | QapiSchemaToken::Event(crate::QapiEvent {
                data: Some(MembersOrRef::Members(fields)),
                ..
            }) => members(fields)?,
            _ => {}
        }
        Ok(())
    }

    /// A value that the grammar refused and that the release's QEMU takes
    /// without checking it, once nothing else is wrong.
    fn unchecked(&self, all: &[usize]) -> SchemaResult {
        for &index in all {
            if let Some(key) = leftover(self.definitions[index].token) {
                return self.located(index, Err(crate::schema_shapes::unsupported(key)));
            }
        }
        Ok(())
    }

    /// The entities of the schema that QEMU builds from 2.6, in the order
    /// it checks them: each definition after the types that QEMU makes for
    /// it, and in 2.11.2 all of them in the order of their names.
    fn entities(&self, all: &[usize]) -> Vec<Entity<'a, 's>> {
        let mut entities = Vec::new();
        let mut made = HashSet::new();
        let mut array = |entities: &mut Vec<_>, value: &'a QapiTypeRef<'s>, owner| {
            if let QapiTypeRef::ArrayRef(element) = value
                && !Builtins::current().defines(element)
                && made.insert(format!("{element}List"))
            {
                entities.push(Entity::Array { element, owner });
            }
        };
        let mut wrapped = HashSet::new();
        for &index in all {
            let token = self.definitions[index].token;
            let name = token.definition_name().unwrap_or_default();
            let implicit = |entities: &mut Vec<_>,
                            array: &mut dyn FnMut(&mut Vec<_>, &'a QapiTypeRef<'s>, usize),
                            role: &str,
                            fields: &'a QapiMembers<'s>| {
                for field in &fields.0 {
                    array(entities, &field.r#type, index);
                }
                if !fields.0.is_empty() {
                    entities.push(Entity::Implicit(format!("q_obj_{name}-{role}")));
                }
            };
            match token {
                QapiSchemaToken::Struct(value) => {
                    for field in &value.data.0 {
                        array(&mut entities, &field.r#type, index);
                    }
                }
                QapiSchemaToken::Union(value) => {
                    if let MembersOrRef::Members(fields) = &value.base {
                        implicit(&mut entities, &mut array, "base", fields);
                    }
                    if value.discriminator.is_none() && !value.legacy_alternate {
                        for branch in &value.data.0 {
                            array(&mut entities, &branch.r#type, index);
                            // A wrapper type for each type that branches use.
                            if wrapped.insert(spelled(&branch.r#type)) {
                                entities.push(Entity::Wrapper {
                                    branch,
                                    owner: index,
                                });
                            }
                        }
                        entities.push(Entity::Kind(index));
                    }
                }
                QapiSchemaToken::Alternate(value) if since((7, 1, 0)) => {
                    for branch in &value.data.0 {
                        array(&mut entities, &branch.r#type, index);
                    }
                }
                QapiSchemaToken::Command(value) => {
                    if let Some(MembersOrRef::Members(fields)) = &value.data {
                        implicit(&mut entities, &mut array, "arg", fields);
                    }
                    if let Some(returns) = &value.returns {
                        array(&mut entities, returns, index);
                    }
                }
                QapiSchemaToken::Event(value) => {
                    if let Some(MembersOrRef::Members(fields)) = &value.data {
                        implicit(&mut entities, &mut array, "arg", fields);
                    }
                }
                _ => {}
            }
            entities.push(Entity::Definition(index));
        }
        if since((2, 11, 2)) && !since((2, 12, 0)) {
            entities.sort_by_cached_key(|entity| self.entity_name(entity));
        }
        entities
    }

    /// The name of an entity in QEMU's schema.
    fn entity_name(&self, entity: &Entity<'_, '_>) -> String {
        let name = |index: usize| {
            self.definitions[index]
                .token
                .definition_name()
                .unwrap_or_default()
        };
        match entity {
            Entity::Array { element, .. } => format!("{element}List"),
            Entity::Implicit(name) => name.clone(),
            Entity::Wrapper { branch, .. } => match &branch.r#type {
                QapiTypeRef::ArrayRef(element) => format!("q_obj_{element}List-wrapper"),
                QapiTypeRef::Ref(name) => format!("q_obj_{name}-wrapper"),
                _ => String::new(),
            },
            Entity::Kind(index) => crate::symbols::implicit_tag(name(*index)),
            Entity::Definition(index) => name(*index).to_owned(),
        }
    }

    /// Check that the documentation of the definition at `index` describes
    /// only what it has.
    pub(crate) fn described(&self, index: usize) -> SchemaResult {
        let item = &self.definitions[index];
        self.located(
            index,
            schema_docs::check_described(item.token, item.content),
        )
    }

    /// The struct that QEMU 2.0 to 2.3 find first by the name `name`: a
    /// definition, or a loose expression, which is `Err` without `data`.
    fn first_struct(&self, name: &str) -> Option<Result<&'a QapiMembers<'s>, &Loose<'a>>> {
        let definition = self
            .definitions
            .iter()
            .enumerate()
            .find_map(|(index, item)| match item.token {
                QapiSchemaToken::Struct(value) if value.name == name => Some((index, &value.data)),
                _ => None,
            });
        let loose = self
            .loose
            .iter()
            .find(|item| item.meta == Some("type") && item.name.as_deref() == Some(name));
        match (definition, loose) {
            (Some((index, _)), Some(item)) if item.at <= index => Some(Err(item)),
            (Some((_, fields)), _) => Some(Ok(fields)),
            (None, Some(item)) => Some(Err(item)),
            (None, None) => None,
        }
    }

    /// The failures of QEMU 2.0 to 2.3 on a missing `data` before their
    /// checks: as they learn the definitions, an enum's, then as they look
    /// for the tag enum of each union with a base and discriminator, its
    /// base's.
    fn missing_data(&self) -> SchemaResult {
        if !since((2, 0, 0)) {
            return Ok(());
        }
        if let Some(item) = self
            .loose
            .iter()
            .find(|item| item.meta == Some("enum") && !item.data)
        {
            return item.crash(item.name.as_deref().unwrap_or_default());
        }
        let mut loose = self.loose.iter().peekable();
        for (index, item) in self.definitions.iter().enumerate() {
            while let Some(union) = loose.next_if(|union| union.at <= index) {
                if let (Some(base), Some(_)) = (&union.base, &union.discriminator)
                    && let Some(Err(base)) = self.first_struct(base)
                {
                    return union.crash(base.name.as_deref().unwrap_or_default());
                }
            }
            if let QapiSchemaToken::Union(value) = item.token
                && let (MembersOrRef::Ref(base), Some(discriminator)) =
                    (&value.base, value.discriminator.as_deref())
                && !base.is_empty()
                && !discriminator.is_empty()
            {
                self.located(index, self.base_data(base).map(drop))?;
            }
        }
        for union in loose {
            if let (Some(base), Some(_)) = (&union.base, &union.discriminator)
                && let Some(Err(base)) = self.first_struct(base)
            {
                return union.crash(base.name.as_deref().unwrap_or_default());
            }
        }
        Ok(())
    }

    /// The members of the struct `base` that QEMU 2.0 to 2.3 find first,
    /// which fail without `data`.
    fn base_data(&self, base: &str) -> SchemaResult<Option<&'a QapiMembers<'s>>> {
        match self.first_struct(base) {
            Some(Ok(fields)) => Ok(Some(fields)),
            Some(Err(_)) => Err(SchemaErrorKind::Crash {
                crash: crate::unified::Crash::MissingData {
                    name: base.to_owned(),
                },
            }
            .into()),
            None => Ok(None),
        }
    }

    /// A loose union, which QEMU 2.0 to 2.3 check as any union, first
    /// looking up its `data`, then its base.
    fn loose_union(&self, item: &Loose<'a>) -> SchemaResult {
        if !since((2, 0, 0)) || item.meta != Some("union") {
            return Ok(());
        }
        let name = item.name.as_deref().unwrap_or_default();
        if !item.data {
            return item.crash(name);
        }
        match item.base.as_deref().map(|base| self.first_struct(base)) {
            Some(Some(Err(_))) => item.crash(item.base.as_deref().unwrap_or_default()),
            _ => Ok(()),
        }
    }

    /// The checks before QEMU 2.4: from 2.0 the bases and discriminators
    /// of unions and, from 2.1, the members of event data. QEMU looks a
    /// name up among the definitions of one kind, the first of them.
    fn early(&self, index: usize) -> SchemaResult {
        let token = self.definitions[index].token;
        let first = |name: &str, kind: fn(&QapiSchemaToken<'_>) -> bool| {
            self.definitions
                .iter()
                .map(|item| item.token)
                .find(|token| token.definition_name() == Some(name) && kind(token))
        };
        let structure = |name: &str| self.base_data(name);
        match token {
            QapiSchemaToken::Union(value) if since((2, 0, 0)) => {
                // An empty base or discriminator counts as none.
                let base = match &value.base {
                    MembersOrRef::Ref(name) if !name.is_empty() => {
                        let Some(fields) = structure(name)? else {
                            return Err(self.not_allowed(name)).within(|| "'base'".into());
                        };
                        require(!fields.0.is_empty(), || SchemaErrorKind::EmptyBase {
                            name: name.to_string(),
                        })?;
                        Some(fields)
                    }
                    MembersOrRef::Members(fields) if !fields.0.is_empty() => {
                        return Err(SchemaErrorKind::AnonymousStruct).within(|| "'base'".into());
                    }
                    _ => None,
                };
                let discriminator = value.discriminator.as_deref().unwrap_or_default();
                if value.legacy_alternate || discriminator.is_empty() {
                    return Ok(());
                }
                let base = base.ok_or(SchemaErrorKind::MissingBase)?;
                // QEMU looks the discriminator up among the keys as
                // written, so an optional member is none.
                let member = base
                    .0
                    .iter()
                    .find(|member| member.name == discriminator && !member.optional)
                    .ok_or_else(|| SchemaErrorKind::DiscriminatorNotMember {
                        discriminator: discriminator.to_owned(),
                    })?;
                let enumeration = match &member.r#type {
                    QapiTypeRef::Ref(name) => {
                        match first(name, |token| matches!(token, QapiSchemaToken::Enum(_))) {
                            Some(QapiSchemaToken::Enum(value)) => Some((
                                name,
                                value
                                    .data
                                    .iter()
                                    .map(|value| value.name.as_ref())
                                    .collect::<Vec<_>>(),
                            )),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                let Some((enumeration, values)) = enumeration else {
                    return Err(SchemaErrorKind::DiscriminatorNotEnum {
                        discriminator: discriminator.to_owned(),
                        discriminator_type: spelled(&member.r#type).trim_matches('\'').to_owned(),
                    }
                    .into());
                };
                for branch in &value.data.0 {
                    self.branch_value(&branch.name, enumeration, &values)?;
                }
                Ok(())
            }
            QapiSchemaToken::Event(value) if since((2, 1, 0)) => {
                // Members of the data, or of the struct it names, may not
                // be objects written out in place.
                let fields = match &value.data {
                    Some(MembersOrRef::Members(fields)) => fields,
                    Some(MembersOrRef::Ref(name)) if !name.is_empty() => match structure(name)? {
                        Some(fields) => fields,
                        None => {
                            return Err(SchemaErrorKind::Crash {
                                crash: crate::unified::Crash::EventData {
                                    name: name.to_string(),
                                },
                            }
                            .into());
                        }
                    },
                    _ => return Ok(()),
                };
                for field in &fields.0 {
                    require(
                        !matches!(field.r#type, QapiTypeRef::InlineStruct(_)),
                        || SchemaErrorKind::NestedEventData,
                    )
                    .within(|| format!("member '{}'", field.name))?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Between their two passes over the definitions, QEMU 2.4 to 4.1 look
    /// each flat union's discriminator up among its base's members as
    /// keys, which fails for a discriminator that is an array or an object
    /// and, from 2.9, a base that is an array; from 2.9 they look the
    /// discriminator's type up as a key, which fails for an array and,
    /// until 3.1, an object written out in place.
    fn discriminator_key(&self, index: usize) -> SchemaResult {
        let QapiSchemaToken::Union(value) = self.definitions[index].token else {
            return Ok(());
        };
        let raw_discriminator = bad(&value.malformed, "discriminator");
        let raw_base = bad(&value.malformed, "base");
        let discriminator = value.discriminator.as_deref().unwrap_or_default();
        let fails = || {
            Err(SchemaErrorKind::Crash {
                crash: crate::unified::Crash::UnhashableDiscriminator {
                    discriminator: raw_discriminator
                        .map_or(discriminator, |raw| raw.text)
                        .to_owned(),
                },
            }
            .into())
        };
        let unhashable = |raw: &RawValue<'_>| matches!(raw.value, Raw::Array(_) | Raw::Object(_));
        if !(raw_discriminator.is_some_and(RawValue::truthy) || !discriminator.is_empty()) {
            return Ok(());
        }
        let base = match (&value.base, raw_base) {
            (_, Some(raw)) if raw.truthy() && unhashable(raw) && since((2, 9, 0)) => {
                return fails();
            }
            (MembersOrRef::Members(fields), None) if since((2, 6, 0)) => fields,
            (MembersOrRef::Ref(base), None) => match self.token(base) {
                Some(QapiSchemaToken::Struct(base)) => &base.data,
                _ => return Ok(()),
            },
            _ => return Ok(()),
        };
        if base.0.is_empty() {
            return Ok(());
        }
        match raw_discriminator {
            Some(raw) if unhashable(raw) => return fails(),
            Some(_) => return Ok(()),
            None => {}
        }
        if !since((2, 9, 0)) {
            return Ok(());
        }
        // The member as written, so not an optional one.
        let member = base
            .0
            .iter()
            .find(|member| member.name == discriminator && !member.optional);
        let unhashable_type = member.is_some_and(|member| match &member.r#type {
            QapiTypeRef::ArrayRef(_) => true,
            QapiTypeRef::InlineStruct(_) => !since((4, 0, 0)),
            _ => bad(&member.malformed, "type").is_some_and(unhashable),
        });
        if unhashable_type { fails() } else { Ok(()) }
    }

    /// Find a flat union's discriminator among the required members that
    /// its base declares itself, as QEMU does before 4.2, and its enum.
    fn discriminator_enum(
        &self,
        base: &QapiMembers<'_>,
        discriminator: &str,
    ) -> SchemaResult<(String, Vec<&'a str>)> {
        let member = base
            .0
            .iter()
            .find(|member| member.name == discriminator && !member.optional)
            .ok_or_else(|| SchemaErrorKind::DiscriminatorNotMember {
                discriminator: discriminator.to_owned(),
            })?;
        require(member.r#if.is_none(), || {
            SchemaErrorKind::ConditionalDiscriminator {
                discriminator: discriminator.to_owned(),
            }
        })?;
        let values = match &member.r#type {
            QapiTypeRef::Ref(name) if self.meta(name) == Some(Meta::Enum) => self.enum_values(name),
            _ => None,
        };
        match values {
            Some(values) => Ok((
                spelled(&member.r#type).trim_matches('\'').to_owned(),
                values,
            )),
            None => Err(SchemaErrorKind::DiscriminatorNotEnum {
                discriminator: discriminator.to_owned(),
                discriminator_type: spelled(&member.r#type).trim_matches('\'').to_owned(),
            }
            .into()),
        }
    }

    /// Require a flat union's branch to be a value of its enum.
    pub(crate) fn branch_value(
        &self,
        branch: &str,
        enumeration: &str,
        values: &[&str],
    ) -> SchemaResult {
        require(values.contains(&branch), || {
            SchemaErrorKind::UnknownBranch {
                branch: branch.to_owned(),
                discriminator_type: enumeration.to_owned(),
            }
        })
    }

    fn predefined(&self, token: &QapiSchemaToken<'_>) -> SchemaResult {
        let name = token.definition_name().unwrap_or_default();
        require(!Builtins::current().reserves(name), || {
            SchemaErrorKind::Predefined {
                name: name.to_owned(),
            }
        })
    }

    /// Record a definition's name, refusing a second definition of it.
    fn duplicate(&self, index: usize, defined: &mut HashMap<&'a str, &'a Path>) -> SchemaResult {
        let item = &self.definitions[index];
        let name = item.token.definition_name().unwrap_or_default();
        match defined.insert(name, item.path) {
            Some(previous) => Err(SchemaErrorKind::Duplicate {
                name: name.to_owned(),
                previous: previous.to_path_buf(),
            }
            .into()),
            None => Ok(()),
        }
    }

    /// The first pass of QEMU 2.4 to 4.1 over a definition: its keys, flag
    /// values and name, that the name is new, and its suffix.
    fn learn(&self, index: usize, defined: &mut HashMap<&'a str, &'a Path>) -> SchemaResult {
        let token = self.definitions[index].token;
        schema_docs::check_required(token, &self.docs)?;
        self.metatypes(token)?;
        self.keys(token)?;
        crate::schema_names::validate_definition(token)?;
        self.predefined(token)?;
        self.duplicate(index, defined)?;
        crate::schema_names::reserved_suffix(token)?;
        schema_docs::check_symbol(token)
    }

    /// The definition pass of QEMU 4.2 and later, which builds the schema:
    /// duplicates.
    fn define(&self, index: usize, defined: &mut HashMap<&'a str, &'a Path>) -> SchemaResult {
        self.build(index)?;
        self.predefined(self.definitions[index].token)?;
        self.duplicate(index, defined)
    }

    /// Refuse an empty condition, from QEMU 3.0 to 6.1; from 6.2 the parser
    /// reads only conditions with configuration names. From 4.2 a blank
    /// name is empty too.
    fn condition(&self, cond: Option<&QapiCond<'_>>, raw: Option<&RawValue<'_>>) -> SchemaResult {
        if let Some(raw) = raw.filter(|_| since((3, 0, 0))) {
            return crate::schema_shapes::condition(raw);
        }
        if !since((3, 0, 0)) || since((6, 2, 0)) {
            return Ok(());
        }
        let empty = |name: &str| {
            let blank = if since((4, 2, 0)) {
                name.trim().is_empty()
            } else {
                name.is_empty()
            };
            require(!blank, || SchemaErrorKind::EmptyCondition {
                condition: format!("'{name}'"),
            })
        };
        match cond {
            Some(QapiCond::ConfigName(name)) => empty(name),
            Some(QapiCond::All(names)) => {
                require(!names.is_empty(), || SchemaErrorKind::EmptyCondition {
                    condition: "[]".into(),
                })?;
                for name in names {
                    if let QapiCond::ConfigName(name) = name {
                        empty(name)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// The keys of a definition's expression that the release knows, as
    /// QEMU's lists of required and optional keys, without the key that
    /// names the definition's kind.
    fn known_keys(token: &QapiSchemaToken<'_>) -> Option<(Vec<&'static str>, Vec<&'static str>)> {
        let era = |keys: &[(&'static str, Version)]| -> Vec<&'static str> {
            keys.iter()
                .filter(|(_, from)| since(*from))
                .map(|(key, _)| *key)
                .collect()
        };
        const ALWAYS: Version = (2, 4, 0);
        let (required, optional) = match token {
            QapiSchemaToken::Enum(_) => (
                vec!["data"],
                era(&[
                    ("prefix", (2, 5, 0)),
                    ("if", (3, 0, 0)),
                    ("features", (5, 0, 0)),
                ]),
            ),
            QapiSchemaToken::Struct(_) => (
                vec!["data"],
                era(&[("base", ALWAYS), ("if", (3, 0, 0)), ("features", (4, 1, 0))]),
            ),
            // From 6.2 every union is flat.
            QapiSchemaToken::Union(_) if since((6, 2, 0)) => (
                vec!["base", "discriminator", "data"],
                vec!["if", "features"],
            ),
            QapiSchemaToken::Union(_) => (
                vec!["data"],
                era(&[
                    ("base", ALWAYS),
                    ("discriminator", ALWAYS),
                    ("if", (3, 0, 0)),
                    ("features", (5, 0, 0)),
                ]),
            ),
            QapiSchemaToken::Alternate(_) => (
                vec!["data"],
                era(&[("if", (3, 0, 0)), ("features", (5, 0, 0))]),
            ),
            QapiSchemaToken::Command(_) => (
                vec![],
                era(&[
                    ("data", ALWAYS),
                    ("returns", ALWAYS),
                    ("gen", ALWAYS),
                    ("success-response", ALWAYS),
                    ("boxed", (2, 7, 0)),
                    ("allow-oob", (2, 12, 0)),
                    ("allow-preconfig", (3, 0, 0)),
                    ("if", (3, 0, 0)),
                    ("features", (4, 2, 0)),
                    ("coroutine", (5, 2, 0)),
                ]),
            ),
            QapiSchemaToken::Event(_) => (
                vec![],
                era(&[
                    ("data", ALWAYS),
                    ("boxed", (2, 7, 0)),
                    ("if", (3, 0, 0)),
                    ("features", (5, 0, 0)),
                ]),
            ),
            _ => return None,
        };
        Some((required, optional))
    }

    /// The kinds of definition that an expression names, from QEMU 2.4:
    /// until 6.1 it must name one, and the kind that QEMU looks for first
    /// takes another kind's key as unknown; from 6.2 exactly one.
    fn metatypes(&self, token: &QapiSchemaToken<'_>) -> SchemaResult {
        const METAS: [&str; 6] = ["enum", "union", "alternate", "struct", "command", "event"];
        if !since((6, 2, 0)) {
            return require(!matches!(token, QapiSchemaToken::NoMetatype(_)), || {
                SchemaErrorKind::MissingMetatype
            });
        }
        let keys = keys_of(token);
        let metas = keys.iter().filter(|key| METAS.contains(&key.as_ref()));
        require(metas.count() == 1, || SchemaErrorKind::Metatypes)
    }

    /// Check the keys of a definition's expression as QEMU does: until 3.1
    /// key by key in the order of the source, an unknown key, a flag's
    /// value and (from 3.0) the condition, then the keys that are missing;
    /// from 4.0 the missing keys, then the unknown ones, and until 4.1 the
    /// flags and the condition key by key. From 4.2 the expression checks
    /// test the flags and the condition after the rest.
    fn keys(&self, token: &QapiSchemaToken<'_>) -> SchemaResult {
        let Some((required, optional)) = Self::known_keys(token) else {
            return Ok(());
        };
        let meta = match token {
            QapiSchemaToken::Enum(_) => "enum",
            QapiSchemaToken::Struct(_) => "struct",
            QapiSchemaToken::Union(_) => "union",
            QapiSchemaToken::Alternate(_) => "alternate",
            QapiSchemaToken::Command(_) => "command",
            _ => "event",
        };
        let keys: Vec<&str> = keys_of(token).iter().map(|key| key.as_ref()).collect();
        let known = |key: &&str| *key == meta || required.contains(key) || optional.contains(key);
        // Until 4.1 the keys' check starts with the name.
        if let Some(raw) = bad(malformed_of(token), meta).filter(|_| !since((4, 2, 0))) {
            crate::schema_shapes::name(raw)?;
        }
        if !since((4, 0, 0)) {
            for key in &keys {
                require(known(key), || SchemaErrorKind::UnknownKey {
                    key: (*key).to_owned(),
                })?;
                self.key_value(token, key)?;
            }
            return missing(&required, &keys);
        }
        dictionary_keys(&keys, &required, &[&[meta], &optional[..]].concat())?;
        if !since((4, 2, 0)) {
            for key in &keys {
                self.key_value(token, key)?;
            }
        }
        Ok(())
    }

    /// The value of a definition's key that QEMU checks with the keys
    /// until 4.1: a flag, and the condition.
    fn key_value(&self, token: &QapiSchemaToken<'_>, key: &str) -> SchemaResult {
        match key {
            "if" => self.condition(condition(token), bad(malformed_of(token), "if")),
            _ => self.flag(token, key),
        }
    }

    /// The keys of a member written as a dictionary, from QEMU 4.0, with
    /// its features from 5.0.
    pub(crate) fn member_keys(&self, field: &QapiMember<'_>) -> SchemaResult {
        let optional: &[&str] = if since((5, 0, 0)) {
            &["if", "features"]
        } else {
            &["if"]
        };
        keyed(field.keys.as_deref(), &["type"], optional)
    }

    /// The keys of a union's or alternate's branch written as a
    /// dictionary, from QEMU 4.0.
    pub(crate) fn branch_keys(&self, branch: &crate::QapiVariant<'_>) -> SchemaResult {
        keyed(branch.keys.as_deref(), &["type"], &["if"])
    }

    /// The keys of an enum value written as a dictionary, from QEMU 4.0
    /// (before, its name is refused instead), with its features from 6.2.
    pub(crate) fn enum_value_keys(&self, value: &crate::QapiEnumValue<'_>) -> SchemaResult {
        if !since((4, 0, 0)) {
            return Ok(());
        }
        let optional: &[&str] = if since((6, 2, 0)) {
            &["if", "features"]
        } else {
            &["if"]
        };
        keyed(
            value.dictionary.then_some(&value.keys[..]),
            &["name"],
            optional,
        )
    }

    /// The keys of a feature written as a dictionary, from QEMU 4.1.
    pub(crate) fn feature_keys(&self, feature: &crate::QapiFeature<'_>) -> SchemaResult {
        keyed(feature.keys.as_deref(), &["name"], &["if"])
    }

    /// The values of command and event flags, and the flags that exclude
    /// each other.
    fn flags(&self, token: &QapiSchemaToken<'_>) -> SchemaResult {
        for key in keys_of(token) {
            self.flag(token, key)?;
        }
        require(
            !matches!(token, QapiSchemaToken::Command(value)
                if value.allow_oob.is_some() && value.coroutine.is_some()),
            || SchemaErrorKind::IncompatibleFlags,
        )
    }

    /// The value of the flag `key` of a command or event: `gen` and
    /// `success-response` only false, the others only true.
    fn flag(&self, token: &QapiSchemaToken<'_>, key: &str) -> SchemaResult {
        let malformed = bad(malformed_of(token), key).is_some();
        let setting = match token {
            QapiSchemaToken::Command(value) => match key {
                "gen" => value.r#gen.as_deref(),
                "success-response" => value.success_response.as_deref(),
                "boxed" => value.boxed,
                "allow-oob" => value.allow_oob,
                "allow-preconfig" => value.allow_preconfig,
                "coroutine" => value.coroutine,
                _ => return Ok(()),
            },
            QapiSchemaToken::Event(value) if key == "boxed" => value.boxed,
            _ => return Ok(()),
        };
        let (flag, value) = match key {
            "gen" => ("gen", false),
            "success-response" => ("success-response", false),
            "boxed" => ("boxed", true),
            "allow-oob" => ("allow-oob", true),
            "allow-preconfig" => ("allow-preconfig", true),
            _ => ("coroutine", true),
        };
        let expected = if value { "true" } else { "false" };
        require(
            !malformed && setting.is_none_or(|setting| setting == expected),
            || SchemaErrorKind::FlagValue { flag, value },
        )
    }

    /// Refuse a member named `u` or `has_*` in C, from QEMU 2.5: until 5.2
    /// the key as written, so that an optional `*u` passes, from 6.0 the
    /// name without its `*`.
    fn reserved(&self, field: &QapiMember<'_>) -> SchemaResult {
        if !since((2, 5, 0)) {
            return Ok(());
        }
        let written = if field.optional {
            format!("*{}", field.name)
        } else {
            field.name.to_string()
        };
        let name = crate::schema_names::c_name(if since((6, 0, 0)) {
            &field.name
        } else {
            &written
        });
        require(name != "u" && !name.starts_with("has_"), || {
            SchemaErrorKind::ReservedName { name: written }
        })
    }

    /// The case of a member name in the expression checks from QEMU 4.2: no
    /// uppercase until 5.2, also no `_` from 6.0, unless the pragma exempts
    /// `owner`.
    fn member_case(&self, name: &str, owner: Option<&str>) -> SchemaResult {
        if owner.is_some_and(|owner| self.case_exempt(owner)) {
            return Ok(());
        }
        lower_case(
            name,
            if since((6, 0, 0)) {
                NameRule::Lowercase
            } else {
                NameRule::NoUppercase
            },
        )
    }

    /// The type of a member, branch, base, argument or return value as
    /// QEMU 2.4 to 4.1 checks it with its expression: defined, of a kind
    /// that `metas` allows and an array only where arrays are allowed.
    fn reference_classic(
        &self,
        value: &QapiTypeRef<'_>,
        allow_array: bool,
        metas: &[Meta],
        allow_untyped: bool,
    ) -> SchemaResult {
        let name = match value {
            QapiTypeRef::Ref(name) => name.as_ref(),
            QapiTypeRef::ArrayRef(name) => {
                require(allow_array, || SchemaErrorKind::ArrayNotAllowed)?;
                name.as_ref()
            }
            QapiTypeRef::InlineStruct(_) | QapiTypeRef::Unset => {
                return Err(SchemaErrorKind::AnonymousStruct.into());
            }
        };
        // 2.4 takes `**` only as a type name, never in an array.
        if name == "**" && Builtins::current().defines(name) {
            return require(
                allow_untyped && matches!(value, QapiTypeRef::Ref(_)),
                || SchemaErrorKind::UntypedWithoutGen,
            );
        }
        let meta = self
            .meta(name)
            .ok_or_else(|| SchemaErrorKind::UnknownType {
                name: name.to_owned(),
            })?;
        require(metas.contains(&meta), || SchemaErrorKind::TypeNotAllowed {
            meta: meta.word(),
            reference: format!("'{name}'"),
        })
    }

    /// The error for a reference to `name` where its kind is not allowed.
    pub(crate) fn not_allowed(&self, name: &str) -> SchemaError {
        match self.meta(name) {
            Some(meta) => SchemaErrorKind::TypeNotAllowed {
                meta: meta.word(),
                reference: format!("'{name}'"),
            },
            None => SchemaErrorKind::UnknownType {
                name: name.to_owned(),
            },
        }
        .into()
    }

    /// The members of an object written out in place, as QEMU 2.4 to 4.1
    /// checks them: names, reserved names and types.
    fn members_classic(&self, fields: &QapiMembers<'_>, allow_untyped: bool) -> SchemaResult {
        for field in &fields.0 {
            let check = || -> SchemaResult {
                crate::schema_names::validate(&field.name)?;
                self.reserved(field)?;
                self.member_keys(field)?;
                if let Some(raw) = bad(&field.malformed, "type") {
                    return crate::schema_shapes::type_value(raw, true, false);
                }
                self.reference_classic(
                    &field.r#type,
                    true,
                    &[
                        Meta::Builtin,
                        Meta::Union,
                        Meta::Alternate,
                        Meta::Struct,
                        Meta::Enum,
                    ],
                    allow_untyped,
                )
            };
            check().within(|| format!("member '{}'", field.name))?;
        }
        Ok(())
    }

    /// QEMU 2.4 and 2.5 compare enum values and branch names by their C
    /// enumeration constants, among them the ones it takes itself.
    fn constants(&self, names: &[&str], automatic: &[&str]) -> SchemaResult {
        if since((2, 6, 0)) {
            return Ok(());
        }
        let mut seen: HashMap<String, String> = automatic
            .iter()
            .map(|name| ((*name).to_owned(), "(automatic)".to_owned()))
            .collect();
        for name in names {
            if let Some(previous) = seen.insert(camel_to_upper(name), (*name).to_owned()) {
                return Err(SchemaErrorKind::MemberClash {
                    name: (*name).to_owned(),
                    previous,
                }
                .into());
            }
        }
        Ok(())
    }

    /// QEMU 2.4 and 2.5 compare the members of a struct, or of a flat
    /// union's branch, with those of the base and its bases as written. A
    /// cycle of bases never ends there.
    fn base_clash(&self, base: &str, fields: &QapiMembers<'_>) -> SchemaResult {
        if since((2, 6, 0)) {
            return Ok(());
        }
        let mut seen = HashSet::new();
        let mut next = Some(base);
        while let Some(name) = next {
            require(seen.insert(name), || SchemaErrorKind::Crash {
                crash: crate::unified::Crash::BaseCycle {
                    name: name.to_owned(),
                },
            })?;
            let Some(QapiSchemaToken::Struct(value)) = self.token(name) else {
                return Ok(());
            };
            for field in &fields.0 {
                require(
                    !value.data.0.iter().any(|member| member.name == field.name),
                    || SchemaErrorKind::BaseClash {
                        name: field.name.to_string(),
                        base: name.to_owned(),
                    },
                )?;
            }
            next = value.base.as_deref();
        }
        Ok(())
    }

    /// The expression checks of QEMU 2.4 to 4.1.
    fn expression_classic(&self, index: usize) -> SchemaResult {
        let token = self.definitions[index].token;
        let name = token.definition_name().unwrap_or_default();
        let malformed = malformed_of(token);
        match token {
            QapiSchemaToken::Enum(value) => {
                if let Some(raw) = bad(malformed, "data") {
                    crate::schema_shapes::array(raw)?;
                }
                if let Some(raw) = bad(malformed, "prefix") {
                    crate::schema_shapes::string(raw)?;
                }
                let names: Vec<_> = value.data.iter().map(|value| value.name.as_ref()).collect();
                for (position, member) in value.data.iter().enumerate() {
                    // From 4.0 QEMU checks an enum value's keys and condition
                    // first.
                    if since((4, 0, 0)) {
                        self.enum_value_keys(member)
                            .and_then(|()| {
                                self.condition(member.r#if.as_ref(), bad(&member.malformed, "if"))
                            })
                            .within(|| format!("member '{}'", member.name))?;
                    }
                    let unnamed = bad(&member.malformed, "name").is_some()
                        || (member.dictionary && !since((4, 0, 0)));
                    if unnamed {
                        return Err(SchemaError::from(SchemaErrorKind::NameNotString))
                            .within(|| format!("member '{}'", member.name));
                    }
                    empty_enum_value(&member.name)?;
                    crate::schema_names::validate_enum_member(&member.name)
                        .within(|| format!("member '{}'", member.name))?;
                    // 2.4 and 2.5 check each value's constant with it.
                    self.constants(&names[..=position], &["MAX"])?;
                }
                Ok(())
            }
            QapiSchemaToken::Struct(value) => {
                match bad(malformed, "data").map(|raw| &raw.value) {
                    // A type name, of no kind that members take.
                    Some(Raw::String(members)) => self
                        .reference_classic(&QapiTypeRef::Ref(members.clone()), false, &[], false)
                        .within(|| "'data'".into())?,
                    Some(_) => {
                        let raw = bad(malformed, "data").unwrap_or_else(|| unreachable!());
                        crate::schema_shapes::type_value(raw, false, true)
                            .within(|| "'data'".into())?;
                    }
                    None => self
                        .members_classic(&value.data, false)
                        .within(|| "'data'".into())?,
                }
                if let Some(raw) = bad(malformed, "base") {
                    crate::schema_shapes::type_value(raw, false, false)
                        .within(|| "'base'".into())?;
                }
                if let Some(base) = value.base.as_deref() {
                    self.reference_classic(
                        &QapiTypeRef::Ref(base.into()),
                        false,
                        &[Meta::Struct],
                        false,
                    )
                    .within(|| "'base'".into())?;
                    self.base_clash(base, &value.data)?;
                }
                // 4.1 checks only true features.
                if let Some(raw) = bad(malformed, "features").filter(|raw| raw.truthy()) {
                    crate::schema_shapes::array(raw)?;
                }
                for feature in value.features.iter().flat_map(|features| features.iter()) {
                    let check = || -> SchemaResult {
                        self.feature_keys(feature)?;
                        self.condition(feature.r#if.as_ref(), bad(&feature.malformed, "if"))?;
                        if let Some(raw) = bad(&feature.malformed, "name") {
                            crate::schema_shapes::name(raw)?;
                        }
                        crate::schema_names::validate_mandatory(&feature.name)
                    };
                    check().within(|| format!("feature '{}'", feature.name))?;
                }
                Ok(())
            }
            QapiSchemaToken::Union(value) => self.union_classic(value),
            QapiSchemaToken::Alternate(value) => {
                match bad(malformed, "data") {
                    Some(raw) => crate::schema_shapes::branches(raw, Some(2))?,
                    None => require(value.data.0.len() >= 2 || !since((2, 6, 0)), || {
                        SchemaErrorKind::AlternateBranches { minimum: 2 }
                    })?,
                }
                // Branch by branch: the name, its constant, the type and
                // its JSON types.
                let names: Vec<_> = value.data.0.iter().map(|b| b.name.as_ref()).collect();
                let mut kinds = HashMap::new();
                for (position, branch) in value.data.0.iter().enumerate() {
                    crate::schema_names::validate_mandatory(&branch.name)
                        .and_then(|()| self.branch_keys(branch))
                        .within(|| format!("branch '{}'", branch.name))?;
                    self.constants(&names[..=position], &["MAX"])?;
                    if let Some(raw) = bad(&branch.malformed, "type") {
                        // `null` has no JSON type: 2.4 and 2.5 assert one.
                        let check = match raw.value {
                            Raw::Null if !since((2, 6, 0)) => {
                                Err(crate::schema_shapes::unchecked("type"))
                            }
                            Raw::Null => Err(SchemaErrorKind::TypeNotAllowed {
                                meta: "null",
                                reference: "null".into(),
                            }
                            .into()),
                            _ => crate::schema_shapes::type_value(raw, false, false),
                        };
                        return check.within(|| format!("branch '{}'", branch.name));
                    }
                    self.reference_classic(
                        &branch.r#type,
                        false,
                        &[Meta::Builtin, Meta::Union, Meta::Struct, Meta::Enum],
                        false,
                    )
                    .within(|| format!("branch '{}'", branch.name))?;
                    self.alternate_kind(branch, &mut kinds)?;
                }
                Ok(())
            }
            QapiSchemaToken::Command(value) => {
                let untyped = value.r#gen.is_some();
                self.arguments_classic(
                    value.data.as_ref(),
                    bad(malformed, "data"),
                    value.boxed,
                    untyped,
                )?;
                if let Some(raw) = bad(malformed, "returns") {
                    return crate::schema_shapes::type_value(raw, true, false)
                        .within(|| "'returns'".into());
                }
                // 2.4 also takes members written out in place.
                if let Some(QapiTypeRef::InlineStruct(fields)) = &value.returns
                    && !since((2, 5, 0))
                {
                    self.members_classic(fields, untyped)
                        .within(|| "'returns'".into())?;
                } else if let Some(returns) = &value.returns {
                    let mut metas = vec![Meta::Union, Meta::Struct];
                    if self.returns_exempt(name) {
                        metas.extend([Meta::Builtin, Meta::Alternate, Meta::Enum]);
                    }
                    self.reference_classic(returns, true, &metas, untyped)
                        .within(|| "'returns'".into())?;
                }
                Ok(())
            }
            QapiSchemaToken::Event(value) => {
                require(since((2, 6, 0)) || name.to_uppercase() != "MAX", || {
                    SchemaErrorKind::ReservedName {
                        name: name.to_owned(),
                    }
                })?;
                self.arguments_classic(
                    value.data.as_ref(),
                    bad(malformed, "data"),
                    value.boxed,
                    false,
                )
            }
            _ => Ok(()),
        }
    }

    /// The arguments of a command or event, from QEMU 2.4 to 4.1: a struct,
    /// or a union too in 2.4; boxed from 2.7, which takes a union or an
    /// alternate too, but only by name.
    fn arguments_classic(
        &self,
        data: Option<&MembersOrRef<'_>>,
        raw: Option<&RawValue<'_>>,
        boxed: Option<&str>,
        untyped: bool,
    ) -> SchemaResult {
        // Boxed arguments without a type are refused with the schema's
        // entities.
        let boxed = boxed == Some("true");
        if let Some(raw) = raw {
            return crate::schema_shapes::type_value(raw, false, !boxed).within(|| "'data'".into());
        }
        match data {
            None | Some(MembersOrRef::Unset) => Ok(()),
            Some(MembersOrRef::Members(fields)) => {
                require(!boxed, || SchemaErrorKind::AnonymousStruct).within(|| "'data'".into())?;
                self.members_classic(fields, untyped)
                    .within(|| "'data'".into())
            }
            Some(MembersOrRef::Ref(name)) => {
                let metas: &[Meta] = if !since((2, 5, 0)) {
                    &[Meta::Union, Meta::Struct]
                } else if boxed {
                    &[Meta::Struct, Meta::Union, Meta::Alternate]
                } else {
                    &[Meta::Struct]
                };
                self.reference_classic(&QapiTypeRef::Ref(name.clone()), false, metas, false)
                    .within(|| "'data'".into())
            }
        }
    }

    /// A union, from QEMU 2.4 to 4.1. Any value but `null` of a base or a
    /// discriminator is one, which the checks of its shape refuse.
    fn union_classic(&self, value: &QapiUnion<'_>) -> SchemaResult {
        let malformed = &value.malformed;
        let raw_base = bad(malformed, "base");
        let raw_discriminator = bad(malformed, "discriminator");
        let raw_data = bad(malformed, "data");
        // A legacy alternate's `{}` is a discriminator too.
        let flat =
            value.discriminator.is_some() || raw_discriminator.is_some() || value.legacy_alternate;
        if !flat {
            require(
                value.base == MembersOrRef::Unset && raw_base.is_none(),
                || SchemaErrorKind::BaseWithoutDiscriminator,
            )?;
            if let Some(raw) = raw_data {
                return crate::schema_shapes::branches(raw, None);
            }
            require(!value.data.0.is_empty() || !since((2, 6, 0)), || {
                SchemaErrorKind::NoBranches
            })?;
            let names: Vec<_> = value.data.0.iter().map(|b| b.name.as_ref()).collect();
            let automatic: &[&str] = if since((2, 5, 0)) {
                &["MAX", "KIND", "TYPE"]
            } else {
                &["MAX"]
            };
            for (position, branch) in value.data.0.iter().enumerate() {
                let check = || -> SchemaResult {
                    crate::schema_names::validate_mandatory(&branch.name)?;
                    self.branch_keys(branch)?;
                    if let Some(raw) = bad(&branch.malformed, "type") {
                        return crate::schema_shapes::type_value(raw, true, false);
                    }
                    self.reference_classic(
                        &branch.r#type,
                        true,
                        &[
                            Meta::Builtin,
                            Meta::Union,
                            Meta::Alternate,
                            Meta::Struct,
                            Meta::Enum,
                        ],
                        false,
                    )
                };
                check().within(|| format!("branch '{}'", branch.name))?;
                self.constants(&names[..=position], automatic)?;
            }
            return Ok(());
        }
        // 2.4 requires a flat union's base to be a type name.
        require(
            since((2, 5, 0)) || (matches!(value.base, MembersOrRef::Ref(_)) && raw_base.is_none()),
            || SchemaErrorKind::StringBase,
        )?;
        let base = match (&value.base, raw_base) {
            (_, Some(raw)) => {
                return crate::schema_shapes::type_value(raw, false, since((2, 6, 0)))
                    .within(|| "'base'".into());
            }
            (MembersOrRef::Unset, None) => return Err(SchemaErrorKind::MissingBase.into()),
            (MembersOrRef::Members(fields), None) => {
                if !since((2, 6, 0)) {
                    return Err(SchemaErrorKind::AnonymousStruct).within(|| "'base'".into());
                }
                self.members_classic(fields, false)
                    .within(|| "'base'".into())?;
                // An empty object is no base.
                require(!fields.0.is_empty(), || SchemaErrorKind::MissingBase)?;
                fields
            }
            (MembersOrRef::Ref(base), None) => {
                self.reference_classic(
                    &QapiTypeRef::Ref(base.clone()),
                    false,
                    &[Meta::Struct],
                    false,
                )
                .within(|| "'base'".into())?;
                let fields = match self.token(base) {
                    Some(QapiSchemaToken::Struct(value)) => &value.data,
                    _ => return Err(self.not_allowed(base)).within(|| "'base'".into()),
                };
                // Until 2.8 QEMU requires the base to have members.
                require(!fields.0.is_empty() || since((2, 9, 0)), || {
                    SchemaErrorKind::EmptyBase {
                        name: base.to_string(),
                    }
                })?;
                fields
            }
        };
        if let Some(raw) = raw_discriminator {
            return crate::schema_shapes::name(raw).within(|| "'discriminator'".into());
        }
        // A legacy alternate's `{}` is no name.
        require(!value.legacy_alternate, || SchemaErrorKind::NameNotString)
            .within(|| "'discriminator'".into())?;
        let discriminator = value.discriminator.as_deref().unwrap_or_default();
        crate::schema_names::validate_mandatory(discriminator)
            .within(|| "'discriminator'".into())?;
        let (enumeration, values) = self.discriminator_enum(base, discriminator)?;
        if let Some(raw) = raw_data {
            return crate::schema_shapes::branches(raw, None);
        }
        require(!value.data.0.is_empty() || !since((2, 6, 0)), || {
            SchemaErrorKind::NoBranches
        })?;
        for branch in &value.data.0 {
            let check = || -> SchemaResult {
                crate::schema_names::validate_mandatory(&branch.name)?;
                self.branch_keys(branch)?;
                if let Some(raw) = bad(&branch.malformed, "type") {
                    return crate::schema_shapes::type_value(raw, false, false);
                }
                self.reference_classic(&branch.r#type, false, &[Meta::Struct], false)?;
                if let (QapiTypeRef::Ref(payload), MembersOrRef::Ref(base)) =
                    (&branch.r#type, &value.base)
                    && let Some(QapiSchemaToken::Struct(payload)) = self.token(payload)
                {
                    self.base_clash(base, &payload.data)?;
                }
                Ok(())
            };
            check().within(|| format!("branch '{}'", branch.name))?;
            self.branch_value(&branch.name, &enumeration, &values)?;
            require(
                !since((2, 5, 0)) || since((2, 6, 0)) || !values.contains(&discriminator),
                || SchemaErrorKind::DiscriminatorClash {
                    discriminator: discriminator.to_owned(),
                    enumeration: enumeration.clone(),
                },
            )?;
        }
        if since((2, 7, 0)) && !since((3, 0, 0)) {
            for name in &values {
                require(
                    value.data.0.iter().any(|branch| branch.name == *name),
                    || SchemaErrorKind::MissingBranch {
                        branch: (*name).to_owned(),
                    },
                )?;
            }
        }
        Ok(())
    }

    /// The JSON types of an alternate's branch, which must differ from
    /// those of the branches before it, recorded in `seen`: from QEMU 2.10
    /// a string branch also takes numbers and booleans, unless it is an
    /// enum without values that look like them.
    pub(crate) fn alternate_kind<'b>(
        &self,
        branch: &'b QapiVariant<'_>,
        seen: &mut HashMap<Json, &'b str>,
    ) -> SchemaResult {
        let numbers = since((2, 10, 0));
        let (json, values) = match &branch.r#type {
            QapiTypeRef::ArrayRef(_) => (Some(Json::Array), None),
            QapiTypeRef::Ref(name) => {
                let builtin = match name.as_ref() {
                    "str" => Some(Json::String),
                    "bool" => Some(Json::Bool),
                    "null" => Some(Json::Null),
                    "number" if numbers => Some(Json::Number),
                    "number" => Some(Json::Float),
                    "int" | "int8" | "int16" | "int32" | "int64" | "uint8" | "uint16"
                    | "uint32" | "uint64" | "size" => {
                        Some(if numbers { Json::Number } else { Json::Integer })
                    }
                    // Until 4.2, QType is a builtin string type.
                    "QType" if !since((4, 2, 0)) => Some(Json::String),
                    _ => None,
                };
                match (builtin, self.meta(name)) {
                    (Some(json), Some(Meta::Builtin | Meta::Enum)) => (Some(json), None),
                    (None, Some(Meta::Enum)) => (Some(Json::String), self.enum_values(name)),
                    (None, Some(Meta::Struct | Meta::Union)) => (Some(Json::Object), None),
                    _ => (None, None),
                }
            }
            _ => (None, None),
        };
        // 2.5 asserts that every branch has a JSON type.
        if json.is_none()
            && !since((2, 6, 0))
            && matches!(&branch.r#type, QapiTypeRef::Ref(name) if name == "any")
        {
            return Err(SchemaErrorKind::Crash {
                crash: crate::unified::Crash::AlternateAny {
                    branch: branch.name.to_string(),
                },
            }
            .into());
        }
        let json = match (json, &branch.r#type) {
            (Some(json), _) => Ok(json),
            (None, QapiTypeRef::Ref(name)) => Err(self.not_allowed(name)),
            (None, other) => Err(SchemaErrorKind::TypeNotAllowed {
                meta: "array",
                reference: spelled(other),
            }
            .into()),
        }
        .within(|| format!("branch '{}'", branch.name))?;
        let mut conflicting = vec![json];
        // From 2.10 QEMU looks for the values of an enum branch.
        if let (true, QapiTypeRef::Ref(name)) = (numbers, &branch.r#type) {
            self.alternate_tag(name)?;
        }
        if numbers && json == Json::String {
            match values {
                Some(values) => {
                    for value in values {
                        if matches!(value, "on" | "off") {
                            conflicting.push(Json::Bool);
                        }
                        if value.starts_with(|c: char| {
                            c.is_ascii_digit() || matches!(c, '-' | '+' | '.')
                        }) {
                            conflicting.push(Json::Number);
                        }
                    }
                }
                None => conflicting.extend([Json::Bool, Json::Number]),
            }
        }
        // 2.10 names the previous branch by the branch's own JSON type,
        // which only another string branch has.
        if since((2, 10, 0))
            && !since((2, 11, 0))
            && !seen.contains_key(&json)
            && conflicting.iter().any(|json| seen.contains_key(json))
        {
            return Err(SchemaErrorKind::Crash {
                crash: crate::unified::Crash::AlternateConflict {
                    branch: branch.name.to_string(),
                },
            }
            .into());
        }
        for json in &conflicting {
            if let Some(previous) = seen.get(json) {
                return Err(SchemaErrorKind::AmbiguousAlternate {
                    branch: branch.name.to_string(),
                    previous: (*previous).to_owned(),
                }
                .into());
            }
        }
        for json in conflicting {
            seen.insert(json, branch.name.as_ref());
        }
        Ok(())
    }

    /// The expression checks of QEMU 4.2 and later.
    fn expression(&self, index: usize) -> SchemaResult {
        let item = &self.definitions[index];
        let token = item.token;
        let name = token.definition_name().unwrap_or_default();
        let malformed = malformed_of(token);
        self.metatypes(token)?;
        if let Some(raw) = meta_key(token).and_then(|meta| bad(malformed, meta)) {
            crate::schema_shapes::name(raw)?;
        }
        // The name, from 6.0 its case, then its suffix.
        crate::schema_names::validate_definition(token)?;
        self.definition_case(token, name)?;
        crate::schema_names::reserved_suffix(token)?;
        schema_docs::check_expression(token, item.content, &self.docs)?;
        self.keys(token)?;
        match token {
            QapiSchemaToken::Enum(value) => {
                if let Some(raw) = bad(malformed, "data") {
                    crate::schema_shapes::array(raw)?;
                }
                if let Some(raw) = bad(malformed, "prefix") {
                    crate::schema_shapes::string(raw)?;
                }
                for member in &value.data {
                    let check = || -> SchemaResult {
                        // 6.0 takes the name first.
                        let named = member.keys.iter().any(|key| key == "name");
                        if since((6, 0, 0)) && !since((6, 1, 0)) && member.dictionary && !named {
                            return Err(crate::schema_shapes::unchecked("name"));
                        }
                        self.enum_value_keys(member)?;
                        if let Some(raw) = bad(&member.malformed, "name") {
                            crate::schema_shapes::name(raw)?;
                        }
                        empty_enum_value(&member.name)?;
                        crate::schema_names::validate_enum_member(&member.name)?;
                        self.member_case(&member.name, Some(name))?;
                        self.condition(member.r#if.as_ref(), bad(&member.malformed, "if"))?;
                        self.features(member.features.as_ref(), bad(&member.malformed, "features"))
                    };
                    check().within(|| format!("member '{}'", member.name))?;
                }
            }
            QapiSchemaToken::Struct(value) => {
                match bad(malformed, "data") {
                    // Until 8.0 a type name passes, and fails later.
                    Some(RawValue {
                        value: Raw::String(_),
                        ..
                    }) if !since((8, 1, 0)) => {}
                    Some(raw) => crate::schema_shapes::type_value(raw, false, true)
                        .within(|| "'data'".into())?,
                    None => self
                        .members(&value.data, Some(name))
                        .within(|| "'data'".into())?,
                }
                if let Some(raw) = bad(malformed, "base") {
                    crate::schema_shapes::type_value(raw, false, false)
                        .within(|| "'base'".into())?;
                }
            }
            QapiSchemaToken::Union(value) => self.union(name, value)?,
            QapiSchemaToken::Alternate(value) => {
                match bad(malformed, "data") {
                    Some(raw) => crate::schema_shapes::branches(raw, Some(1))?,
                    None => require(!value.data.0.is_empty(), || {
                        SchemaErrorKind::AlternateBranches { minimum: 1 }
                    })?,
                }
                for branch in &value.data.0 {
                    let check = || -> SchemaResult {
                        crate::schema_names::validate(&branch.name)?;
                        self.member_case(&branch.name, None)?;
                        self.branch_keys(branch)?;
                        self.condition(branch.r#if.as_ref(), bad(&branch.malformed, "if"))?;
                        if let Some(raw) = bad(&branch.malformed, "type") {
                            return crate::schema_shapes::type_value(raw, since((7, 1, 0)), false);
                        }
                        require(
                            since((7, 1, 0)) || !matches!(branch.r#type, QapiTypeRef::ArrayRef(_)),
                            || SchemaErrorKind::ArrayNotAllowed,
                        )
                    };
                    check().within(|| format!("branch '{}'", branch.name))?;
                }
            }
            QapiSchemaToken::Command(value) => {
                self.arguments(token, value.data.as_ref(), value.boxed)?;
                if let Some(raw) = bad(malformed, "returns") {
                    crate::schema_shapes::type_value(raw, true, false)
                        .within(|| "'returns'".into())?;
                }
            }
            QapiSchemaToken::Event(value) => {
                self.arguments(token, value.data.as_ref(), value.boxed)?
            }
            _ => {}
        }
        // From 5.0 QEMU checks the definition's condition before its
        // features.
        if since((5, 0, 0)) {
            self.condition(condition(token), bad(malformed, "if"))?;
            self.features(features(token).as_ref(), bad(malformed, "features"))?;
        } else {
            self.features(features(token).as_ref(), bad(malformed, "features"))?;
            self.condition(condition(token), bad(malformed, "if"))?;
        }
        self.flags(token)
    }

    /// A union's expression, from QEMU 4.2. Any value but `null` of a
    /// union's base or discriminator is one, which the checks of its shape
    /// refuse; the base first.
    fn union(&self, name: &str, value: &QapiUnion<'_>) -> SchemaResult {
        let malformed = &value.malformed;
        let raw_base = bad(malformed, "base");
        let raw_discriminator = bad(malformed, "discriminator");
        let base = value.base != MembersOrRef::Unset || raw_base.is_some();
        // A legacy alternate's `{}` is a discriminator too.
        let flat =
            value.discriminator.is_some() || raw_discriminator.is_some() || value.legacy_alternate;
        match (&value.base, flat) {
            (_, false) if base && !since((6, 2, 0)) => {
                return Err(SchemaErrorKind::BaseWithoutDiscriminator.into());
            }
            _ if raw_base.is_some() => {
                let raw = raw_base.unwrap_or_else(|| unreachable!());
                crate::schema_shapes::type_value(raw, false, true).within(|| "'base'".into())?;
            }
            (MembersOrRef::Unset, true) => return Err(SchemaErrorKind::MissingBase.into()),
            (MembersOrRef::Members(fields), true) => {
                self.members(fields, Some(name))
                    .within(|| "'base'".into())?;
                // Until 6.1 an empty object is no base.
                require(!fields.0.is_empty() || since((6, 2, 0)), || {
                    SchemaErrorKind::MissingBase
                })?;
            }
            _ => {}
        }
        if let Some(raw) = raw_discriminator {
            crate::schema_shapes::name(raw)?;
        }
        require(!value.legacy_alternate, || SchemaErrorKind::NameNotString)?;
        if let Some(raw) = bad(malformed, "data") {
            return crate::schema_shapes::branches(raw, None);
        }
        let simple = !flat;
        for branch in &value.data.0 {
            let check = || -> SchemaResult {
                // From 6.0 a flat union's branches are only enum values.
                if simple || !since((6, 0, 0)) {
                    crate::schema_names::validate(&branch.name)?;
                    self.member_case(&branch.name, None)?;
                }
                self.branch_keys(branch)?;
                self.condition(branch.r#if.as_ref(), bad(&branch.malformed, "if"))?;
                if let Some(raw) = bad(&branch.malformed, "type") {
                    return crate::schema_shapes::type_value(raw, simple, false);
                }
                // From 8.1 QEMU asks for a type name.
                require(
                    simple || !matches!(branch.r#type, QapiTypeRef::ArrayRef(_)),
                    || match since((8, 1, 0)) {
                        true => SchemaErrorKind::ValueShape {
                            expected: crate::unified::Shape::TypeName,
                        },
                        false => SchemaErrorKind::ArrayNotAllowed,
                    },
                )
            };
            check().within(|| format!("branch '{}'", branch.name))?;
        }
        Ok(())
    }

    /// The case of a definition's name, from QEMU 6.0: events in
    /// uppercase, commands in lowercase unless the pragma exempts them,
    /// types in CamelCase. A downstream `__RFQDN_` prefix and an `x-` do
    /// not count.
    fn definition_case(&self, token: &QapiSchemaToken<'_>, name: &str) -> SchemaResult {
        if !since((6, 0, 0)) {
            return Ok(());
        }
        let stem = stem(name);
        let (valid, rule) = match token {
            QapiSchemaToken::Event(_) => (
                !stem.contains(|c: char| c.is_ascii_lowercase() || c == '-'),
                NameRule::Uppercase,
            ),
            QapiSchemaToken::Command(_) => {
                let exempt = self.pragma.command_names.contains(&name);
                (
                    !stem.contains(|c: char| c.is_ascii_uppercase() || (c == '_' && !exempt)),
                    NameRule::Lowercase,
                )
            }
            _ => {
                let bytes = stem.as_bytes();
                (
                    bytes.first().is_some_and(u8::is_ascii_uppercase)
                        && bytes.iter().all(u8::is_ascii_alphanumeric)
                        && bytes[1..].iter().any(u8::is_ascii_lowercase),
                    NameRule::CamelCase,
                )
            }
        };
        require(valid, || SchemaErrorKind::NameCase {
            name: name.to_owned(),
            rule,
        })
    }

    /// The members of an object written out in place, from QEMU 4.2: names,
    /// case unless the pragma exempts `owner`, reserved names, features
    /// and the shape of the type.
    fn members(&self, fields: &QapiMembers<'_>, owner: Option<&str>) -> SchemaResult {
        for field in &fields.0 {
            let check = || -> SchemaResult {
                crate::schema_names::validate(&field.name)?;
                self.member_case(&field.name, owner)?;
                self.reserved(field)?;
                self.member_keys(field)?;
                self.condition(field.r#if.as_ref(), bad(&field.malformed, "if"))?;
                self.features(field.features.as_ref(), bad(&field.malformed, "features"))?;
                match bad(&field.malformed, "type") {
                    Some(raw) => crate::schema_shapes::type_value(raw, true, false),
                    None => Ok(()),
                }
            };
            check().within(|| format!("member '{}'", field.name))?;
        }
        Ok(())
    }

    /// The features, from QEMU 4.2: an array, whose names are strings with
    /// no uppercase until 5.2, and no `_` either in 6.0 and from 7.1.
    fn features(
        &self,
        features: Option<&QapiFeatures<'_>>,
        raw: Option<&RawValue<'_>>,
    ) -> SchemaResult {
        if let Some(raw) = raw {
            crate::schema_shapes::array(raw)?;
        }
        for feature in features.iter().flat_map(|features| features.iter()) {
            let check = || -> SchemaResult {
                self.feature_keys(feature)?;
                if let Some(name) = bad(&feature.malformed, "name") {
                    crate::schema_shapes::name(name)?;
                }
                crate::schema_names::validate(&feature.name)?;
                if !since((6, 0, 0)) {
                    lower_case(&feature.name, NameRule::NoUppercase)?;
                } else if since((7, 1, 0)) || !since((6, 1, 0)) {
                    lower_case(&feature.name, NameRule::Lowercase)?;
                }
                self.condition(feature.r#if.as_ref(), bad(&feature.malformed, "if"))
            };
            check().within(|| format!("feature '{}'", feature.name))?;
        }
        Ok(())
    }

    /// The shape of a command's or event's arguments, from QEMU 4.2. Python
    /// takes any true value for `boxed`, which the flags refuse later.
    fn arguments(
        &self,
        token: &QapiSchemaToken<'_>,
        data: Option<&MembersOrRef<'_>>,
        boxed: Option<&str>,
    ) -> SchemaResult {
        let raw_boxed = bad(malformed_of(token), "boxed");
        let boxed = boxed == Some("true") || raw_boxed.is_some_and(RawValue::truthy);
        if let Some(raw) = bad(malformed_of(token), "data") {
            return crate::schema_shapes::type_value(raw, false, !boxed).within(|| "'data'".into());
        }
        match data {
            Some(MembersOrRef::Ref(_)) => Ok(()),
            Some(MembersOrRef::Members(fields)) => {
                require(!boxed, || SchemaErrorKind::AnonymousStruct).within(|| "'data'".into())?;
                self.members(fields, None).within(|| "'data'".into())
            }
            None | Some(MembersOrRef::Unset) => {
                require(!boxed, || SchemaErrorKind::BoxedWithoutType)
            }
        }
    }

    /// The checks of an entity of the schema that QEMU builds: from 2.6
    /// the clashes between members and the cycles, from 4.2 also that each
    /// type a definition uses is defined and of the right kind. An error
    /// is located at the definition the entity belongs to.
    fn entity(&self, entity: &Entity<'a, 's>, objects: &mut Objects) -> SchemaResult {
        let resolve = since((4, 2, 0));
        match *entity {
            Entity::Array { element, owner } if resolve => self.located(
                owner,
                self.resolve(&QapiTypeRef::Ref(element.into())).map(drop),
            ),
            Entity::Array { element, owner } => self.located(owner, self.alternate_tag(element)),
            Entity::Implicit(ref name) => objects.check(self, name).map(drop),
            Entity::Wrapper { branch, owner } => match &branch.r#type {
                // The array type that a branch wraps is checked itself.
                // QEMU fails to describe the wrapper's member, until 6.1
                // when simple unions end.
                QapiTypeRef::Ref(_) if resolve => self.located(
                    owner,
                    self.resolve(&branch.r#type).map(drop).map_err(|_| {
                        SchemaErrorKind::Crash {
                            crash: crate::unified::Crash::SimpleBranchType {
                                branch: branch.name.to_string(),
                            },
                        }
                        .into()
                    }),
                ),
                QapiTypeRef::Ref(name) => self.located(owner, self.alternate_tag(name)),
                _ => Ok(()),
            },
            Entity::Kind(index) => {
                let token = self.definitions[index].token;
                let QapiSchemaToken::Union(value) = token else {
                    return Ok(());
                };
                let owner = crate::symbols::implicit_tag(value.name.as_ref());
                let mut seen = HashMap::new();
                let clashes = value.data.0.iter().try_for_each(|branch| {
                    crate::union_layout::clash(self, &mut seen, Member::new(&branch.name, &owner))
                });
                self.located(index, clashes)
            }
            Entity::Definition(index) => {
                self.located(index, self.definition_entity(index, objects))
            }
        }
    }

    /// The checks of a definition's entity.
    fn definition_entity(&self, index: usize, objects: &mut Objects) -> SchemaResult {
        let token = self.definitions[index].token;
        let name = token.definition_name().unwrap_or_default();
        let resolve = since((4, 2, 0));
        match token {
            QapiSchemaToken::Enum(value) => {
                self.definition_features(token)?;
                let mut seen = HashMap::new();
                for member in &value.data {
                    crate::union_layout::clash(self, &mut seen, Member::new(&member.name, name))?;
                }
                Ok(())
            }
            QapiSchemaToken::Struct(_) | QapiSchemaToken::Union(_) => {
                objects.check(self, name).map(drop)
            }
            QapiSchemaToken::Alternate(value) => {
                self.definition_features(token)?;
                // From 4.2 QEMU resolves every branch first, then checks
                // them in turn.
                if resolve {
                    for branch in &value.data.0 {
                        self.resolve(&branch.r#type)
                            .within(|| format!("branch '{}'", branch.name))?;
                    }
                }
                let mut seen = HashMap::new();
                let mut kinds = HashMap::new();
                for branch in &value.data.0 {
                    if let QapiTypeRef::Ref(branch_type) = &branch.r#type {
                        self.alternate_tag(branch_type)?;
                    }
                    crate::union_layout::clash(self, &mut seen, Member::new(&branch.name, name))?;
                    if resolve {
                        self.alternate_kind(branch, &mut kinds)?;
                    }
                }
                Ok(())
            }
            QapiSchemaToken::Command(value) => {
                self.definition_features(token)?;
                self.arguments_entity(
                    objects,
                    name,
                    value.data.as_ref(),
                    value.boxed == Some("true"),
                )?;
                if let Some(QapiTypeRef::Ref(returns)) = &value.returns {
                    self.alternate_tag(returns)?;
                }
                if let (true, Some(returns)) = (resolve, &value.returns) {
                    let element = self.resolve(returns).within(|| "'returns'".into())?;
                    require(
                        self.returns_exempt(name) || matches!(element, Meta::Struct | Meta::Union),
                        || SchemaErrorKind::TypeNotAllowed {
                            meta: match returns {
                                QapiTypeRef::ArrayRef(_) => "array",
                                _ => element.word(),
                            },
                            reference: spelled(returns),
                        },
                    )
                    .within(|| "'returns'".into())?;
                }
                Ok(())
            }
            QapiSchemaToken::Event(value) => {
                self.definition_features(token)?;
                self.arguments_entity(
                    objects,
                    name,
                    value.data.as_ref(),
                    value.boxed == Some("true"),
                )
            }
            _ => Ok(()),
        }
    }

    /// Resolve a type reference, from QEMU 4.2, to the kind of the type it
    /// names or, for an array, of its elements.
    pub(crate) fn resolve(&self, value: &QapiTypeRef<'_>) -> SchemaResult<Meta> {
        let name = match value {
            QapiTypeRef::Ref(name) | QapiTypeRef::ArrayRef(name) => name.as_ref(),
            QapiTypeRef::InlineStruct(_) | QapiTypeRef::Unset => {
                return Err(SchemaErrorKind::AnonymousStruct.into());
            }
        };
        match self.meta(name) {
            Some(meta) if !matches!(meta, Meta::Command | Meta::Event) => Ok(meta),
            _ => Err(SchemaErrorKind::UnknownType {
                name: name.to_owned(),
            }
            .into()),
        }
    }

    /// The features of a definition: no two clash, from QEMU 4.1, and from
    /// 5.0 a type has none of the special ones. In 4.2 QEMU connects each
    /// feature to the documentation as it checks it.
    pub(crate) fn definition_features(&self, token: &QapiSchemaToken<'_>) -> SchemaResult {
        let features = features(token);
        let owner = token.definition_name().unwrap_or_default();
        let mut seen = HashMap::new();
        for feature in features.iter().flat_map(|features| features.iter()) {
            self.feature_clash(&mut seen, &feature.name, owner)?;
            if since((4, 2, 0)) && !since((5, 0, 0)) {
                schema_docs::connect_feature(token, &feature.name)?;
            }
        }
        if matches!(
            token,
            QapiSchemaToken::Command(_) | QapiSchemaToken::Event(_)
        ) || !since((5, 0, 0))
        {
            return Ok(());
        }
        let special: &[&str] = if since((6, 2, 0)) {
            &["deprecated", "unstable"]
        } else {
            &["deprecated"]
        };
        for feature in features.iter().flat_map(|features| features.iter()) {
            require(!special.contains(&feature.name.as_ref()), || {
                SchemaErrorKind::SpecialFeature {
                    name: feature.name.to_string(),
                }
            })?;
        }
        Ok(())
    }

    /// Refuse two features of one owner whose C names clash.
    pub(crate) fn features_clash(
        &self,
        features: Option<&QapiFeatures<'_>>,
        owner: &str,
    ) -> SchemaResult {
        let mut seen = HashMap::new();
        for feature in features.iter().flat_map(|features| features.iter()) {
            self.feature_clash(&mut seen, &feature.name, owner)?;
        }
        Ok(())
    }

    fn feature_clash(
        &self,
        seen: &mut crate::union_layout::Seen,
        feature: &str,
        owner: &str,
    ) -> SchemaResult {
        crate::union_layout::clash(self, seen, Member::new(feature, owner)).map_err(|error| {
            match error.kind() {
                SchemaErrorKind::MemberClash { name, previous } => SchemaErrorKind::FeatureClash {
                    name: name.clone(),
                    previous: previous.clone(),
                }
                .into(),
                _ => error,
            }
        })
    }

    /// A command's or event's arguments in the schema: an object type, a
    /// union only boxed, and from 8.1 conditional members only boxed; from
    /// 2.7 to 4.1 boxed ones of a type with members. QEMU checks the type
    /// with the command or event from 2.7 to 4.1 and from 8.1.
    fn arguments_entity(
        &self,
        objects: &mut Objects,
        name: &str,
        data: Option<&MembersOrRef<'_>>,
        boxed: bool,
    ) -> SchemaResult {
        let resolve = since((4, 2, 0));
        let checked = since((2, 7, 0)) && !resolve || since((8, 1, 0));
        if !resolve && matches!(data, None | Some(MembersOrRef::Unset)) {
            return require(!boxed, || SchemaErrorKind::BoxedWithoutType);
        }
        let object = match data {
            Some(MembersOrRef::Members(fields)) if !fields.0.is_empty() && checked => {
                objects.check(self, &format!("q_obj_{name}-arg"))?
            }
            Some(MembersOrRef::Ref(argument)) => {
                let meta = self
                    .resolve(&QapiTypeRef::Ref(argument.clone()))
                    .within(|| "'data'".into())?;
                match meta {
                    Meta::Struct => {}
                    Meta::Union if boxed => {}
                    Meta::Union => {
                        return Err(SchemaErrorKind::UnboxedUnion {
                            name: argument.to_string(),
                        }
                        .into());
                    }
                    // Until 4.1 an alternate is valid boxed, and never
                    // empty; QEMU checks it again.
                    Meta::Alternate if boxed && !resolve => {
                        if let Some(index) = self.index(argument) {
                            self.entity(&Entity::Definition(index), objects)?;
                            self.described(index)?;
                        }
                        return Ok(());
                    }
                    _ => {
                        return Err(SchemaErrorKind::TypeNotAllowed {
                            meta: meta.word(),
                            reference: format!("'{argument}'"),
                        })
                        .within(|| "'data'".into());
                    }
                }
                if !checked {
                    return Ok(());
                }
                let object = objects.check(self, argument)?;
                require(
                    !boxed || resolve || !object.members.is_empty() || object.branches,
                    || SchemaErrorKind::BoxedEmpty {
                        name: argument.to_string(),
                    },
                )?;
                object
            }
            _ => return Ok(()),
        };
        require(
            boxed || !since((8, 1, 0)) || !object.members.iter().any(|member| member.conditional),
            || SchemaErrorKind::ConditionalArguments,
        )
    }

    /// At most 64 distinct features, from QEMU 10.0, counting its own
    /// `deprecated` and `unstable`.
    fn feature_limit(&self, indices: &[usize]) -> SchemaResult {
        if !since((10, 0, 0)) {
            return Ok(());
        }
        let mut names: HashSet<&str> = ["deprecated", "unstable"].into();
        for &index in indices {
            for feature in features_in_order(self.definitions[index].token) {
                if names.insert(feature) && names.len() > 64 {
                    return self.located(index, Err(SchemaErrorKind::TooManyFeatures.into()));
                }
            }
        }
        Ok(())
    }
}

/// The key of the first value of a definition that the grammar refused.
fn leftover<'t>(token: &'t QapiSchemaToken<'_>) -> Option<&'t str> {
    fn first<'t>(malformed: &'t Malformed<'_>) -> Option<&'t str> {
        malformed.first().map(|(key, _)| key.as_ref())
    }
    fn feature_list<'t>(features: &'t Option<QapiFeatures<'_>>) -> Option<&'t str> {
        features
            .iter()
            .flat_map(|features| features.iter())
            .find_map(|feature| first(&feature.malformed))
    }
    fn members<'t>(fields: &'t QapiMembers<'_>) -> Option<&'t str> {
        fields.0.iter().find_map(|field| {
            first(&field.malformed)
                .or_else(|| feature_list(&field.features))
                .or_else(|| match &field.r#type {
                    QapiTypeRef::InlineStruct(fields) => members(fields),
                    _ => None,
                })
        })
    }
    fn arguments<'t>(data: &'t Option<MembersOrRef<'_>>) -> Option<&'t str> {
        match data {
            Some(MembersOrRef::Members(fields)) => members(fields),
            _ => None,
        }
    }
    first(malformed_of(token))
        .or_else(|| feature_list(features(token)))
        .or_else(|| match token {
            QapiSchemaToken::Enum(value) => value.data.iter().find_map(|value| {
                first(&value.malformed).or_else(|| feature_list(&value.features))
            }),
            QapiSchemaToken::Struct(value) => members(&value.data),
            QapiSchemaToken::Union(value) => match &value.base {
                MembersOrRef::Members(fields) => members(fields),
                _ => None,
            }
            .or_else(|| {
                value
                    .data
                    .0
                    .iter()
                    .find_map(|branch| first(&branch.malformed))
            }),
            QapiSchemaToken::Alternate(value) => value
                .data
                .0
                .iter()
                .find_map(|branch| first(&branch.malformed)),
            QapiSchemaToken::Command(value) => {
                arguments(&value.data).or_else(|| match &value.returns {
                    Some(QapiTypeRef::InlineStruct(fields)) => members(fields),
                    _ => None,
                })
            }
            QapiSchemaToken::Event(value) => arguments(&value.data),
            _ => None,
        })
}

/// From 2.6 QEMU looks at the first character of an enum value's name,
/// which an empty one lacks.
fn empty_enum_value(name: &str) -> SchemaResult {
    require(!name.is_empty() || !since((2, 6, 0)), || {
        SchemaErrorKind::Crash {
            crash: crate::unified::Crash::UncheckedValue { key: "name".into() },
        }
    })
}

/// The key that names a definition's kind.
fn meta_key(token: &QapiSchemaToken<'_>) -> Option<&'static str> {
    Some(match token {
        QapiSchemaToken::Enum(_) => "enum",
        QapiSchemaToken::Struct(_) if !since((2, 4, 0)) => "type",
        QapiSchemaToken::Struct(_) => "struct",
        QapiSchemaToken::Union(_) => "union",
        QapiSchemaToken::Alternate(_) => "alternate",
        QapiSchemaToken::Command(_) => "command",
        QapiSchemaToken::Event(_) => "event",
        _ => return None,
    })
}

/// The values of a definition's keys that the grammar refused.
pub(crate) fn malformed_of<'t, 's>(token: &'t QapiSchemaToken<'s>) -> &'t Malformed<'s> {
    static NONE: Malformed<'static> = Vec::new();
    match token {
        QapiSchemaToken::Enum(value) => &value.malformed,
        QapiSchemaToken::Struct(value) => &value.malformed,
        QapiSchemaToken::Union(value) => &value.malformed,
        QapiSchemaToken::Alternate(value) => &value.malformed,
        QapiSchemaToken::Command(value) => &value.malformed,
        QapiSchemaToken::Event(value) => &value.malformed,
        _ => &NONE,
    }
}

/// A definition's own features.
fn features<'t, 's>(token: &'t QapiSchemaToken<'s>) -> &'t Option<QapiFeatures<'s>> {
    match token {
        QapiSchemaToken::Enum(value) => &value.features,
        QapiSchemaToken::Struct(value) => &value.features,
        QapiSchemaToken::Union(value) => &value.features,
        QapiSchemaToken::Alternate(value) => &value.features,
        QapiSchemaToken::Command(value) => &value.features,
        QapiSchemaToken::Event(value) => &value.features,
        _ => &None,
    }
}

fn condition<'t, 's>(token: &'t QapiSchemaToken<'s>) -> Option<&'t QapiCond<'s>> {
    match token {
        QapiSchemaToken::Enum(value) => value.r#if.as_ref(),
        QapiSchemaToken::Struct(value) => value.r#if.as_ref(),
        QapiSchemaToken::Union(value) => value.r#if.as_ref(),
        QapiSchemaToken::Alternate(value) => value.r#if.as_ref(),
        QapiSchemaToken::Command(value) => value.r#if.as_ref(),
        QapiSchemaToken::Event(value) => value.r#if.as_ref(),
        _ => None,
    }
}

/// The keys of a definition's expression, in their order in the source.
fn keys_of<'t, 's>(token: &'t QapiSchemaToken<'s>) -> &'t [crate::QapiString<'s>] {
    match token {
        QapiSchemaToken::Enum(value) => &value.keys,
        QapiSchemaToken::Struct(value) => &value.keys,
        QapiSchemaToken::Union(value) => &value.keys,
        QapiSchemaToken::Alternate(value) => &value.keys,
        QapiSchemaToken::Command(value) => &value.keys,
        QapiSchemaToken::Event(value) => &value.keys,
        _ => &[],
    }
}

/// The first of the `required` keys, in QEMU's order, that `keys` lacks.
fn missing(required: &[&str], keys: &[&str]) -> SchemaResult {
    match required.iter().find(|key| !keys.contains(key)) {
        Some(key) => Err(SchemaErrorKind::MissingKey {
            key: (*key).to_owned(),
        }
        .into()),
        None => Ok(()),
    }
}

/// Check the keys of a dictionary as QEMU does from 4.0: the first missing
/// key, then the first unknown one, each in sorted order.
fn dictionary_keys(keys: &[&str], required: &[&str], optional: &[&str]) -> SchemaResult {
    let mut absent: Vec<_> = required.iter().filter(|key| !keys.contains(key)).collect();
    absent.sort();
    missing(&absent.into_iter().copied().collect::<Vec<_>>(), &[])?;
    let mut unknown: Vec<_> = keys
        .iter()
        .filter(|key| !required.contains(key) && !optional.contains(key))
        .collect();
    unknown.sort();
    match unknown.first() {
        Some(key) => Err(SchemaErrorKind::UnknownKey {
            key: (**key).to_owned(),
        }
        .into()),
        None => Ok(()),
    }
}

/// [`dictionary_keys`] of a member, branch, enum value or feature written
/// as a dictionary; one written in its short form has no keys to check.
fn keyed(
    keys: Option<&[crate::QapiString<'_>]>,
    required: &[&str],
    optional: &[&str],
) -> SchemaResult {
    let Some(keys) = keys else {
        return Ok(());
    };
    let keys: Vec<&str> = keys.iter().map(|key| key.as_ref()).collect();
    dictionary_keys(&keys, required, optional)
}

/// The feature names of a definition, in the order QEMU's schema meets
/// them: its own, then those of its members.
fn features_in_order<'t>(token: &'t QapiSchemaToken<'_>) -> Vec<&'t str> {
    fn add<'t>(names: &mut Vec<&'t str>, features: &'t Option<QapiFeatures<'_>>) {
        names.extend(
            features
                .iter()
                .flat_map(|features| features.iter())
                .map(|feature| feature.name.as_ref()),
        );
    }
    let mut names = Vec::new();
    add(&mut names, features(token));
    let fields = match token {
        QapiSchemaToken::Enum(value) => {
            for member in &value.data {
                add(&mut names, &member.features);
            }
            None
        }
        QapiSchemaToken::Struct(value) => Some(&value.data),
        QapiSchemaToken::Union(value) => match &value.base {
            MembersOrRef::Members(fields) => Some(fields),
            _ => None,
        },
        QapiSchemaToken::Command(value) => match &value.data {
            Some(MembersOrRef::Members(fields)) => Some(fields),
            _ => None,
        },
        QapiSchemaToken::Event(value) => match &value.data {
            Some(MembersOrRef::Members(fields)) => Some(fields),
            _ => None,
        },
        _ => None,
    };
    for field in fields.iter().flat_map(|fields| &fields.0) {
        add(&mut names, &field.features);
    }
    names
}

/// A name without the downstream `__RFQDN_` prefix and the `x-` of
/// experimental names, which the case rules from QEMU 6.0 leave out.
fn stem(name: &str) -> &str {
    let name = name
        .strip_prefix("__")
        .and_then(|rest| rest.split_once('_').map(|(_, stem)| stem))
        .unwrap_or(name);
    name.strip_prefix("x-")
        .or_else(|| name.strip_prefix("X-"))
        .unwrap_or(name)
}

/// Refuse uppercase in `name` and, for [`NameRule::Lowercase`], `_`: from
/// QEMU 6.0 in its stem, before in the whole name.
pub(crate) fn lower_case(name: &str, rule: NameRule) -> SchemaResult {
    let checked = if since((6, 0, 0)) { stem(name) } else { name };
    require(
        !checked.contains(|c: char| {
            c.is_ascii_uppercase() || (c == '_' && rule == NameRule::Lowercase)
        }),
        || SchemaErrorKind::NameCase {
            name: name.to_owned(),
            rule,
        },
    )
}

/// QEMU's `camel_to_upper` of 2.4 and 2.5: the C enumeration constant of a
/// value.
fn camel_to_upper(value: &str) -> String {
    let name: Vec<char> = value
        .chars()
        .map(|c| if matches!(c, '.' | '-') { '_' } else { c })
        .collect();
    if value.chars().any(char::is_alphabetic) && !value.chars().any(char::is_lowercase) {
        return name.iter().collect();
    }
    let mut upper = String::new();
    for (i, &c) in name.iter().enumerate() {
        if c.is_uppercase()
            && i > 0
            && name[i - 1] != '_'
            && (name.get(i + 1).is_some_and(|next| next.is_lowercase())
                || name[i - 1].is_ascii_digit())
        {
            upper.push('_');
        }
        upper.push(c);
    }
    upper.trim_start_matches('_').to_uppercase()
}

#[cfg(test)]
pub(crate) mod tests;
