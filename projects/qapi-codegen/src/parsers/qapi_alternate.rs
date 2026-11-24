use super::common::QapiString;
use super::common::{Malformed, keep, malformed_key};
use super::common::{expression_text, other_key, qstring, take_keyed_dict, take_kv};
use super::qapi_alternatives::QapiAlternatives;
use super::qapi_alternatives::take_alternatives;
use super::qapi_cond::QapiCond;
use super::qapi_cond::take_cond;
use super::qapi_documentation::QapiDocumentation;
use super::qapi_features::QapiFeatures;
use super::qapi_features::take_features;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::combinator::opt;
use nom::error::{Error, ErrorKind};

/// The keys whose `null` QEMU 2.4 to 4.1 take for an absent key.
const ABSENT: &[&str] = &["if"];

/// The keys that the parser reads; from 2.4 the others are kept for the
/// validation.
const READ: &[&str] = &["alternate", "data", "if", "features"];
pub fn take_alternate(input: &str) -> IResult<&str, QapiAlternate<'_>> {
    QapiAlternate::parse(input)
}

#[derive(Debug, Clone, Default)]
pub struct QapiAlternate<'i> {
    pub name: QapiString<'i>,
    pub data: QapiAlternatives<'i>,
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

impl<'i> QapiAlternate<'i> {
    /// ALTERNATE = { 'alternate': STRING,
    ///               'data': ALTERNATIVES,
    ///               '*if': COND,
    ///               '*features': FEATURES }
    ///
    /// From 2.4: the generators of earlier releases skip it.
    pub fn parse(input: &'i str) -> IResult<&'i str, Self> {
        if super::parser_version() < (2, 4, 0) {
            let error = nom::error::Error::new(input, nom::error::ErrorKind::Tag);
            return Err(nom::Err::Error(error));
        }
        Self::parse_v1(input)
    }

    pub fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let (input, doc) = opt(QapiDocumentation::parse).parse(input)?;
        let mut s = Self {
            doc,
            ..Default::default()
        };
        let start = input;
        let (input, (_, keys)) = take_keyed_dict(alt((
            take_kv("alternate", qstring).map(|v| s.name = v),
            take_alternatives.map(|v| s.data = v),
            take_cond.map(|v| s.r#if = Some(v)),
            take_features.map(|v| s.features = Some(v)),
            other_key(READ),
            malformed_key(READ).map(|entry| keep(&mut s.malformed, ABSENT, entry)),
        )))
        .parse(input)?;
        if !super::common::named(&keys, "alternate", &s.name) {
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

    const VALID_INPUTS: [&str; 5] = [
        "{'alternate':'SOMENAME','data':{'membername':'membertype'}}",
        "{ 'alternate': 'SOMENAME', 'data': {'membername':{'if':'CONFIG_OPTION', 'type': ['sometype']}}, 'features': ['yes']}",
        "{'alternate': 'SOMENAME', 'data':{'membername':'membertype'}, 'if':{'not':'CONFIG_VALUE'}}",
        "{'alternate': 'SOMENAME', 'data':{'membername':'membertype'}}",
        "{'alternate': 'SOMENAME', 'data':{'membername':{'type':'membertype'}}}",
    ];

    #[test]
    fn test_valid() {
        for input in VALID_INPUTS {
            let result = QapiAlternate::parse(input);
            match result {
                Ok((remaining, _)) => {
                    assert_eq!(remaining, "");
                }
                _ => panic!("Failed to parse: ```\n{input}\n```"),
            }
        }
    }
}
