use super::common::QapiString;
use super::common::{Malformed, keep, malformed_key};
use super::common::{
    expression_text, other_key, qbool, qstring, read_since, take_keyed_dict, take_kv,
};
use super::qapi_cond::QapiCond;
use super::qapi_cond::take_cond;
use super::qapi_documentation::QapiDocumentation;
use super::qapi_features::QapiFeatures;
use super::qapi_features::take_features;
use super::qapi_members::MembersOrRef;
use super::qapi_members::take_members_or_ref;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::combinator::opt;
use nom::error::{Error, ErrorKind};

/// The keys whose `null` QEMU 2.4 to 4.1 take for an absent key.
const ABSENT: &[&str] = &["data", "if"];

/// The keys that the parser reads; from 2.4 the others are kept for the
/// validation.
const READ: &[&str] = &["event", "boxed", "if", "features", "data"];
#[derive(Debug, Clone, Default)]
pub struct QapiEvent<'i> {
    pub name: QapiString<'i>,
    pub data: Option<MembersOrRef<'i>>,
    pub boxed: Option<&'i str>,
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

impl<'i> QapiEvent<'i> {
    /// EVENT = { 'event': STRING,
    ///           (
    ///           '*data': ( MEMBERS | STRING ),
    ///           |
    ///           'data': STRING,
    ///           'boxed': true,
    ///           )
    ///           '*if': COND,
    ///           '*features': FEATURES }
    ///
    /// The generators of releases before 2.1 have no events and skip them.
    pub(crate) fn parse(input: &'i str) -> IResult<&'i str, Self> {
        if super::parser_version() < (2, 1, 0) {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }
        Self::parse_v1(input)
    }

    pub(crate) fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let (input, doc) = opt(QapiDocumentation::parse).parse(input)?;
        let mut s = Self {
            doc,
            ..Default::default()
        };
        let start = input;
        let (input, (_, keys)) = take_keyed_dict(alt((
            take_kv("event", qstring).map(|v| s.name = v),
            read_since((2, 4, 0), take_kv("boxed", qbool)).map(|v| s.boxed = Some(v)),
            read_since((2, 4, 0), take_cond).map(|v| s.r#if = Some(v)),
            read_since((2, 4, 0), take_features).map(|v| s.features = Some(v)),
            take_kv("data", take_members_or_ref).map(|v| s.data = Some(v)),
            other_key(READ),
            malformed_key(READ).map(|entry| keep(&mut s.malformed, ABSENT, entry)),
        )))
        .parse(input)?;
        if !super::common::named(&keys, "event", &s.name) {
            return Err(nom::Err::Error(Error::new(start, ErrorKind::Tag)));
        }
        // Named boxed payload requirements are checked after schema resolution.
        // the qapi spec needs to be correct here, data being equal to None is allowed for us.
        s.keys = keys;
        s.text = expression_text(start, input);
        Ok((input, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_INPUTS: [&str; 1] = [r#"{ 'event': 'BLOCK_IMAGE_CORRUPTED',
  'data': { 'device'     : 'str',
            '*node-name' : 'str',
            'msg'        : 'str',
            '*offset'    : 'int',
            '*size'      : 'int',
            'fatal'      : 'bool' } }"#];

    #[test]
    fn test_valid() {
        for input in VALID_INPUTS {
            let result = QapiEvent::parse(input);
            match result {
                Ok((remaining, _)) => {
                    assert_eq!(remaining, "");
                }
                _ => panic!("Failed to parse: ```\n{input}\n```"),
            }
        }
    }
}
