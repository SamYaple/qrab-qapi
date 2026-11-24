use super::common::QapiString;
use super::common::{expression_text, qstring, qtag, take_kv};
use nom::IResult;
use nom::Parser;
use nom::sequence::delimited;

pub fn take_include(input: &str) -> IResult<&str, QapiInclude<'_>> {
    QapiInclude::parse(input)
}

/// An include directive: the file name it gives, and its text in its
/// schema file, from `{` to `}`.
#[derive(Debug, Clone)]
pub struct QapiInclude<'i>(pub QapiString<'i>, pub &'i str);
impl<'i> QapiInclude<'i> {
    /// INCLUDE = { 'include': STRING }, from 2.1: earlier releases skip it.
    pub fn parse(input: &'i str) -> IResult<&'i str, Self> {
        if super::parser_version() < (2, 1, 0) {
            let error = nom::error::Error::new(input, nom::error::ErrorKind::Tag);
            return Err(nom::Err::Error(error));
        }
        Self::parse_v1(input)
    }

    pub fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let (rest, name) =
            delimited(qtag("{"), take_kv("include", qstring), qtag("}")).parse(input)?;
        Ok((rest, Self(name, expression_text(input, rest))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_INPUTS: [&str; 3] = [
        "{'include':'path/to/file.json'}",
        "{ 'include' : '../file.json' }",
        r#"{ # comment
            'include' : # comment
            '../file.json' # comemnt
        }"#,
    ];

    #[test]
    fn test_valid() {
        for input in VALID_INPUTS {
            let result = QapiInclude::parse(input);
            match result {
                Ok((remaining, _)) => {
                    assert_eq!(remaining, "");
                }
                _ => panic!("Failed to parse: ```\n{input}\n```"),
            }
        }
    }
}
