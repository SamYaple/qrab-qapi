use super::{
    common::{is_whitespace, line_ending, lines, qcomment, whitespace1},
    qapi_alternate::QapiAlternate,
    qapi_command::QapiCommand,
    qapi_documentation::{
        QapiDocumentation, QapiFreeformDoc, QapiSectionDocumentation,
    },
    qapi_enum::QapiEnum,
    qapi_event::QapiEvent,
    qapi_include::QapiInclude,
    qapi_pragma::QapiPragma,
    qapi_struct::QapiStruct,
    qapi_union::QapiUnion,
    vec_newtype,
};
use nom::{
    IResult, Parser, branch::alt, bytes::complete::tag,
    character::complete::space0, combinator::recognize, multi::many0,
    sequence::terminated,
};

pub(crate) fn take_schema(input: &str) -> IResult<&str, QapiSchema<'_>> {
    QapiSchema::parse(input)
}

#[derive(Debug, Clone)]
pub enum QapiSchemaToken<'i> {
    Alternate(QapiAlternate<'i>),
    Command(QapiCommand<'i>),
    Enum(QapiEnum<'i>),
    Event(QapiEvent<'i>),
    Struct(QapiStruct<'i>),
    Union(QapiUnion<'i>),
    Include(QapiInclude<'i>),
    Pragma(QapiPragma<'i>),
    Doc(QapiSectionDocumentation<'i>),
    FreeformDoc(QapiFreeformDoc<'i>),
    Comment(&'i str),
    EmptyLines,
    /// Before 2.4, an expression with a metatype that the release's
    /// generators read loosely, ignoring keys and values they do not use,
    /// but that its grammar does not describe.
    Unsupported(&'i str),
    /// From 2.4, an expression without a key that names a kind of
    /// definition or directive, which the validation refuses.
    NoMetatype(&'i str),
}

impl<'i> QapiSchemaToken<'i> {
    pub(crate) fn definition_name(&self) -> Option<&str> {
        match self {
            Self::Alternate(value) => Some(value.name.as_ref()),
            Self::Command(value) => Some(value.name.as_ref()),
            Self::Enum(value) => Some(value.name.as_ref()),
            Self::Event(value) => Some(value.name.as_ref()),
            Self::Struct(value) => Some(value.name.as_ref()),
            Self::Union(value) => Some(value.name.as_ref()),
            _ => None,
        }
    }

    /// The definition's expression in its schema file.
    pub(crate) fn definition_text(&self) -> Option<&'i str> {
        match self {
            Self::Alternate(value) => Some(value.text),
            Self::Command(value) => Some(value.text),
            Self::Enum(value) => Some(value.text),
            Self::Event(value) => Some(value.text),
            Self::Struct(value) => Some(value.text),
            Self::Union(value) => Some(value.text),
            Self::NoMetatype(text) => Some(text),
            _ => None,
        }
    }

    pub(crate) fn documentation_mut(
        &mut self,
    ) -> Option<&mut Option<QapiDocumentation<'i>>> {
        match self {
            Self::Alternate(value) => Some(&mut value.doc),
            Self::Command(value) => Some(&mut value.doc),
            Self::Enum(value) => Some(&mut value.doc),
            Self::Event(value) => Some(&mut value.doc),
            Self::Struct(value) => Some(&mut value.doc),
            Self::Union(value) => Some(&mut value.doc),
            _ => None,
        }
    }

    pub(crate) fn documentation(&self) -> Option<&QapiDocumentation<'i>> {
        match self {
            Self::Alternate(value) => value.doc.as_ref(),
            Self::Command(value) => value.doc.as_ref(),
            Self::Enum(value) => value.doc.as_ref(),
            Self::Event(value) => value.doc.as_ref(),
            Self::Struct(value) => value.doc.as_ref(),
            Self::Union(value) => value.doc.as_ref(),
            _ => None,
        }
    }
}

// To preserve the token ordering at this stage, the parsers schema output is a
// Vec<Token> with each token being a parsed element from the qapi spec. At this
// stage, all ordering is preserved and the `&str` references can still be used
// to highlight and debug the original schema file.
#[derive(Debug, Clone)]
pub struct QapiSchema<'i>(pub Vec<QapiSchemaToken<'i>>);
impl<'i> QapiSchema<'i> {
    pub(crate) fn parse(input: &'i str) -> IResult<&'i str, Self> {
        Self::parse_v1(input)
    }

    pub(crate) fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let mut token_parser = alt((
            whitespace1.map(|_| QapiSchemaToken::EmptyLines),
            QapiSectionDocumentation::parse.map(QapiSchemaToken::Doc),
            QapiFreeformDoc::parse.map(QapiSchemaToken::FreeformDoc),
            qcomment.map(QapiSchemaToken::Comment),
            QapiAlternate::parse.map(QapiSchemaToken::Alternate),
            QapiCommand::parse.map(QapiSchemaToken::Command),
            QapiEnum::parse.map(QapiSchemaToken::Enum),
            QapiEvent::parse.map(QapiSchemaToken::Event),
            QapiStruct::parse.map(QapiSchemaToken::Struct),
            QapiUnion::parse.map(QapiSchemaToken::Union),
            QapiInclude::parse.map(QapiSchemaToken::Include),
            QapiPragma::parse.map(QapiSchemaToken::Pragma),
            documented_expression,
            invalid_documentation,
            // Bare ## separator (not followed by doc content, common in v2.x)
            bare_doc_separator.map(|_| QapiSchemaToken::EmptyLines),
            legacy_comment.map(QapiSchemaToken::Comment),
            ignored_expression,
        ));
        many0(|input| token_parser.parse(input)).map(Self).parse(input)
    }
}

/// An expression that the definitions' grammar does not read. Before 2.4,
/// one without a metatype that the release's generators read (`enum`,
/// `type`, `union`, `command`, and from 2.1 `event` and `include`), which
/// they skip, such as a `'struct'`, `'alternate'` or `'pragma'` of later
/// releases, or else one whose values they read loosely, which the IR
/// cannot represent. The keys they do not read are ignored as they are.
/// From 2.4, an object without a metatype or directive key, which QEMU
/// refuses only with the definitions' checks.
fn ignored_expression(input: &str) -> IResult<&str, QapiSchemaToken<'_>> {
    let version = super::parser_version();
    let metas: &[&str] = if version >= (2, 9, 0) {
        &[
            "enum",
            "struct",
            "union",
            "alternate",
            "command",
            "event",
            "include",
            "pragma",
        ]
    } else if version >= (2, 4, 0) {
        &["enum", "struct", "union", "alternate", "command", "event", "include"]
    } else if version >= (2, 1, 0) {
        &["enum", "type", "union", "command", "event", "include"]
    } else {
        &["enum", "type", "union", "command"]
    };
    let refuse = || {
        nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))
    };
    match super::common::top_level_expression(input) {
        Ok((Some(keys), rest)) => {
            let text = super::common::expression_text(input, rest)
                .trim_end_matches(is_whitespace);
            let read = keys.iter().any(|key| metas.contains(&key.as_ref()));
            Ok((
                rest,
                match (version >= (2, 4, 0), read) {
                    // The grammar of the definition or directive refuses it.
                    (true, true) => return Err(refuse()),
                    (true, false) => QapiSchemaToken::NoMetatype(text),
                    (false, true) => QapiSchemaToken::Unsupported(text),
                    (false, false) => QapiSchemaToken::Comment(text),
                },
            ))
        }
        _ => Err(refuse()),
    }
}

/// From 2.4, an expression without a metatype after a documentation
/// comment, which QEMU refuses for its metatype first.
fn documented_expression(input: &str) -> IResult<&str, QapiSchemaToken<'_>> {
    let (rest, _) = QapiDocumentation::parse(input)?;
    match ignored_expression(rest)? {
        (rest, token @ QapiSchemaToken::NoMetatype(_)) => Ok((rest, token)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn legacy_comment(input: &str) -> IResult<&str, &str> {
    if super::parser_version() < (2, 9, 0) {
        super::common::any_comment(input)
    } else {
        qcomment(input)
    }
}

// Failed documentation must not fall back to ordinary comments and vanish.
fn invalid_documentation(input: &str) -> IResult<&str, QapiSchemaToken<'_>> {
    let mut lines = lines(input);
    let first = lines.next().unwrap_or("").trim();
    let shorthand = first
        .strip_prefix("##")
        .is_some_and(|rest| rest.trim_start().starts_with('@'));
    let symbol = if first == "##" {
        lines
            .map(str::trim)
            .find(|line| !line.is_empty() && *line != "#")
            .and_then(|line| line.strip_prefix('#'))
            .is_some_and(|line| line.trim_start().starts_with('@'))
    } else {
        false
    };
    if super::parser_version() >= (2, 9, 0) && (shorthand || symbol) {
        // A comment that QEMU reads without fault: what follows it fails.
        // Otherwise the documentation check reports the comment's fault in
        // place of this error, whose kind no message shows.
        let error = match super::doc_comment::read(
            input.trim_start_matches(is_whitespace),
            &mut (),
        ) {
            Ok((_, rest)) => {
                nom::error::Error::new(rest, nom::error::ErrorKind::Eof)
            }
            Err(_) => {
                nom::error::Error::new(input, nom::error::ErrorKind::Fail)
            }
        };
        return Err(nom::Err::Failure(error));
    }
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Verify,
    )))
}

/// Matches a bare `##` line (with optional trailing whitespace) that acts as a
/// separator.
fn bare_doc_separator(input: &str) -> IResult<&str, &str> {
    recognize(terminated((space0, tag("##"), space0), line_ending)).parse(input)
}

vec_newtype!(QapiSchema, QapiSchemaToken<'i>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let (rem, s) = QapiSchema::parse("").unwrap();
        assert_eq!(rem, "");
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_single_enum() {
        let input = "{ 'enum': 'MyEnum', 'data': ['val-a', 'val-b'] }\n";
        let (rem, s) = QapiSchema::parse(input).unwrap();
        assert_eq!(rem, "");
        assert!(s.iter().any(|t| matches!(t, QapiSchemaToken::Enum(_))));
    }

    #[test]
    fn test_single_struct() {
        let input = "{ 'struct': 'MyStruct', 'data': { 'name': 'str' } }\n";
        let (rem, s) = QapiSchema::parse(input).unwrap();
        assert_eq!(rem, "");
        assert!(s.iter().any(|t| matches!(t, QapiSchemaToken::Struct(_))));
    }

    #[test]
    fn test_single_command() {
        let input = "{ 'command': 'my-cmd', 'data': { 'arg': 'str' } }\n";
        let (rem, s) = QapiSchema::parse(input).unwrap();
        assert_eq!(rem, "");
        assert!(s.iter().any(|t| matches!(t, QapiSchemaToken::Command(_))));
    }

    #[test]
    fn test_include() {
        let input = "{ 'include': 'other.json' }\n";
        let (rem, s) = QapiSchema::parse(input).unwrap();
        assert_eq!(rem, "");
        assert!(s.iter().any(|t| matches!(t, QapiSchemaToken::Include(_))));
    }

    #[test]
    fn test_comment_and_whitespace() {
        // Comments before a definition get absorbed by clean_lines during
        // parsing. A standalone comment only appears as a Comment token when
        // not adjacent to a definition.
        let input = "# a comment\n{ 'enum': 'E', 'data': ['a'] }\n";
        let (rem, s) = QapiSchema::parse(input).unwrap();
        assert_eq!(rem, "");
        let has_enum = s.iter().any(|t| matches!(t, QapiSchemaToken::Enum(_)));
        assert!(has_enum);
    }

    #[test]
    fn test_multi_entry_schema() {
        let input = concat!(
            "{ 'enum': 'Status', 'data': ['ok', 'error'] }\n",
            "{ 'struct': 'Info', 'data': { 'name': 'str' } }\n",
            "{ 'command': 'get-info', 'returns': 'Info' }\n",
            "{ 'event': 'STATUS_CHANGE', 'data': { 'status': 'Status' } }\n",
        );
        let (rem, s) = QapiSchema::parse(input).unwrap();
        assert_eq!(rem, "");
        let type_count = s
            .iter()
            .filter(|t| {
                !matches!(
                    t,
                    QapiSchemaToken::EmptyLines | QapiSchemaToken::Comment(_)
                )
            })
            .count();
        assert_eq!(type_count, 4);
    }
}

#[cfg(test)]
mod freeform_schema_tests {
    use super::*;
    use crate::parsers::set_parser_version;

    #[test]
    fn test_freeform_doc_direct() {
        set_parser_version((99, 0, 0));
        // Test QapiFreeformDoc directly on the input
        let input = "##\n# title\n# body\n##\n";
        let result = QapiFreeformDoc::parse(input);
        assert!(
            result.is_ok(),
            "Direct FreeformDoc parse failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_freeform_doc_in_schema() {
        set_parser_version((99, 0, 0));
        let input = "##\n# title\n# body\n##\n{ 'include': 'other.json' }\n";
        let (rem, s) = QapiSchema::parse(input).unwrap();
        assert_eq!(
            rem,
            "",
            "Unparsed: {:?}",
            &rem[..std::cmp::min(rem.len(), 100)]
        );
        let has_freeform =
            s.iter().any(|t| matches!(t, QapiSchemaToken::FreeformDoc(_)));
        assert!(has_freeform, "Should have freeform doc");
        let has_include =
            s.iter().any(|t| matches!(t, QapiSchemaToken::Include(_)));
        assert!(has_include, "Should have include");
    }
}
