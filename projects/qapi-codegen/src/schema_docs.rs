//! Definitions' documentation: comments that legacy schemas wrote before an
//! intervening helper definition, and from 2.9 the rules that QEMU's
//! generator checks between a definition and its documentation.
//!
//! QEMU checks them in steps, which the validation calls in its order: the
//! expression checks ([`check_expression`] from 4.2; [`check_required`],
//! [`check_symbol`] and [`check_sections`] at their own places before), and
//! with the definition's types [`connect`] and [`check_described`], which
//! from 9.0 runs after every definition's types instead.
use crate::parsers::doc_comment::{self, Describe};
use crate::unified::{SchemaContext, SchemaError, SchemaErrorKind, SchemaResult, require};
use crate::{MembersOrRef, QapiFeatures, QapiMembers, QapiPragma, QapiSchema, QapiSchemaToken};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Before 2.9, give a comment that documents another definition of its
/// file, one without a comment of its own, to that definition, before or
/// after it.
///
/// Such comments sit before a helper definition that the documented one
/// uses. A comment whose target has its own documentation, or that names
/// no other definition of the file, stays where it is, as does a later
/// comment for the same target.
pub(crate) fn resolve_legacy(schemas: &mut [(&Path, QapiSchema<'_>)]) {
    if crate::parser_version() >= (2, 9, 0) {
        return;
    }
    for (_, schema) in schemas {
        let undocumented: HashSet<_> = schema
            .0
            .iter()
            .filter(|token| token.documentation().is_none())
            .filter_map(|token| token.definition_name().map(str::to_owned))
            .collect();
        let mut pending = HashMap::new();
        for token in &mut schema.0 {
            let Some(name) = token.definition_name().map(str::to_owned) else {
                continue;
            };
            let Some(slot) = token.documentation_mut() else {
                continue;
            };
            // The first comment that names a target moves; later ones stay.
            if let Some(doc) = slot.take_if(|doc| {
                doc.name != name
                    && undocumented.contains(doc.name)
                    && !pending.contains_key(doc.name)
            }) {
                pending.insert(doc.name, doc);
            }
        }
        // Before 2.4 a type and a command may share a name: the comment
        // goes to the one without documentation.
        for token in &mut schema.0 {
            let Some(name) = token.definition_name().map(str::to_owned) else {
                continue;
            };
            if let Some(slot) = token.documentation_mut().filter(|slot| slot.is_none())
                && let Some(doc) = pending.remove(name.as_str())
            {
                *slot = Some(doc);
            }
        }
    }
}

/// The pragmas that the documentation rules read: QEMU keeps one set for a
/// schema and its includes, and the value set last wins.
#[derive(Debug, Default)]
pub(crate) struct Pragmas {
    doc_required: bool,
    documentation_exceptions: HashSet<String>,
}

impl Pragmas {
    /// The rules of the schema's pragma directives, given in the order QEMU
    /// reads them.
    pub(crate) fn of(pragmas: &[&QapiPragma<'_>]) -> Self {
        let mut rules = Self::default();
        for pragma in pragmas {
            if let Some(value) = pragma.doc_required {
                rules.doc_required = value;
            }
            if let Some(names) = &pragma.documentation_exceptions {
                rules.documentation_exceptions =
                    names.iter().map(|name| name.to_string()).collect();
            }
        }
        rules
    }
}

/// What a definition's documentation describes, as QEMU's parser reads it.
#[derive(Default)]
struct DocComment<'i> {
    /// The described members and features, in the comment's order, with
    /// their comment lines.
    members: Vec<(String, &'i str)>,
    features: Vec<(String, &'i str)>,
    returns: Option<&'i str>,
    errors: Option<&'i str>,
    /// From 10.0: the sections after the introduction, and whether they
    /// all describe members.
    sections: (usize, bool),
}

impl<'i> Describe<'i> for DocComment<'i> {
    fn member(&mut self, name: &str, at: &'i str) {
        self.members.push((name.to_owned(), at));
    }

    fn feature(&mut self, name: &str, at: &'i str) {
        self.features.push((name.to_owned(), at));
    }

    fn returns(&mut self, at: &'i str) {
        self.returns = Some(at);
    }

    fn errors(&mut self, at: &'i str) {
        self.errors = Some(at);
    }

    fn sections(&mut self, count: usize, members: bool) {
        self.sections = (count, members);
    }
}

fn describes(descriptions: &[(String, &str)], name: &str) -> bool {
    descriptions.iter().any(|(described, _)| described == name)
}

/// A definition's documentation as QEMU's parser reads it.
fn comment<'i>(token: &QapiSchemaToken<'i>) -> Option<DocComment<'i>> {
    let doc = token.documentation()?;
    let mut comment = DocComment::default();
    // The file has parsed, so the comment reads without a fault.
    doc_comment::read(doc.text, &mut comment).ok()?;
    Some(comment)
}

fn fail(kind: SchemaErrorKind, position: Option<(usize, usize)>) -> SchemaResult {
    Err(SchemaError::from(kind).at(position))
}

/// The checks of a definition's expression from 4.2: the definition the
/// documentation names and the sections only commands may have, or else,
/// with the `doc-required` pragma, that there is documentation. Errors are
/// located in `content`, the definition's file.
pub(crate) fn check_expression(
    token: &QapiSchemaToken<'_>,
    content: &str,
    pragmas: &Pragmas,
) -> SchemaResult {
    check_required(token, pragmas)?;
    check_symbol(token)?;
    check_sections(token, content)
}

/// From 2.9, with the `doc-required` pragma, a definition must have
/// documentation.
pub(crate) fn check_required(token: &QapiSchemaToken<'_>, pragmas: &Pragmas) -> SchemaResult {
    if crate::parser_version() < (2, 9, 0) || token.definition_name().is_none() {
        return Ok(());
    }
    require(comment(token).is_some() || !pragmas.doc_required, || {
        SchemaErrorKind::DocumentationRequired
    })
}

/// From 2.9, a definition's documentation must name the definition.
pub(crate) fn check_symbol(token: &QapiSchemaToken<'_>) -> SchemaResult {
    let (Some(name), Some(doc)) = (token.definition_name(), token.documentation()) else {
        return Ok(());
    };
    if crate::parser_version() < (2, 9, 0) || comment(token).is_none() {
        return Ok(());
    }
    require(doc.name == name, || SchemaErrorKind::DocumentationSymbol {
        documented: doc.name.to_owned(),
        name: name.to_owned(),
    })
}

/// From 2.9, the sections that only commands may have: Returns, from 9.0
/// only for a command that returns something, and Errors from 9.0.
pub(crate) fn check_sections(token: &QapiSchemaToken<'_>, content: &str) -> SchemaResult {
    let version = crate::parser_version();
    if version < (2, 9, 0) || token.definition_name().is_none() {
        return Ok(());
    }
    let Some(comment) = comment(token) else {
        return Ok(());
    };
    // The reader gives the place where QEMU reports a section: until 8.2
    // the comment's opening line, from 9.0 the section's.
    let at = |section: &str| crate::locate(content, section);
    if let QapiSchemaToken::Command(value) = token {
        return match comment.returns {
            Some(returns) if version >= (9, 0, 0) && value.returns.is_none() => {
                fail(SchemaErrorKind::ReturnsWithoutType, at(returns))
            }
            _ => Ok(()),
        };
    }
    match (comment.returns, comment.errors) {
        (Some(returns), _) => fail(SchemaErrorKind::ReturnsSection, at(returns)),
        // 9.0 fails on it, looking for the Returns section to report.
        (None, Some(errors)) if version < (9, 1, 0) => {
            fail(SchemaErrorKind::ErrorsWithoutReturns, at(errors))
        }
        (None, Some(errors)) => fail(SchemaErrorKind::ErrorsSection, at(errors)),
        (None, None) => Ok(()),
    }
}

/// A member or feature that QEMU connects to a definition's documentation,
/// in its order.
enum Connected<'a> {
    Member { role: &'static str, name: &'a str },
    Feature(&'a str),
}

/// Features, which a release that does not know them has refused as
/// unknown keys.
fn features<'a>(features: Option<&'a QapiFeatures<'_>>, connected: &mut Vec<Connected<'a>>) {
    connected.extend(
        features
            .iter()
            .flat_map(|features| features.iter())
            .map(|feature| Connected::Feature(&feature.name)),
    );
}

fn members<'a>(fields: &'a QapiMembers<'_>, connected: &mut Vec<Connected<'a>>) {
    for field in fields.iter() {
        connected.push(Connected::Member {
            role: "member",
            name: &field.name,
        });
        features(field.features.as_ref(), connected);
    }
}

/// The members and features that QEMU connects to the documentation of
/// `token`, in its order: the definition's features, then its members
/// with theirs. Before 4.2, the members only of the definitions whose
/// documentation QEMU checked: structs, unions, alternates and inline,
/// non-empty arguments.
fn connected<'a>(token: &'a QapiSchemaToken<'_>) -> Option<Vec<Connected<'a>>> {
    let version = crate::parser_version();
    let modern = version >= (4, 2, 0);
    let mut connected = Vec::new();
    let arguments = |data: &'a Option<MembersOrRef<'_>>, connected: &mut Vec<Connected<'a>>| {
        if let Some(MembersOrRef::Members(fields)) = data
            && !fields.is_empty()
        {
            members(fields, connected);
            true
        } else {
            modern
        }
    };
    let checked = match token {
        QapiSchemaToken::Struct(value) => {
            features(value.features.as_ref(), &mut connected);
            members(&value.data, &mut connected);
            true
        }
        QapiSchemaToken::Enum(value) => {
            features(value.features.as_ref(), &mut connected);
            for member in &value.data {
                connected.push(Connected::Member {
                    role: "value",
                    name: &member.name,
                });
                features(member.features.as_ref(), &mut connected);
            }
            modern
        }
        QapiSchemaToken::Alternate(value) => {
            features(value.features.as_ref(), &mut connected);
            connected.extend(value.data.iter().map(|branch| Connected::Member {
                role: "branch",
                name: &branch.name,
            }));
            true
        }
        QapiSchemaToken::Union(value) => {
            features(value.features.as_ref(), &mut connected);
            if let MembersOrRef::Members(fields) = &value.base {
                members(fields, &mut connected);
            }
            // A simple union's tag member (simple unions end with 6.1).
            if value.discriminator.is_none() {
                connected.push(Connected::Member {
                    role: "member",
                    name: "type",
                });
            }
            true
        }
        QapiSchemaToken::Command(value) => {
            features(value.features.as_ref(), &mut connected);
            arguments(&value.data, &mut connected)
        }
        QapiSchemaToken::Event(value) => {
            features(value.features.as_ref(), &mut connected);
            arguments(&value.data, &mut connected)
        }
        _ => false,
    };
    checked.then_some(connected)
}

/// QEMU's connection of a definition's members and features to its
/// documentation, from 4.2: a feature must be described, and from 9.0 a
/// member too, unless the `documentation-exceptions` pragma names the
/// definition. The first undescribed one, in QEMU's order, fails. From
/// 10.0 to 11.0 QEMU adds a section for an exempt member after the member
/// descriptions, and fails when nothing follows them.
pub(crate) fn connect(token: &QapiSchemaToken<'_>, pragmas: &Pragmas) -> SchemaResult {
    let version = crate::parser_version();
    if version < (4, 2, 0) {
        return Ok(());
    }
    let (Some(comment), Some(connected), Some(name)) =
        (comment(token), connected(token), token.definition_name())
    else {
        return Ok(());
    };
    let exempt = pragmas.documentation_exceptions.contains(name);
    let stubs = ((10, 0, 0)..(11, 1, 0)).contains(&version);
    let (mut sections, members) = comment.sections;
    for item in connected {
        match item {
            Connected::Feature(feature) => {
                require(describes(&comment.features, feature), || {
                    SchemaErrorKind::UndocumentedFeature {
                        name: feature.to_owned(),
                    }
                })?;
            }
            Connected::Member { role, name } => {
                if version < (9, 0, 0) || describes(&comment.members, name) {
                    continue;
                }
                require(exempt, || SchemaErrorKind::UndocumentedMember {
                    role,
                    name: name.to_owned(),
                })?;
                if stubs {
                    require(sections == 0 || !members, || {
                        SchemaErrorKind::ExemptMemberAfterMembers {
                            role,
                            name: name.to_owned(),
                        }
                    })?;
                    sections += 1;
                }
            }
        }
    }
    Ok(())
}

/// QEMU 4.2's connection of one feature of a definition to its
/// documentation, as it checks the feature: it must be described.
pub(crate) fn connect_feature(token: &QapiSchemaToken<'_>, feature: &str) -> SchemaResult {
    let Some(comment) = comment(token) else {
        return Ok(());
    };
    require(describes(&comment.features, feature), || {
        SchemaErrorKind::UndocumentedFeature {
            name: feature.to_owned(),
        }
    })
}

/// QEMU's check that a definition's documentation describes only members
/// and features it has: from 2.9 for structs, unions, alternates and
/// inline, non-empty arguments, from 4.2 for every definition and for
/// features too. Every such name is listed. Until 8.2 the error names the
/// comment's opening line, from 9.0 the first such description.
pub(crate) fn check_described<'i>(token: &QapiSchemaToken<'i>, content: &str) -> SchemaResult {
    let version = crate::parser_version();
    if version < (2, 9, 0) {
        return Ok(());
    }
    let (Some(doc), Some(comment), Some(connected)) =
        (token.documentation(), comment(token), connected(token))
    else {
        return Ok(());
    };
    let known = |feature: bool| -> HashSet<&str> {
        connected
            .iter()
            .filter_map(|item| match item {
                Connected::Member { name, .. } if !feature => Some(*name),
                Connected::Feature(name) if feature => Some(*name),
                _ => None,
            })
            .collect()
    };
    let unknown = |described: &[(String, &'i str)], known: HashSet<&str>| {
        let names: Vec<_> = described
            .iter()
            .filter(|(name, _)| !known.contains(name.as_str()))
            .collect();
        let at = if version < (9, 0, 0) {
            Some(doc.text)
        } else {
            names.first().map(|(_, at)| *at)
        };
        (
            names
                .iter()
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>(),
            at.and_then(|at| crate::locate(content, at)),
        )
    };
    let (names, at) = unknown(&comment.members, known(false));
    require(names.is_empty(), || {
        SchemaErrorKind::UnknownDocumentedMember { names }
    })
    .at(|| at)?;
    if version < (4, 2, 0) {
        return Ok(());
    }
    let (names, at) = unknown(&comment.features, known(true));
    // Until 6.1 QEMU words them as members.
    let word = if version < (6, 2, 0) {
        "member"
    } else {
        "feature"
    };
    require(names.is_empty(), || {
        SchemaErrorKind::UnknownDocumentedFeature { names, word }
    })
    .at(|| at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QapiSchemaToken;
    use std::path::PathBuf;

    fn legacy(input: &str) -> Vec<(String, Option<String>)> {
        crate::with_parser_version((2, 3, 0), || {
            let schema = crate::parse_schema(input).unwrap();
            let mut schemas = [(Path::new("legacy.json"), schema)];
            resolve_legacy(&mut schemas);
            let [(_, schema)] = schemas;
            schema
                .0
                .iter()
                .filter_map(|token| {
                    let name = token.definition_name()?.to_owned();
                    let doc = token.documentation().map(|doc| doc.body.join("\n"));
                    Some((name, doc))
                })
                .collect()
        })
    }

    #[test]
    fn legacy_forward_docs_move_only_to_a_definition_without_its_own() {
        let forward = "##\n# @Backend:\n# Forward description.\n##\n{'type':'Helper','data':{}}\n";
        assert_eq!(
            legacy(&format!(
                "{forward}{{'union':'Backend','data':{{'empty':'Helper'}}}}"
            )),
            [
                ("Helper".into(), None),
                ("Backend".into(), Some("Forward description.".into()))
            ]
        );
        // A copied header keeps its text where it is (InputAxis in 2.0).
        let own = "##\n# @Backend:\n# Own description.\n##\n";
        assert_eq!(
            legacy(&format!(
                "{forward}{own}{{'union':'Backend','data':{{'empty':'Helper'}}}}"
            )),
            [
                ("Helper".into(), Some("Forward description.".into())),
                ("Backend".into(), Some("Own description.".into()))
            ]
        );
    }

    #[test]
    fn the_first_legacy_doc_that_names_a_target_moves() {
        let input = concat!(
            "##\n# @Target:\n# First.\n##\n{'type':'One','data':{}}\n",
            "##\n# @Target:\n# Second.\n##\n{'type':'Two','data':{}}\n",
            "{'type':'Target','data':{}}\n",
        );
        assert_eq!(
            legacy(input),
            [
                ("One".into(), None),
                ("Two".into(), Some("Second.".into())),
                ("Target".into(), Some("First.".into()))
            ]
        );
    }

    #[test]
    fn harmless_case_typos_and_unknown_legacy_doc_targets_stay_attached() {
        for name in ["EXAMPLE", "Unknown"] {
            let input =
                format!("##\n# @{name}:\n# Description.\n##\n{{'type':'Example','data':{{}}}}");
            assert_eq!(
                legacy(&input),
                [("Example".into(), Some("Description.".into()))]
            );
        }
    }

    #[test]
    fn formal_docs_are_never_moved() {
        let input = "##\n# @Target:\n# Text.\n##\n{ 'struct': 'Other', 'data': {} }\n{ 'struct': 'Target', 'data': {} }\n";
        let schema = crate::parse_schema_with_version(input, (11, 1, 3)).unwrap();
        let mut schemas = [(Path::new("schema.json"), schema)];
        crate::with_parser_version((11, 1, 3), || resolve_legacy(&mut schemas));
        let token = schemas[0]
            .1
            .0
            .iter()
            .find(|token| token.definition_name() == Some("Other"));
        let error = crate::with_parser_version((11, 1, 3), || {
            check_expression(token.unwrap(), input, &Pragmas::default())
        })
        .unwrap_err();
        assert!(matches!(
            error.kind(),
            SchemaErrorKind::DocumentationSymbol { .. }
        ));
    }

    fn definition<'i>(schema: &'i QapiSchema<'i>, name: &str) -> &'i QapiSchemaToken<'i> {
        schema
            .0
            .iter()
            .find(|token| token.definition_name() == Some(name))
            .unwrap()
    }

    #[test]
    fn the_first_undocumented_feature_in_declaration_order_fails() {
        let input = "##\n# @Example:\n#\n# @m: Member.\n#\n# Features:\n#\n# @b: Documented.\n##\n{ 'struct': 'Example', 'data': { 'm': { 'type': 'int', 'features': [ 'a' ] } }, 'features': [ 'c', 'b' ] }";
        // 4.2 knows no features of members.
        let plain = input.replace("{ 'type': 'int', 'features': [ 'a' ] }", "'int'");
        for (version, input) in [((4, 2, 0), plain.as_str()), ((11, 1, 3), input)] {
            let schema = crate::parse_schema_with_version(input, version).unwrap();
            let error = crate::with_parser_version(version, || {
                connect(definition(&schema, "Example"), &Pragmas::default())
            })
            .unwrap_err();
            assert_eq!(
                error.kind(),
                &SchemaErrorKind::UndocumentedFeature { name: "c".into() }
            );
        }
        // The member's feature comes after the definition's.
        let input = input.replace("[ 'c', 'b' ]", "[ 'b' ]");
        let schema = crate::parse_schema_with_version(&input, (5, 0, 0)).unwrap();
        let error = crate::with_parser_version((5, 0, 0), || {
            connect(definition(&schema, "Example"), &Pragmas::default())
        })
        .unwrap_err();
        assert_eq!(error.to_string(), "feature 'a' lacks documentation");
    }

    /// Validate one schema file as a release's generator does.
    fn validate(version: (u32, u32, u32), input: &str) -> SchemaResult {
        crate::with_parser_version(version, || {
            let sources = [(PathBuf::from("schema.json"), input.to_owned())];
            let schemas = crate::builtins::parse_sources(&sources, &Default::default())?;
            crate::schema_validation::validate(&schemas, &sources)
        })
    }

    /// Each rule in the release that introduced it and the one before, and
    /// where QEMU moved the error, at the line QEMU reports.
    #[test]
    fn definition_documentation_rules_follow_their_release() {
        for (version, input, expected) in [
            (
                (2, 8, 0),
                "##\n# @Foo:\n#\n# @a: A.\n#\n# Returns: x\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n#\n# @a: A.\n#\n# Returns: x\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((1, "ReturnsSection")),
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n#\n# @a: A.\n#\n# Returns: x\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((6, "ReturnsSection")),
            ),
            (
                (8, 2, 0),
                "##\n# @foo:\n#\n# @a: A.\n#\n# Returns: x\n##\n{ 'command': 'foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (9, 0, 0),
                "##\n# @foo:\n#\n# @a: A.\n#\n# Returns: x\n##\n{ 'command': 'foo', 'data': { 'a': 'int' } }\n",
                Some((6, "ReturnsWithoutType")),
            ),
            (
                (8, 2, 0),
                "##\n# @Foo:\n#\n# @a: A.\n#\n# Errors: x\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (9, 1, 0),
                "##\n# @Foo:\n#\n# @a: A.\n#\n# Errors: x\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((6, "ErrorsSection")),
            ),
            (
                (4, 1, 0),
                "##\n# @Foo:\n#\n# @x: X.\n#\n# @y: Y.\n##\n{ 'enum': 'Foo', 'data': [ 'x' ] }\n",
                None,
            ),
            (
                (4, 2, 0),
                "##\n# @Foo:\n#\n# @x: X.\n#\n# @y: Y.\n##\n{ 'enum': 'Foo', 'data': [ 'x' ] }\n",
                Some((1, "UnknownDocumentedMember")),
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n#\n# @x: X.\n#\n# @y: Y.\n##\n{ 'enum': 'Foo', 'data': [ 'x' ] }\n",
                Some((6, "UnknownDocumentedMember")),
            ),
            (
                (2, 8, 0),
                "##\n# @Foo:\n#\n# @a: A.\n# @zz: Z.\n# @yy: Y.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n#\n# @a: A.\n# @zz: Z.\n# @yy: Y.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((1, "UnknownDocumentedMember")),
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n#\n# @a: A.\n# @zz: Z.\n# @yy: Y.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((5, "UnknownDocumentedMember")),
            ),
            (
                (4, 1, 0),
                "##\n# @foo:\n#\n# @a: A.\n##\n{ 'command': 'foo', 'data': 'Foo' }\n##\n# @Foo:\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (4, 2, 0),
                "##\n# @foo:\n#\n# @a: A.\n##\n{ 'command': 'foo', 'data': 'Foo' }\n##\n# @Foo:\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((1, "UnknownDocumentedMember")),
            ),
            (
                (8, 2, 0),
                "##\n# @Foo:\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int', 'b': 'int' } }\n",
                None,
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int', 'b': 'int' } }\n",
                Some((6, "UndocumentedMember")),
            ),
            (
                (9, 0, 0),
                "{ 'pragma': { 'documentation-exceptions': [ 'Foo' ] } }\n##\n# @Foo:\n#\n# @a: A.\n#\n# Since: 1.0\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int', 'b': 'int' } }\n",
                None,
            ),
            (
                (11, 1, 3),
                "{ 'pragma': { 'documentation-exceptions': [ 'Foo' ] } }\n##\n# @Foo:\n#\n# @a: A.\n#\n# Since: 1.0\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int', 'b': 'int' } }\n",
                None,
            ),
            (
                (2, 9, 0),
                "{ 'pragma': { 'doc-required': true } }\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((2, "DocumentationRequired")),
            ),
            (
                (11, 1, 3),
                "{ 'pragma': { 'doc-required': true } }\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((2, "DocumentationRequired")),
            ),
            (
                (2, 9, 0),
                "{ 'pragma': { 'doc-required': true } }\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n{ 'pragma': { 'doc-required': false } }\n",
                None,
            ),
            (
                (11, 1, 3),
                "{ 'pragma': { 'doc-required': true } }\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n{ 'pragma': { 'doc-required': false } }\n",
                None,
            ),
            (
                (4, 1, 0),
                "##\n# @Foo:\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' }, 'features': [ 'f' ] }\n",
                None,
            ),
            (
                (4, 2, 0),
                "##\n# @Foo:\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' }, 'features': [ 'f' ] }\n",
                Some((6, "UndocumentedFeature")),
            ),
            (
                (4, 1, 0),
                "##\n# @Foo:\n#\n# @a: A.\n#\n# Features:\n# @g: G.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (4, 2, 0),
                "##\n# @Foo:\n#\n# @a: A.\n#\n# Features:\n# @g: G.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((1, "UnknownDocumentedFeature")),
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n#\n# @a: A.\n#\n# Features:\n# @g: G.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((7, "UnknownDocumentedFeature")),
            ),
            (
                (2, 8, 0),
                "##\n# @Bar:\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (2, 9, 0),
                "##\n# @Bar:\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((6, "DocumentationSymbol")),
            ),
            (
                (4, 2, 0),
                "{ 'struct': 'Xx', 'data': {} }\n##\n# @Uu:\n#\n# @type: tag\n##\n{ 'union': 'Uu', 'data': { 'x': 'Xx' } }\n",
                None,
            ),
            (
                (6, 1, 0),
                "{ 'struct': 'Xx', 'data': {} }\n##\n# @Uu:\n#\n# @type: tag\n##\n{ 'union': 'Uu', 'data': { 'x': 'Xx' } }\n",
                None,
            ),
            (
                (2, 8, 0),
                "{ 'struct': 'Xx', 'data': {} }\n##\n# @Uu:\n#\n# @x: branch\n##\n{ 'union': 'Uu', 'data': { 'x': 'Xx' } }\n",
                None,
            ),
            (
                (2, 9, 0),
                "{ 'struct': 'Xx', 'data': {} }\n##\n# @Uu:\n#\n# @x: branch\n##\n{ 'union': 'Uu', 'data': { 'x': 'Xx' } }\n",
                Some((2, "UnknownDocumentedMember")),
            ),
            (
                (6, 1, 0),
                "{ 'struct': 'Xx', 'data': {} }\n##\n# @Uu:\n#\n# @x: branch\n##\n{ 'union': 'Uu', 'data': { 'x': 'Xx' } }\n",
                Some((2, "UnknownDocumentedMember")),
            ),
        ] {
            let actual = validate(version, input).err().map(|error| {
                let kind = format!("{:?}", error.kind());
                let kind = kind
                    .split([' ', '{', '('])
                    .next()
                    .unwrap_or_default()
                    .to_owned();
                (error.line().unwrap_or_default(), kind)
            });
            let expected = expected.map(|(line, kind)| (line, kind.to_owned()));
            assert_eq!(actual, expected, "{version:?} {input}");
        }
    }

    #[test]
    fn every_unknown_description_is_listed_in_the_documentation_order() {
        let input = "##
# @Foo:
#
# @zz: Z.
# @a: A.
# @yy: Y.
##
{ 'struct': 'Foo', 'data': { 'a': 'int' } }
";
        let error = validate((11, 1, 3), input).unwrap_err();
        assert_eq!(
            error.kind(),
            &SchemaErrorKind::UnknownDocumentedMember {
                names: vec!["zz".into(), "yy".into()]
            }
        );
        assert_eq!(error.line(), Some(4));
    }

    /// The kind and line of the error that a release reports first.
    fn outcome(version: (u32, u32, u32), input: &str) -> Option<(String, usize)> {
        validate(version, input).err().map(|error| {
            let kind = format!("{:?}", error.kind());
            let kind = kind.split([' ', '{', '(']).next().unwrap_or_default();
            (kind.to_owned(), error.line().unwrap_or_default())
        })
    }

    #[test]
    fn qemu_4_2_checks_a_features_documentation_with_the_feature() {
        let types = "##\n# @Foo:\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'Unknown' }, 'features': [ 'f' ] }\n";
        let clash = "##\n# @Foo:\n#\n# @a: A.\n#\n# @b: B.\n##\n{ 'struct': 'Foo', 'data': { 'a-b': 'int', 'a_b': 'int' }, 'features': [ 'f' ] }\n";
        let returns =
            "##\n# @foo:\n##\n{ 'command': 'foo', 'returns': 'Unknown', 'features': [ 'f' ] }\n";
        // A base, checked with the struct before it, with its own comment.
        let base = "##\n# @Foo:\n#\n# @b: B.\n##\n{ 'struct': 'Foo', 'base': 'Base', 'data': { 'b': 'Unknown' } }\n##\n# @Base:\n#\n# @a: A.\n##\n{ 'struct': 'Base', 'data': { 'a': 'int' }, 'features': [ 'g' ] }\n";
        let undocumented = |line| Some(("UndocumentedFeature".to_owned(), line));
        for (version, input, expected) in [
            ((4, 2, 0), types, undocumented(6)),
            ((5, 0, 0), types, Some(("UnknownType".to_owned(), 6))),
            ((4, 2, 0), clash, undocumented(8)),
            ((5, 0, 0), clash, Some(("MemberClash".to_owned(), 8))),
            ((4, 2, 0), returns, undocumented(4)),
            ((5, 0, 0), returns, Some(("UnknownType".to_owned(), 4))),
            ((4, 2, 0), base, undocumented(12)),
            ((5, 0, 0), base, Some(("UnknownType".to_owned(), 6))),
        ] {
            assert_eq!(outcome(version, input), expected, "{version:?} {input}");
        }
    }

    #[test]
    fn the_crashes_of_qemus_generator_are_refused_where_it_crashes() {
        // 9.0 fails on an Errors section outside a command without a
        // Returns section, which it looks for to report.
        let errors = "##\n# @Foo:\n#\n# @a: A.\n#\n# Errors: x\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n";
        // 10.0 to 11.0 fail to add a section for an exempt member when
        // only member descriptions follow the introduction.
        let exempt = |doc: &str| {
            format!(
                "{{ 'pragma': {{ 'documentation-exceptions': [ 'Ss' ] }} }}\n##\n# @Ss:\n{doc}##\n{{ 'struct': 'Ss', 'data': {{ 'a': 'int', 'b': 'int' }} }}\n"
            )
        };
        let described = exempt("#\n# @a: A.\n");
        let bare = exempt("");
        let since = exempt("#\n# @a: A.\n#\n# Since: 1.0\n");
        let crash = |line| Some(("ExemptMemberAfterMembers".to_owned(), line));
        for (version, input, expected) in [
            ((8, 2, 0), errors, None),
            (
                (9, 0, 0),
                errors,
                Some(("ErrorsWithoutReturns".to_owned(), 6)),
            ),
            ((9, 1, 0), errors, Some(("ErrorsSection".to_owned(), 6))),
            ((9, 2, 0), described.as_str(), None),
            ((10, 0, 0), &described, crash(7)),
            ((11, 0, 5), &described, crash(7)),
            ((11, 1, 0), &described, None),
            // The first member gets a section, the second fails.
            ((9, 2, 0), &bare, None),
            ((10, 0, 0), &bare, crash(5)),
            ((11, 1, 0), &bare, None),
            ((10, 0, 0), &since, None),
        ] {
            assert_eq!(outcome(version, input), expected, "{version:?} {input}");
        }
        let error = validate((10, 0, 0), &bare).unwrap_err();
        assert_eq!(
            error.kind(),
            &SchemaErrorKind::ExemptMemberAfterMembers {
                role: "member",
                name: "b".into()
            }
        );
    }

    #[test]
    fn each_documentation_rule_holds_on_both_sides_of_its_releases() {
        // Before 2.9 a comment moves to the definition it names (see
        // below), from 2.9 the definition after it must be the one it
        // names.
        let forward = "##\n# @Backend:\n#\n# Forward.\n##\n{ 'struct': 'Helper', 'data': {} }\n{ 'union': 'Backend', 'data': { 'empty': 'Helper' } }\n";
        // Until 8.2 QEMU names the comment's opening line, from 9.0 the
        // line of the section or description.
        let returns = "##\n# @Foo:\n#\n# @a: A.\n#\n# Returns: x\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n";
        let member = "##\n# @Foo:\n#\n# @a: A.\n#\n# @zz: Z.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n";
        let feature = "##\n# @Foo:\n#\n# @a: A.\n#\n# Features:\n# @g: G.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n";
        // Until 8.2 each definition's descriptions with it, from 9.0 after
        // the types of every definition.
        let order = "##\n# @Aa:\n#\n# @a: A.\n#\n# @zz: bogus\n##\n{ 'struct': 'Aa', 'data': { 'a': 'int' } }\n##\n# @Bb:\n#\n# @b: B.\n##\n{ 'struct': 'Bb', 'data': { 'b': 'Unknown' } }\n";
        let at = |kind: &str, line| Some((kind.to_owned(), line));
        for (version, input, expected) in [
            ((2, 8, 0), forward, None),
            ((2, 9, 0), forward, at("DocumentationSymbol", 6)),
            ((8, 2, 0), returns, at("ReturnsSection", 1)),
            ((9, 0, 0), returns, at("ReturnsSection", 6)),
            ((8, 2, 0), member, at("UnknownDocumentedMember", 1)),
            ((9, 0, 0), member, at("UnknownDocumentedMember", 6)),
            ((8, 2, 0), feature, at("UnknownDocumentedFeature", 1)),
            ((9, 0, 0), feature, at("UnknownDocumentedFeature", 7)),
            ((8, 2, 0), order, at("UnknownDocumentedMember", 1)),
            ((9, 0, 0), order, at("UnknownType", 14)),
        ] {
            assert_eq!(outcome(version, input), expected, "{version:?} {input}");
        }
        // 2.8 moves the comment to the definition it names.
        crate::with_parser_version((2, 8, 0), || {
            let sources = [(PathBuf::from("schema.json"), forward.to_owned())];
            let schemas = crate::builtins::parse_sources(&sources, &Default::default()).unwrap();
            let documented: Vec<_> = schemas[0]
                .1
                .0
                .iter()
                .filter_map(|token| {
                    Some((token.definition_name()?, token.documentation().is_some()))
                })
                .collect();
            assert_eq!(documented, [("Helper", false), ("Backend", true)]);
        });
        // 4.2 to 6.1 word unknown features as members.
        for (version, message) in [
            ((6, 1, 0), "documented member 'g' does not exist"),
            ((6, 2, 0), "documented feature 'g' does not exist"),
        ] {
            let error = validate(version, feature).unwrap_err();
            assert_eq!(error.kind().to_string(), message, "{version:?}");
        }
    }

    #[test]
    fn legacy_comments_move_when_sources_are_read() {
        let input = "##
# @Backend:
# Forward.
##
{'type':'Helper','data':{}}
{'union':'Backend','data':{'empty':'Helper'}}
";
        crate::with_parser_version((2, 3, 0), || {
            let sources = [(PathBuf::from("schema.json"), input.to_owned())];
            let schemas = crate::builtins::parse_sources(&sources, &Default::default()).unwrap();
            let names: Vec<_> = schemas[0]
                .1
                .0
                .iter()
                .filter_map(|token| {
                    Some((token.definition_name()?, token.documentation().is_some()))
                })
                .collect();
            assert_eq!(names, [("Helper", false), ("Backend", true)]);
        });
    }
}
