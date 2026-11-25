use super::common::QapiString;
use super::common::{
    Malformed, keep, malformed_key, other_key, qstring, qtag, raw_value, take_keyed_dict,
};
use super::qapi_cond::QapiCond;
use super::qapi_cond::take_cond;
use super::qapi_features::QapiFeatures;
use super::qapi_features::take_features;
use super::qapi_type_ref::QapiTypeRef;
use super::qapi_type_ref::take_type_ref;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::error::{Error, ErrorKind};
use nom::sequence::terminated;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct QapiMember<'i> {
    pub name: QapiString<'i>,
    pub optional: bool,
    pub r#type: QapiTypeRef<'i>,
    pub r#if: Option<QapiCond<'i>>,
    pub features: Option<QapiFeatures<'i>>,
    /// The keys of its dictionary form, in their order in the source;
    /// `None` for the short form.
    pub keys: Option<Vec<QapiString<'i>>>,
    /// From 2.4, the values that the grammar refused, for the validation:
    /// the type of the short form is its `type`.
    pub malformed: Malformed<'i>,
}

impl<'i> QapiMember<'i> {
    /// MEMBER = STRING : TYPE-REF
    ///        | STRING : { 'type': TYPE-REF,
    ///                     '*if': COND,
    ///                     '*features': FEATURES }
    pub(crate) fn parse(input: &'i str) -> IResult<&'i str, Self> {
        Self::parse_v1(input)
    }

    pub(crate) fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let start = input;
        let (input, mut member) = alt((
            Self::simple_parser,
            Self::complex_parser,
            Self::malformed_parser,
        ))
        .parse(input)?;
        // The validation refuses a dictionary without 'type', and from 2.4
        // an empty name.
        let lenient = super::parser_version() >= (2, 4, 0);
        if (member.name.is_empty() && !lenient)
            || (member.r#type == QapiTypeRef::Unset
                && member.keys.is_none()
                && member.malformed.is_empty())
        {
            return Err(nom::Err::Error(Error::new(start, ErrorKind::Tag)));
        }
        if member.name.starts_with('*') {
            member.optional = true;
            member.name = match member.name {
                QapiString::Borrowed(value) => QapiString::Borrowed(&value[1..]),
                QapiString::Owned(value) => QapiString::Owned(value[1..].to_owned()),
            };
        }
        if member.name.is_empty() && !lenient {
            return Err(nom::Err::Error(Error::new(start, ErrorKind::Verify)));
        }
        Ok((input, member))
    }

    /// From 2.4, STRING : VALUE of another shape, which the validation
    /// refuses as the member's type.
    fn malformed_parser(input: &'i str) -> IResult<&'i str, Self> {
        let mut s = Self::default();
        let (input, (name, _, value)) = (qstring, qtag(":"), raw_value).parse(input)?;
        s.name = name;
        s.malformed.push(("type".into(), value));
        Ok((input, s))
    }

    /// STRING : TYPE-REF
    fn simple_parser(input: &'i str) -> IResult<&'i str, Self> {
        let mut s = Self::default();
        let name_parser = terminated(qstring, qtag(":")).map(|v| s.name = v);
        let type_parser = QapiTypeRef::parse.map(|v| s.r#type = v);
        let (input, _) = (name_parser, type_parser).parse(input)?;
        Ok((input, s))
    }

    /// STRING : { 'type': TYPE-REF,
    ///            '*if': COND,
    ///            '*features': FEATURES }
    fn complex_parser(input: &'i str) -> IResult<&'i str, Self> {
        // From 2.4 to 3.1 a dictionary is an anonymous struct, which the
        // validation refuses.
        if ((2, 4, 0)..(4, 0, 0)).contains(&super::parser_version()) {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }
        let mut s = Self::default();
        let name_parser = terminated(qstring, qtag(":")).map(|v| s.name = v);
        let type_parser = take_type_ref.map(|v| s.r#type = v);
        let cond_parser = take_cond.map(|v| s.r#if = Some(v));
        let features_parser = take_features.map(|v| s.features = Some(v));
        const READ: &[&str] = &["type", "if", "features"];
        let other = other_key(READ);
        let malformed = malformed_key(READ).map(|entry| keep(&mut s.malformed, &["if"], entry));
        let (input, (_, (_, keys))) = (
            name_parser,
            take_keyed_dict(alt((
                features_parser,
                type_parser,
                cond_parser,
                other,
                malformed,
            ))),
        )
            .parse(input)?;
        s.keys = Some(keys);
        Ok((input, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_INPUTS: [&str; 4] = [
        "'name':'type'",
        "'name':['type']",
        "'name':{'if':'CONFIG_OPTION', 'type': ['sometype'], 'features': ['yes']}",
        "'name':{'if':'CONFIG_OPTION', 'type': 'sometype', 'features': [{'name':'yes', 'if': 'CONFIG_MAP'}]}",
    ];

    #[test]
    fn test_valid() {
        for input in VALID_INPUTS {
            let result = QapiMember::parse(input);
            match result {
                Ok((remaining, _)) => {
                    assert_eq!(remaining, "");
                }
                _ => panic!("Failed to parse: ```\n{input}\n```"),
            }
        }
    }
}
