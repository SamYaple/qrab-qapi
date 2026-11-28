use super::{
    common::{
        Malformed, QapiString, expression_text, keep, malformed_key, other_key,
        qstring, qtag, read_since, take_keyed_dict, take_kv,
    },
    parser_version,
    qapi_branches::{QapiBranches, take_branches},
    qapi_cond::{QapiCond, take_cond},
    qapi_documentation::QapiDocumentation,
    qapi_features::{QapiFeatures, take_features},
    qapi_members::{MembersOrRef, take_members_or_ref},
};
use nom::{
    IResult, Parser,
    branch::alt,
    combinator::opt,
    error::{Error, ErrorKind},
};

/// The keys whose `null` QEMU 2.4 to 4.1 take for an absent key.
const ABSENT: &[&str] = &["base", "discriminator", "if"];

/// The keys that the parser reads; from 2.4 the others are kept for the
/// validation.
const READ: &[&str] =
    &["union", "data", "discriminator", "if", "features", "base"];
#[derive(Debug, Clone, Default)]
pub struct QapiUnion<'i> {
    pub name: QapiString<'i>,
    pub data: QapiBranches<'i>,
    pub base: MembersOrRef<'i>,
    /// Named for flat unions; absent for simple unions and legacy alternates.
    pub discriminator: Option<QapiString<'i>>,
    /// An empty discriminator object denoted an untagged alternate.
    pub legacy_alternate: bool,
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

impl<'i> QapiUnion<'i> {
    pub(crate) fn parse(input: &'i str) -> IResult<&'i str, Self> {
        let v = parser_version();
        if v < (1, 6, 0) {
            Self::parse_v1(input)
        } else if v < (6, 2, 0) {
            Self::parse_v2(input)
        } else {
            Self::parse_v3(input)
        }
    }

    /// v1 (QEMU v0.15–v1.5): simple unions only. Their generators read
    /// nothing but `'union'` and `'data'`, so a `'base'` or `'discriminator'`
    /// of later releases is ignored.
    pub(crate) fn parse_v1(input: &'i str) -> IResult<&'i str, Self> {
        let (input, doc) = opt(QapiDocumentation::parse).parse(input)?;
        let mut s = Self { doc, ..Default::default() };
        let start = input;
        let mut has_data = false;
        let (input, (_, keys)) = take_keyed_dict(alt((
            take_kv("union", qstring).map(|v| s.name = v),
            take_kv("data", take_branches).map(|v| {
                has_data = true;
                s.data = v;
            }),
            other_key(READ),
            malformed_key(READ)
                .map(|entry| keep(&mut s.malformed, ABSENT, entry)),
        )))
        .parse(input)?;
        if !super::common::named(&keys, "union", &s.name) || !has_data {
            return Err(nom::Err::Error(Error::new(start, ErrorKind::Tag)));
        }
        s.keys = keys;
        s.text = expression_text(start, input);
        Ok((input, s))
    }

    /// v2 (QEMU v1.6.0–v6.1.0): simple unions + discriminated unions coexist.
    pub(crate) fn parse_v2(input: &'i str) -> IResult<&'i str, Self> {
        let (input, doc) = opt(QapiDocumentation::parse).parse(input)?;
        let mut s = Self { doc, ..Default::default() };
        let start = input;
        let mut has_data = false;
        let (input, (_, keys)) = take_keyed_dict(alt((
            take_kv("union", qstring).map(|v| s.name = v),
            take_kv("data", take_branches).map(|v| {
                has_data = true;
                s.data = v;
            }),
            take_kv("discriminator", qstring)
                .map(|v| s.discriminator = Some(v)),
            // Historical untagged alternate syntax.
            take_kv("discriminator", (qtag("{"), qtag("}")))
                .map(|_| s.legacy_alternate = true),
            read_since((2, 4, 0), take_cond).map(|v| s.r#if = Some(v)),
            read_since((2, 4, 0), take_features).map(|v| s.features = Some(v)),
            take_kv("base", take_members_or_ref).map(|v| s.base = v),
            other_key(READ),
            malformed_key(READ)
                .map(|entry| keep(&mut s.malformed, ABSENT, entry)),
        )))
        .parse(input)?;
        // From 2.4 the validation refuses a missing 'data'.
        if !super::common::named(&keys, "union", &s.name)
            || (!has_data && parser_version() < (2, 4, 0))
        {
            return Err(nom::Err::Error(Error::new(start, ErrorKind::Tag)));
        }
        // Before 2.4 an empty discriminator names none: a simple union.
        if s.discriminator.as_deref() == Some("")
            && parser_version() < (2, 4, 0)
        {
            s.discriminator = None;
        }
        s.keys = keys;
        s.text = expression_text(start, input);
        // v2: both simple (no discriminator) and flat (with discriminator) are valid
        Ok((input, s))
    }

    /// v3 (QEMU v6.2.0+): discriminated (flat) unions only — base + discriminator required.
    pub(crate) fn parse_v3(input: &'i str) -> IResult<&'i str, Self> {
        let (input, doc) = opt(QapiDocumentation::parse).parse(input)?;
        let mut s = Self { doc, ..Default::default() };
        let start = input;
        let mut has_data = false;
        let (input, (_, keys)) = take_keyed_dict(alt((
            take_kv("union", qstring).map(|v| s.name = v),
            take_kv("data", take_branches).map(|v| {
                has_data = true;
                s.data = v;
            }),
            take_kv("discriminator", qstring)
                .map(|v| s.discriminator = Some(v)),
            take_cond.map(|v| s.r#if = Some(v)),
            take_features.map(|v| s.features = Some(v)),
            take_kv("base", take_members_or_ref).map(|v| s.base = v),
            other_key(READ),
            malformed_key(READ)
                .map(|entry| keep(&mut s.malformed, ABSENT, entry)),
        )))
        .parse(input)?;
        if !super::common::named(&keys, "union", &s.name) {
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
    fn flat_unions_allow_implicit_empty_payloads_but_require_data() {
        for version in [(2, 1, 0), (6, 2, 0), (11, 1, 3)] {
            set_parser_version(version);
            let prefix = "{ 'union': 'EmptyPayload', 'base': {'kind': 'Kind'}, 'discriminator': 'kind'";
            // From 2.4 the validation refuses a missing 'data'.
            let missing = format!("{prefix} }}");
            assert_eq!(
                QapiUnion::parse(&missing).is_err(),
                version < (2, 4, 0)
            );
            let input = format!("{prefix}, 'data': {{}} }}");
            let (remaining, value) = QapiUnion::parse(&input).unwrap();
            assert!(remaining.is_empty());
            assert!(value.data.is_empty());
        }
    }

    #[test]
    fn empty_discriminator_is_distinct_from_simple_union_and_rejects_nonempty_objects()
     {
        set_parser_version((1, 7, 0));
        let (_, alternate) = QapiUnion::parse(
            "{ 'union': 'Ref', 'discriminator': {}, 'data': {'reference':'str'} }",
        )
        .unwrap();
        assert!(alternate.legacy_alternate);
        let (_, tagged) =
            QapiUnion::parse("{ 'union': 'Ref', 'data': {'reference':'str'} }")
                .unwrap();
        assert!(!tagged.legacy_alternate);
        assert!(
            QapiUnion::parse(
                "{ 'union': 'Ref', 'discriminator': {'ignored'}, 'data': {'reference':'str'} }"
            )
            .is_err()
        );
        // Before 2.4 the lowering checks a legacy alternate's base, from
        // 2.4 the validation.
        let legacy = "{ 'union': 'Ref', 'discriminator': {}, 'base': 'Base', 'data': {'reference':'str'} }";
        assert!(QapiUnion::parse(legacy).unwrap().1.legacy_alternate);
        set_parser_version((2, 4, 0));
        assert!(QapiUnion::parse(legacy).unwrap().1.legacy_alternate);
    }

    #[test]
    fn test_flat_union_v3() {
        set_parser_version((10, 2, 0));
        let input = "{'union':'SOMENAME', 'discriminator': 'testdiscrim', 'base': 'testbase', 'data':{'branchname':'branchtype'}}";
        let (rem, u) = QapiUnion::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(u.name, "SOMENAME");
        assert_eq!(u.discriminator, Some("testdiscrim".into()));
    }

    #[test]
    fn a_simple_union_in_v3_keeps_its_keys_for_the_validation() {
        set_parser_version((10, 2, 0));
        let input = "{ 'union': 'Test', 'data': { 'a': 'TypeA' } }";
        let (_, value) = QapiUnion::parse(input).unwrap();
        assert_eq!(value.keys, ["union", "data"]);
    }

    #[test]
    fn test_simple_union_accepted_in_v2() {
        set_parser_version((2, 0, 0));
        let input = "{ 'union': 'BlockdevAction', 'data': { 'blockdev-snapshot-sync': 'BlockdevSnapshot' } }";
        let (rem, u) = QapiUnion::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(u.name, "BlockdevAction");
        assert!(u.discriminator.is_none());
        assert_eq!(u.base, MembersOrRef::Unset);
    }

    #[test]
    fn test_empty_discriminator_v1() {
        set_parser_version((1, 6, 0));
        let input = "{ 'union': 'BlockdevRef', 'discriminator': {}, 'data': { 'definition': 'BlockdevOptions', 'reference': 'str' } }";
        let (rem, u) = QapiUnion::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(u.name, "BlockdevRef");
        assert!(u.discriminator.is_none());
        assert!(u.legacy_alternate);
    }

    #[test]
    fn unions_before_1_6_ignore_the_keys_of_later_releases() {
        set_parser_version((1, 5, 0));
        for input in [
            "{ 'union': 'U', 'discriminator': {}, 'data': { 'a': 'str' } }",
            "{ 'union': 'U', 'base': 'B', 'discriminator': 'kind', 'data': { 'a': 'str' } }",
            "{ 'union': 'U', 'data': { 'a': 'str' }, 'if': 'CONFIG_X', 'gen': [ 'any', { } ] }",
        ] {
            let (rem, u) = QapiUnion::parse(input).unwrap();
            assert_eq!(rem, "", "{input}");
            assert!(
                u.discriminator.is_none()
                    && !u.legacy_alternate
                    && u.r#if.is_none()
            );
            assert_eq!(u.base, MembersOrRef::Unset);
            assert_eq!(u.data.len(), 1);
        }
        set_parser_version((1, 6, 0));
        let (_, flat) = QapiUnion::parse(
            "{ 'union': 'U', 'base': 'B', 'discriminator': 'kind', 'data': { 'a': 'str' } }",
        )
        .unwrap();
        assert_eq!(flat.discriminator, Some("kind".into()));
        assert_eq!(flat.base, MembersOrRef::Ref("B".into()));
    }

    #[test]
    fn test_flat_union_accepted_in_v2() {
        set_parser_version((3, 0, 0));
        let input = "{'union': 'TestUnion', 'discriminator': 'type', 'base': 'TestBase', 'data': {'a': 'TypeA'}}";
        let (rem, u) = QapiUnion::parse(input).unwrap();
        assert_eq!(rem, "");
        assert_eq!(u.discriminator, Some("type".into()));
    }
}
