use super::common::QapiString;
use super::common::{Malformed, keep, malformed_key};
use super::common::{
    expression_text, other_key, qstring, read_since, take_keyed_dict, take_kv, take_list,
};
use super::qapi_cond::QapiCond;
use super::qapi_cond::take_cond;
use super::qapi_documentation::QapiDocumentation;
use super::qapi_enum_value::QapiEnumValue;
use super::qapi_enum_value::take_enum_value;
use super::qapi_features::QapiFeatures;
use super::qapi_features::take_features;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::combinator::opt;
use nom::error::{Error, ErrorKind};

/// The keys whose `null` QEMU 2.4 to 4.1 take for an absent key.
const ABSENT: &[&str] = &["prefix", "if"];

/// The keys that the parser reads; from 2.4 the others are kept for the
/// validation.
const READ: &[&str] = &["enum", "prefix", "data", "if", "features"];
pub fn take_enum(input: &str) -> IResult<&str, QapiEnum<'_>> {
    QapiEnum::parse(input)
}

#[derive(Debug, Clone, Default)]
pub struct QapiEnum<'i> {
    pub name: QapiString<'i>,
    pub data: Vec<QapiEnumValue<'i>>,
    pub r#if: Option<QapiCond<'i>>,
    pub prefix: Option<QapiString<'i>>,
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

impl<'i> QapiEnum<'i> {
    /// ENUM = { 'enum': STRING,
    ///          'data': [ ENUM-VALUE, ... ],
    ///          '*prefix': STRING,
    ///          '*if': COND,
    ///          '*features': FEATURES }
    pub fn parse(input: &'i str) -> IResult<&'i str, Self> {
        Self::parse_v1(input)
    }

    pub fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let (input, doc) = opt(QapiDocumentation::parse).parse(input)?;
        let mut s = Self {
            doc,
            ..Default::default()
        };
        let start = input;
        let mut has_data = false;
        let (input, (_, keys)) = take_keyed_dict(alt((
            take_kv("enum", qstring).map(|v| s.name = v),
            read_since((2, 4, 0), take_kv("prefix", qstring)).map(|v| s.prefix = Some(v)),
            take_kv("data", take_list(take_enum_value)).map(|v| {
                has_data = true;
                s.data = v;
            }),
            read_since((2, 4, 0), take_cond).map(|v| s.r#if = Some(v)),
            read_since((2, 4, 0), take_features).map(|v| s.features = Some(v)),
            other_key(READ),
            malformed_key(READ).map(|entry| keep(&mut s.malformed, ABSENT, entry)),
        )))
        .parse(input)?;
        // From 2.4 the validation refuses a missing 'data'.
        if !super::common::named(&keys, "enum", &s.name)
            || (!has_data && super::parser_version() < (2, 4, 0))
        {
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

    const VALID_INPUTS: [&str; 1] = [
        "{ 'enum': 'Qcow2CompressionType', 'data': [ 'zlib', { 'name': 'zstd', 'if': 'CONFIG_ZSTD' } ] }",
    ];

    #[test]
    fn empty_enum_retains_required_data_key() {
        let (remaining, value) = QapiEnum::parse("{ 'enum': 'Empty', 'data': [] }").unwrap();
        assert!(remaining.is_empty());
        assert!(value.data.is_empty());
        // From 2.4 the validation refuses a missing 'data'.
        let (_, value) = QapiEnum::parse("{ 'enum': 'MissingData', 'other': [] }").unwrap();
        assert_eq!(value.keys, ["enum", "other"]);
        crate::set_parser_version((2, 3, 0));
        assert!(QapiEnum::parse("{ 'enum': 'MissingData' }").is_err());
        crate::set_parser_version((11, 1, 3));
    }

    #[test]
    fn test_valid() {
        for input in VALID_INPUTS {
            let result = QapiEnum::parse(input);
            match result {
                Ok((remaining, _)) => {
                    assert_eq!(remaining, "");
                }
                _ => panic!("Failed to parse: ```\n{input}\n```"),
            }
        }
    }
}
