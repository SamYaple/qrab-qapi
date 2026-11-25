use super::common::QapiString;
use super::common::{
    Malformed, keep, malformed_key, other_key, qstring, raw_value, take_keyed_dict, take_kv,
};
use super::qapi_cond::QapiCond;
use super::qapi_cond::take_cond;
use super::qapi_features::QapiFeatures;
use super::qapi_features::take_features;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::error::{Error, ErrorKind};

pub(crate) fn take_enum_value(input: &str) -> IResult<&str, QapiEnumValue<'_>> {
    QapiEnumValue::parse(input)
}

#[derive(Debug, Clone, Default)]
pub struct QapiEnumValue<'i> {
    pub name: QapiString<'i>,
    pub r#if: Option<QapiCond<'i>>,
    pub features: Option<QapiFeatures<'i>>,
    /// Written as an object, `{ 'name': ... }`, which QEMU reads from 4.0:
    /// from 2.4 to 3.1 it requires a string name.
    pub dictionary: bool,
    /// The keys of its dictionary form, in their order in the source.
    pub keys: Vec<QapiString<'i>>,
    /// From 2.4, the values that the grammar refused, for the validation:
    /// a value that is neither a string nor an object is its `name`.
    pub malformed: Malformed<'i>,
}

impl<'i> QapiEnumValue<'i> {
    /// ENUM-VALUE = STRING
    ///            | { 'name': STRING,
    ///                '*if': COND,
    ///                '*features': FEATURES }
    pub(crate) fn parse(input: &'i str) -> IResult<&'i str, Self> {
        Self::parse_v1(input)
    }

    pub(crate) fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let start = input;
        let (input, (enumvalue, simple)) = alt((
            Self::simple_parser.map(|value| (value, true)),
            Self::complex_parser.map(|value| (value, false)),
            Self::malformed_parser.map(|value| (value, false)),
        ))
        .parse(input)?;
        // From 2.4 the validation refuses a dictionary without 'name', and
        // from 2.6 an empty name, on which QEMU fails.
        let unnamed = !simple && super::parser_version() < (2, 4, 0);
        if enumvalue.name.is_empty() && unnamed && enumvalue.malformed.is_empty() {
            return Err(nom::Err::Error(Error::new(start, ErrorKind::Tag)));
        }
        Ok((input, enumvalue))
    }

    /// STRING
    fn simple_parser(input: &'i str) -> IResult<&'i str, Self> {
        let mut s = Self::default();
        let (input, _) = qstring.map(|v| s.name = v).parse(input)?;
        Ok((input, s))
    }

    /// From 2.4, a value that is neither a string nor an object, which the
    /// validation refuses as its name.
    fn malformed_parser(input: &'i str) -> IResult<&'i str, Self> {
        let (input, value) = raw_value(input)?;
        if matches!(value.value, super::common::Raw::Object(_)) {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }
        let mut s = Self::default();
        s.malformed.push(("name".into(), value));
        Ok((input, s))
    }

    /// { 'name': STRING,
    ///   '*if': COND,
    ///   '*features': FEATURES }
    fn complex_parser(input: &'i str) -> IResult<&'i str, Self> {
        let mut s = Self::default();
        let (input, (_, keys)) = take_keyed_dict(alt((
            take_kv("name", qstring).map(|v| s.name = v),
            take_features.map(|v| s.features = Some(v)),
            take_cond.map(|v| s.r#if = Some(v)),
            other_key(&["name", "features", "if"]),
            malformed_key(&["name", "features", "if"])
                .map(|entry| keep(&mut s.malformed, &["if"], entry)),
        )))
        .parse(input)?;
        s.dictionary = true;
        s.keys = keys;
        Ok((input, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        let (rem, v) = QapiEnumValue::parse("'my-value'").unwrap();
        assert_eq!(rem, "");
        assert_eq!(v.name, "my-value");
        assert!(v.r#if.is_none());
        assert!(v.features.is_none());
    }

    #[test]
    fn test_dict_name_only() {
        let (rem, v) = QapiEnumValue::parse("{ 'name': 'my-value' }").unwrap();
        assert_eq!(rem, "");
        assert_eq!(v.name, "my-value");
        assert!(v.r#if.is_none());
        assert!(v.features.is_none());
    }

    #[test]
    fn test_dict_with_condition() {
        let input = "{ 'name': 'my-value', 'if': 'CONFIG_X' }";
        let (rem, v) = QapiEnumValue::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(v.name, "my-value");
        assert!(v.r#if.is_some());
    }

    #[test]
    fn test_dict_with_features() {
        let input = "{ 'name': 'my-value', 'features': ['deprecated'] }";
        let (rem, v) = QapiEnumValue::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(v.name, "my-value");
        assert!(v.features.is_some());
    }

    #[test]
    fn test_dict_with_all_fields() {
        let input = "{ 'name': 'my-value', 'if': 'CONFIG_X', 'features': ['deprecated'] }";
        let (rem, v) = QapiEnumValue::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(v.name, "my-value");
        assert!(v.r#if.is_some());
        assert!(v.features.is_some());
    }

    #[test]
    fn test_empty_name_is_kept_for_the_validation() {
        crate::set_parser_version((11, 1, 3));
        assert_eq!(QapiEnumValue::parse("''").unwrap().1.name, "");
    }

    #[test]
    fn legacy_empty_enum_values_do_not_allow_missing_dictionary_names() {
        crate::set_parser_version((2, 5, 0));
        assert_eq!(QapiEnumValue::parse("''").unwrap().1.name, "");
        // The validation refuses a dictionary for its name.
        assert!(QapiEnumValue::parse("{}").unwrap().1.dictionary);
        crate::set_parser_version((2, 3, 0));
        assert!(QapiEnumValue::parse("{ 'if': 'CONFIG_X' }").is_err());
        crate::set_parser_version((11, 1, 3));
    }

    #[test]
    fn dictionaries_keep_their_keys_for_the_validation() {
        let (_, value) = QapiEnumValue::parse("{ 'if': 'CONFIG_X' }").unwrap();
        assert_eq!(value.name, "");
        assert_eq!(value.keys, ["if"]);
        let (_, value) = QapiEnumValue::parse("{ 'name': 'val', 'unknown': 'x' }").unwrap();
        assert_eq!(value.keys, ["name", "unknown"]);
    }

    #[test]
    fn test_invalid_input_fails() {
        assert!(QapiEnumValue::parse("not_valid").is_err());
    }

    #[test]
    fn values_of_read_keys_are_kept_from_2_4() {
        let (_, value) = QapiEnumValue::parse("{ 'name': ['val'] }").unwrap();
        assert_eq!(value.malformed[0].1.text, "['val']");
        let (_, value) = QapiEnumValue::parse("true").unwrap();
        assert_eq!(value.malformed[0].0, "name");
        crate::set_parser_version((2, 3, 0));
        assert!(QapiEnumValue::parse("{ 'name': ['val'] }").is_err());
        crate::set_parser_version((11, 1, 3));
    }
}
