use super::common::QapiString;
use super::common::{
    Malformed, keep, malformed_key, other_key, qstring, qtag, raw_value, take_keyed_dict,
};
use super::qapi_cond::QapiCond;
use super::qapi_cond::take_cond;
use super::qapi_type_ref::QapiTypeRef;
use super::qapi_type_ref::take_type_ref;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::error::{Error, ErrorKind};
use nom::sequence::terminated;

pub fn take_variant(input: &str) -> IResult<&str, QapiVariant<'_>> {
    QapiVariant::parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string_type() {
        let (rem, v) = QapiVariant::parse("'branch-name': 'SomeType'").unwrap();
        assert_eq!(rem, "");
        assert_eq!(v.name, "branch-name");
        assert_eq!(v.r#type, QapiTypeRef::Ref("SomeType".into()));
        assert!(v.r#if.is_none());
    }

    #[test]
    fn test_simple_array_type() {
        let (rem, v) = QapiVariant::parse("'items': ['ItemType']").unwrap();
        assert_eq!(rem, "");
        assert_eq!(v.name, "items");
        assert_eq!(v.r#type, QapiTypeRef::ArrayRef("ItemType".into()));
    }

    #[test]
    fn test_complex_with_condition() {
        let input = "'branch-name': { 'type': 'SomeType', 'if': 'CONFIG_X' }";
        let (rem, v) = QapiVariant::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(v.name, "branch-name");
        assert_eq!(v.r#type, QapiTypeRef::Ref("SomeType".into()));
        assert!(v.r#if.is_some());
    }

    #[test]
    fn test_complex_array_with_condition() {
        let input = "'data': { 'type': ['MyType'], 'if': 'CONFIG_Y' }";
        let (rem, v) = QapiVariant::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(v.name, "data");
        assert_eq!(v.r#type, QapiTypeRef::ArrayRef("MyType".into()));
        assert!(v.r#if.is_some());
    }

    #[test]
    fn test_whitespace_tolerance() {
        let input = " 'name' : 'Type' ";
        let (_, v) = QapiVariant::parse(input).unwrap();
        assert_eq!(v.name, "name");
        assert_eq!(v.r#type, QapiTypeRef::Ref("Type".into()));
    }

    #[test]
    fn test_missing_name_fails() {
        assert!(QapiVariant::parse("").is_err());
    }

    #[test]
    fn test_missing_type_fails() {
        assert!(QapiVariant::parse("'name':").is_err());
    }

    #[test]
    fn test_invalid_format() {
        assert!(QapiVariant::parse("not_valid").is_err());
    }
}

/// A named type reference with an optional condition.
/// Used for both union branches and alternate alternatives in the QAPI schema.
#[derive(Debug, Clone, Default)]
pub struct QapiVariant<'i> {
    pub name: QapiString<'i>,
    pub r#type: QapiTypeRef<'i>,
    pub r#if: Option<QapiCond<'i>>,
    /// The keys of its dictionary form, in their order in the source;
    /// `None` for the short form.
    pub keys: Option<Vec<QapiString<'i>>>,
    /// From 2.4, the values that the grammar refused, for the validation:
    /// the type of the short form is its `type`.
    pub malformed: Malformed<'i>,
}

impl<'i> QapiVariant<'i> {
    /// STRING : TYPE-REF
    /// | STRING : { 'type': TYPE-REF, '*if': COND }
    pub fn parse(input: &'i str) -> IResult<&'i str, Self> {
        Self::parse_v1(input)
    }

    pub fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let start = input;
        let (input, variant) = alt((
            Self::simple_parser,
            Self::complex_parser,
            Self::malformed_parser,
        ))
        .parse(input)?;
        // The validation refuses a dictionary without 'type', and from 2.4
        // an empty name.
        if (variant.name.is_empty() && super::parser_version() < (2, 4, 0))
            || (variant.r#type == QapiTypeRef::Unset
                && variant.keys.is_none()
                && variant.malformed.is_empty())
        {
            return Err(nom::Err::Error(Error::new(start, ErrorKind::Tag)));
        }
        Ok((input, variant))
    }

    /// STRING : TYPE-REF
    fn simple_parser(input: &'i str) -> IResult<&'i str, Self> {
        let mut s = Self::default();
        let name_parser = terminated(qstring, qtag(":")).map(|v| s.name = v);
        let type_parser = QapiTypeRef::parse.map(|v| s.r#type = v);
        let (input, _) = (name_parser, type_parser).parse(input)?;
        Ok((input, s))
    }

    /// From 2.4, STRING : VALUE of another shape, which the validation
    /// refuses as the branch's type.
    fn malformed_parser(input: &'i str) -> IResult<&'i str, Self> {
        let mut s = Self::default();
        let (input, (name, _, value)) = (qstring, qtag(":"), raw_value).parse(input)?;
        s.name = name;
        s.malformed.push(("type".into(), value));
        Ok((input, s))
    }

    /// STRING : { 'type': TYPE-REF, '*if': COND }: from 2.4 to 3.1 a
    /// dictionary is an anonymous struct, which the validation refuses.
    fn complex_parser(input: &'i str) -> IResult<&'i str, Self> {
        if ((2, 4, 0)..(4, 0, 0)).contains(&super::parser_version()) {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }
        let mut s = Self::default();
        let name_parser = terminated(qstring, qtag(":")).map(|v| s.name = v);
        let type_parser = take_type_ref.map(|v| s.r#type = v);
        let cond_parser = take_cond.map(|v| s.r#if = Some(v));
        let (input, (_, (_, keys))) = (
            name_parser,
            take_keyed_dict(alt((
                type_parser,
                cond_parser,
                other_key(&["type", "if"]),
                malformed_key(&["type", "if"]).map(|entry| keep(&mut s.malformed, &["if"], entry)),
            ))),
        )
            .parse(input)?;
        s.keys = Some(keys);
        Ok((input, s))
    }
}
