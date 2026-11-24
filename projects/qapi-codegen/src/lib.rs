#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod arrays;
mod builtins;
pub mod parsers;
pub mod qapi_ir;
mod schema_docs;
mod schema_names;
mod schema_shapes;
mod schema_validation;
mod symbols;
pub mod unified;
mod union_layout;
pub mod versions;

pub use parsers::{
    parser_version,
    qapi_alternate::{QapiAlternate, take_alternate},
    qapi_alternative::{QapiAlternative, take_alternative},
    qapi_alternatives::{QapiAlternatives, take_alternatives},
    qapi_branch::{QapiBranch, take_branch},
    qapi_branches::{QapiBranches, take_branches},
    qapi_command::{QapiCommand, take_command},
    qapi_cond::{QapiCond, take_cond},
    qapi_documentation::{
        QapiDocumentation, QapiFreeformDoc, QapiSectionDocumentation, extract_since_from_comment,
    },
    qapi_enum::{QapiEnum, take_enum},
    qapi_enum_value::{QapiEnumValue, take_enum_value},
    qapi_event::{QapiEvent, take_event},
    qapi_feature::{QapiFeature, take_feature},
    qapi_features::{QapiFeatures, take_features},
    qapi_include::{QapiInclude, take_include},
    qapi_member::{QapiMember, take_member},
    qapi_members::{MembersOrRef, QapiMembers, take_members, take_members_or_ref},
    qapi_pragma::{QapiPragma, take_pragma},
    qapi_schema::{QapiSchema, QapiSchemaToken, take_schema},
    qapi_struct::{QapiStruct, take_struct},
    qapi_type_ref::{QapiTypeRef, take_type_ref},
    qapi_union::{QapiUnion, take_union},
    qapi_variant::{QapiVariant, take_variant},
    set_parser_version, with_parser_version,
};

pub use parsers::common::QapiString;
pub use parsers::doc_comment::DocumentationError;

use nom::{Parser, combinator::all_consuming, error::ErrorKind};
use std::path::{Path, PathBuf};

/// Schema text that the QAPI grammar of the current [`parser_version`] rejects.
///
/// ```
/// use nom::error::ErrorKind;
///
/// let schema = "{ 'enum': 'E', 'data': ['a'] }\n  { 'pragma': { 'doc-required': 'x' } }";
/// let error = qapi_codegen::parse_schema_with_version(schema, (11, 1, 3)).unwrap_err();
/// assert_eq!((error.line, error.column), (2, 3));
/// assert_eq!(error.kind, ErrorKind::Eof);
/// assert_eq!(error.near, "{ 'pragma': { 'doc-required': 'x' } }");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}", describe(self))]
#[non_exhaustive]
pub struct ParseError {
    /// The 1-based line of the first input the grammar could not consume.
    pub line: usize,
    /// The 1-based column on that line, counted in characters.
    pub column: usize,
    /// The nom parser that rejected the input: [`ErrorKind::Eof`] when no
    /// definition parses at that position, the kind of a parser that commits
    /// to a definition and then fails (such as [`ErrorKind::Verify`] for a
    /// duplicate key), or [`ErrorKind::Complete`] for truncated input. An
    /// expression that QEMU's parser finds malformed is reported where that
    /// parser stops, as [`ErrorKind::Escaped`] for a bad string,
    /// [`ErrorKind::Verify`] for a duplicate key and [`ErrorKind::Char`] for
    /// any other unexpected character.
    pub kind: ErrorKind,
    /// Up to 160 characters of the input from that position.
    pub near: String,
    /// The rule of documentation comments that the input breaks, from
    /// QEMU 2.9; `kind` is then [`ErrorKind::Verify`].
    pub documentation: Option<Box<DocumentationError>>,
    /// Input on which the release's own parser fails instead of reporting
    /// an error; `kind` is then [`ErrorKind::Fail`].
    pub crash: Option<ParserCrash>,
}

/// What a release's QAPI parser fails on, in [`ParseError::crash`].
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
    if let Some(crash) = &error.crash {
        return format!("QEMU's QAPI parser fails at line {line}, column {column} on {crash}");
    }
    match &error.documentation {
        Some(rule) => format!("QAPI documentation error at line {line}, column {column}: {rule}"),
        None => format!(
            "QAPI parse error at line {line}, column {column} ({:?}): {:?}",
            error.kind, error.near
        ),
    }
}

impl ParseError {
    /// The fault without its position, for a message that gives it already.
    pub(crate) fn fault(&self) -> String {
        if let Some(crash) = &self.crash {
            return format!("QEMU's QAPI parser fails on {crash}");
        }
        match &self.documentation {
            Some(rule) => format!("QAPI documentation error: {rule}"),
            None => format!("QAPI parse error ({:?}): {:?}", self.kind, self.near),
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

/// Parse one file with an explicit grammar, restoring the caller's parser version.
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
pub fn parse_schema(input: &str) -> Result<QapiSchema<'_>, ParseError> {
    if let Some((at, crash)) = parsers::common::unreadable_line(input) {
        return Err(ParseError {
            crash: Some(crash),
            ..parse_error(input, at, ErrorKind::Fail)
        });
    }
    let parsed = all_consuming(take_schema).parse(input);
    let documentation = parsers::doc_comment::check(input);
    let offset = |at: &str| input.len() - at.len();
    let failure = match parsed {
        Ok((_, schema)) => match documentation {
            None => return Ok(schema),
            Some(fault) => return Err(documentation_error(input, fault)),
        },
        Err(nom::Err::Error(error) | nom::Err::Failure(error)) => {
            let at = error
                .input
                .trim_start_matches(parsers::common::is_whitespace);
            // No definition parses here: report where QEMU's parser
            // stops, if it finds the expression malformed.
            match error.code {
                ErrorKind::Eof => parsers::common::syntax_error(at),
                _ => None,
            }
            .unwrap_or((at, error.code))
        }
        Err(nom::Err::Incomplete(_)) => (&input[input.len()..], ErrorKind::Complete),
    };
    // QEMU reads a comment before the expressions after it.
    Err(match documentation {
        Some(fault) if offset(fault.comment) <= offset(failure.0) => {
            documentation_error(input, fault)
        }
        _ => parse_error(input, failure.0, failure.1),
    })
}

fn documentation_error(input: &str, fault: parsers::doc_comment::Fault<'_>) -> ParseError {
    ParseError {
        documentation: fault.error.map(Box::new),
        ..parse_error(input, fault.at, fault.kind)
    }
}

fn parse_error(input: &str, remainder: &str, kind: ErrorKind) -> ParseError {
    let (line, column) = position(input, input.len().saturating_sub(remainder.len()));
    ParseError {
        line,
        column,
        kind,
        near: remainder.chars().take(160).collect(),
        documentation: None,
        crash: None,
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

/// Read a schema file and the files it includes, parsing each to find the includes.
///
/// `Some(version)` reads under that grammar and restores the caller's afterwards,
/// as [`with_parser_version`] does; `None` reads under the current [`parser_version`].
/// The parser handles the format of each release directly, except that from
/// 1.2 to 1.5 an expression with characters outside strings that QEMU's
/// tokenizer drops, such as `true`, is read as that tokenizer reads it.
pub fn read_schema_with_version(
    schema_file: &Path,
    version: Option<(u32, u32, u32)>,
) -> Result<Vec<(PathBuf, String)>, ReadError> {
    let read = || {
        let path = schema_file
            .canonicalize()
            .map_err(|source| ReadError::Resolve {
                path: schema_file.to_path_buf(),
                source,
            })?;
        let content = std::fs::read_to_string(&path).map_err(|source| ReadError::Read {
            path: path.clone(),
            source,
        })?;
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
    let schema_tokens = parse_schema(&content).map_err(|source| ReadError::Parse {
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
            fail(IncludeError::Unreadable {
                name: name.to_string(),
                source,
            })
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
                ReadError::Read {
                    path: target.clone(),
                    source,
                }
            } else {
                unreadable(source)
            }
        })?;
        read_schema_file(target, included, sources, active)?;
    }
    active.pop();
    Ok(())
}
