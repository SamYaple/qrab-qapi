#![doc = include_str!("../README.md")]
//!
//! ## Generated code
//!
//! Code that this crate generates is an output: fix generation bugs here,
//! never in the generated files, and regenerate them. The repository's
//! `docs/codegen.md` describes how to run the generator, the pipeline and its
//! correctness boundaries.
//!
//! ## API
//!
//! The API is the generator, [`unified::generate`] with the release pins of
//! [`versions`] and the errors they report; items hidden from the
//! documentation are internal to the generator and its tests and not part
//! of the API.
#![forbid(unsafe_code)]

mod arrays;
mod builtins;
mod metadata_emit;
#[doc(hidden)]
pub mod namespace;
#[doc(hidden)]
pub mod parsers;
#[doc(hidden)]
pub mod qapi_ir;
mod schema_docs;
mod schema_names;
mod schema_shapes;
mod schema_validation;
mod symbols;
pub mod unified;
mod union_layout;
pub mod versions;

use nom::{Finish, Parser, combinator::all_consuming, error::ErrorKind};
pub use parsers::doc_comment::DocumentationError;
pub(crate) use parsers::{
    common::QapiString,
    qapi_alternate::QapiAlternate,
    qapi_alternative::QapiAlternative,
    qapi_alternatives::QapiAlternatives,
    qapi_command::QapiCommand,
    qapi_cond::QapiCond,
    qapi_documentation::extract_since_from_comment,
    qapi_enum::QapiEnum,
    qapi_enum_value::QapiEnumValue,
    qapi_event::QapiEvent,
    qapi_feature::QapiFeature,
    qapi_features::QapiFeatures,
    qapi_include::QapiInclude,
    qapi_member::QapiMember,
    qapi_members::{MembersOrRef, QapiMembers},
    qapi_pragma::QapiPragma,
    qapi_schema::{QapiSchema, take_schema},
    qapi_struct::QapiStruct,
    qapi_type_ref::QapiTypeRef,
    qapi_union::QapiUnion,
    qapi_variant::QapiVariant,
};
#[doc(hidden)]
pub use parsers::{
    parser_version, qapi_schema::QapiSchemaToken, set_parser_version,
    with_parser_version,
};
use std::path::{Path, PathBuf};

/// Schema text that the QAPI grammar of its release rejects, in
/// [`ReadError::Parse`] and [`SchemaErrorKind::Parse`].
///
/// [`SchemaErrorKind::Parse`]: crate::unified::SchemaErrorKind::Parse
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}", describe(self))]
#[non_exhaustive]
pub struct ParseError {
    /// The 1-based line of the first input the grammar could not consume.
    pub line: usize,
    /// The 1-based column on that line, counted in characters.
    pub column: usize,
    /// What is wrong there.
    pub kind: ParseErrorKind,
    /// Up to 160 characters of the input from that position, which the
    /// message quotes unless they are only space.
    pub near: String,
}

/// What is wrong with schema text, in [`ParseError::kind`].
///
/// An expression that QEMU's parser finds malformed is reported where that
/// parser stops: [`UnexpectedCharacter`], [`UnexpectedEnd`],
/// [`MalformedString`] or, from 2.0, [`DuplicateKey`]. Before 1.6 that
/// parser reports no position, so such an expression is [`NoDefinition`]
/// at its start, unless the release's parser fails on it ([`Crash`]).
///
/// [`UnexpectedCharacter`]: ParseErrorKind::UnexpectedCharacter
/// [`UnexpectedEnd`]: ParseErrorKind::UnexpectedEnd
/// [`MalformedString`]: ParseErrorKind::MalformedString
/// [`DuplicateKey`]: ParseErrorKind::DuplicateKey
/// [`NoDefinition`]: ParseErrorKind::NoDefinition
/// [`Crash`]: ParseErrorKind::Crash
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ParseErrorKind {
    /// An expression that no definition or directive of the release's
    /// grammar reads, such as a `pragma` whose value has the wrong shape.
    #[error("no definition or directive parses here")]
    NoDefinition,
    /// A character that QEMU's parser does not expect: one outside strings
    /// that is no token, or a token out of its place in the expression.
    /// Before 2.4, `true` and `false` are such characters.
    #[error("unexpected character")]
    UnexpectedCharacter,
    /// Input that ends inside an expression.
    #[error("unexpected end of input")]
    UnexpectedEnd,
    /// A string that QEMU's lexer refuses: one without its closing quote,
    /// or with an escape or a character that the release does not allow.
    #[error("malformed string")]
    MalformedString,
    /// From QEMU 2.0, a key that its object has already, at the key's value
    /// as QEMU reports it.
    #[error("duplicate key")]
    DuplicateKey,
    /// From QEMU 2.9, a documentation comment that breaks a rule of the
    /// release's parser.
    #[error("{0}")]
    Documentation(Box<DocumentationError>),
    /// Before 1.6, input on which the release's own parser fails instead of
    /// reporting an error.
    #[error("QEMU's QAPI parser fails on {0}")]
    Crash(ParserCrash),
}

/// What a release's QAPI parser fails on, in [`ParseErrorKind::Crash`].
///
/// The parsers before 1.6 skip only the lines that start with `#` and the
/// empty ones, and read the rest as tokens of punctuation and strings.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ParserCrash {
    /// Before 1.2, a character outside strings that is neither punctuation,
    /// a space nor a newline: a letter, a digit, a tab, or the `#` of a
    /// comment that does not start its line. Those tokenizers never finish
    /// there; from 1.2 they drop such characters.
    #[error("a character outside strings that it never reads past")]
    StrayCharacter,
    /// Before 1.6, a line of an expression that starts at its first
    /// column, which those parsers take as the start of the next
    /// expression, failing on the incomplete one before it.
    #[error("a line of an expression at its first column")]
    UnindentedLine,
    /// Before 1.6, an expression whose tokens end before it does, where
    /// those parsers run out of tokens: from 1.2 a value that the tokenizer
    /// drops, such as `true`, punctuation in a comment of the expression,
    /// or an expression of only an indented comment.
    #[error("an expression whose tokens end before it does")]
    IncompleteExpression,
    /// Before 1.6, an expression that is not an object.
    #[error("an expression that is not an object")]
    NotAnObject,
    /// Before 1.6, a string without its closing quote.
    #[error("a string without its closing quote")]
    UnterminatedString,
}

fn describe(error: &ParseError) -> String {
    let (line, column) = (error.line, error.column);
    match &error.kind {
        ParseErrorKind::Crash(crash) => {
            format!(
                "QEMU's QAPI parser fails at line {line}, column {column} on {crash}"
            )
        }
        ParseErrorKind::Documentation(rule) => {
            format!(
                "QAPI documentation error at line {line}, column {column}: {rule}"
            )
        }
        kind => format!(
            "QAPI parse error at line {line}, column {column}: {kind}{}",
            quoted_near(error)
        ),
    }
}

/// The input near a syntax error, quoted after a colon; nothing when only
/// space is left, as at the end of the input.
fn quoted_near(error: &ParseError) -> String {
    if error.near.chars().all(parsers::common::is_whitespace) {
        String::new()
    } else {
        format!(": {:?}", error.near)
    }
}

impl ParseError {
    /// The fault without its position, for a message that gives it already.
    pub(crate) fn fault(&self) -> String {
        match &self.kind {
            ParseErrorKind::Crash(_) => self.kind.to_string(),
            ParseErrorKind::Documentation(rule) => {
                format!("QAPI documentation error: {rule}")
            }
            kind => format!("QAPI parse error: {kind}{}", quoted_near(self)),
        }
    }
}

/// Why reading a schema and its includes failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReadError {
    /// The schema file could not be resolved to a canonical path.
    #[error("Failed to resolve schema file: {}", path.display())]
    Resolve {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// An include directive that QEMU refuses, located at the directive.
    #[error("{}:{line}:{column}: {kind}", from.display())]
    Include {
        /// The file with the directive.
        from: PathBuf,
        /// The directive's line.
        line: usize,
        /// The directive's column.
        column: usize,
        /// What is wrong with it.
        #[source]
        kind: IncludeError,
    },
    /// The schema file could not be read, or it or a file it includes is
    /// not UTF-8 text.
    #[error("Failed to read schema file: {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// A schema file does not parse under the grammar it was read with.
    #[error(
        "Failed to parse schema file {} with the grammar of QEMU {}.{}.{}",
        path.display(),
        version.0,
        version.1,
        version.2
    )]
    Parse {
        path: PathBuf,
        version: (u32, u32, u32),
        #[source]
        source: ParseError,
    },
}

/// An include directive that QEMU refuses.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IncludeError {
    /// The directive names a file that is still being read: the file
    /// itself or one that includes it.
    #[error("inclusion loop for {}", path.display())]
    Loop {
        /// The file named.
        path: PathBuf,
    },
    /// The file named cannot be read.
    #[error("can't read include file '{name}'")]
    Unreadable {
        /// The name the directive gives.
        name: String,
        /// Why it cannot be read.
        #[source]
        source: std::io::Error,
    },
}

/// Parse one file with an explicit grammar, restoring the caller's parser
/// version.
#[doc(hidden)]
pub fn parse_schema_with_version(
    input: &str,
    version: (u32, u32, u32),
) -> Result<QapiSchema<'_>, ParseError> {
    with_parser_version(version, || parse_schema(input))
}

/// Parse one schema file using the current [`parser_version`].
///
/// Documentation comments must follow the rules of the release's own
/// parser, which reports the first fault in the file, in an expression or
/// in documentation.
#[doc(hidden)]
pub fn parse_schema(input: &str) -> Result<QapiSchema<'_>, ParseError> {
    if let Some((at, crash)) = parsers::common::unreadable_line(input) {
        return Err(parse_error(input, at, ParseErrorKind::Crash(crash)));
    }
    // The grammar's parsers are complete: they never ask for more input.
    let parsed = all_consuming(take_schema).parse(input).finish();
    let documentation = parsers::doc_comment::check(input);
    let offset = |at: &str| input.len() - at.len();
    let failure = match parsed {
        Ok((_, schema)) => match documentation {
            None => return Ok(schema),
            Some(fault) => {
                return Err(parse_error(input, fault.at, fault.kind));
            }
        },
        Err(error) => {
            let at =
                error.input.trim_start_matches(parsers::common::is_whitespace);
            match error.code {
                // No definition parses here: report where QEMU's parser
                // stops, if it finds the expression malformed.
                ErrorKind::Eof => parsers::common::syntax_error(at),
                // A definition's grammar commits to it, refusing a key
                // that it has already, or before 2.4 a boolean value.
                // QEMU reports the duplicate at the token after its colon.
                ErrorKind::Verify => Some((
                    parsers::common::next_token(error.input),
                    ParseErrorKind::DuplicateKey,
                )),
                ErrorKind::Char => {
                    Some((at, ParseErrorKind::UnexpectedCharacter))
                }
                _ => None,
            }
            .unwrap_or((at, ParseErrorKind::NoDefinition))
        }
    };
    // QEMU reads a comment before the expressions after it.
    Err(match documentation {
        Some(fault) if offset(fault.comment) <= offset(failure.0) => {
            parse_error(input, fault.at, fault.kind)
        }
        _ => parse_error(input, failure.0, failure.1),
    })
}

fn parse_error(
    input: &str,
    remainder: &str,
    kind: ParseErrorKind,
) -> ParseError {
    let (line, column) =
        position(input, input.len().saturating_sub(remainder.len()));
    ParseError {
        line,
        column,
        kind,
        near: remainder.chars().take(160).collect(),
    }
}

/// The line and column at which `at`, a slice of `source`, starts; `None`
/// for text from elsewhere, such as a predefined definition.
pub(crate) fn locate(source: &str, at: &str) -> Option<(usize, usize)> {
    let offset = (at.as_ptr() as usize).checked_sub(source.as_ptr() as usize)?;
    (offset + at.len() <= source.len()).then(|| position(source, offset))
}

/// The 1-based line and column, in characters, at byte `offset` of `input`.
pub(crate) fn position(input: &str, offset: usize) -> (usize, usize) {
    let (mut line, mut column) = (1, 1);
    let mut chars = input[..offset].chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' || c == '\n' {
            if c == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

/// Read a schema file and the files it includes, parsing each to find the
/// includes.
///
/// `Some(version)` reads under that grammar and restores the caller's
/// afterwards, as [`with_parser_version`] does; `None` reads under the current
/// [`parser_version`]. The parser handles the format of each release directly,
/// except that from 1.2 to 1.5 an expression with characters outside strings
/// that QEMU's tokenizer drops, such as `true`, is read as that tokenizer reads
/// it.
#[doc(hidden)]
pub fn read_schema_with_version(
    schema_file: &Path,
    version: Option<(u32, u32, u32)>,
) -> Result<Vec<(PathBuf, String)>, ReadError> {
    let read = || {
        let path = schema_file.canonicalize().map_err(|source| {
            ReadError::Resolve { path: schema_file.to_path_buf(), source }
        })?;
        let content = std::fs::read_to_string(&path)
            .map_err(|source| ReadError::Read { path: path.clone(), source })?;
        let mut sources = Vec::new();
        read_schema_file(path, content, &mut sources, &mut Vec::new())?;
        if sources.len() > 1 {
            sources.rotate_left(1);
        }
        Ok(sources)
    };
    match version {
        Some(version) => with_parser_version(version, read),
        None => read(),
    }
}

fn read_schema_file(
    schema_file: PathBuf,
    content: String,
    sources: &mut Vec<(PathBuf, String)>,
    active: &mut Vec<PathBuf>,
) -> Result<(), ReadError> {
    // From 1.2 to 1.5, an expression as QEMU's parser reads it.
    let content = parsers::common::legacy_text(&content).unwrap_or(content);
    active.push(schema_file.clone());
    sources.push((schema_file.clone(), content.clone()));
    let schema_tokens =
        parse_schema(&content).map_err(|source| ReadError::Parse {
            path: schema_file.clone(),
            version: parser_version(),
            source,
        })?;
    for token in schema_tokens.0 {
        let QapiSchemaToken::Include(QapiInclude(name, text)) = token else {
            continue;
        };
        // QEMU names the directive.
        let (line, column) = locate(&content, text).unwrap_or((1, 1));
        let fail = |kind| ReadError::Include {
            from: schema_file.clone(),
            line,
            column,
            kind,
        };
        let unreadable = |source| {
            fail(IncludeError::Unreadable { name: name.to_string(), source })
        };
        let include_path = schema_file.parent().unwrap().join(name.as_ref());
        let target = include_path.canonicalize().map_err(unreadable)?;
        if active.contains(&target) {
            return Err(fail(IncludeError::Loop { path: target }));
        }
        if sources.iter().any(|(path, _)| path == &target) {
            continue;
        }
        // A file that is not UTF-8 text is read, but not as a schema.
        let included = std::fs::read_to_string(&target).map_err(|source| {
            if source.kind() == std::io::ErrorKind::InvalidData {
                ReadError::Read { path: target.clone(), source }
            } else {
                unreadable(source)
            }
        })?;
        read_schema_file(target, included, sources, active)?;
    }
    active.pop();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ParseErrorKind, ParserCrash, parse_schema_with_version};

    /// A parse error gives the line and column where the grammar stops, and
    /// the input from there, which its message quotes.
    #[test]
    fn a_parse_error_locates_its_fault_and_quotes_the_input_there() {
        let schema =
            "{ 'enum': 'E', 'data': ['a'] }\n  { 'enum': 'F' 'data': [] }";
        let error = parse_schema_with_version(schema, (11, 1, 3)).unwrap_err();
        assert_eq!((error.line, error.column), (2, 17));
        assert_eq!(error.kind, ParseErrorKind::UnexpectedCharacter);
        assert_eq!(error.near, "'data': [] }");
        assert_eq!(
            error.to_string(),
            r#"QAPI parse error at line 2, column 17: unexpected character: "'data': [] }""#
        );
    }

    /// Each kind of parse error where it arises, with its message.
    #[test]
    fn each_kind_of_parse_error_names_its_fault_in_words() {
        let documented = "##\n# @Example:\n#\n# @member:\n##\n{ 'struct': 'Example', 'data': { 'member': 'int' } }\n";
        for (version, input, kind, message) in [
            (
                (11, 1, 3),
                "{ 'pragma': { 'doc-required': 'x' } }",
                ParseErrorKind::NoDefinition,
                r#"QAPI parse error at line 1, column 1: no definition or directive parses here: "{ 'pragma': { 'doc-required': 'x' } }""#,
            ),
            (
                (11, 1, 3),
                "{ 'enum': 'E' 'data': [] }",
                ParseErrorKind::UnexpectedCharacter,
                r#"QAPI parse error at line 1, column 15: unexpected character: "'data': [] }""#,
            ),
            (
                (2, 3, 0),
                "{ 'command': 'c', 'gen': false }",
                ParseErrorKind::UnexpectedCharacter,
                r#"QAPI parse error at line 1, column 26: unexpected character: "false }""#,
            ),
            (
                (11, 1, 3),
                "{ 'enum': 'E',",
                ParseErrorKind::UnexpectedEnd,
                "QAPI parse error at line 1, column 15: unexpected end of input",
            ),
            (
                (11, 1, 3),
                r"{ 'enum': 'E\x', 'data': [] }",
                ParseErrorKind::MalformedString,
                r#"QAPI parse error at line 1, column 11: malformed string: "'E\\x', 'data': [] }""#,
            ),
            (
                (11, 1, 3),
                "{ 'enum': 'E', 'enum': 'F', 'data': [] }",
                ParseErrorKind::DuplicateKey,
                r#"QAPI parse error at line 1, column 24: duplicate key: "'F', 'data': [] }""#,
            ),
            (
                (11, 1, 3),
                documented,
                ParseErrorKind::Documentation(Box::new(
                    crate::DocumentationError::EmptySection {
                        section: "Member".into(),
                    },
                )),
                "QAPI documentation error at line 4, column 1: text required after 'Member:'",
            ),
            (
                (1, 1, 0),
                "{ 'type': 'Foo', 'data': {} } # plain words",
                ParseErrorKind::Crash(ParserCrash::StrayCharacter),
                "QEMU's QAPI parser fails at line 1, column 31 on a character outside strings that it never reads past",
            ),
        ] {
            let error = parse_schema_with_version(input, version).unwrap_err();
            assert_eq!(error.kind, kind, "{version:?} {input}");
            assert_eq!(error.to_string(), message, "{version:?} {input}");
        }
    }

    /// A key without its colon at the end of the input, a duplicate or not,
    /// in an object of a definition or one nested in it, and a duplicate
    /// key, where QEMU's parser reports them: a key's colon is expected at
    /// the end, after space and comments.
    #[test]
    fn keys_without_colons_and_duplicate_keys_are_reported_where_qemu_does() {
        for (input, (line, column)) in [
            ("{ 'enum': 'E', 'enum'", (1, 22)),
            ("{ 'enum': 'E', 'enum' ", (1, 23)),
            ("{ 'enum': 'E', 'enum'\n", (1, 22)),
            ("{ 'enum': 'E', 'enum'\r\n", (1, 22)),
            ("{ 'enum': 'E', 'enum' \n\n", (2, 1)),
            ("{ 'enum': 'E', 'enum' # comment", (1, 32)),
            ("{ 'enum': 'E', 'enum' # comment\n", (1, 32)),
            ("{ 'enum': 'E', 'b'", (1, 19)),
            ("{ 'enum': 'E', 'b' # comment\n", (1, 29)),
            ("{ 'struct': 'S', 'data': { 'a': 'int', 'a'", (1, 43)),
            ("{ 'struct': 'S', 'data': { 'a': 'int', 'b'", (1, 43)),
            ("{ 'command': 'c', 'data': { 'a': 'int' }, 'data'", (1, 49)),
            ("{ 'union': 'U', 'base': { 'a': 'int', 'a'", (1, 42)),
            ("{ 'alternate': 'A', 'data': { 'a': 'int', 'a'", (1, 46)),
            ("{ 'enum': 'E', 'data': [ { 'name': 'a', 'name'", (1, 47)),
        ] {
            let error =
                parse_schema_with_version(input, (11, 1, 3)).unwrap_err();
            assert_eq!(error.kind, ParseErrorKind::UnexpectedEnd, "{input:?}");
            assert_eq!(
                error.to_string(),
                format!(
                    "QAPI parse error at line {line}, column {column}: unexpected end of input"
                ),
                "{input:?}"
            );
        }
        // Before the colon of a duplicate key, a token is out of its place.
        let error = parse_schema_with_version(
            "{ 'enum': 'E', 'enum' 'x' }",
            (11, 1, 3),
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            r#"QAPI parse error at line 1, column 23: unexpected character: "'x' }""#
        );
        // A duplicate key is reported at the token after its colon, as for
        // QEMU: past space and comments, or at the end of the input.
        for (input, (line, column), near) in [
            ("{ 'enum': 'E', 'enum':\n", (1, 23), String::new()),
            ("{ 'enum': 'E', 'enum': # c\n", (1, 27), String::new()),
            ("{ 'enum': 'E', 'enum':  \n", (1, 25), String::new()),
            (
                "{ 'enum': 'E', 'enum': # c\n  'F' }",
                (2, 3),
                r#": "'F' }""#.to_owned(),
            ),
            (
                "{ 'enum': 'E', 'enum':\n##\n'F' }",
                (2, 1),
                r###": "##\n'F' }""###.to_owned(),
            ),
        ] {
            let error =
                parse_schema_with_version(input, (11, 1, 3)).unwrap_err();
            assert_eq!(
                (error.to_string(), error.kind),
                (
                    format!(
                        "QAPI parse error at line {line}, column {column}: duplicate key{near}"
                    ),
                    ParseErrorKind::DuplicateKey,
                ),
                "{input:?}"
            );
        }
    }
}
