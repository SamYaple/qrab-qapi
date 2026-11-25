use super::common::QapiString;
use super::common::{
    Malformed, keep, malformed_key, other_key, qstring, raw_value, take_keyed_dict, take_kv,
};
use super::qapi_cond::{QapiCond, take_cond};
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::error::{Error, ErrorKind};

/// Holds information on a specific Feature in a single option
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QapiFeature<'i> {
    pub name: QapiString<'i>,
    pub r#if: Option<QapiCond<'i>>,
    /// The keys of its dictionary form, in their order in the source;
    /// `None` for the short form.
    pub keys: Option<Vec<QapiString<'i>>>,
    /// From 2.4, the values that the grammar refused, for the validation:
    /// a value that is neither a string nor an object is its `name`.
    pub malformed: Malformed<'i>,
}

impl<'i> QapiFeature<'i> {
    /// Parses a `FEATURE` as defined in the QAPI spec
    ///     FEATURE = STRING
    ///             | { 'name': STRING, '*if': COND }
    pub(crate) fn parse(input: &'i str) -> IResult<&'i str, Self> {
        Self::parse_v1(input)
    }

    pub(crate) fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let start = input;
        let (input, feature) = alt((
            Self::string_parser,
            Self::dict_parser,
            Self::malformed_parser,
        ))
        .parse(input)?;
        // The validation refuses a dictionary without 'name'.
        if feature.name.is_empty() && feature.keys.is_none() && feature.malformed.is_empty() {
            return Err(nom::Err::Error(Error::new(start, ErrorKind::Tag)));
        }
        Ok((input, feature))
    }

    /// STRING
    fn string_parser(input: &'i str) -> IResult<&'i str, Self> {
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

    /// { 'name': STRING, '*if': COND }
    fn dict_parser(input: &'i str) -> IResult<&'i str, Self> {
        let mut s = Self::default();
        let name_parser = take_kv("name", qstring).map(|v| s.name = v);
        let cond_parser = take_cond.map(|v| s.r#if = Some(v));
        let other = other_key(&["name", "if"]);
        let malformed =
            malformed_key(&["name", "if"]).map(|entry| keep(&mut s.malformed, &["if"], entry));
        let (input, (_, keys)) =
            take_keyed_dict(alt((name_parser, cond_parser, other, malformed))).parse(input)?;
        s.keys = Some(keys);
        Ok((input, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_feature() {
        let input = "'feature_name'";
        let expected = QapiFeature {
            malformed: Vec::new(),
            name: "feature_name".into(),
            r#if: None,
            keys: None,
        };
        let (remaining, result) = QapiFeature::parse(input).expect("Parsing failed");
        assert_eq!(remaining, "");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_dict_feature_minimal() {
        let input = "{'name':'feature_name'}";
        let expected = QapiFeature {
            malformed: Vec::new(),
            name: "feature_name".into(),
            r#if: None,
            keys: Some(vec!["name".into()]),
        };
        let (remaining, result) = QapiFeature::parse(input).expect("Parsing failed");
        assert_eq!(remaining, "");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_dict_feature_with_if() {
        let input = "{ 'name': 'feature_name', 'if': 'CONFIG_CONDITION' }";
        let expected = QapiFeature {
            malformed: Vec::new(),
            name: "feature_name".into(),
            r#if: Some(QapiCond::ConfigName("CONFIG_CONDITION".into())),
            keys: Some(vec!["name".into(), "if".into()]),
        };
        let (remaining, result) = QapiFeature::parse(input).expect("Parsing failed");
        assert_eq!(remaining, "");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_dict_feature_with_complex_if() {
        let input = "{ 'name': 'feature_name', 'if': { 'not': 'CONFIG_CONDITION' } }";
        let expected = QapiFeature {
            malformed: Vec::new(),
            name: "feature_name".into(),
            r#if: Some(QapiCond::Not(Box::new(QapiCond::ConfigName(
                "CONFIG_CONDITION".into(),
            )))),
            keys: Some(vec!["name".into(), "if".into()]),
        };
        let (remaining, result) = QapiFeature::parse(input).expect("Parsing failed");
        assert_eq!(remaining, "");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_dict_feature_with_any_if() {
        let input = "{ 'name': 'feature_name', 'if': { 'any': [ 'CONFIG_A', 'CONFIG_B' ] } }";
        let expected = QapiFeature {
            malformed: Vec::new(),
            name: "feature_name".into(),
            r#if: Some(QapiCond::Any(vec![
                QapiCond::ConfigName("CONFIG_A".into()),
                QapiCond::ConfigName("CONFIG_B".into()),
            ])),
            keys: Some(vec!["name".into(), "if".into()]),
        };
        let (remaining, result) = QapiFeature::parse(input).expect("Parsing failed");
        assert_eq!(remaining, "");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_dict_feature_with_all_if() {
        let input = "{ 'name': 'feature_name', 'if': { 'all': [ 'CONFIG_A', 'CONFIG_B' ] } }";
        let expected = QapiFeature {
            malformed: Vec::new(),
            name: "feature_name".into(),
            r#if: Some(QapiCond::All(vec![
                QapiCond::ConfigName("CONFIG_A".into()),
                QapiCond::ConfigName("CONFIG_B".into()),
            ])),
            keys: Some(vec!["name".into(), "if".into()]),
        };
        let (remaining, result) = QapiFeature::parse(input).expect("Parsing failed");
        assert_eq!(remaining, "");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_feature_with_nested_conditions() {
        let input =
            "{ 'name': 'feature_name', 'if': { 'all': [ 'CONFIG_A', { 'not': 'CONFIG_B' } ] } }";
        let expected = QapiFeature {
            malformed: Vec::new(),
            name: "feature_name".into(),
            r#if: Some(QapiCond::All(vec![
                QapiCond::ConfigName("CONFIG_A".into()),
                QapiCond::Not(Box::new(QapiCond::ConfigName("CONFIG_B".into()))),
            ])),
            keys: Some(vec!["name".into(), "if".into()]),
        };
        let (remaining, result) = QapiFeature::parse(input).expect("Parsing failed");
        assert_eq!(remaining, "");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_feature_with_whitespace() {
        let input = " { 'name' : 'feature_name' , 'if' : 'CONFIG_CONDITION' } ";
        let expected = QapiFeature {
            malformed: Vec::new(),
            name: "feature_name".into(),
            r#if: Some(QapiCond::ConfigName("CONFIG_CONDITION".into())),
            keys: Some(vec!["name".into(), "if".into()]),
        };
        let (remaining, result) = QapiFeature::parse(input).expect("Parsing failed");
        assert_eq!(remaining.trim(), "");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_invalid_missing_name() {
        let input = "{'if':'CONFIG_CONDITION'}";
        let (_, feature) = QapiFeature::parse(input).unwrap();
        assert_eq!(feature.name, "");
        assert_eq!(feature.keys.unwrap(), ["if"]);
    }

    #[test]
    fn test_parse_invalid_unknown_field() {
        let input = "{'name':'feature_name','unknown':'value'}";
        let (_, feature) = QapiFeature::parse(input).unwrap();
        assert_eq!(feature.keys.unwrap(), ["name", "unknown"]);
    }

    #[test]
    fn test_parse_invalid_unexpected_format() {
        let input = "invalid_input";
        let result = QapiFeature::parse(input);
        assert!(
            result.is_err(),
            "Expected parsing to fail due to invalid format"
        );
    }

    #[test]
    fn test_parse_invalid_incorrect_quotes() {
        let inputs = [
            "\"feature_name\"",
            "{'name':\"feature_name\"}",
            "{'name':'feature_name', 'if':\"condition\"}",
        ];
        for input in inputs.iter() {
            let result = QapiFeature::parse(input);
            assert!(
                result.is_err(),
                "Expected parsing to fail for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_parse_invalid_missing_quotes() {
        let inputs = [
            "'feature_name",
            "{'name':'feature_name}",
            "{'name':'feature_name', 'if':'condition}",
        ];
        for input in inputs.iter() {
            let result = QapiFeature::parse(input);
            assert!(
                result.is_err(),
                "Expected parsing to fail due to missing closing quote"
            );
        }
    }

    #[test]
    fn test_parse_trailing_comma_rejected() {
        crate::set_parser_version((11, 1, 3));
        let inputs = [
            "{'name':'feature_name',}",
            "{'name':'feature_name', 'if':'CONFIG_CONDITION',}",
        ];
        for input in inputs.iter() {
            let result = QapiFeature::parse(input);
            assert!(
                result.is_err(),
                "Modern QAPI rejects trailing commas: {}",
                input
            );
        }
    }

    #[test]
    fn test_parse_invalid_missing_wrapping_quotes() {
        let inputs = [
            "{'name':feature_name}",
            "{'name':'feature_name', 'if':condition}",
        ];
        for input in inputs.iter() {
            let result = QapiFeature::parse(input);
            assert!(
                result.is_err(),
                "Expected parsing to fail due to missing wrapping quotes"
            );
        }
    }

    #[test]
    fn test_parse_invalid_extra_fields() {
        let input = "{'name':'feature_name', 'if':'CONFIG_CONDITION', 'extra':'value'}";
        let (_, feature) = QapiFeature::parse(input).unwrap();
        assert_eq!(feature.keys.unwrap(), ["name", "if", "extra"]);
    }
}
