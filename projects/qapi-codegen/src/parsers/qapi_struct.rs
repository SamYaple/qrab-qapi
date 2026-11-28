use super::{
    common::{
        Malformed, QapiString, expression_text, keep, malformed_key, other_key,
        qstring, read_since, take_keyed_dict, take_kv,
    },
    parser_version,
    qapi_cond::{QapiCond, take_cond},
    qapi_documentation::QapiDocumentation,
    qapi_features::{QapiFeatures, take_features},
    qapi_members::{QapiMembers, take_members},
};
use nom::{
    IResult, Parser,
    branch::alt,
    combinator::opt,
    error::{Error, ErrorKind},
};

/// The keys whose `null` QEMU 2.4 to 4.1 take for an absent key.
const ABSENT: &[&str] = &["base", "if"];

/// The keys that the parser reads; from 2.4 the others are kept for the
/// validation.
const READ: &[&str] = &["type", "struct", "base", "if", "features", "data"];
#[derive(Debug, Clone, Default)]
pub struct QapiStruct<'i> {
    pub name: QapiString<'i>,
    pub data: QapiMembers<'i>,
    pub base: Option<QapiString<'i>>,
    pub r#if: Option<QapiCond<'i>>,
    pub features: Option<QapiFeatures<'i>>,
    pub doc: Option<QapiDocumentation<'i>>,
    /// From 2.4, the keys whose values the grammar refused, for the
    /// validation.
    pub malformed: Malformed<'i>,
    /// The keys of the definition's expression, in their order there.
    pub keys: Vec<QapiString<'i>>,
    /// The definition's expression in its schema file, from `{` to `}`.
    pub text: &'i str,
}

impl<'i> QapiStruct<'i> {
    /// Dispatches to the correct versioned parser.
    pub(crate) fn parse(input: &'i str) -> IResult<&'i str, Self> {
        let v = parser_version();
        if v < (2, 4, 0) {
            Self::parse_v1(input)
        } else {
            Self::parse_v2(input)
        }
    }

    /// v1 (QEMU v1.0–v2.3.0): the `'type'` keyword. Their generators ignore
    /// an expression with the later `'struct'` keyword, and the keys they do
    /// not read: `'base'` before 1.7.
    pub(crate) fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let (input, doc) = opt(QapiDocumentation::parse).parse(input)?;
        let mut s = Self { doc, ..Default::default() };
        let start = input;
        let mut has_data = false;
        let take_name = take_kv("type", qstring);
        let (input, (_, keys)) = take_keyed_dict(alt((
            take_name.map(|v| s.name = v),
            read_since((1, 7, 0), take_kv("base", qstring))
                .map(|v| s.base = Some(v)),
            read_since((2, 4, 0), take_cond).map(|v| s.r#if = Some(v)),
            read_since((2, 4, 0), take_features).map(|v| s.features = Some(v)),
            take_kv("data", take_members).map(|v| {
                has_data = true;
                s.data = v;
            }),
            other_key(READ),
            malformed_key(READ)
                .map(|entry| keep(&mut s.malformed, ABSENT, entry)),
        )))
        .parse(input)?;
        if !super::common::named(&keys, "type", &s.name) || !has_data {
            return Err(nom::Err::Error(Error::new(start, ErrorKind::Tag)));
        }
        s.keys = keys;
        s.text = expression_text(start, input);
        Ok((input, s))
    }

    /// v2 (QEMU v2.4.0+): only `'struct'` keyword.
    pub(crate) fn parse_v2(input: &'i str) -> IResult<&'i str, Self> {
        let (input, doc) = opt(QapiDocumentation::parse).parse(input)?;
        let mut s = Self { doc, ..Default::default() };
        let start = input;
        let mut has_data = false;
        let (input, (_, keys)) = take_keyed_dict(alt((
            take_kv("struct", qstring).map(|v| s.name = v),
            take_kv("base", qstring).map(|v| s.base = Some(v)),
            take_cond.map(|v| s.r#if = Some(v)),
            take_features.map(|v| s.features = Some(v)),
            take_kv("data", take_members).map(|v| {
                has_data = true;
                s.data = v;
            }),
            other_key(READ),
            malformed_key(READ)
                .map(|entry| keep(&mut s.malformed, ABSENT, entry)),
        )))
        .parse(input)?;
        if !super::common::named(&keys, "struct", &s.name) {
            return Err(nom::Err::Error(Error::new(start, ErrorKind::Tag)));
        }
        s.keys = keys;
        s.text = expression_text(start, input);
        Ok((input, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::set_parser_version;

    const VALID_INPUTS: [&str; 4] = [
        "{'struct':'SOMENAME','data':{'membername':'membertype'}}",
        "{ 'struct': 'SOMENAME', 'data': {'membername':{'if':'CONFIG_OPTION', 'type': ['sometype'], 'features': ['yes']}}}",
        "{'struct': 'SOMENAME', 'data':{'membername':'membertype'}, 'if':{'not':'CONFIG_VALUE'}}",
        "{'struct': 'SOMENAME', 'data':{'membername':'membertype'}, 'base':'SOMETHING'}",
    ];

    #[test]
    fn data_is_required_even_when_empty() {
        for version in [(1, 0, 0), (11, 1, 3)] {
            set_parser_version(version);
            let keyword = if version < (2, 4, 0) { "type" } else { "struct" };
            // From 2.4 the validation refuses a missing 'data'.
            let missing = format!("{{ '{keyword}': 'MissingData' }}");
            assert_eq!(
                QapiStruct::parse(&missing).is_err(),
                version < (2, 4, 0)
            );
            assert_eq!(
                QapiStruct::parse(&format!(
                    "{{ '{keyword}': 'Empty', 'data': {{}} }}"
                ))
                .unwrap()
                .0,
                ""
            );
        }
    }

    #[test]
    fn test_valid_v2() {
        set_parser_version((10, 2, 0));
        for input in VALID_INPUTS {
            let result = QapiStruct::parse(input);
            match result {
                Ok((remaining, _)) => assert_eq!(remaining, ""),
                _ => panic!("Failed to parse: ```\n{input}\n```"),
            }
        }
    }

    #[test]
    fn test_type_keyword_accepted_in_v1() {
        set_parser_version((2, 0, 0));
        let input = "{ 'type': 'NameInfo', 'data': {'*name': 'str'} }";
        let result = QapiStruct::parse(input);
        match result {
            Ok((remaining, s)) => {
                assert_eq!(remaining, "");
                assert_eq!(s.name, "NameInfo");
            }
            Err(e) => panic!("v1 should accept 'type': {}", e),
        }
    }

    #[test]
    fn test_struct_keyword_defines_no_struct_in_v1() {
        set_parser_version((2, 3, 0));
        let input = "{ 'struct': 'NameInfo', 'data': {'*name': 'str'} }";
        assert!(QapiStruct::parse(input).is_err());
        let schema = crate::parse_schema(input).unwrap();
        assert!(matches!(schema.0[..], [crate::QapiSchemaToken::Comment(_)]));
    }

    #[test]
    fn test_type_keyword_rejected_in_v2() {
        set_parser_version((10, 2, 0));
        let input = "{ 'type': 'NameInfo', 'data': {'*name': 'str'} }";
        let result = QapiStruct::parse(input);
        assert!(result.is_err(), "'type' should be rejected in v2 (modern)");
    }
}
