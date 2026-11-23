#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod parsers;
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

use nom::{Parser, combinator::all_consuming, error::ErrorKind};
use std::path::{Path, PathBuf};

/// Schema text that the QAPI grammar of the current [`parser_version`] rejects.
///
/// ```
/// use nom::error::ErrorKind;
///
/// let schema = "{ 'enum': 'E', 'data': ['a'] }\n  { 'enum': 'F' }";
/// let error = qapi_codegen::parse_schema_with_version(schema, (11, 1, 3)).unwrap_err();
/// assert_eq!((error.line, error.column), (2, 3));
/// assert_eq!(error.kind, ErrorKind::Eof);
/// assert_eq!(error.near, "{ 'enum': 'F' }");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("QAPI parse error at line {line}, column {column} ({kind:?}): {near:?}")]
#[non_exhaustive]
pub struct ParseError {
    /// The 1-based line of the first input the grammar could not consume.
    pub line: usize,
    /// The 1-based column on that line, counted in characters.
    pub column: usize,
    /// The nom parser that rejected the input: [`ErrorKind::Eof`] when no
    /// definition parses at that position, the kind of a parser that commits
    /// to a definition and then fails (such as [`ErrorKind::Verify`] for a
    /// duplicate key), or [`ErrorKind::Complete`] for truncated input.
    pub kind: ErrorKind,
    /// Up to 160 characters of the input from that position.
    pub near: String,
}

/// Why reading a schema and its includes failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReadError {
    /// A schema file or include could not be resolved to a canonical path.
    #[error("Failed to resolve schema file: {}", path.display())]
    Resolve {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// An include leads back to a file that is still being read.
    #[error("QAPI inclusion loop: {}", path.display())]
    IncludeLoop { path: PathBuf },
    /// A resolved schema file could not be read as UTF-8 text.
    #[error("Failed to read schema file: {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// A schema file does not parse under the grammar it was read with.
    #[error("Failed to parse schema file: {} (version: {version:?})", path.display())]
    Parse {
        path: PathBuf,
        version: (u32, u32, u32),
        #[source]
        source: ParseError,
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
pub fn parse_schema(input: &str) -> Result<QapiSchema<'_>, ParseError> {
    let (_, schema) = all_consuming(take_schema)
        .parse(input)
        .map_err(|error| match error {
            nom::Err::Error(error) | nom::Err::Failure(error) => {
                parse_error(input, error.input, error.code)
            }
            nom::Err::Incomplete(_) => parse_error(input, "", ErrorKind::Complete),
        })?;
    Ok(schema)
}

fn parse_error(input: &str, remainder: &str, kind: ErrorKind) -> ParseError {
    let remainder = remainder.trim_start_matches(parsers::common::is_whitespace);
    let offset = input.len().saturating_sub(remainder.len());
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
    ParseError {
        line,
        column,
        kind,
        near: remainder.chars().take(160).collect(),
    }
}

/// Read a schema file and the files it includes, parsing each to find the includes.
///
/// `Some(version)` reads under that grammar and restores the caller's afterwards,
/// as [`with_parser_version`] does; `None` reads under the current [`parser_version`].
/// No preprocessing is applied — the parser handles all format variations directly.
pub fn read_schema_with_version(
    schema_file: &Path,
    version: Option<(u32, u32, u32)>,
) -> Result<Vec<(PathBuf, String)>, ReadError> {
    let read = || {
        let mut sources = Vec::new();
        read_schema_file(schema_file, &mut sources, &mut Vec::new())?;
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
    schema_file: &Path,
    sources: &mut Vec<(PathBuf, String)>,
    active: &mut Vec<PathBuf>,
) -> Result<(), ReadError> {
    let schema_file = schema_file
        .canonicalize()
        .map_err(|source| ReadError::Resolve {
            path: schema_file.to_path_buf(),
            source,
        })?;
    if active.contains(&schema_file) {
        return Err(ReadError::IncludeLoop { path: schema_file });
    }
    if sources.iter().any(|(path, _)| path == &schema_file) {
        return Ok(());
    }
    active.push(schema_file.clone());

    let content = std::fs::read_to_string(&schema_file).map_err(|source| ReadError::Read {
        path: schema_file.clone(),
        source,
    })?;
    sources.push((schema_file.to_path_buf(), content.clone()));
    let schema_tokens = parse_schema(&content).map_err(|source| ReadError::Parse {
        path: schema_file.clone(),
        version: parser_version(),
        source,
    })?;
    let includes: Vec<_> = schema_tokens
        .into_iter()
        .filter_map(|token| match token {
            QapiSchemaToken::Include(filename) => Some(filename.0),
            _ => None,
        })
        .collect();
    for include in includes {
        let include_path = schema_file.parent().unwrap().join(include.as_ref());
        read_schema_file(&include_path, sources, active)?;
    }
    active.pop();
    Ok(())
}
