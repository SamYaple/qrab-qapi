//! Plan generated symbols against declarations from every selected release.
use crate::{
    QapiSchema, QapiSchemaToken,
    qapi_ir::{
        Attribute, Enum, EnumVariantKind, Metadata, Struct, rustify_name,
        rustify_type,
    },
    unified::Version,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Enum,
    Object,
    Alternate,
}

/// The kind of a type definition, which one Rust symbol keeps for a run of
/// releases.
fn kind(token: &QapiSchemaToken<'_>) -> Option<Kind> {
    Some(match token {
        QapiSchemaToken::Enum(_) => Kind::Enum,
        QapiSchemaToken::Struct(_) => Kind::Object,
        QapiSchemaToken::Alternate(_) => Kind::Alternate,
        QapiSchemaToken::Union(value) if value.legacy_alternate => {
            Kind::Alternate
        }
        QapiSchemaToken::Union(_) => Kind::Object,
        _ => return None,
    })
}

/// What a planned name stands for. A type and a command may share a name,
/// since the lowering merges a command into its argument type or gives the
/// command a `Command` name; any other two claims need names of their own.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Owner {
    Type,
    Command,
    Role,
}

/// The releases of `all` that start a run: those in which `state` is present
/// and differs from the release before.
fn run_starts<T: PartialEq>(
    all: &BTreeSet<Version>,
    state: impl Fn(&Version) -> Option<T>,
) -> Vec<Version> {
    let mut previous = None;
    let mut starts = Vec::new();
    for version in all {
        let current = state(version);
        if current.is_some() && current != previous {
            starts.push(*version);
        }
        previous = current;
    }
    starts
}

/// The name of a simple union's tag enum, which QEMU defines implicitly:
/// `_make_implicit_enum_type` in its `scripts/qapi/schema.py`, in the `Kind`
/// namespace that it reserved until all simple unions became flat ones. A
/// flat union that replaced a simple one declares that enum by the same name,
/// so both are one type.
pub(crate) fn implicit_tag(union: &str) -> String {
    format!("{union}Kind")
}

/// Marks the IR name of an item that the lowering names itself. That name is
/// a Rust name already, and it can equal the QAPI name of another type of the
/// release: a type that a later release adds yields its Rust name to the
/// earlier helper type, which the IR names in the same space as the QAPI
/// names. No schema name contains a NUL.
pub(crate) const SYNTHESIZED: char = '\0';

/// The IR name of an item that the lowering names `rust_name`.
pub(crate) fn synthesized(rust_name: &str) -> String {
    format!("{SYNTHESIZED}{rust_name}")
}

/// A helper type's or event's claim on a name in one release.
enum RoleClaim<'a> {
    /// A union's branch enum or a command's request type.
    Helper,
    /// The `run`th run of an event that has `runs` of them.
    Event { raw: &'a String, run: usize, runs: usize },
}

/// The Rust symbols of every selected release.
///
/// Each type, command and event, and each helper type of one (a union's
/// branch enum, a command's request type), claims its Rust name in
/// the first release that has it. The earliest claim keeps the plain name; a
/// later claim that would clash gets `Generated`, then `Generated2` and so on.
/// So does each later run of releases: a type starts a run when it changes
/// kind or returns after an absence, a command or event when it returns. A
/// newer release therefore never renames a symbol of the older ones.
///
/// The plan holds no name for the enum of a command's return types: only a
/// command whose return type still changes once the merge has coalesced the
/// wire-identical successors needs one, and [`crate::namespace`] gives it,
/// refusing a name that another symbol holds.
#[derive(Default)]
pub(crate) struct SchemaNames {
    versions: BTreeSet<Version>,
    /// Every planned name, which the types that the lowering extracts avoid.
    pub names: HashSet<String>,
    owners: HashMap<String, Owner>,
    roles: BTreeMap<String, String>,
    unions: BTreeMap<String, BTreeSet<Version>>,
    aliases: BTreeMap<String, String>,
    shared_requests: BTreeSet<(String, String, Version)>,
    shared_roots: HashSet<String>,
    declarations: BTreeMap<String, BTreeMap<Version, Kind>>,
    type_runs: BTreeMap<String, BTreeMap<Version, String>>,
    commands: BTreeMap<String, BTreeSet<Version>>,
    events: BTreeMap<String, BTreeSet<Version>>,
    command_aliases: BTreeMap<String, String>,
    event_aliases: BTreeMap<String, String>,
    command_runs: BTreeMap<String, BTreeMap<Version, String>>,
    separate_requests: HashSet<String>,
    event_runs: BTreeMap<String, BTreeMap<Version, String>>,
}

impl SchemaNames {
    pub fn merge(&mut self, other: Self) {
        self.versions.extend(other.versions);
        self.shared_requests.extend(other.shared_requests);
        for (name, versions) in other.unions {
            self.unions.entry(name).or_default().extend(versions);
        }
        for (name, versions) in other.commands {
            self.commands.entry(name).or_default().extend(versions);
        }
        for (name, versions) in other.events {
            self.events.entry(name).or_default().extend(versions);
        }
        for (name, versions) in other.declarations {
            self.declarations.entry(name).or_default().extend(versions);
        }
    }

    /// Leave out the predefined types that no release of the selection uses.
    pub fn keep_predefined(&mut self, used: &BTreeSet<&str>) {
        for name in crate::builtins::PREDEFINED {
            if !used.contains(name) {
                self.declarations.remove(name);
            }
        }
    }

    pub fn collect(&mut self, schema: &QapiSchema<'_>) {
        let version = crate::parser_version();
        self.versions.insert(version);
        for token in &schema.0 {
            let Some(name) = token.definition_name() else {
                continue;
            };
            if let Some(kind) = kind(token) {
                self.declarations
                    .entry(name.to_owned())
                    .or_default()
                    .insert(version, kind);
            }
            match token {
                QapiSchemaToken::Union(value) if !value.legacy_alternate => {
                    self.unions
                        .entry(name.to_owned())
                        .or_default()
                        .insert(version);
                    if value.discriminator.is_none() {
                        self.declarations
                            .entry(implicit_tag(name))
                            .or_default()
                            .insert(version, Kind::Enum);
                    }
                }
                QapiSchemaToken::Command(value) => {
                    self.commands
                        .entry(name.to_owned())
                        .or_default()
                        .insert(version);
                    if let Some(crate::MembersOrRef::Ref(data)) = &value.data
                        && rustify_name(name) == rustify_name(data)
                    {
                        self.shared_requests.insert((
                            rustify_name(name),
                            data.to_string(),
                            version,
                        ));
                    }
                }
                QapiSchemaToken::Event(_) => {
                    self.events
                        .entry(name.to_owned())
                        .or_default()
                        .insert(version);
                }
                _ => {}
            }
        }
    }

    /// Take `preferred` unless another claim holds it.
    fn take_plain(&mut self, preferred: &str, owner: Owner) -> Option<String> {
        let free = match self.owners.get(preferred) {
            None => true,
            Some(Owner::Type) => owner == Owner::Command,
            Some(Owner::Command) => owner == Owner::Type,
            Some(Owner::Role) => false,
        };
        free.then(|| {
            self.owners.entry(preferred.to_owned()).or_insert(owner);
            preferred.to_owned()
        })
    }

    /// Take the first of `{preferred}Generated`, `{preferred}Generated2`, ...
    /// that no claim holds.
    fn take_generated(&mut self, preferred: &str, owner: Owner) -> String {
        let base = format!("{preferred}Generated");
        let mut name = base.clone();
        let mut suffix = 2;
        while self.owners.contains_key(&name) {
            name = format!("{base}{suffix}");
            suffix += 1;
        }
        self.owners.insert(name.clone(), owner);
        name
    }

    /// Name the claims of one release, in order: plain names first, so that
    /// no generated name takes the plain name of a claim of the same release.
    /// A later run never takes the plain name, which its first run holds. A
    /// claim is `(preferred, owner, later)`.
    fn take_all(&mut self, claims: &[(String, Owner, bool)]) -> Vec<String> {
        let plain: Vec<_> =
            claims
                .iter()
                .map(|(preferred, owner, later)| {
                    if *later {
                        None
                    } else {
                        self.take_plain(preferred, *owner)
                    }
                })
                .collect();
        plain
            .into_iter()
            .zip(claims)
            .map(|(name, (preferred, owner, _))| {
                name.unwrap_or_else(|| self.take_generated(preferred, *owner))
            })
            .collect()
    }

    pub fn finish(&mut self) {
        // Spellings of one Rust name that never coexist share one identity.
        let mut groups: BTreeMap<String, Vec<&String>> = BTreeMap::new();
        for raw in self.declarations.keys() {
            groups.entry(rustify_name(raw)).or_default().push(raw);
        }
        let mut identities: Vec<(
            String,
            Vec<String>,
            BTreeMap<Version, Kind>,
        )> = Vec::new();
        for (root, mut raws) in groups {
            raws.sort_by_key(|raw| {
                let since = self.declarations[*raw].keys().next().copied();
                (since, **raw != root, (*raw).clone())
            });
            let mut slots: Vec<(Vec<String>, BTreeMap<Version, Kind>)> =
                Vec::new();
            for raw in raws {
                let versions = &self.declarations[raw];
                let slot = slots.iter_mut().find(|(_, history)| {
                    !versions
                        .keys()
                        .any(|version| history.contains_key(version))
                });
                match slot {
                    Some((members, history)) => {
                        members.push(raw.clone());
                        history.extend(versions);
                    }
                    None => slots.push((vec![raw.clone()], versions.clone())),
                }
            }
            identities.extend(
                slots
                    .into_iter()
                    .map(|(members, history)| (root.clone(), members, history)),
            );
        }
        let type_starts: Vec<_> = identities
            .iter()
            .map(|(_, _, history)| {
                run_starts(&self.versions, |version| history.get(version))
            })
            .collect();
        let presence = |present: &BTreeSet<Version>| {
            run_starts(&self.versions, |version| {
                present.contains(version).then_some(())
            })
        };
        let command_starts: BTreeMap<_, _> = self
            .commands
            .iter()
            .map(|(raw, present)| (raw.clone(), presence(present)))
            .collect();
        let event_starts: BTreeMap<_, _> = self
            .events
            .iter()
            .map(|(raw, present)| (raw.clone(), presence(present)))
            .collect();
        let mut identity_names = vec![String::new(); identities.len()];
        let versions: Vec<_> = self.versions.iter().copied().collect();
        for version in versions {
            // The types that start a run in this release.
            let mut claims = Vec::new();
            let mut runs = Vec::new();
            for (index, starts) in type_starts.iter().enumerate() {
                if let Some(run) =
                    starts.iter().position(|since| *since == version)
                {
                    claims.push((
                        identities[index].0.clone(),
                        Owner::Type,
                        run > 0,
                    ));
                    runs.push((index, run));
                }
            }
            for ((index, run), name) in
                runs.into_iter().zip(self.take_all(&claims))
            {
                if run == 0 {
                    for raw in &identities[index].1 {
                        if rustify_name(raw) != name {
                            self.aliases.insert(raw.clone(), name.clone());
                        }
                    }
                    identity_names[index].clone_from(&name);
                }
                if type_starts[index].len() > 1 {
                    self.type_runs
                        .entry(identity_names[index].clone())
                        .or_default()
                        .insert(version, name);
                }
            }

            // The commands that start a run in this release.
            let mut claims = Vec::new();
            let mut runs = Vec::new();
            for (raw, starts) in &command_starts {
                if let Some(run) =
                    starts.iter().position(|since| *since == version)
                {
                    claims.push((rustify_name(raw), Owner::Command, run > 0));
                    runs.push((raw, run, starts.len()));
                }
            }
            for ((raw, run, count), name) in
                runs.into_iter().zip(self.take_all(&claims))
            {
                if run == 0 && name != rustify_name(raw) {
                    self.command_aliases.insert(raw.clone(), name.clone());
                }
                if count > 1 {
                    // The first run's request type has the command's own name.
                    let name = if run == 0 { String::new() } else { name };
                    self.command_runs
                        .entry(raw.clone())
                        .or_default()
                        .insert(version, name);
                }
            }

            // The helper types and event types that this release adds.
            let mut claims = Vec::new();
            let mut helpers = BTreeSet::new();
            for (raw, versions) in &self.unions {
                if versions.contains(&version) {
                    let root = self.type_name_at(raw, version);
                    helpers.insert(format!("{root}Branch"));
                }
            }
            for (raw, starts) in &command_starts {
                if starts.first() == Some(&version) {
                    helpers
                        .insert(format!("{}Command", self.command_root(raw)));
                }
            }
            helpers.retain(|preferred| !self.roles.contains_key(preferred));
            claims.extend(
                helpers
                    .into_iter()
                    .map(|preferred| (preferred, RoleClaim::Helper)),
            );
            for (raw, starts) in &event_starts {
                if let Some(run) =
                    starts.iter().position(|since| *since == version)
                {
                    let preferred = rustify_name(&format!("{raw}Event"));
                    let runs = starts.len();
                    claims
                        .push((preferred, RoleClaim::Event { raw, run, runs }));
                }
            }
            claims.sort_by(|(left, _), (right, _)| left.cmp(right));
            let requests: Vec<_> = claims
                .iter()
                .map(|(preferred, claim)| {
                    let later = matches!(claim, RoleClaim::Event { run, .. } if *run > 0);
                    // A composed name is escaped like a schema name.
                    let name = crate::qapi_ir::escape_reserved(preferred.clone());
                    (name, Owner::Role, later)
                })
                .collect();
            let names = self.take_all(&requests);
            for ((preferred, claim), name) in claims.into_iter().zip(names) {
                match claim {
                    RoleClaim::Helper => {
                        self.roles.insert(preferred, name);
                    }
                    RoleClaim::Event { raw, run, runs } => {
                        if run == 0 {
                            // The first event of a Rust name holds its role;
                            // another is an alias of its own.
                            match self.roles.entry(preferred) {
                                std::collections::btree_map::Entry::Occupied(_) => {
                                    self.event_aliases.insert(raw.clone(), name.clone());
                                }
                                std::collections::btree_map::Entry::Vacant(entry) => {
                                    entry.insert(name.clone());
                                }
                            }
                        }
                        if runs > 1 {
                            self.event_runs
                                .entry(raw.clone())
                                .or_default()
                                .insert(version, name);
                        }
                    }
                }
            }
        }
        self.names = self.owners.keys().cloned().collect();

        // Wire-distinct commands never share request identity, even when they
        // occur in disjoint releases and normalize to the same Rust name.
        for raw in self.commands.keys() {
            if self
                .command_aliases
                .keys()
                .any(|alias| rustify_name(alias) == rustify_name(raw))
            {
                self.separate_requests.insert(raw.clone());
            }
        }
        self.separate_requests.extend(self.command_runs.keys().cloned());
        for (root, data, _) in &self.shared_requests {
            for (command, present) in &self.commands {
                if rustify_name(command) == *root
                    && self.declarations.get(data).is_some_and(|versions| {
                        versions
                            .keys()
                            .any(|version| !present.contains(version))
                    })
                {
                    self.separate_requests.insert(command.clone());
                }
            }
        }
        // A later named DTO can continue an earlier anonymous request type.
        for (root, data, version) in &self.shared_requests {
            let split = self
                .separate_requests
                .iter()
                .any(|name| rustify_name(name) == *root);
            if !split && &self.type_name_at(data, *version) == root {
                self.shared_roots.insert(root.clone());
            }
        }
    }

    pub fn separate_request(&self, name: &str) -> bool {
        self.separate_requests.contains(name)
    }

    pub fn command_root(&self, name: &str) -> String {
        self.command_aliases
            .get(name)
            .cloned()
            .unwrap_or_else(|| rustify_name(name))
    }

    fn event_root(&self, name: &str) -> String {
        self.event_aliases.get(name).cloned().unwrap_or_else(|| {
            self.role(&rustify_name(&format!("{name}Event")))
        })
    }

    pub fn historical_command(&self, name: &str) -> Option<String> {
        self.command_runs
            .get(name)
            .and_then(|runs| runs.range(..=crate::parser_version()).next_back())
            .map(|(_, name)| name)
            .filter(|name| !name.is_empty())
            .cloned()
    }

    pub fn event_name(&self, name: &str) -> String {
        self.event_runs
            .get(name)
            .and_then(|runs| runs.range(..=crate::parser_version()).next_back())
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| self.event_root(name))
    }

    pub fn role(&self, preferred: &str) -> String {
        self.roles[preferred].clone()
    }

    pub fn alias_name(&self, name: &str) -> String {
        self.aliases.get(name).cloned().unwrap_or_else(|| rustify_name(name))
    }

    pub fn data_symbols(&self) -> HashSet<String> {
        self.declarations
            .keys()
            .map(|name| self.alias_name(name))
            .chain(
                self.type_runs.values().flat_map(|runs| runs.values().cloned()),
            )
            .filter(|name| !self.shared_roots.contains(name))
            .collect()
    }

    pub fn type_name(&self, name: &str) -> String {
        self.type_name_at(name, crate::parser_version())
    }

    fn type_name_at(&self, name: &str, version: Version) -> String {
        let root = self.alias_name(name);
        self.type_runs
            .get(&root)
            .and_then(|runs| runs.range(..=version).next_back())
            .map_or(root.clone(), |(_, name)| name.clone())
    }

    /// Give the lowered release the planned Rust names: a type whose name the
    /// plan changed keeps its QAPI name as its `name` attribute, and the
    /// lowering's own items lose their [`SYNTHESIZED`] mark. References follow.
    pub fn rewrite_types(
        &self,
        declared: &HashSet<String>,
        enums: &mut HashMap<String, Enum>,
        structs: &mut HashMap<String, Struct>,
    ) {
        let assigned: HashMap<_, _> = declared
            .iter()
            .map(|name| (name.as_str(), self.type_name(name)))
            .collect();
        // A reference resolves to the declared type even where the name spells
        // a builtin, since a release defines no type that its builtins spell.
        let reference = |name: &mut String| {
            if let Some(rust) = name.strip_prefix(SYNTHESIZED) {
                *name = rust.to_owned();
            } else if let Some(assigned) = assigned.get(name.as_str())
                && rustify_type(name) != *assigned
            {
                name.clone_from(assigned);
            }
        };
        let rewrite_name = |name: &mut String, meta: &mut Metadata| {
            if assigned
                .get(name.as_str())
                .is_some_and(|assigned| *assigned != rustify_name(name))
                && !meta
                    .attributes
                    .iter()
                    .any(|attr| matches!(attr, Attribute::Item { name, .. } if name == "name"))
            {
                meta.attributes.push(Attribute::with_value("name", &*name));
            }
            reference(name);
        };
        for value in enums.values_mut() {
            rewrite_name(&mut value.name, &mut value.meta);
            for variant in &mut value.variants {
                if let EnumVariantKind::Tuple(ty) = &mut variant.kind {
                    reference(ty);
                }
            }
        }
        for value in structs.values_mut() {
            rewrite_name(&mut value.name, &mut value.meta);
            for field in &mut value.fields {
                reference(&mut field.r#type);
            }
        }
    }

    pub fn declared_names<'a>(
        schemas: &[(&std::path::Path, QapiSchema<'a>)],
    ) -> HashSet<String> {
        schemas
            .iter()
            .flat_map(|(_, schema)| &schema.0)
            .filter(|token| kind(token).is_some())
            .filter_map(QapiSchemaToken::definition_name)
            .map(str::to_owned)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qapi_ir::StructField;

    fn planned(releases: &[(Version, &str)]) -> SchemaNames {
        let mut symbols = SchemaNames::default();
        for (version, text) in releases {
            crate::set_parser_version(*version);
            symbols.collect(&crate::parse_schema(text).unwrap());
        }
        symbols.finish();
        symbols
    }

    /// The Rust name of every definition and helper type of every release.
    fn symbols(releases: &[(Version, &str)]) -> BTreeMap<String, String> {
        let plan = planned(releases);
        let mut symbols = BTreeMap::new();
        for (version, text) in releases {
            crate::set_parser_version(*version);
            for token in crate::parse_schema(text).unwrap().0 {
                let Some(name) = token.definition_name() else {
                    continue;
                };
                let key = |what: &str| format!("{version:?} {what} {name}");
                match &token {
                    QapiSchemaToken::Command(_) => {
                        let request = plan
                            .historical_command(name)
                            .unwrap_or_else(|| plan.command_root(name));
                        symbols.insert(key("request"), request);
                        let role = plan.role(&format!(
                            "{}Command",
                            plan.command_root(name)
                        ));
                        symbols.insert(key("command"), role);
                    }
                    QapiSchemaToken::Event(_) => {
                        symbols.insert(key("event"), plan.event_name(name));
                    }
                    _ => {
                        symbols.insert(key("type"), plan.type_name(name));
                    }
                }
                if let QapiSchemaToken::Union(value) = &token {
                    let root = plan.type_name(name);
                    symbols.insert(
                        key("branch"),
                        plan.role(&format!("{root}Branch")),
                    );
                    if value.discriminator.is_none() {
                        symbols.insert(
                            key("tag"),
                            plan.type_name(&implicit_tag(name)),
                        );
                    }
                }
            }
        }
        symbols
    }

    const OLDER: [(Version, &str); 2] = [
        (
            (5, 0, 0),
            "{'union':'Choice','data':{'one':'str'}}
             {'struct':'Gone','data':{}}
             {'enum':'Shift','data':['a']}
             {'command':'ping'}
             {'command':'bar-baz'}
             {'event':'TICK'}",
        ),
        (
            (5, 1, 0),
            "{'union':'Choice','data':{'one':'str'}}
             {'enum':'Shift','data':['a']}
             {'command':'bar_baz'}",
        ),
    ];

    #[test]
    fn a_newer_release_renames_no_symbol_of_the_older_ones() {
        // The newer release brings back what the older ones removed, changes
        // a type's kind, declares the names of the helper types, and turns the
        // simple union flat with the tag enum it had implicitly.
        let newer = (
            (6, 2, 0),
            "{'enum':'ChoiceKind','data':['one']}
             {'struct':'Wrapper','data':{'value':'str'}}
             {'union':'Choice','base':{'type':'ChoiceKind'},'discriminator':'type',
              'data':{'one':'Wrapper'}}
             {'struct':'ChoiceBranch','data':{}}
             {'struct':'Gone','data':{}}
             {'struct':'Shift','data':{}}
             {'struct':'PingCommand','data':{}}
             {'command':'ping'}
             {'command':'bar_baz'}
             {'event':'TICK'}",
        );
        let before = symbols(&OLDER);
        let mut releases = OLDER.to_vec();
        releases.push(newer);
        let after = symbols(&releases);
        for (key, name) in &before {
            assert_eq!(after.get(key), Some(name), "{key}");
        }
        let at = |what: &str| after[&format!("(6, 2, 0) {what}")].as_str();
        assert_eq!(at("type ChoiceKind"), "ChoiceKind");
        assert_eq!(before["(5, 0, 0) tag Choice"], "ChoiceKind");
        assert_eq!(at("type ChoiceBranch"), "ChoiceBranchGenerated");
        assert_eq!(at("branch Choice"), "ChoiceBranch");
        assert_eq!(at("type Gone"), "GoneGenerated");
        assert_eq!(at("type Shift"), "ShiftGenerated");
        assert_eq!(at("type PingCommand"), "PingCommandGenerated");
        assert_eq!(at("request ping"), "PingGenerated");
        assert_eq!(at("event TICK"), "TickEventGenerated");
        assert_eq!(before["(5, 0, 0) request bar-baz"], "BarBaz");
        assert_eq!(at("request bar_baz"), "BarBazGenerated");
    }

    #[test]
    fn exclusive_spellings_share_a_symbol_and_coexisting_ones_do_not() {
        let after = symbols(&[
            ((5, 0, 0), "{'struct':'FooBAR','data':{}}"),
            ((5, 1, 0), "{'struct':'FooBar','data':{}}"),
            (
                (5, 2, 0),
                "{'struct':'FooBar','data':{}} {'struct':'FOOBar','data':{}}",
            ),
        ]);
        assert_eq!(after["(5, 0, 0) type FooBAR"], "FooBar");
        assert_eq!(after["(5, 1, 0) type FooBar"], "FooBar");
        assert_eq!(after["(5, 2, 0) type FooBar"], "FooBar");
        assert_eq!(after["(5, 2, 0) type FOOBar"], "FooBarGenerated");
        // In one release the exact spelling of the Rust name keeps it.
        let after = symbols(&[(
            (5, 0, 0),
            "{'struct':'FooBAR','data':{}} {'enum':'FooBar','data':[]}",
        )]);
        assert_eq!(after["(5, 0, 0) type FooBar"], "FooBar");
        assert_eq!(after["(5, 0, 0) type FooBAR"], "FooBarGenerated");
        let plan = planned(&[(
            (5, 0, 0),
            "{'struct':'FooBar','data':{}} {'struct':'FooBarGenerated','data':{}}
             {'struct':'FooBAR','data':{}}",
        )]);
        crate::set_parser_version((5, 0, 0));
        assert_eq!(plan.type_name("FooBarGenerated"), "FooBarGenerated");
        assert_eq!(plan.type_name("FooBAR"), "FooBarGenerated2");
    }

    /// The plan holds no `Returns` name: a type named like the enum of a
    /// command's return types keeps its name, and a command that would need
    /// that enum is refused after the merge (`crate::namespace`).
    #[test]
    fn types_named_like_an_enum_of_return_types_keep_their_names() {
        let after = symbols(&[(
            (5, 0, 0),
            "{'command':'ping'} {'struct':'PingReturns','data':{}}
             {'struct':'PingCommandReturns','data':{}}",
        )]);
        assert_eq!(after["(5, 0, 0) request ping"], "Ping");
        assert_eq!(after["(5, 0, 0) type PingReturns"], "PingReturns");
        assert_eq!(
            after["(5, 0, 0) type PingCommandReturns"],
            "PingCommandReturns"
        );
    }

    #[test]
    fn wire_distinct_events_of_one_rust_name_get_names_of_their_own() {
        // The earlier event keeps the plain name, whatever the spellings'
        // order.
        let after = symbols(&[
            ((5, 0, 0), "{'event':'NOTICE__BAR'}"),
            ((5, 1, 0), "{'event':'NOTICE_BAR'} {'event':'NOTICE__BAR'}"),
        ]);
        assert_eq!(after["(5, 0, 0) event NOTICE__BAR"], "NoticeBarEvent");
        assert_eq!(after["(5, 1, 0) event NOTICE__BAR"], "NoticeBarEvent");
        assert_eq!(
            after["(5, 1, 0) event NOTICE_BAR"],
            "NoticeBarEventGenerated"
        );
    }

    #[test]
    fn requests_share_their_argument_type_only_over_its_whole_lifetime() {
        let plan = planned(&[
            (
                (5, 0, 0),
                "{'struct':'Late','data':{}} {'struct':'Same','data':{}}",
            ),
            (
                (5, 1, 0),
                "{'struct':'Late','data':{}} {'struct':'Same','data':{}}
                 {'command':'late','data':'Late'}",
            ),
        ]);
        assert!(plan.separate_request("late"));
        let plan = planned(&[(
            (5, 0, 0),
            "{'struct':'Same','data':{}} {'command':'same','data':'Same'}",
        )]);
        assert!(!plan.separate_request("same"));
        assert!(!plan.data_symbols().contains("Same"));
    }

    #[test]
    fn renamed_types_keep_their_schema_name_and_synthesized_names_are_unmarked()
    {
        let plan = planned(&[(
            (5, 0, 0),
            "{'struct':'FooBAR','data':{}} {'enum':'FooBar','data':[]}",
        )]);
        crate::set_parser_version((5, 0, 0));
        let declared: HashSet<_> =
            ["FooBAR".to_owned(), "FooBar".to_owned()].into();
        let record = |name: &str, field_type: &str| Struct {
            name: name.into(),
            fields: vec![StructField {
                name: "value".into(),
                r#type: field_type.into(),
                optional: false,
                array: false,
                meta: Metadata::default(),
            }],
            meta: Metadata::default(),
        };
        let mut wired = record("FooBAR", "FooBar");
        wired.meta.attributes.push(Attribute::with_value("name", "wire"));
        let mut structs = HashMap::from([
            ("FooBAR".to_owned(), record("FooBAR", "FooBAR")),
            ("wired".to_owned(), wired),
            (
                synthesized("Helper"),
                record(&synthesized("Helper"), &synthesized("Other")),
            ),
        ]);
        plan.rewrite_types(&declared, &mut HashMap::new(), &mut structs);
        let renamed = &structs["FooBAR"];
        assert_eq!(renamed.name, "FooBarGenerated");
        assert_eq!(renamed.fields[0].r#type, "FooBarGenerated");
        assert_eq!(
            renamed.meta.attributes,
            [Attribute::with_value("name", "FooBAR")]
        );
        assert_eq!(
            structs["wired"].meta.attributes,
            [Attribute::with_value("name", "wire")]
        );
        assert_eq!(structs["wired"].fields[0].r#type, "FooBar");
        let helper = &structs[&synthesized("Helper")];
        assert_eq!(
            (helper.name.as_str(), helper.fields[0].r#type.as_str()),
            ("Helper", "Other")
        );
        assert!(helper.meta.attributes.is_empty());
    }
}
