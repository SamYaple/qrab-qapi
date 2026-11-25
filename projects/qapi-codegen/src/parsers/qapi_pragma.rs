use super::common::QapiString;
use super::common::{qbool, qstring, take_dict, take_kv, take_list};
use super::parser_version;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;

enum ParserToken<'i> {
    DocRequired(&'i str),
    CommandReturnsExceptions(Vec<QapiString<'i>>),
    CommandNameExceptions(Vec<QapiString<'i>>),
    DocumentationExceptions(Vec<QapiString<'i>>),
    MemberNameExceptions(Vec<QapiString<'i>>),
}

/// A `pragma` directive, from QEMU 2.9. Each field is `None` when the
/// directive does not set it.
#[derive(Debug, Clone, Default)]
pub struct QapiPragma<'i> {
    /// `doc-required`: every definition needs a documentation comment.
    pub doc_required: Option<bool>,
    /// `command-name-exceptions` (from 6.0): commands whose names may use `_`.
    pub command_name_exceptions: Option<Vec<QapiString<'i>>>,
    /// `command-returns-exceptions` (from 6.0), `returns-whitelist` before:
    /// commands that may return any type.
    pub command_returns_exceptions: Option<Vec<QapiString<'i>>>,
    /// `documentation-exceptions` (from 9.0): definitions whose members need
    /// no documentation.
    pub documentation_exceptions: Option<Vec<QapiString<'i>>>,
    /// `member-name-exceptions` (from 6.0), `name-case-whitelist` before:
    /// definitions whose member names need not be lowercase.
    pub member_name_exceptions: Option<Vec<QapiString<'i>>>,
}

impl<'i> QapiPragma<'i> {
    pub(crate) fn parse(input: &'i str) -> IResult<&'i str, Self> {
        let v = parser_version();
        if v < (2, 9, 0) {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        } else if v < (6, 0, 0) {
            Self::parse_v1(input)
        } else {
            Self::parse_v2(input)
        }
    }

    /// v1 (QEMU v2.9.0–v5.2.0): `doc-required`, `returns-whitelist` and
    /// `name-case-whitelist`.
    pub(crate) fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        Self::parse_common(input, true)
    }

    /// v2 (QEMU v6.0.0+): `doc-required`, `command-name-exceptions`,
    /// `command-returns-exceptions`, `member-name-exceptions` and, from 9.0,
    /// `documentation-exceptions`.
    pub(crate) fn parse_v2(input: &'i str) -> IResult<&'i str, Self> {
        Self::parse_common(input, false)
    }

    fn parse_common(input: &'i str, old_keys: bool) -> IResult<&'i str, Self> {
        let documentation = parser_version() >= (9, 0, 0);
        let entry = move |input: &'i str| -> IResult<&'i str, ParserToken<'i>> {
            let list = |key| take_kv(key, take_list(qstring));
            let mut doc_required = take_kv("doc-required", qbool).map(ParserToken::DocRequired);
            if old_keys {
                alt((
                    doc_required,
                    list("returns-whitelist").map(ParserToken::CommandReturnsExceptions),
                    list("name-case-whitelist").map(ParserToken::MemberNameExceptions),
                ))
                .parse(input)
            } else if let Ok(parsed) = doc_required.parse(input) {
                Ok(parsed)
            } else if documentation
                && let Ok(parsed) = list("documentation-exceptions")
                    .map(ParserToken::DocumentationExceptions)
                    .parse(input)
            {
                Ok(parsed)
            } else {
                alt((
                    list("command-name-exceptions").map(ParserToken::CommandNameExceptions),
                    list("command-returns-exceptions").map(ParserToken::CommandReturnsExceptions),
                    list("member-name-exceptions").map(ParserToken::MemberNameExceptions),
                ))
                .parse(input)
            }
        };
        take_dict(take_kv("pragma", take_dict(entry).map(Self::from_tokens)))
            .map_opt(|mut values| values.pop())
            .parse(input)
    }

    fn from_tokens(tokens: Vec<ParserToken<'i>>) -> Self {
        let mut pragma = Self::default();
        for token in tokens {
            match token {
                ParserToken::DocRequired(v) => pragma.doc_required = Some(v == "true"),
                ParserToken::CommandReturnsExceptions(v) => {
                    pragma.command_returns_exceptions = Some(v)
                }
                ParserToken::CommandNameExceptions(v) => pragma.command_name_exceptions = Some(v),
                ParserToken::DocumentationExceptions(v) => {
                    pragma.documentation_exceptions = Some(v)
                }
                ParserToken::MemberNameExceptions(v) => pragma.member_name_exceptions = Some(v),
            }
        }
        pragma
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::set_parser_version;

    #[test]
    fn empty_pragma_is_valid_but_missing_pragma_is_not() {
        set_parser_version((11, 1, 3));
        assert_eq!(QapiPragma::parse("{ 'pragma': {} }").unwrap().0, "");
        assert!(QapiPragma::parse("{}").is_err());
    }

    #[test]
    fn duplicate_pragma_keys_are_rejected() {
        set_parser_version((11, 1, 3));
        for input in [
            "{ 'pragma': { 'doc-required': true, 'doc-required': false } }",
            "{ 'pragma': {}, 'pragma': {} }",
            "{ 'pragma': { 'member-name-exceptions': [], 'member-name-exceptions': ['X'] } }",
        ] {
            assert!(QapiPragma::parse(input).is_err(), "accepted {input}");
        }
    }

    #[test]
    fn test_valid_v2() {
        set_parser_version((10, 2, 0));
        let input = r#"{ 'pragma': {
            'command-name-exceptions': ['add_client'],
            'command-returns-exceptions': ['human-monitor-command'],
            'documentation-exceptions': ['AbortWrapper'],
            'member-name-exceptions': ['ACPISlotType'] } }"#;
        let (rem, _) = QapiPragma::parse(input).unwrap();
        assert_eq!(rem, "");
    }

    #[test]
    fn test_old_keys_accepted_in_v1() {
        set_parser_version((3, 0, 0));
        let input = "{ 'pragma': { 'returns-whitelist': ['human-monitor-command'], 'name-case-whitelist': ['ACPISlotType'] } }";
        let (rem, _) = QapiPragma::parse(input).unwrap();
        assert_eq!(rem, "");
    }

    #[test]
    fn test_old_keys_rejected_in_v2() {
        set_parser_version((10, 2, 0));
        let input = "{ 'pragma': { 'returns-whitelist': ['test'] } }";
        let result = QapiPragma::parse(input);
        assert!(result.is_err(), "Old pragma keys should be rejected in v2");
    }

    #[test]
    fn pragma_keys_follow_their_releases() {
        for (version, key, accepted) in [
            ((2, 8, 0), "'doc-required': true", false),
            ((2, 9, 0), "'doc-required': true", true),
            ((5, 2, 0), "'member-name-exceptions': []", false),
            ((5, 2, 0), "'name-case-whitelist': []", true),
            ((6, 0, 0), "'member-name-exceptions': []", true),
            ((8, 2, 0), "'documentation-exceptions': []", false),
            ((9, 0, 0), "'documentation-exceptions': []", true),
        ] {
            set_parser_version(version);
            let input = format!("{{ 'pragma': {{ {key} }} }}");
            assert_eq!(
                QapiPragma::parse(&input).is_ok(),
                accepted,
                "{version:?} {key}"
            );
        }
        set_parser_version((9, 0, 0));
        let (_, pragma) = QapiPragma::parse(
            "{ 'pragma': { 'doc-required': true, 'documentation-exceptions': ['A'] } }",
        )
        .unwrap();
        assert_eq!(pragma.doc_required, Some(true));
        assert_eq!(pragma.documentation_exceptions, Some(vec!["A".into()]));
    }

    #[test]
    fn test_doc_required() {
        set_parser_version((10, 2, 0));
        let input = "{ 'pragma': { 'doc-required': true } }";
        let (rem, _) = QapiPragma::parse(input).unwrap();
        assert_eq!(rem, "");
    }
}
