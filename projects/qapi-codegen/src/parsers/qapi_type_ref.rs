use super::{
    common::{QapiString, qstring, take_list},
    parser_version,
    qapi_members::QapiMembers,
};
use nom::{IResult, Parser, branch::alt};

pub(crate) fn take_type_ref(input: &str) -> IResult<&str, QapiTypeRef<'_>> {
    use super::common::take_kv;
    take_kv("type", QapiTypeRef::parse).parse(input)
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum QapiTypeRef<'i> {
    #[default]
    Unset,
    Ref(QapiString<'i>),
    ArrayRef(QapiString<'i>),
    /// An anonymous struct written out in place: a dictionary before 4.0.
    InlineStruct(QapiMembers<'i>),
}

impl<'i> QapiTypeRef<'i> {
    /// Dispatches to the correct versioned parser.
    pub(crate) fn parse(input: &'i str) -> IResult<&'i str, Self> {
        // Until 4.0 a dictionary in place of a type is an anonymous
        // struct, which the validation refuses from 2.4 where QEMU does;
        // 4.0 made `{ 'type': ... }` the long form of a member.
        let v = parser_version();
        if v < (4, 0, 0) {
            Self::parse_v1(input)
        } else {
            Self::parse_v2(input)
        }
    }

    /// Before 4.0: a string, an array, or an anonymous struct.
    pub(crate) fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        use super::qapi_members::take_members;
        let array_parser =
            take_list(qstring).map_opt(|items| match items.as_slice() {
                [name] => Some(name.clone()),
                _ => None,
            });
        alt((
            qstring.map(Self::Ref),
            array_parser.map(Self::ArrayRef),
            take_members.map(Self::InlineStruct),
        ))
        .parse(input)
    }

    /// From 4.0: only a string or an array.
    pub(crate) fn parse_v2(input: &'i str) -> IResult<&'i str, Self> {
        let array_parser =
            take_list(qstring).map_opt(|items| match items.as_slice() {
                [name] => Some(name.clone()),
                _ => None,
            });
        alt((qstring.map(Self::Ref), array_parser.map(Self::ArrayRef)))
            .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::set_parser_version;

    #[test]
    fn arrays_require_one_element_and_follow_versioned_separator_rules() {
        for version in [(1, 0, 0), (11, 1, 3)] {
            set_parser_version(version);
            assert!(QapiTypeRef::parse("[]").is_err());
            assert!(QapiTypeRef::parse("['str', 'int']").is_err());
            assert_eq!(
                QapiTypeRef::parse("['str',]").is_ok(),
                version < (2, 4, 0)
            );
        }
    }

    #[test]
    fn test_string_ref() {
        set_parser_version((10, 2, 0));
        let (rem, parsed) = QapiTypeRef::parse("'str'").unwrap();
        assert_eq!(rem, "");
        assert_eq!(parsed, QapiTypeRef::Ref("str".into()));
    }

    #[test]
    fn test_array_ref() {
        set_parser_version((10, 2, 0));
        let (rem, parsed) = QapiTypeRef::parse("['int']").unwrap();
        assert_eq!(rem, "");
        assert_eq!(parsed, QapiTypeRef::ArrayRef("int".into()));
    }

    #[test]
    fn test_inline_struct_not_parsed_in_v2() {
        set_parser_version((10, 2, 0));
        let input = "{ 'major': 'int', 'minor': 'int' }";
        let result = QapiTypeRef::parse(input);
        assert!(result.is_err(), "v2 should not accept inline structs");
    }

    #[test]
    fn test_inline_struct_parsed_in_v1() {
        set_parser_version((3, 1, 0));
        let input = "{ 'major': 'int', 'minor': 'int' }";
        let result = QapiTypeRef::parse(input);
        assert!(result.is_ok(), "v1 should accept inline structs");
        let (rem, parsed) = result.unwrap();
        assert_eq!(rem, "");
        match parsed {
            QapiTypeRef::InlineStruct(members) => {
                assert_eq!(members.len(), 2);
            }
            _ => panic!("Expected InlineStruct"),
        }
    }
}
