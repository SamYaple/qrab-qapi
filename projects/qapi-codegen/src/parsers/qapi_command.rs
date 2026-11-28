use super::{
    common::{
        Malformed, QapiString, expression_text, keep, malformed_key, other_key,
        qbool, qstring, read_since, take_keyed_dict, take_kv,
    },
    parser_version,
    qapi_cond::{QapiCond, take_cond},
    qapi_documentation::QapiDocumentation,
    qapi_features::{QapiFeatures, take_features},
    qapi_members::{MembersOrRef, take_members_or_ref},
    qapi_type_ref::QapiTypeRef,
};
use nom::{
    IResult, Parser,
    branch::alt,
    combinator::opt,
    error::{Error, ErrorKind},
};

/// The keys whose `null` QEMU 2.4 to 4.1 take for an absent key.
const ABSENT: &[&str] = &["data", "returns", "if"];

/// The keys that the parser reads; from 2.4 the others are kept for the
/// validation.
const READ: &[&str] = &[
    "command",
    "boxed",
    "if",
    "features",
    "returns",
    "success-response",
    "gen",
    "allow-oob",
    "coroutine",
    "allow-preconfig",
    "data",
];
#[derive(Debug, Clone, Default)]
pub struct QapiCommand<'i> {
    pub name: QapiString<'i>,
    pub data: Option<MembersOrRef<'i>>,
    pub boxed: Option<&'i str>,
    pub r#if: Option<QapiCond<'i>>,
    pub features: Option<QapiFeatures<'i>>,
    pub returns: Option<QapiTypeRef<'i>>,
    pub success_response: Option<QapiString<'i>>,
    pub r#gen: Option<QapiString<'i>>,
    pub allow_oob: Option<&'i str>,
    pub allow_preconfig: Option<&'i str>,
    pub coroutine: Option<&'i str>,
    pub doc: Option<QapiDocumentation<'i>>,
    /// From 2.4, the keys whose values the grammar refused, for the
    /// validation.
    pub malformed: Malformed<'i>,
    /// The keys of the definition's expression, in their order there.
    pub keys: Vec<QapiString<'i>>,
    /// The definition's expression in its schema file, from `{` to `}`.
    pub text: &'i str,
}

impl<'i> QapiCommand<'i> {
    pub(crate) fn parse(input: &'i str) -> IResult<&'i str, Self> {
        let v = parser_version();
        if v < (2, 4, 0) {
            Self::parse_v1(input)
        } else {
            Self::parse_v2(input)
        }
    }

    /// v1 (QEMU v1.0–v2.3.0): accepts `'gen': 'no'`/`'gen': 'yes'` (string
    /// booleans).
    pub(crate) fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        Self::parse_common(input, true)
    }

    /// v2 (QEMU v2.4.0+): only `'gen': false`/`'gen': true` (real booleans).
    pub(crate) fn parse_v2(input: &'i str) -> IResult<&'i str, Self> {
        Self::parse_common(input, false)
    }

    fn parse_common(
        input: &'i str,
        gen_string_as_bool: bool,
    ) -> IResult<&'i str, Self> {
        let (input, doc) = opt(QapiDocumentation::parse).parse(input)?;
        let mut s = Self { doc, ..Default::default() };
        let start = input;
        let flag = move |input: &'i str| -> IResult<&'i str, QapiString<'i>> {
            if gen_string_as_bool {
                alt((qbool.map(QapiString::Borrowed), qstring)).parse(input)
            } else {
                qbool.map(QapiString::Borrowed).parse(input)
            }
        };
        let (input, (_, keys)) = take_keyed_dict(alt((
            take_kv("command", qstring).map(|v| s.name = v),
            read_since((2, 4, 0), take_kv("boxed", qbool))
                .map(|v| s.boxed = Some(v)),
            read_since((2, 4, 0), take_cond).map(|v| s.r#if = Some(v)),
            read_since((2, 4, 0), take_features).map(|v| s.features = Some(v)),
            take_kv("returns", QapiTypeRef::parse).map(|v| s.returns = Some(v)),
            read_since((1, 1, 0), take_kv("success-response", flag))
                .map(|v| s.success_response = Some(v)),
            read_since((1, 1, 0), take_kv("gen", flag))
                .map(|v| s.r#gen = Some(v)),
            read_since((2, 4, 0), take_kv("allow-oob", qbool))
                .map(|v| s.allow_oob = Some(v)),
            read_since((2, 4, 0), take_kv("coroutine", qbool))
                .map(|v| s.coroutine = Some(v)),
            read_since((2, 4, 0), take_kv("allow-preconfig", qbool))
                .map(|v| s.allow_preconfig = Some(v)),
            take_kv("data", take_members_or_ref).map(|v| s.data = Some(v)),
            other_key(READ),
            malformed_key(READ)
                .map(|entry| keep(&mut s.malformed, ABSENT, entry)),
        )))
        .parse(input)?;
        if !super::common::named(&keys, "command", &s.name) {
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

    #[test]
    fn test_valid_v2() {
        set_parser_version((10, 2, 0));
        let input = r#"{ 'command': 'block-set-write-threshold',
  'data': { 'node-name': 'str', 'write-threshold': 'uint64' },
  'allow-preconfig': true }"#;
        let (rem, cmd) = QapiCommand::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(cmd.name, "block-set-write-threshold");
    }

    #[test]
    fn test_gen_string_accepted_in_v1() {
        set_parser_version((1, 1, 0));
        let input =
            "{ 'command': 'query-name', 'returns': 'NameInfo', 'gen': 'no' }";
        let (rem, cmd) = QapiCommand::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(cmd.name, "query-name");
        assert_eq!(cmd.r#gen, Some("no".into()));
    }

    #[test]
    fn test_gen_string_rejected_in_v2() {
        set_parser_version((10, 2, 0));
        let input = "{ 'command': 'test', 'gen': 'no' }";
        // Kept for the validation, which refuses the flag's value.
        let (_, command) = QapiCommand::parse(input).unwrap();
        assert_eq!(command.r#gen, None);
        assert_eq!(command.malformed[0].0, "gen");
        set_parser_version((2, 3, 0));
        assert_eq!(
            QapiCommand::parse(input).unwrap().1.r#gen,
            Some("no".into())
        );
    }

    #[test]
    fn test_gen_bool_accepted_in_v2() {
        set_parser_version((10, 2, 0));
        let input = "{ 'command': 'test', 'gen': false }";
        let (rem, cmd) = QapiCommand::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(cmd.r#gen, Some("false".into()));
    }
}
