use super::validate;
use crate::unified::{NameRule, SchemaError, SchemaErrorKind as Kind, SchemaResult, Version};
use std::path::{Path, PathBuf};

fn check_files(version: Version, files: &[(&str, &str)]) -> SchemaResult {
    crate::set_parser_version(version);
    let sources: Vec<(PathBuf, String)> = files
        .iter()
        .map(|(path, text)| (PathBuf::from(path), text.to_string()))
        .collect();
    let schemas = crate::builtins::parse_sources(&sources, &crate::builtins::PREDEFINED.into())?;
    validate(&schemas, &sources)
}

fn check(version: Version, text: &str) -> SchemaResult {
    check_files(version, &[("schema.json", text)])
}

fn rejected(version: Version, text: &str) -> SchemaError {
    match check(version, text) {
        Ok(()) => panic!("{version:?} accepts {text}"),
        Err(error) => error,
    }
}

/// Each rule rejects in the releases that have it, and only there: the
/// expected kind, or `None` where the release accepts the schema.
fn rules(cases: &[(Version, &str, Option<Kind>)]) {
    for (version, text, expected) in cases {
        match (check(*version, text), expected) {
            (Ok(()), None) => {}
            (Err(error), Some(kind)) => {
                assert_eq!(error.kind(), kind, "{version:?} {text}: {error}")
            }
            (Ok(()), Some(kind)) => panic!("{version:?} accepts {text}, expected {kind:?}"),
            (Err(error), None) => panic!("{version:?} rejects {text}: {error}"),
        }
    }
}

fn unknown_key(key: &str) -> Option<Kind> {
    Some(Kind::UnknownKey { key: key.into() })
}

fn case(name: &str, rule: NameRule) -> Option<Kind> {
    Some(Kind::NameCase {
        name: name.into(),
        rule,
    })
}

fn clash(name: &str, previous: &str) -> Option<Kind> {
    Some(Kind::MemberClash {
        name: name.into(),
        previous: previous.into(),
    })
}

fn not_allowed(meta: &'static str, reference: &str) -> Option<Kind> {
    Some(Kind::TypeNotAllowed {
        meta,
        reference: reference.into(),
    })
}

const FLAT: &str = "{'enum':'Ee','data':['a','b']} {'struct':'Ss','data':{'x':'int'}}
    {'struct':'Bb','data':{'d':'Ee'}}";

#[test]
fn before_2_4_only_unions_and_event_data_are_checked() {
    let flat = |base: &str, data: &str| {
        format!(
            "{{'enum':'Ee','data':['a']}} {{'type':'Bb','data':{{'d':'Ee','s':'str'}}}}
             {{'union':'Uu','base':{base},'discriminator':'d','data':{data}}}"
        )
    };
    let missing = flat("'Bb'", "{'a':'Nope'}");
    let unknown = flat("'Bb'", "{'c':'Bb'}");
    let not_struct = flat("'Ee'", "{}");
    let string = flat("'Bb'", "{}").replace("'discriminator':'d'", "'discriminator':'s'");
    let no_base = "{'enum':'Ee','data':['a']} {'union':'Uu','discriminator':'d','data':{}}";
    let nested = "{'event':'EV','data':{'a':{'b':'int'}}}";
    rules(&[
        ((1, 7, 0), &unknown, None),
        ((1, 7, 0), &not_struct, None),
        ((1, 7, 0), &string, None),
        ((1, 7, 0), no_base, None),
        ((2, 0, 0), nested, None),
        ((2, 3, 0), &missing, None),
        (
            (2, 0, 0),
            &unknown,
            Some(Kind::UnknownBranch {
                branch: "c".into(),
                discriminator_type: "Ee".into(),
            }),
        ),
        ((2, 0, 0), &not_struct, not_allowed("enum", "'Ee'")),
        (
            (2, 3, 0),
            &string,
            Some(Kind::DiscriminatorNotEnum {
                discriminator: "s".into(),
                discriminator_type: "str".into(),
            }),
        ),
        ((2, 0, 0), no_base, Some(Kind::MissingBase)),
        ((2, 1, 0), nested, Some(Kind::NestedEventData)),
        (
            (2, 3, 0),
            "{'struct':'Foo','data':{'a':'Nope','B':'int'}}",
            None,
        ),
        (
            (2, 3, 0),
            "{'command':'Foo_Bar','gen':'no'} {'command':'Foo_Bar'}",
            None,
        ),
    ]);
}

#[test]
fn keys_are_unknown_before_their_release() {
    rules(&[
        (
            (2, 4, 0),
            "{'enum':'Foo','data':[],'prefix':'BAR'}",
            unknown_key("prefix"),
        ),
        ((2, 5, 0), "{'enum':'Foo','data':[],'prefix':'BAR'}", None),
        (
            (2, 6, 0),
            "{'command':'foo','boxed':true,'data':'Foo'} {'struct':'Foo','data':{'a':'int'}}",
            unknown_key("boxed"),
        ),
        (
            (2, 7, 0),
            "{'command':'foo','boxed':true,'data':'Foo'} {'struct':'Foo','data':{'a':'int'}}",
            None,
        ),
        (
            (2, 6, 0),
            "{'event':'FOO','boxed':true,'data':'Foo'} {'struct':'Foo','data':{'a':'int'}}",
            unknown_key("boxed"),
        ),
        (
            (2, 11, 0),
            "{'command':'foo','allow-oob':true}",
            unknown_key("allow-oob"),
        ),
        ((2, 12, 0), "{'command':'foo','allow-oob':true}", None),
        (
            (2, 12, 0),
            "{'command':'foo','allow-preconfig':true}",
            unknown_key("allow-preconfig"),
        ),
        ((3, 0, 0), "{'command':'foo','allow-preconfig':true}", None),
        (
            (2, 12, 0),
            "{'struct':'Foo','data':{},'if':'CONFIG_FOO'}",
            unknown_key("if"),
        ),
        (
            (3, 0, 0),
            "{'struct':'Foo','data':{},'if':'CONFIG_FOO'}",
            None,
        ),
        // Before 4.0 a member's dictionary is an anonymous struct.
        (
            (3, 1, 0),
            "{'struct':'Foo','data':{'a':{'type':'int','if':'CONFIG_A'}}}",
            Some(Kind::AnonymousStruct),
        ),
        (
            (4, 0, 0),
            "{'struct':'Foo','data':{'a':{'type':'int','if':'CONFIG_A'}}}",
            None,
        ),
        (
            (3, 1, 0),
            "{'enum':'Foo','data':[{'name':'a','if':'CONFIG_A'}]}",
            Some(Kind::NameNotString),
        ),
        (
            (4, 0, 0),
            "{'enum':'Foo','data':[{'name':'a','if':'CONFIG_A'}]}",
            None,
        ),
        (
            (4, 0, 0),
            "{'struct':'Foo','data':{},'features':['f']}",
            unknown_key("features"),
        ),
        (
            (4, 1, 0),
            "{'struct':'Foo','data':{},'features':[{'name':'f','if':'CONFIG_F'}]}",
            None,
        ),
        (
            (4, 1, 0),
            "{'command':'foo','features':['f']}",
            unknown_key("features"),
        ),
        ((4, 2, 0), "{'command':'foo','features':['f']}", None),
        (
            (4, 2, 0),
            "{'enum':'Foo','data':[],'features':['f']}",
            unknown_key("features"),
        ),
        ((5, 0, 0), "{'enum':'Foo','data':[],'features':['f']}", None),
        (
            (4, 2, 0),
            "{'struct':'Foo','data':{'a':{'type':'int','features':['f']}}}",
            unknown_key("features"),
        ),
        (
            (5, 0, 0),
            "{'struct':'Foo','data':{'a':{'type':'int','features':['f']}}}",
            None,
        ),
        (
            (6, 1, 0),
            "{'enum':'Foo','data':[{'name':'a','features':['f']}]}",
            unknown_key("features"),
        ),
        (
            (6, 2, 0),
            "{'enum':'Foo','data':[{'name':'a','features':['f']}]}",
            None,
        ),
        (
            (5, 1, 0),
            "{'command':'foo','coroutine':true}",
            unknown_key("coroutine"),
        ),
        ((5, 2, 0), "{'command':'foo','coroutine':true}", None),
    ]);
    let error = rejected((3, 1, 0), "{'enum':'Foo','data':['a'],'if':''}");
    assert_eq!(
        error.kind(),
        &Kind::EmptyCondition {
            condition: "''".into()
        }
    );
}

#[test]
fn flags_take_one_value_and_out_of_band_excludes_coroutines() {
    let flag = |flag, value| Some(Kind::FlagValue { flag, value });
    rules(&[
        ((2, 3, 0), "{'command':'foo','gen':'yes'}", None),
        (
            (2, 4, 0),
            "{'command':'foo','gen':true}",
            flag("gen", false),
        ),
        (
            (2, 4, 0),
            "{'command':'foo','success-response':true}",
            flag("success-response", false),
        ),
        (
            (4, 1, 0),
            "{'command':'foo','gen':true}",
            flag("gen", false),
        ),
        (
            (2, 7, 0),
            "{'struct':'Foo','data':{}} {'command':'foo','boxed':false,'data':'Foo'}",
            flag("boxed", true),
        ),
        (
            (11, 1, 3),
            "{'command':'foo','boxed':false,'data':{'a':'int'}}",
            flag("boxed", true),
        ),
        (
            (2, 12, 0),
            "{'command':'foo','allow-oob':false}",
            flag("allow-oob", true),
        ),
        (
            (11, 1, 3),
            "{'command':'foo','allow-preconfig':false}",
            flag("allow-preconfig", true),
        ),
        (
            (5, 2, 0),
            "{'command':'foo','allow-oob':true,'coroutine':true}",
            Some(Kind::IncompatibleFlags),
        ),
    ]);
}

#[test]
fn names_follow_each_releases_rules() {
    rules(&[
        ((2, 3, 0), "{'struct':'1Foo','data':{'a b':'int'}}", None),
        (
            (2, 4, 0),
            "{'struct':'Foo','data':{'a b':'int'}}",
            Some(Kind::InvalidName { name: "a b".into() }),
        ),
        ((2, 4, 0), "{'enum':'Foo','data':['1a']}", None),
        (
            (2, 4, 0),
            "{'union':'Foo','data':{'1a':'int'}}",
            Some(Kind::InvalidName { name: "1a".into() }),
        ),
        (
            (2, 4, 0),
            "{'alternate':'Foo','data':{'1a':'int','b':'str'}}",
            Some(Kind::InvalidName { name: "1a".into() }),
        ),
        (
            (4, 1, 0),
            "{'struct':'Foo','data':{},'features':['a b']}",
            Some(Kind::InvalidName { name: "a b".into() }),
        ),
        // From 6.0 a flat union's branches are only enum values.
        (
            (5, 2, 0),
            &format!(
                "{FLAT} {{'enum':'Ed','data':['1a']}} {{'struct':'Bd','data':{{'d':'Ed'}}}} {{'union':'Uu','base':'Bd','discriminator':'d','data':{{'1a':'Ss'}}}}"
            ),
            Some(Kind::InvalidName { name: "1a".into() }),
        ),
        (
            (6, 0, 0),
            &format!(
                "{FLAT} {{'enum':'Ed','data':['1a']}} {{'struct':'Bd','data':{{'d':'Ed'}}}} {{'union':'Uu','base':'Bd','discriminator':'d','data':{{'1a':'Ss'}}}}"
            ),
            None,
        ),
        (
            (2, 4, 0),
            "{'event':'max'}",
            Some(Kind::ReservedName { name: "max".into() }),
        ),
        (
            (2, 5, 0),
            "{'event':'MAX'}",
            Some(Kind::ReservedName { name: "MAX".into() }),
        ),
        ((2, 6, 0), "{'event':'MAX'}", None),
    ]);
}

#[test]
fn reserved_member_names_follow_the_c_name_of_each_release() {
    rules(&[
        (
            (2, 4, 0),
            "{'struct':'Foo','data':{'has-a':'int','u':'int'}}",
            None,
        ),
        (
            (2, 5, 0),
            "{'struct':'Foo','data':{'has-a':'int'}}",
            Some(Kind::ReservedName {
                name: "has-a".into(),
            }),
        ),
        (
            (2, 5, 0),
            "{'command':'foo','data':{'u':'int'}}",
            Some(Kind::ReservedName { name: "u".into() }),
        ),
        (
            (5, 2, 0),
            "{'struct':'Foo','data':{'*u':'int','*has-a':'int'}}",
            None,
        ),
        (
            (6, 0, 0),
            "{'struct':'Foo','data':{'*u':'int'}}",
            Some(Kind::ReservedName { name: "*u".into() }),
        ),
        (
            (6, 0, 0),
            &format!(
                "{FLAT} {{'union':'Uu','base':{{'d':'Ee','*has-x':'int'}},'discriminator':'d','data':{{}}}}"
            ),
            Some(Kind::ReservedName {
                name: "*has-x".into(),
            }),
        ),
        // Only members written out in place are checked.
        ((11, 1, 3), "{'enum':'Foo','data':['u','has-a']}", None),
    ]);
}

#[test]
fn case_rules_follow_each_release_and_its_pragmas() {
    use NameRule::*;
    rules(&[
        ((2, 5, 0), "{'struct':'Foo','data':{'Abc':'int'}}", None),
        (
            (2, 6, 0),
            "{'struct':'Foo','data':{'Abc':'int'}}",
            case("Abc", NoUppercase),
        ),
        (
            (2, 6, 0),
            "{'struct':'UuidInfo','data':{'Abc':'int'}}",
            None,
        ),
        (
            (2, 6, 0),
            "{'enum':'Foo','data':['Abc']}",
            case("Abc", NoUppercase),
        ),
        (
            (2, 6, 0),
            "{'command':'foo','data':{'Abc':'int'}}",
            case("Abc", NoUppercase),
        ),
        (
            (2, 6, 0),
            "{'alternate':'Foo','data':{'Ab':'int','b':'str'}}",
            case("Ab", NoUppercase),
        ),
        (
            (2, 9, 0),
            "{'struct':'UuidInfo','data':{'Abc':'int'}}",
            case("Abc", NoUppercase),
        ),
        (
            (2, 9, 0),
            "{'pragma':{'name-case-whitelist':['Foo']}} {'struct':'Foo','data':{'Abc':'int'}}",
            None,
        ),
        (
            (4, 1, 0),
            "{'struct':'Foo','data':{},'features':['Abc']}",
            case("Abc", NoUppercase),
        ),
        (
            (4, 2, 0),
            "{'struct':'Foo','data':{'Abc':'int','a_b':'int'}}",
            case("Abc", NoUppercase),
        ),
        ((5, 2, 0), "{'struct':'Foo','data':{'a_b':'int'}}", None),
        (
            (4, 2, 0),
            "{'pragma':{'name-case-whitelist':['Foo']}} {'command':'Foo','data':{'Abc':'int'}}",
            case("Abc", NoUppercase),
        ),
        (
            (6, 0, 0),
            "{'struct':'Foo','data':{'a_b':'int'}}",
            case("a_b", Lowercase),
        ),
        (
            (6, 0, 0),
            "{'pragma':{'member-name-exceptions':['Foo']}} {'struct':'Foo','data':{'a_b':'int','Abc':'int'}}",
            None,
        ),
        (
            (6, 0, 0),
            "{'pragma':{'member-name-exceptions':['Foo']}} {'union':'Foo','data':{'a_b':'int'}}",
            case("a_b", Lowercase),
        ),
        (
            (6, 0, 0),
            "{'struct':'Foo','data':{'__ORG.qemu_abc':'int','x-def':'int'}}",
            None,
        ),
        (
            (5, 2, 0),
            "{'struct':'Foo','data':{'__ORG.qemu_abc':'int'}}",
            case("__ORG.qemu_abc", NoUppercase),
        ),
        (
            (5, 2, 0),
            "{'struct':'fooBar','data':{}} {'command':'Foo_Bar'} {'event':'foo'}",
            None,
        ),
        (
            (6, 0, 0),
            "{'struct':'fooBar','data':{}}",
            case("fooBar", CamelCase),
        ),
        (
            (6, 0, 0),
            "{'struct':'X86CPU','data':{}}",
            case("X86CPU", CamelCase),
        ),
        ((6, 0, 0), "{'struct':'x-FooBar','data':{}}", None),
        (
            (6, 0, 0),
            "{'command':'foo_bar'}",
            case("foo_bar", Lowercase),
        ),
        (
            (6, 0, 0),
            "{'pragma':{'command-name-exceptions':['foo_bar']}} {'command':'foo_bar'}",
            None,
        ),
        ((6, 0, 0), "{'event':'FOO-BAR'}", case("FOO-BAR", Uppercase)),
        (
            (6, 0, 0),
            "{'struct':'Foo','data':{},'features':['a_b']}",
            case("a_b", Lowercase),
        ),
        (
            (6, 1, 0),
            "{'struct':'Foo','data':{},'features':['a_b','Abc']}",
            None,
        ),
        (
            (7, 1, 0),
            "{'struct':'Foo','data':{},'features':['a_b']}",
            case("a_b", Lowercase),
        ),
    ]);
}

#[test]
fn clashes_follow_the_c_names_of_each_release() {
    rules(&[
        (
            (2, 3, 0),
            "{'enum':'Foo','data':['foo-bar','fooBar']}",
            None,
        ),
        (
            (2, 4, 0),
            "{'enum':'Foo','data':['fooBar','foo-bar']}",
            clash("foo-bar", "fooBar"),
        ),
        (
            (2, 5, 0),
            "{'enum':'Foo','data':['max']}",
            clash("max", "(automatic)"),
        ),
        (
            (2, 6, 0),
            "{'enum':'Foo','data':['max','foo-bar','foo_bar']}",
            clash("foo_bar", "foo-bar"),
        ),
        ((2, 4, 0), "{'union':'Foo','data':{'kind':'int'}}", None),
        (
            (2, 5, 0),
            "{'union':'Foo','data':{'kind':'int'}}",
            clash("kind", "(automatic)"),
        ),
        (
            (2, 6, 0),
            "{'union':'Foo','data':{'kind':'int','a-b':'int','a_b':'str'}}",
            clash("a_b", "a-b"),
        ),
        (
            (2, 5, 0),
            "{'alternate':'Foo','data':{'max':'int','b':'str'}}",
            clash("max", "(automatic)"),
        ),
        (
            (2, 5, 0),
            "{'struct':'Base','data':{'*a':'int'}} {'struct':'Foo','base':'Base','data':{'a':'int'}}",
            Some(Kind::BaseClash {
                name: "a".into(),
                base: "Base".into(),
            }),
        ),
        (
            (2, 5, 0),
            "{'struct':'Base','data':{'a-b':'int'}} {'struct':'Foo','base':'Base','data':{'a_b':'int'}}",
            None,
        ),
        (
            (2, 6, 0),
            "{'struct':'Base','data':{'a-b':'int'}} {'struct':'Foo','base':'Base','data':{'a_b':'int'}}",
            clash("a_b", "a-b"),
        ),
        (
            (2, 4, 0),
            &format!(
                "{FLAT} {{'struct':'Sx','data':{{'d':'int'}}}} {{'union':'Uu','base':'Bb','discriminator':'d','data':{{'a':'Sx'}}}}"
            ),
            Some(Kind::BaseClash {
                name: "d".into(),
                base: "Bb".into(),
            }),
        ),
        (
            (11, 1, 3),
            &format!(
                "{FLAT} {{'struct':'Sx','data':{{'d':'int'}}}} {{'union':'Uu','base':'Bb','discriminator':'d','data':{{'a':'Sx'}}}}"
            ),
            clash("d", "d"),
        ),
        (
            (11, 1, 3),
            "{'command':'foo','data':{'a-b':'int','a_b':'int'}}",
            Some(Kind::NameCase {
                name: "a_b".into(),
                rule: NameRule::Lowercase,
            }),
        ),
        (
            (5, 2, 0),
            "{'command':'foo','data':{'a-b':'int','a_b':'int'}}",
            clash("a_b", "a-b"),
        ),
        (
            (4, 1, 0),
            "{'struct':'Foo','data':{},'features':['a-b','a_b']}",
            Some(Kind::FeatureClash {
                name: "a_b".into(),
                previous: "a-b".into(),
            }),
        ),
        (
            (5, 0, 0),
            "{'struct':'Foo','data':{'m':{'type':'int','features':['f','f']}}}",
            Some(Kind::FeatureClash {
                name: "f".into(),
                previous: "f".into(),
            }),
        ),
        (
            (11, 1, 3),
            "{'enum':'Foo','data':[{'name':'a','features':['f','f']}]}",
            None,
        ),
    ]);
}

#[test]
fn types_may_not_have_special_features_and_schemas_have_at_most_64() {
    let special = |name: &str| Some(Kind::SpecialFeature { name: name.into() });
    let many: String = (0..63)
        .map(|n| format!("{{'struct':'St{n}','data':{{}},'features':['f{n}']}}"))
        .collect();
    rules(&[
        (
            (4, 2, 0),
            "{'struct':'Foo','data':{},'features':['deprecated']}",
            None,
        ),
        (
            (5, 0, 0),
            "{'struct':'Foo','data':{},'features':['deprecated']}",
            special("deprecated"),
        ),
        (
            (5, 0, 0),
            "{'command':'foo','features':['deprecated']}",
            None,
        ),
        (
            (6, 1, 0),
            "{'enum':'Foo','data':[],'features':['unstable']}",
            None,
        ),
        (
            (6, 2, 0),
            "{'enum':'Foo','data':[],'features':['unstable']}",
            special("unstable"),
        ),
        ((9, 2, 0), &many, None),
        ((10, 0, 0), &many, Some(Kind::TooManyFeatures)),
        ((10, 0, 0), &many[..many.rfind("{'struct'").unwrap()], None),
    ]);
    let error = rejected((10, 0, 0), &many.replace("}{", "}\n{"));
    assert_eq!(error.definition(), Some("struct 'St62'"));
    assert_eq!(error.line(), Some(63));
}

#[test]
fn type_references_follow_each_release() {
    let unknown = |name: &str| Some(Kind::UnknownType { name: name.into() });
    rules(&[
        (
            (2, 3, 0),
            "{'struct':'Foo','data':{'a':'Nope'},'base':'Nope'}",
            None,
        ),
        (
            (2, 4, 0),
            "{'struct':'Foo','data':{'a':'Nope'}}",
            unknown("Nope"),
        ),
        (
            (4, 2, 0),
            "{'struct':'Foo','data':{'a':['Nope']}}",
            unknown("Nope"),
        ),
        (
            (2, 4, 0),
            "{'command':'foo'} {'struct':'Foo','data':{'a':'foo'}}",
            not_allowed("command", "'foo'"),
        ),
        (
            (4, 2, 0),
            "{'command':'foo'} {'struct':'Foo','data':{'a':'foo'}}",
            unknown("foo"),
        ),
        (
            (2, 4, 0),
            "{'struct':'Foo','data':{'a':'**'}}",
            Some(Kind::UntypedWithoutGen),
        ),
        (
            (2, 4, 0),
            "{'command':'foo','data':{'a':'**'},'returns':'**','gen':false}",
            None,
        ),
        (
            (2, 5, 0),
            "{'command':'foo','data':{'a':'any'},'returns':'**','gen':false}",
            unknown("**"),
        ),
        (
            (2, 4, 0),
            "{'enum':'Ee','data':[]} {'command':'foo','returns':'Ee'}",
            not_allowed("enum", "'Ee'"),
        ),
        (
            (2, 4, 0),
            "{'enum':'Ee','data':[]} {'command':'user_def_cmd3','returns':'Ee'}",
            None,
        ),
        (
            (2, 4, 0),
            "{'command':'qom-get','returns':'int'}",
            not_allowed("built-in", "'int'"),
        ),
        ((2, 5, 0), "{'command':'qom-get','returns':'int'}", None),
        (
            (2, 9, 0),
            "{'command':'qom-get','returns':'int'}",
            not_allowed("built-in", "'int'"),
        ),
        (
            (2, 9, 0),
            "{'pragma':{'returns-whitelist':['qom-get']}} {'command':'qom-get','returns':['int']}",
            None,
        ),
        (
            (11, 1, 3),
            "{'command':'foo','returns':['int']}",
            not_allowed("array", "['int']"),
        ),
        (
            (11, 1, 3),
            "{'pragma':{'command-returns-exceptions':['foo']}} {'command':'foo','returns':['int']}",
            None,
        ),
        (
            (11, 1, 3),
            "{'struct':'Foo','data':{}} {'command':'foo','returns':['Foo']}",
            None,
        ),
        // QEMU's own implicit tag enum of a simple union is a type.
        (
            (6, 1, 0),
            "{'union':'Uu','data':{'a':'int'}} {'struct':'Foo','data':{'k':'UuKind'}}",
            None,
        ),
        (
            (4, 2, 0),
            "{'struct':'Foo','data':{'a':'QTypeList'}}",
            unknown("QTypeList"),
        ),
    ]);
    let error = rejected((2, 4, 0), "{'struct':'Foo','data':{'a':'Nope'}}");
    assert_eq!(error.context(), ["'data'", "member 'a'"]);
}

#[test]
fn arguments_follow_each_release() {
    let union = "{'enum':'Ee','data':['a']} {'struct':'Bb','data':{'d':'Ee'}} {'struct':'Ss','data':{'x':'int'}}
        {'union':'Uu','base':'Bb','discriminator':'d','data':{'a':'Ss'}}
        {'alternate':'Al','data':{'i':'int','s':'Ss'}} {'struct':'Empty','data':{}}";
    let with = |command: &str| format!("{union} {command}");
    let unboxed = Some(Kind::UnboxedUnion { name: "Uu".into() });
    rules(&[
        ((2, 4, 0), &with("{'command':'foo','data':'Uu'}"), None),
        (
            (2, 5, 0),
            &with("{'command':'foo','data':'Uu'}"),
            not_allowed("union", "'Uu'"),
        ),
        // Until 4.1 a union is no type that unboxed arguments take.
        (
            (4, 1, 0),
            &with("{'command':'foo','data':'Uu'}"),
            not_allowed("union", "'Uu'"),
        ),
        (
            (2, 7, 0),
            &with("{'command':'foo','data':'Uu','boxed':true}"),
            None,
        ),
        (
            (4, 2, 0),
            &with("{'command':'foo','data':'Uu'}"),
            unboxed.clone(),
        ),
        (
            (11, 1, 3),
            &with("{'command':'foo','data':'Uu'}"),
            unboxed.clone(),
        ),
        (
            (4, 1, 0),
            &with("{'command':'foo','data':'Al','boxed':true}"),
            None,
        ),
        (
            (4, 2, 0),
            &with("{'command':'foo','data':'Al','boxed':true}"),
            not_allowed("alternate", "'Al'"),
        ),
        (
            (2, 7, 0),
            &with("{'event':'FOO','data':'Empty','boxed':true}"),
            Some(Kind::BoxedEmpty {
                name: "Empty".into(),
            }),
        ),
        (
            (4, 2, 0),
            &with("{'event':'FOO','data':'Empty','boxed':true}"),
            None,
        ),
        (
            (2, 7, 0),
            &with("{'command':'foo','boxed':true}"),
            Some(Kind::BoxedWithoutType),
        ),
        (
            (4, 1, 0),
            &with("{'command':'foo','boxed':true,'data':{'a':'int'}}"),
            Some(Kind::AnonymousStruct),
        ),
        (
            (11, 1, 3),
            &with("{'command':'foo','boxed':true,'data':{'a':'int'}}"),
            Some(Kind::AnonymousStruct),
        ),
        (
            (11, 1, 3),
            &with("{'command':'foo','boxed':true}"),
            Some(Kind::BoxedWithoutType),
        ),
        (
            (11, 1, 3),
            &with("{'command':'foo','data':'int'}"),
            not_allowed("built-in", "'int'"),
        ),
        (
            (8, 0, 0),
            "{'command':'foo','data':{'a':{'type':'int','if':'CONFIG_A'}}}",
            None,
        ),
        (
            (8, 1, 0),
            "{'command':'foo','data':{'a':{'type':'int','if':'CONFIG_A'}}}",
            Some(Kind::ConditionalArguments),
        ),
        (
            (8, 1, 0),
            "{'struct':'Base','data':{'a':{'type':'int','if':'CONFIG_A'}}} {'struct':'Args','base':'Base','data':{}} {'event':'FOO','data':'Args'}",
            Some(Kind::ConditionalArguments),
        ),
        (
            (8, 1, 0),
            "{'struct':'Base','data':{'a':{'type':'int','if':'CONFIG_A'}}} {'event':'FOO','data':'Base','boxed':true}",
            None,
        ),
    ]);
}

#[test]
fn alternates_follow_each_release() {
    let ambiguous = |branch: &str, previous: &str| {
        Some(Kind::AmbiguousAlternate {
            branch: branch.into(),
            previous: previous.into(),
        })
    };
    rules(&[
        ((2, 5, 0), "{'alternate':'Al','data':{'i':'int'}}", None),
        (
            (2, 6, 0),
            "{'alternate':'Al','data':{'i':'int'}}",
            Some(Kind::AlternateBranches { minimum: 2 }),
        ),
        ((4, 2, 0), "{'alternate':'Al','data':{'i':'int'}}", None),
        (
            (4, 2, 0),
            "{'alternate':'Al','data':{}}",
            Some(Kind::AlternateBranches { minimum: 1 }),
        ),
        (
            (2, 9, 0),
            "{'alternate':'Al','data':{'s':'str','i':'int','n':'number'}}",
            None,
        ),
        (
            (2, 10, 0),
            "{'alternate':'Al','data':{'s':'str','i':'int'}}",
            ambiguous("i", "s"),
        ),
        (
            (2, 10, 0),
            "{'alternate':'Al','data':{'i':'int','n':'number'}}",
            ambiguous("n", "i"),
        ),
        (
            (2, 10, 0),
            "{'enum':'Ee','data':['x']} {'alternate':'Al','data':{'e':'Ee','i':'int','b':'bool'}}",
            None,
        ),
        (
            (2, 10, 0),
            "{'enum':'Ee','data':['on']} {'alternate':'Al','data':{'e':'Ee','b':'bool'}}",
            ambiguous("b", "e"),
        ),
        (
            (2, 11, 0),
            "{'alternate':'Al','data':{'i':'int','q':'QType'}}",
            ambiguous("q", "i"),
        ),
        (
            (4, 2, 0),
            "{'alternate':'Al','data':{'i':'int','q':'QType'}}",
            None,
        ),
        (
            (2, 4, 0),
            "{'alternate':'Al','data':{'i':'int','a':'any'}}",
            Some(Kind::UnknownType { name: "any".into() }),
        ),
        (
            (2, 5, 0),
            "{'alternate':'Al','data':{'i':'int','a':'any'}}",
            Some(Kind::Crash {
                crash: crate::unified::Crash::AlternateAny { branch: "a".into() },
            }),
        ),
        (
            (2, 6, 0),
            "{'alternate':'Al','data':{'i':'int','a':'any'}}",
            not_allowed("built-in", "'any'"),
        ),
        (
            (2, 9, 0),
            "{'alternate':'Al','data':{'i':'int','n':'null'}}",
            Some(Kind::UnknownType {
                name: "null".into(),
            }),
        ),
        (
            (2, 10, 0),
            "{'alternate':'Al','data':{'i':'int','n':'null'}}",
            None,
        ),
        (
            (7, 0, 0),
            "{'alternate':'Al','data':{'i':'int','l':['int']}}",
            Some(Kind::ArrayNotAllowed),
        ),
        (
            (7, 1, 0),
            "{'alternate':'Al','data':{'i':'int','l':['int']}}",
            None,
        ),
        (
            (7, 1, 0),
            "{'alternate':'Al','data':{'l':['str'],'m':['int']}}",
            ambiguous("m", "l"),
        ),
        (
            (2, 4, 0),
            "{'alternate':'Ab','data':{'i':'int','s':'str'}} {'alternate':'Al','data':{'i':'int','a':'Ab'}}",
            not_allowed("alternate", "'Ab'"),
        ),
        (
            (11, 1, 3),
            "{'alternate':'Ab','data':{'i':'int','b':'bool'}} {'alternate':'Al','data':{'i':'int','a':'Ab'}}",
            not_allowed("alternate", "'Ab'"),
        ),
    ]);
}

#[test]
fn unions_follow_each_release() {
    let union = |base: &str, data: &str| {
        format!("{FLAT} {{'union':'Uu','base':{base},'discriminator':'d','data':{data}}}")
    };
    let empty = Some(Kind::NoBranches);
    rules(&[
        (
            (2, 4, 0),
            "{'struct':'Bb','data':{}} {'union':'Uu','base':'Bb','data':{'a':'int'}}",
            Some(Kind::BaseWithoutDiscriminator),
        ),
        (
            (2, 3, 0),
            "{'type':'Bb','data':{}} {'union':'Uu','base':'Bb','data':{'a':'int'}}",
            Some(Kind::EmptyBase { name: "Bb".into() }),
        ),
        // Before 2.4 QEMU skips a 'struct', which so defines no base.
        (
            (2, 3, 0),
            "{'struct':'Bb','data':{}} {'union':'Uu','base':'Bb','data':{'a':'int'}}",
            Some(Kind::UnknownType { name: "Bb".into() }),
        ),
        (
            (2, 3, 0),
            "{'union':'Uu','discriminator':{},'data':{'a':'int','b':'str'}}",
            None,
        ),
        (
            (2, 4, 0),
            "{'union':'Uu','discriminator':{},'data':{'a':'int','b':'str'}}",
            Some(Kind::StringBase),
        ),
        (
            (2, 5, 0),
            "{'union':'Uu','discriminator':{},'data':{'a':'int','b':'str'}}",
            Some(Kind::MissingBase),
        ),
        (
            (2, 5, 0),
            &union("{'d':'Ee'}", "{'a':'Ss'}"),
            Some(Kind::AnonymousStruct),
        ),
        ((2, 6, 0), &union("{'d':'Ee'}", "{'a':'Ss','b':'Ss'}"), None),
        (
            (2, 4, 0),
            &union("'Bb'", "{'a':'int'}"),
            not_allowed("built-in", "'int'"),
        ),
        (
            (2, 4, 0),
            &union("'Bb'", "{'a':['Ss']}"),
            Some(Kind::ArrayNotAllowed),
        ),
        (
            (8, 0, 0),
            &union("'Bb'", "{'a':['Ss']}"),
            Some(Kind::ArrayNotAllowed),
        ),
        // From 8.1 QEMU asks for a type name.
        (
            (8, 1, 0),
            &union("'Bb'", "{'a':['Ss']}"),
            Some(Kind::ValueShape {
                expected: crate::unified::Shape::TypeName,
            }),
        ),
        ((2, 4, 0), "{'union':'Uu','data':{'a':['int']}}", None),
        ((2, 5, 0), &union("'Bb'", "{}"), None),
        ((2, 6, 0), &union("'Bb'", "{}"), empty.clone()),
        ((4, 2, 0), &union("'Bb'", "{}"), None),
        ((2, 6, 0), "{'union':'Uu','data':{}}", empty.clone()),
        (
            (4, 2, 0),
            "{'enum':'Ez','data':[]} {'struct':'Bz','data':{'d':'Ez'}} {'union':'Uu','base':'Bz','discriminator':'d','data':{}}",
            empty.clone(),
        ),
        ((2, 6, 0), &union("'Bb'", "{'a':'Ss'}"), None),
        (
            (2, 7, 0),
            &union("'Bb'", "{'a':'Ss'}"),
            Some(Kind::MissingBranch { branch: "b".into() }),
        ),
        (
            (2, 12, 0),
            &union("'Bb'", "{'a':'Ss'}"),
            Some(Kind::MissingBranch { branch: "b".into() }),
        ),
        ((3, 0, 0), &union("'Bb'", "{'a':'Ss'}"), None),
        (
            (2, 4, 0),
            &union("'Bb'", "{'a':'Ss','b':'Ss'}").replace("['a','b']", "['a','b','d']"),
            None,
        ),
        (
            (2, 5, 0),
            &union("'Bb'", "{'a':'Ss','b':'Ss'}").replace("['a','b']", "['a','b','d']"),
            Some(Kind::DiscriminatorClash {
                discriminator: "d".into(),
                enumeration: "Ee".into(),
            }),
        ),
        (
            (2, 6, 0),
            &union("'Bb'", "{'a':'Ss','b':'Ss'}").replace("['a','b']", "['a','b','d']"),
            None,
        ),
        (
            (2, 7, 0),
            "{'enum':'Ee','data':['a']} {'struct':'Ss','data':{}} {'struct':'Bb','data':{'d':{'type':'Ee','if':'CONFIG_D'}}} {'union':'Uu','base':'Bb','discriminator':'d','data':{'a':'Ss'}}",
            Some(Kind::AnonymousStruct),
        ),
        // 2.9 to 3.1 look the discriminator's type up before the
        // expression checks.
        (
            (3, 1, 0),
            "{'enum':'Ee','data':['a']} {'struct':'Ss','data':{}} {'struct':'Bb','data':{'d':{'type':'Ee','if':'CONFIG_D'}}} {'union':'Uu','base':'Bb','discriminator':'d','data':{'a':'Ss'}}",
            Some(Kind::Crash {
                crash: crate::unified::Crash::UnhashableDiscriminator {
                    discriminator: "d".into(),
                },
            }),
        ),
        (
            (4, 0, 0),
            "{'enum':'Ee','data':['a']} {'struct':'Ss','data':{}} {'struct':'Bb','data':{'d':{'type':'Ee','if':'CONFIG_D'}}} {'union':'Uu','base':'Bb','discriminator':'d','data':{'a':'Ss'}}",
            Some(Kind::ConditionalDiscriminator {
                discriminator: "d".into(),
            }),
        ),
        (
            (11, 1, 3),
            "{'enum':'Ee','data':['a']} {'struct':'Ss','data':{}} {'struct':'Bb','data':{'d':{'type':'Ee','if':'CONFIG_D'}}} {'union':'Uu','base':'Bb','discriminator':'d','data':{'a':'Ss'}}",
            Some(Kind::ConditionalDiscriminator {
                discriminator: "d".into(),
            }),
        ),
        (
            (4, 1, 0),
            &format!(
                "{FLAT} {{'struct':'Bc','base':'Bb','data':{{}}}} {{'union':'Uu','base':'Bc','discriminator':'d','data':{{'a':'Ss'}}}}"
            ),
            Some(Kind::DiscriminatorNotMember {
                discriminator: "d".into(),
            }),
        ),
        (
            (4, 2, 0),
            &format!(
                "{FLAT} {{'struct':'Bc','base':'Bb','data':{{}}}} {{'union':'Uu','base':'Bc','discriminator':'d','data':{{'a':'Ss'}}}}"
            ),
            None,
        ),
        (
            (4, 1, 0),
            &union("{'*d':'Ee'}", "{'a':'Ss','b':'Ss'}"),
            Some(Kind::DiscriminatorNotMember {
                discriminator: "d".into(),
            }),
        ),
        (
            (4, 2, 0),
            &union("{'*d':'Ee'}", "{'a':'Ss','b':'Ss'}"),
            Some(Kind::OptionalDiscriminator {
                discriminator: "d".into(),
            }),
        ),
        (
            (8, 0, 0),
            &format!(
                "{FLAT} {{'union':'Uv','base':'Bb','discriminator':'d','data':{{}}}} {{'struct':'Bo','data':{{'e':'Ee'}}}} {{'union':'Uu','base':'Bo','discriminator':'e','data':{{'a':'Uv'}}}}"
            ),
            not_allowed("union", "'Uv'"),
        ),
        (
            (8, 1, 0),
            &format!(
                "{FLAT} {{'union':'Uv','base':'Bb','discriminator':'d','data':{{}}}} {{'struct':'Bo','data':{{'e':'Ee'}}}} {{'union':'Uu','base':'Bo','discriminator':'e','data':{{'a':'Uv'}}}}"
            ),
            None,
        ),
        (
            (8, 1, 0),
            &format!(
                "{FLAT} {{'union':'Uv','base':'Bb','discriminator':'d','data':{{}}}} {{'union':'Uu','base':'Bb','discriminator':'d','data':{{'a':'Uv'}}}}"
            ),
            clash("d", "d"),
        ),
        (
            (8, 1, 0),
            &format!("{FLAT} {{'union':'Uu','base':'Bb','discriminator':'d','data':{{'a':'Uu'}}}}"),
            Some(Kind::InheritanceCycle { name: "Uu".into() }),
        ),
    ]);
}

#[test]
fn branch_names_follow_each_release() {
    use NameRule::*;
    rules(&[
        (
            (5, 2, 0),
            "{'union':'Uu','data':{'Ab':'int'}}",
            case("Ab", NoUppercase),
        ),
        ((5, 2, 0), "{'union':'Uu','data':{'a_b':'int'}}", None),
        (
            (6, 0, 0),
            "{'union':'Uu','data':{'a_b':'int'}}",
            case("a_b", Lowercase),
        ),
        (
            (4, 2, 0),
            "{'alternate':'Al','data':{'Ab':'int','b':'str'}}",
            case("Ab", NoUppercase),
        ),
        (
            (6, 0, 0),
            "{'alternate':'Al','data':{'a_b':'int','b':'str'}}",
            case("a_b", Lowercase),
        ),
    ]);
}

#[test]
fn an_object_that_contains_itself_is_refused_where_the_cycle_starts() {
    let cycle = "{'struct':'Other','data':{}}\n{'struct':'First','base':'Second','data':{}}\n{'struct':'Second','base':'First','data':{}}";
    for version in [(2, 4, 0), (2, 6, 0), (11, 1, 3)] {
        let error = rejected(version, cycle);
        // 2.4 and 2.5 recurse until Python's limit.
        let expected = if version < (2, 6, 0) {
            Kind::Crash {
                crash: crate::unified::Crash::BaseCycle {
                    name: "Second".into(),
                },
            }
        } else {
            Kind::InheritanceCycle {
                name: "First".into(),
            }
        };
        assert_eq!(error.kind(), &expected, "{version:?}");
        assert_eq!(
            (error.line(), error.column()),
            (Some(2), Some(1)),
            "{version:?}"
        );
    }
}

#[test]
fn errors_name_the_definition_qemu_names() {
    // A base is checked before the definition that uses it, where its own
    // errors belong.
    let text = "{ 'struct': 'Child', 'base': 'Base', 'data': {} }\n{ 'struct': 'Other', 'data': { 'o': 'Nope' } }\n  { 'struct': 'Base', 'data': { 'b': 'Nope2' } }";
    let error = rejected((4, 2, 0), text);
    assert_eq!(error.definition(), Some("struct 'Base'"));
    assert_eq!((error.line(), error.column()), (Some(3), Some(3)));
    assert_eq!(error.file(), Some(Path::new("schema.json")));
    let error = rejected((4, 1, 0), text);
    assert_eq!(error.definition(), Some("struct 'Other'"));
    assert_eq!(error.line(), Some(2));
}

#[test]
fn from_4_2_every_expression_is_checked_before_duplicates_and_types() {
    let duplicate = "{'struct':'First','data':{'bad name':'int'}}\n{'struct':'Second','data':{}}\n{'struct':'Second','data':{}}";
    let error = rejected((4, 1, 0), duplicate);
    assert!(matches!(error.kind(), Kind::Duplicate { .. }), "{error}");
    assert_eq!(error.line(), Some(3));
    let error = rejected((4, 2, 0), duplicate);
    assert_eq!(
        error.kind(),
        &Kind::InvalidName {
            name: "bad name".into()
        }
    );
    assert_eq!(error.line(), Some(1));
    let types =
        "{'struct':'First','data':{'m':'Nope'}}\n{'struct':'Second','data':{'bad name':'int'}}";
    assert_eq!(
        error_line((4, 1, 0), types),
        (
            Kind::UnknownType {
                name: "Nope".into()
            },
            1
        )
    );
    assert_eq!(
        error_line((4, 2, 0), types),
        (
            Kind::InvalidName {
                name: "bad name".into()
            },
            2
        )
    );
}

fn error_line(version: Version, text: &str) -> (Kind, usize) {
    let error = rejected(version, text);
    (error.kind().clone(), error.line().unwrap())
}

#[test]
fn definitions_share_one_namespace_from_2_4() {
    let files = [
        ("a.json", "{'struct':'Foo','data':{}}"),
        ("b.json", "{'command':'Foo'}"),
    ];
    check_files((2, 3, 0), &files).unwrap();
    let error = check_files((2, 4, 0), &files).unwrap_err();
    assert_eq!(error.file(), Some(Path::new("b.json")));
    assert_eq!((error.line(), error.column()), (Some(1), Some(1)));
    assert_eq!(error.definition(), Some("command 'Foo'"));
    assert_eq!(
        error.kind(),
        &Kind::Duplicate {
            name: "Foo".into(),
            previous: "a.json".into(),
        }
    );
    let error = rejected(
        (11, 1, 3),
        "{'enum':'Foo','data':[]}\n  {'struct':'Foo','data':{}}",
    );
    assert_eq!((error.line(), error.column()), (Some(2), Some(3)));
}

#[test]
fn each_era_reserves_its_own_builtin_names() {
    for (version, name, reserved) in [
        ((2, 4, 0), "dict", false),
        ((2, 4, 0), "Int8", false),
        ((2, 5, 0), "any", true),
        ((2, 5, 0), "QType", false),
        ((2, 6, 0), "QType", true),
        ((2, 6, 0), "null", false),
        ((2, 10, 0), "null", true),
        ((5, 2, 0), "str", true),
    ] {
        let result = check(version, &format!("{{'enum':'{name}','data':[]}}"));
        if reserved {
            let error = result.unwrap_err();
            assert_eq!(error.definition(), Some(format!("enum '{name}'").as_str()));
            assert_eq!(
                error.kind(),
                &Kind::Predefined { name: name.into() },
                "{version:?} {name}"
            );
        } else {
            result.unwrap();
        }
    }
    // From 2.4 commands share the namespace; before, QEMU checks no names.
    rules(&[
        (
            (2, 3, 0),
            "{'type':'dict','data':{}} {'command':'int'}",
            None,
        ),
        (
            (2, 4, 0),
            "{'command':'int'}",
            Some(Kind::Predefined { name: "int".into() }),
        ),
    ]);
    // The name rules come first.
    assert_eq!(
        rejected((2, 6, 0), "{'struct':'q_empty','data':{}}").kind(),
        &Kind::InvalidName {
            name: "q_empty".into()
        }
    );
}

/// The variant of a kind, from its `Debug` form.
pub(crate) fn kind_name(debug: String) -> String {
    let name = debug.split([' ', '(', '{']).next().unwrap_or_default();
    // The shape that a value lacks, too.
    match debug.split_once("expected: ") {
        Some((_, shape)) if name == "ValueShape" => {
            format!("{name}::{}", shape.trim_end_matches([' ', '}']))
        }
        _ => name.to_owned(),
    }
}

/// Where an error is: the file's name, the line and the column.
type At = Option<(String, usize, usize)>;

fn at(file: &Path, line: usize, column: usize) -> At {
    let name = file.file_name()?.to_string_lossy().into_owned();
    Some((name, line, column))
}

/// What the pipeline of release 11.1.3 makes of one of QEMU's test schemas:
/// the kind of its error, as the fixture table writes it, or `accepted`,
/// and where the error is. The kind of a documentation fault is `Doc:` and
/// its variant, that of an include error `Include:` and its variant.
fn fixture_outcome(path: &Path) -> (String, At) {
    crate::set_parser_version((11, 1, 3));
    match crate::read_schema_with_version(path, None) {
        Err(crate::ReadError::Parse { source, path, .. }) => {
            let kind = match &source.documentation {
                Some(fault) => format!("Doc:{}", kind_name(format!("{fault:?}"))),
                None => "Parse".to_owned(),
            };
            (kind, at(&path, source.line, source.column))
        }
        Err(crate::ReadError::Include {
            from,
            line,
            column,
            kind,
        }) => (
            format!("Include:{}", kind_name(format!("{kind:?}"))),
            at(&from, line, column),
        ),
        Err(error) => (format!("Read: {error}"), None),
        Ok(sources) => match crate::unified::process_version_schema(&sources) {
            Ok(_) => ("accepted".to_owned(), None),
            Err(error) => (
                kind_name(format!("{:?}", error.kind())),
                error
                    .file()
                    .zip(error.line().zip(error.column()))
                    .and_then(|(file, (line, column))| at(file, line, column)),
            ),
        },
    }
}

/// The kind of error that each of QEMU's own negative test schemas gets at
/// the pinned release, in the file and at the line (and column, where QEMU
/// gives one) of its `.err` file: `Parse` for an expression that the
/// parser refuses, `Doc:` for a fault of a documentation comment,
/// `Include:` for an include directive.
pub(crate) const FIXTURES: &[(&str, &str)] = &[
    ("allow-preconfig-test", "FlagValue"),
    ("alternate-any", "TypeNotAllowed"),
    ("alternate-base", "UnknownKey"),
    ("alternate-branch-if-invalid", "InvalidCondition"),
    ("alternate-clash", "NameCase"),
    ("alternate-conflict-bool-string", "AmbiguousAlternate"),
    ("alternate-conflict-dict", "AmbiguousAlternate"),
    ("alternate-conflict-enum-bool", "AmbiguousAlternate"),
    ("alternate-conflict-enum-int", "AmbiguousAlternate"),
    ("alternate-conflict-lists", "AmbiguousAlternate"),
    ("alternate-conflict-num-string", "AmbiguousAlternate"),
    ("alternate-conflict-string", "AmbiguousAlternate"),
    ("alternate-data-invalid", "ValueShape::Object"),
    ("alternate-empty", "AlternateBranches"),
    ("alternate-invalid-dict", "MissingKey"),
    ("alternate-nested", "TypeNotAllowed"),
    ("alternate-unknown", "UnknownType"),
    ("args-alternate", "TypeNotAllowed"),
    ("args-any", "TypeNotAllowed"),
    ("args-array-empty", "ValueShape::ArrayOfOneTypeName"),
    ("args-array-unknown", "UnknownType"),
    ("args-bad-boxed", "FlagValue"),
    ("args-boxed-anon", "AnonymousStruct"),
    ("args-boxed-string", "TypeNotAllowed"),
    ("args-if-implicit", "ConditionalArguments"),
    ("args-if-unboxed", "ConditionalArguments"),
    ("args-int", "TypeNotAllowed"),
    ("args-invalid", "ValueShape::ObjectOrTypeName"),
    ("args-member-array-bad", "ValueShape::ArrayOfOneTypeName"),
    ("args-member-case", "NameCase"),
    ("args-member-unknown", "UnknownType"),
    ("args-union", "UnboxedUnion"),
    ("args-unknown", "UnknownType"),
    ("bad-base", "TypeNotAllowed"),
    ("bad-data", "ValueShape::ObjectOrTypeName"),
    ("bad-ident", "InvalidName"),
    ("bad-if", "ValueShape::Condition"),
    ("bad-if-all", "ValueShape::Array"),
    ("bad-if-empty", "InvalidCondition"),
    ("bad-if-empty-list", "EmptyCondition"),
    ("bad-if-key", "UnknownKey"),
    ("bad-if-keys", "ConditionKeys"),
    ("bad-if-list", "InvalidCondition"),
    ("bad-if-not", "InvalidCondition"),
    ("bad-type-bool", "NameNotString"),
    ("bad-type-dict", "NameNotString"),
    ("bad-type-int", "Parse"),
    ("base-cycle-direct", "InheritanceCycle"),
    ("base-cycle-indirect", "InheritanceCycle"),
    ("command-int", "Predefined"),
    ("doc-bad-alternate-member", "UnknownDocumentedMember"),
    ("doc-bad-boxed-command-arg", "UnknownDocumentedMember"),
    ("doc-bad-command-arg", "UnknownDocumentedMember"),
    ("doc-bad-enum-member", "UnknownDocumentedMember"),
    ("doc-bad-event-arg", "UnknownDocumentedMember"),
    ("doc-bad-feature", "UnknownDocumentedFeature"),
    ("doc-bad-indent", "Doc:UnexpectedDeIndent"),
    ("doc-bad-space-between-sentences", "Doc:SingleSpace"),
    ("doc-bad-symbol", "DocumentationSymbol"),
    ("doc-bad-union-member", "UnknownDocumentedMember"),
    ("doc-before-include", "Doc:NotFollowedByDefinition"),
    ("doc-before-pragma", "Doc:NotFollowedByDefinition"),
    ("doc-duplicate-features", "Doc:DuplicateFeatures"),
    ("doc-duplicated-arg", "Doc:DuplicateName"),
    ("doc-duplicated-return", "Doc:DuplicateSection"),
    ("doc-duplicated-since", "Doc:DuplicateSection"),
    ("doc-empty-arg", "Doc:EmptyName"),
    ("doc-empty-features", "Doc:NoFeatureDescriptions"),
    ("doc-empty-section", "Doc:EmptySection"),
    ("doc-empty-symbol", "Doc:EmptySymbol"),
    ("doc-interleaved-section", "Doc:DescriptionAfterSection"),
    ("doc-invalid-end", "Doc:Unterminated"),
    ("doc-invalid-end2", "Doc:JunkAtEnd"),
    ("doc-invalid-return", "ReturnsSection"),
    ("doc-invalid-return2", "ReturnsWithoutType"),
    ("doc-invalid-section", "Doc:NotInFreeform"),
    ("doc-invalid-start", "Doc:JunkAtStart"),
    ("doc-long-line", "Doc:LongLine"),
    ("doc-missing", "DocumentationRequired"),
    ("doc-missing-colon", "Doc:SymbolWithoutColon"),
    ("doc-missing-expr", "Doc:NotFollowedByDefinition"),
    ("doc-missing-space", "Doc:MissingSpace"),
    ("doc-no-symbol", "Doc:DefinitionDocumentationRequired"),
    ("doc-undoc-feature", "UndocumentedFeature"),
    ("double-type", "Metatypes"),
    ("duplicate-key", "Parse"),
    ("enum-bad-member", "NameNotString"),
    ("enum-bad-name", "InvalidName"),
    ("enum-bad-prefix", "ValueShape::String"),
    ("enum-clash-member", "MemberClash"),
    ("enum-dict-member-unknown", "UnknownKey"),
    ("enum-dict-no-name", "MissingKey"),
    ("enum-if-invalid", "UnknownKey"),
    ("enum-int-member", "Parse"),
    ("enum-member-case", "NameCase"),
    ("enum-missing-data", "MissingKey"),
    ("enum-wrong-data", "ValueShape::Array"),
    ("event-args-if-unboxed", "ConditionalArguments"),
    ("event-boxed-empty", "BoxedWithoutType"),
    ("event-case", "NameCase"),
    ("event-member-invalid-dict", "MissingKey"),
    ("event-nest-struct", "ValueShape::TypeNameOrArray"),
    ("features-bad-type", "NameNotString"),
    ("features-deprecated-type", "SpecialFeature"),
    ("features-duplicate-name", "FeatureClash"),
    ("features-if-invalid", "ValueShape::Condition"),
    ("features-missing-name", "MissingKey"),
    ("features-name-bad-type", "NameNotString"),
    ("features-no-list", "ValueShape::Array"),
    ("features-too-many", "TooManyFeatures"),
    ("features-unknown-key", "UnknownKey"),
    ("funny-char", "Parse"),
    ("funny-word", "Parse"),
    ("ident-with-escape", "Parse"),
    ("include-before-err", "Parse"),
    ("include-cycle", "Include:Loop"),
    ("include-extra-junk", "Parse"),
    ("include-nested-err", "Parse"),
    ("include-no-file", "Include:Unreadable"),
    ("include-non-file", "Parse"),
    ("include-self-cycle", "Include:Loop"),
    ("leading-comma-list", "Parse"),
    ("leading-comma-object", "Parse"),
    ("missing-array-rsqb", "Parse"),
    ("missing-colon", "Parse"),
    ("missing-comma-list", "Parse"),
    ("missing-comma-object", "Parse"),
    ("missing-object-member-element", "Parse"),
    ("missing-type", "Metatypes"),
    ("nested-struct-data", "ValueShape::TypeNameOrArray"),
    ("nested-struct-data-invalid-dict", "MissingKey"),
    ("non-objects", "Parse"),
    ("oob-coroutine", "IncompatibleFlags"),
    ("oob-test", "FlagValue"),
    ("pragma-extra-junk", "Parse"),
    ("pragma-non-dict", "Parse"),
    ("pragma-unknown", "Parse"),
    ("pragma-value-not-bool", "Parse"),
    ("pragma-value-not-list", "Parse"),
    ("pragma-value-not-list-of-str", "Parse"),
    ("quoted-structural-chars", "Parse"),
    ("redefined-command", "Duplicate"),
    ("redefined-event", "Duplicate"),
    ("redefined-predefined", "Predefined"),
    ("redefined-type", "Duplicate"),
    ("reserved-command-q", "InvalidName"),
    ("reserved-enum-q", "InvalidName"),
    ("reserved-member-has", "ReservedName"),
    ("reserved-member-q", "InvalidName"),
    ("reserved-member-u", "ReservedName"),
    ("reserved-member-underscore", "InvalidName"),
    ("reserved-type-list", "ReservedSuffix"),
    ("returns-alternate", "TypeNotAllowed"),
    ("returns-array-bad", "ValueShape::ArrayOfOneTypeName"),
    ("returns-bad-type", "TypeNotAllowed"),
    ("returns-dict", "ValueShape::TypeNameOrArray"),
    ("returns-unknown", "UnknownType"),
    ("string-code-point-127", "Parse"),
    ("string-code-point-31", "Parse"),
    ("struct-base-clash", "MemberClash"),
    ("struct-base-clash-deep", "MemberClash"),
    ("struct-data-invalid", "ValueShape::ObjectOrTypeName"),
    ("struct-data-typename", "ValueShape::ObjectOrTypeName"),
    ("struct-member-if-invalid", "ValueShape::Condition"),
    ("struct-member-invalid", "ValueShape::TypeNameOrArray"),
    ("struct-member-invalid-dict", "MissingKey"),
    ("struct-member-name-clash", "MemberClash"),
    ("trailing-comma-list", "Parse"),
    ("trailing-comma-object", "Parse"),
    ("type-bypass-bad-gen", "FlagValue"),
    ("type-case", "NameCase"),
    ("unclosed-list", "Parse"),
    ("unclosed-object", "Parse"),
    ("unclosed-string", "Parse"),
    ("union-array-branch", "ValueShape::TypeName"),
    ("union-bad-base", "MemberClash"),
    ("union-bad-discriminator", "NameNotString"),
    ("union-base-any", "TypeNotAllowed"),
    ("union-base-empty", "DiscriminatorNotMember"),
    ("union-base-no-discriminator", "MissingKey"),
    ("union-base-union", "TypeNotAllowed"),
    ("union-branch-if-invalid", "InvalidCondition"),
    ("union-branch-invalid-dict", "MissingKey"),
    ("union-clash-member", "MemberClash"),
    ("union-discriminator-bad-name", "DiscriminatorNotMember"),
    ("union-empty", "NoBranches"),
    ("union-inline-invalid-dict", "MissingKey"),
    ("union-int-branch", "TypeNotAllowed"),
    ("union-invalid-base", "TypeNotAllowed"),
    ("union-invalid-branch-key", "UnknownBranch"),
    ("union-invalid-data", "ValueShape::Object"),
    ("union-invalid-discriminator", "DiscriminatorNotMember"),
    ("union-invalid-if-discriminator", "ConditionalDiscriminator"),
    ("union-invalid-union-subfield", "MemberClash"),
    ("union-invalid-union-subtype", "MemberClash"),
    ("union-no-base", "MissingKey"),
    ("union-optional-discriminator", "OptionalDiscriminator"),
    ("union-string-discriminator", "DiscriminatorNotEnum"),
    ("union-unknown", "UnknownType"),
    ("unknown-escape", "Parse"),
    ("unknown-expr-key", "UnknownKey"),
];

/// QEMU's negative test schemas that the table does not check, and why:
/// none at the pinned release.
const SKIPPED: &[(&str, &str)] = &[];

fn qemu_fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/qemu/tests/qapi-schema")
}

/// QEMU's test schemas whose `.err` file is empty (`false`) or not.
fn fixture_names(negative: bool) -> Vec<String> {
    let mut names: Vec<_> = std::fs::read_dir(qemu_fixtures())
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let stem = path.file_stem()?.to_string_lossy().into_owned();
            (path.extension()? == "err"
                && path.with_extension("json").exists()
                && std::fs::read_to_string(&path).unwrap().trim().is_empty() != negative)
                .then_some(stem)
        })
        .collect();
    names.sort();
    names
}

#[test]
fn qemu_negative_fixtures_fail_with_their_kind_where_qemu_reports_them() {
    let root = qemu_fixtures();
    let mut listed: Vec<_> = FIXTURES
        .iter()
        .chain(SKIPPED)
        .map(|(name, _)| (*name).to_owned())
        .collect();
    listed.sort();
    assert_eq!(
        fixture_names(true),
        listed,
        "every negative fixture is in the table or the skip list, once"
    );
    let mut failures = Vec::new();
    for (name, expected) in FIXTURES {
        let err = std::fs::read_to_string(root.join(format!("{name}.err"))).unwrap();
        let report = err
            .lines()
            .find(|line| !line.contains(": In ") && !line.starts_with("In file"))
            .unwrap();
        let mut parts = report.split(':');
        let file = parts
            .next()
            .and_then(|file| Path::new(file).file_name()?.to_str());
        let mut position = parts.map_while(|part| part.parse::<usize>().ok());
        let (line, column) = (position.next(), position.next());
        let (kind, at) = fixture_outcome(&root.join(format!("{name}.json")));
        let located = at.as_ref().is_some_and(|(at_file, at_line, at_column)| {
            Some(at_file.as_str()) == file
                && Some(*at_line) == line
                && column.is_none_or(|column| column == *at_column)
        });
        if kind != *expected || !located {
            failures.push(format!("{name}: {kind} at {at:?}, QEMU: {report}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn qemu_positive_fixtures_lower() {
    let root = qemu_fixtures();
    let names = fixture_names(false);
    assert!(!names.is_empty(), "QEMU's positive fixtures are missing");
    let failures: Vec<_> = names
        .iter()
        .filter_map(|name| {
            let (kind, at) = fixture_outcome(&root.join(format!("{name}.json")));
            (kind != "accepted").then(|| format!("{name}: {kind} at {at:?}"))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn dictionaries_in_place_of_types_are_anonymous_structs_until_4_0() {
    let member = "{'struct':'Foo','data':{'a':{'type':'int'}}}";
    let branch =
        "{'struct':'Ss','data':{}} {'alternate':'Aa','data':{'i':{'type':'int'},'s':'Ss'}}";
    let returns = "{'command':'c','returns':{'a':'int'}}";
    rules(&[
        ((2, 3, 0), member, None),
        ((2, 4, 0), member, Some(Kind::AnonymousStruct)),
        ((3, 1, 0), member, Some(Kind::AnonymousStruct)),
        ((4, 0, 0), member, None),
        ((2, 4, 0), branch, Some(Kind::AnonymousStruct)),
        ((3, 1, 0), branch, Some(Kind::AnonymousStruct)),
        ((4, 0, 0), branch, None),
        // Only 2.4 takes members of a return type in place.
        ((2, 4, 0), returns, None),
        ((2, 5, 0), returns, Some(Kind::AnonymousStruct)),
        (
            (2, 4, 0),
            "{'command':'c','returns':{'a b':'int'}}",
            Some(Kind::InvalidName { name: "a b".into() }),
        ),
    ]);
    assert_eq!(
        rejected((3, 1, 0), member).context(),
        ["'data'", "member 'a'"]
    );
}

#[test]
fn empty_conditions_are_refused_where_each_release_checks_them() {
    let empty = |condition: &str| {
        Some(Kind::EmptyCondition {
            condition: condition.into(),
        })
    };
    let top = |cond: &str| format!("{{'struct':'Foo','data':{{'a':'int'}},'if':{cond}}}");
    let member =
        |cond: &str| format!("{{'struct':'Foo','data':{{'a':{{'type':'int','if':{cond}}}}}}}");
    let value = |cond: &str| format!("{{'enum':'Foo','data':[{{'name':'a','if':{cond}}}]}}");
    rules(&[
        ((3, 0, 0), &top("' '"), None),
        ((3, 0, 0), &top("''"), empty("''")),
        ((3, 0, 0), &top("[]"), empty("[]")),
        ((3, 0, 0), &top("['A','']"), empty("''")),
        ((4, 1, 0), &top("' '"), None),
        ((4, 2, 0), &top("' '"), empty("' '")),
        ((6, 1, 0), &top("[]"), empty("[]")),
        // Members and branches from 4.2, enum values from 4.0.
        ((4, 1, 0), &member("''"), None),
        ((4, 1, 0), &member("[]"), None),
        ((4, 2, 0), &member("''"), empty("''")),
        ((4, 2, 0), &member("[]"), empty("[]")),
        ((3, 1, 0), &value("''"), Some(Kind::NameNotString)),
        ((4, 0, 0), &value("''"), empty("''")),
        ((4, 2, 0), &value("' '"), empty("' '")),
        (
            (4, 2, 0),
            "{'struct':'Ss','data':{}} {'alternate':'Aa','data':{'s':{'type':'Ss','if':[]}}}",
            empty("[]"),
        ),
        (
            (4, 2, 0),
            "{'struct':'Foo','data':{},'features':[{'name':'f','if':''}]}",
            empty("''"),
        ),
        ((2, 3, 0), &top("''"), None),
    ]);
    assert_eq!(
        rejected((3, 0, 0), &top("[]")).to_string(),
        "Invalid QAPI struct 'Foo' in schema.json:1:1: 'if' condition [] is useless"
    );
}

#[test]
fn enum_values_written_as_objects_require_4_0() {
    let value = "{'enum':'Foo','data':['b',{'name':'a'}]}";
    rules(&[
        ((2, 3, 0), value, None),
        ((2, 4, 0), value, Some(Kind::NameNotString)),
        ((3, 1, 0), value, Some(Kind::NameNotString)),
        ((4, 0, 0), value, None),
    ]);
    let error = rejected((2, 4, 0), value);
    assert_eq!(error.context(), ["member 'a'"]);
}

#[test]
fn two_members_of_one_name_clash_from_2_6() {
    // Until 2.5 the validation leaves them to the lowering, which cannot
    // represent them.
    let members = "{'struct':'Foo','data':{'a':'int','*a':'str'}}";
    let arguments = "{'command':'c','data':{'a':'int','*a':'str'}}";
    rules(&[
        ((2, 5, 0), members, None),
        ((2, 5, 0), arguments, None),
        ((2, 6, 0), members, clash("a", "a")),
        ((2, 6, 0), arguments, clash("a", "a")),
        ((4, 2, 0), members, clash("a", "a")),
    ]);
}

#[test]
fn qemu_2_11_2_checks_its_entities_in_name_order() {
    let structs = "{'struct':'Zstruct','data':{'a-b':'int','a_b':'int'}}\n{'struct':'Astruct','data':{'c-d':'int','c_d':'int'}}";
    // A command checks its arguments, the type q_obj_Bcmd-arg, itself.
    let arguments = "{'command':'Bcmd','data':{'a-b':'int','a_b':'int'}}\n{'struct':'Ccc','data':{'c-d':'int','c_d':'int'}}";
    // The tag enum of a simple union, UuKind, checks the branch names
    // after its union.
    let branches = "{'union':'Uu','data':{'a-b':'int','a_b':'str'}}\n{'struct':'Tt','data':{'c-d':'int','c_d':'int'}}";
    for (version, text, line) in [
        ((2, 11, 0), structs, 1),
        ((2, 11, 2), structs, 2),
        ((2, 12, 0), structs, 1),
        ((2, 11, 0), arguments, 1),
        ((2, 11, 2), arguments, 1),
        ((2, 11, 0), branches, 1),
        ((2, 11, 2), branches, 2),
    ] {
        let error = rejected(version, text);
        assert!(matches!(error.kind(), Kind::MemberClash { .. }), "{error}");
        assert_eq!(error.line(), Some(line), "{version:?} {text}");
    }
}

#[test]
fn a_chain_of_5000_bases_needs_no_deeper_stack() {
    // QEMU takes chains as long as its recursion allows; the walk keeps
    // its own stack, so a chain of any length is checked from its end.
    let mut text = String::new();
    for index in (1..5000).rev() {
        text.push_str(&format!(
            "{{'struct':'Sx{index}','base':'Sx{}','data':{{'m{index}':'int'}}}}\n",
            index - 1
        ));
    }
    let chain = format!("{text}{{'struct':'Sx0','data':{{'m0':'int'}}}}\n");
    let cycle = format!("{text}{{'struct':'Sx0','base':'Sx4999','data':{{'m0':'int'}}}}\n");
    check((4, 1, 0), &chain).unwrap();
    check((11, 1, 3), &chain).unwrap();
    assert_eq!(
        error_line((11, 1, 3), &cycle),
        (
            Kind::InheritanceCycle {
                name: "Sx4999".into()
            },
            1
        )
    );
}

#[test]
fn before_2_4_bases_and_event_data_are_looked_up_as_qemu_does() {
    let empty =
        "{'enum':'Ee','data':['a']} {'type':'Bb','data':{}} {'type':'Ss','data':{'x':'int'}}
        {'union':'Uu','base':'Bb','discriminator':'d','data':{'a':'Ss'}}";
    let nested = "{'type':'Str','data':{'a':{'b':'int'}}} {'event':'EVT','data':'Str'}";
    rules(&[
        // An empty base is none valid; an empty discriminator is none.
        ((1, 7, 0), empty, None),
        ((2, 0, 0), empty, Some(Kind::EmptyBase { name: "Bb".into() })),
        (
            (2, 3, 0),
            "{'type':'Ss','data':{'x':'int'}} {'union':'Uu','discriminator':'','data':{'a':'Ss'}}",
            None,
        ),
        (
            (2, 3, 0),
            "{'type':'Ss','data':{'x':'int'}} {'union':'Uu','base':{},'discriminator':'d','data':{'a':'Ss'}}",
            Some(Kind::MissingBase),
        ),
        // The base is the first struct of its name, the discriminator's
        // enum the first enum.
        (
            (2, 3, 0),
            "{'enum':'Ee','data':['x']} {'enum':'Bb','data':['y']} {'type':'Bb','data':{'d':'Ee'}}
             {'type':'Ss','data':{'z':'int'}} {'union':'Uu','base':'Bb','discriminator':'d','data':{'x':'Ss'}}",
            None,
        ),
        // QEMU looks a discriminator up as the key is written.
        (
            (2, 3, 0),
            "{'enum':'Ee','data':['a']} {'type':'Bb','data':{'*d':'Ee'}}
             {'union':'Uu','base':'Bb','discriminator':'d','data':{}}",
            Some(Kind::DiscriminatorNotMember {
                discriminator: "d".into(),
            }),
        ),
        // The struct that event data names may not nest objects either.
        ((2, 0, 0), nested, None),
        ((2, 1, 0), nested, Some(Kind::NestedEventData)),
        ((2, 3, 0), nested, Some(Kind::NestedEventData)),
        // QEMU asserts that it finds the struct.
        (
            (2, 1, 0),
            "{'enum':'Ee','data':[]} {'event':'EVT','data':'Ee'}",
            Some(Kind::Crash {
                crash: crate::unified::Crash::EventData { name: "Ee".into() },
            }),
        ),
    ]);
}

#[test]
fn flat_union_bases_need_members_until_2_8() {
    let union = |base: &str| {
        format!(
            "{{'enum':'Ee','data':['a']}} {{'struct':'Bb','data':{{}}}} {{'struct':'Ss','data':{{'x':'int'}}}}
             {{'union':'Uu','base':{base},'discriminator':'d','data':{{'a':'Ss'}}}}"
        )
    };
    let not_member = Some(Kind::DiscriminatorNotMember {
        discriminator: "d".into(),
    });
    rules(&[
        (
            (2, 4, 0),
            &union("'Bb'"),
            Some(Kind::EmptyBase { name: "Bb".into() }),
        ),
        (
            (2, 8, 0),
            &union("'Bb'"),
            Some(Kind::EmptyBase { name: "Bb".into() }),
        ),
        ((2, 9, 0), &union("'Bb'"), not_member.clone()),
        // An empty object written out in place is no base until 6.1.
        ((2, 5, 0), &union("{}"), Some(Kind::AnonymousStruct)),
        ((2, 6, 0), &union("{}"), Some(Kind::MissingBase)),
        ((6, 1, 0), &union("{}"), Some(Kind::MissingBase)),
        ((6, 2, 0), &union("{}"), not_member),
    ]);
}

#[test]
fn discriminators_are_refused_in_qemus_words() {
    let union = |base: &str, data: &str| {
        format!(
            "{{'enum':'Ee','data':['a']}} {{'struct':'Ss','data':{{'x':'int'}}}}
             {{'union':'Uu','base':{base},'discriminator':'d','data':{data}}}"
        )
    };
    let not_enum = |name: &str| {
        Some(Kind::DiscriminatorNotEnum {
            discriminator: "d".into(),
            discriminator_type: name.into(),
        })
    };
    let array = union("{'d':['Ee']}", "{'a':'Ss'}");
    let qtype = union("{'d':'QType'}", "{'qnum':'Ss'}");
    rules(&[
        ((2, 6, 0), &array, not_enum("['Ee']")),
        ((4, 2, 0), &array, not_enum("['Ee']")),
        ((11, 1, 3), &array, not_enum("['Ee']")),
        (
            (11, 1, 3),
            &union("{'*d':'Ee'}", "{'a':'Ss'}"),
            Some(Kind::OptionalDiscriminator {
                discriminator: "d".into(),
            }),
        ),
        // Until 4.1 QType is a builtin type, no enum.
        ((2, 6, 0), &qtype, not_enum("QType")),
        ((4, 1, 0), &qtype, not_enum("QType")),
        ((4, 2, 0), &qtype, None),
        (
            (4, 1, 0),
            "{'struct':'Str','base':'QType','data':{}}",
            not_allowed("built-in", "'QType'"),
        ),
        (
            (4, 2, 0),
            "{'struct':'Str','base':'QType','data':{}}",
            not_allowed("enum", "'QType'"),
        ),
    ]);
}

#[test]
fn alternate_branches_are_checked_in_the_order_of_each_release() {
    let ambiguous = |branch: &str, previous: &str| {
        Some(Kind::AmbiguousAlternate {
            branch: branch.into(),
            previous: previous.into(),
        })
    };
    let invalid = "{'alternate':'Alt','data':{'a':'int','b':'int','c!':'str'}}";
    let unknown = "{'alternate':'Alt','data':{'a':'int','b':'int','c':'Nope'}}";
    let clash = "{'alternate':'Alt','data':{'a-b':'int','a_b':'str','c':'Nope'}}";
    let late_clash = "{'alternate':'Alt','data':{'a-x':'int','b':'int','a_x':'str'}}";
    let nope = Some(Kind::UnknownType {
        name: "Nope".into(),
    });
    rules(&[
        // Until 4.1 each branch in turn: name, constant, type, JSON type.
        ((2, 4, 0), invalid, ambiguous("b", "a")),
        ((4, 1, 0), invalid, ambiguous("b", "a")),
        ((4, 1, 0), unknown, ambiguous("b", "a")),
        ((2, 5, 0), clash, clash_kind("a_b", "a-b")),
        ((2, 6, 0), clash, nope.clone()),
        // From 4.2 the expressions first, then every branch's type, then
        // each branch's name and JSON type.
        (
            (4, 2, 0),
            invalid,
            Some(Kind::InvalidName { name: "c!".into() }),
        ),
        ((4, 2, 0), unknown, nope.clone()),
        ((5, 2, 0), clash, nope),
        ((4, 2, 0), late_clash, ambiguous("b", "a-x")),
    ]);
}

fn clash_kind(name: &str, previous: &str) -> Option<Kind> {
    clash(name, previous)
}

#[test]
fn enum_values_are_checked_one_by_one_in_2_4_and_2_5() {
    let value = "{'enum':'Enm','data':['a-b','a_b','Bad!']}";
    rules(&[
        ((2, 4, 0), value, clash("a_b", "a-b")),
        ((2, 5, 0), value, clash("a_b", "a-b")),
        (
            (2, 6, 0),
            value,
            Some(Kind::InvalidName {
                name: "Bad!".into(),
            }),
        ),
    ]);
}

#[test]
fn from_4_2_the_types_made_for_a_definition_are_checked_before_it() {
    let nope = Some(Kind::UnknownType {
        name: "Nope".into(),
    });
    let feature_clash = Some(Kind::FeatureClash {
        name: "f_x".into(),
        previous: "f-x".into(),
    });
    let union_base = "{'enum':'Enm','data':['a']} {'struct':'Str','data':{'x':'int'}}
        {'union':'Uni','base':{'kind':'Enm','y':'Nope'},'discriminator':'kind','data':{'a':'Str'},'features':['FEATURE']}";
    rules(&[
        // A command's arguments, the type q_obj_cmd-arg, before its
        // features.
        (
            (4, 2, 0),
            "{'command':'cmd','data':{'a-b':'int','a_b':'int'},'features':['f-x','f_x']}",
            clash("a_b", "a-b"),
        ),
        (
            (4, 2, 0),
            "{'command':'cmd','data':{'a':'Nope'},'features':['f-x','f_x']}",
            nope.clone(),
        ),
        (
            (5, 0, 0),
            "{'event':'EVT','data':{'a':'Nope'},'features':['f-x','f_x']}",
            nope.clone(),
        ),
        // A union's base written out in place before the union's features.
        (
            (5, 0, 0),
            &union_base.replace("FEATURE", "f-x','f_x"),
            nope.clone(),
        ),
        (
            (5, 0, 0),
            &union_base.replace("FEATURE", "deprecated"),
            nope.clone(),
        ),
        // An array type before the definition that first uses it.
        (
            (4, 2, 0),
            "{'struct':'Str','data':{'a':['Nope']},'features':['f-x','f_x']}",
            nope.clone(),
        ),
        (
            (5, 0, 0),
            "{'struct':'Str','data':{'a':['Nope']},'features':['deprecated']}",
            nope.clone(),
        ),
        (
            (4, 2, 0),
            "{'command':'cmd','returns':['Nope'],'features':['f-x','f_x']}",
            nope.clone(),
        ),
        // A simple union's branch types and tag enum before the union.
        (
            (5, 0, 0),
            "{'union':'Uni','data':{'a-b':'int','a_b':'str'},'features':['f-x','f_x']}",
            clash("a_b", "a-b"),
        ),
        // QEMU fails to describe the member of a branch's wrapper.
        (
            (5, 2, 0),
            "{'union':'Uni','data':{'a-b':'int','a_b':'Nope'}}",
            Some(Kind::Crash {
                crash: crate::unified::Crash::SimpleBranchType {
                    branch: "a_b".into(),
                },
            }),
        ),
        (
            (4, 2, 0),
            "{'struct':'Str','data':{'a':'int'},'features':['f-x','f_x']}",
            feature_clash,
        ),
    ]);
    // The array type belongs to the definition that uses it first.
    let arrays = "{'struct':'Aa','data':{'x':'int'}}\n{'struct':'Bb','data':{'l':['Nope']}}\n{'struct':'Cc','data':{'m':['Nope']},'features':['g','g']}";
    assert_eq!(error_line((4, 2, 0), arrays).1, 2);
    assert!(rejected((4, 2, 0), arrays).context().is_empty());
}

#[test]
fn commands_check_their_argument_type_until_4_1_and_from_8_1() {
    let unknown = "{'command':'cmd','data':'Ss'}\n{'struct':'Xx','data':{'a':'Nope'}}\n{'struct':'Ss','data':{'b':'Nope2'}}";
    let clashes = |definition: &str| {
        format!(
            "{definition}\n{{'struct':'Xx','data':{{'a-b':'int','a_b':'int'}}}}\n{{'struct':'Ss','data':{{'c-d':'int','c_d':'int'}}}}"
        )
    };
    let command = clashes("{'command':'cmd','data':'Ss'}");
    let event = clashes("{'event':'EVT','data':'Ss'}");
    let alternate = "{'command':'cmd','boxed':true,'data':'Alt'}\n{'struct':'Xx','data':{'a-b':'int','a_b':'int'}}\n{'alternate':'Alt','data':{'c-d':'int','c_d':'str'}}";
    for (version, text, line) in [
        ((2, 6, 0), command.as_str(), 2),
        ((2, 7, 0), &command, 3),
        ((4, 1, 0), &command, 3),
        ((4, 2, 0), &command, 2),
        ((2, 6, 0), &event, 2),
        ((2, 7, 0), &event, 3),
        ((5, 2, 0), &event, 2),
        ((4, 2, 0), unknown, 2),
        ((8, 0, 0), unknown, 2),
        ((8, 1, 0), unknown, 3),
        // A boxed alternate too, until 4.1.
        ((2, 7, 0), alternate, 3),
        ((4, 1, 0), alternate, 3),
    ] {
        assert_eq!(error_line(version, text).1, line, "{version:?} {text}");
    }
}

#[test]
fn boxed_arguments_without_a_type_name_are_refused_where_qemu_does() {
    let late = "{'command':'cmd','boxed':true}\n{'struct':'Xx','data':{'a b':'int'}}";
    // Until 4.1 with the schema's types, so after every expression.
    assert_eq!(
        error_line((2, 7, 0), late),
        (Kind::InvalidName { name: "a b".into() }, 2)
    );
    assert_eq!(error_line((4, 2, 0), late), (Kind::BoxedWithoutType, 1));
    rules(&[(
        (2, 7, 0),
        "{'command':'cmd','boxed':true,'data':{'a':'int'}}",
        Some(Kind::AnonymousStruct),
    )]);
}

#[test]
fn a_definition_name_is_checked_before_its_suffix() {
    let suffix = |name: &str, suffix: &'static str| {
        Some(Kind::ReservedSuffix {
            name: name.into(),
            suffix,
        })
    };
    rules(&[
        (
            (5, 2, 0),
            "{'struct':'fooList','data':{}}",
            suffix("fooList", "List"),
        ),
        (
            (6, 0, 0),
            "{'struct':'fooList','data':{}}",
            case("fooList", NameRule::CamelCase),
        ),
        (
            (6, 1, 0),
            "{'struct':'fooKind','data':{}}",
            case("fooKind", NameRule::CamelCase),
        ),
        (
            (6, 0, 0),
            "{'command':'foo-List'}",
            case("foo-List", NameRule::Lowercase),
        ),
        (
            (6, 0, 0),
            "{'event':'FOOKind'}",
            case("FOOKind", NameRule::Uppercase),
        ),
        (
            (5, 2, 0),
            "{'struct':'intList','data':{}}",
            suffix("intList", "List"),
        ),
        (
            (6, 0, 0),
            "{'struct':'intList','data':{}}",
            case("intList", NameRule::CamelCase),
        ),
        (
            (6, 0, 0),
            "{'struct':'FooList','data':{}}",
            suffix("FooList", "List"),
        ),
    ]);
    // Until 4.1 a name that is defined again is refused before its suffix.
    let twice = "{'struct':'FooKind','data':{}}\n{'struct':'FooKind','data':{}}";
    assert!(matches!(
        error_line((2, 4, 0), &twice.replace("Kind", "List")).0,
        Kind::Duplicate { .. }
    ));
    assert_eq!(
        error_line((2, 5, 0), &twice.replace("Kind", "List")),
        (
            Kind::ReservedSuffix {
                name: "FooList".into(),
                suffix: "List"
            },
            1
        )
    );
}

#[test]
fn from_5_0_a_definitions_condition_comes_before_its_features() {
    let condition = Some(Kind::EmptyCondition {
        condition: "''".into(),
    });
    let invalid = Some(Kind::InvalidName {
        name: "bad name".into(),
    });
    for text in [
        "{'struct':'Str','data':{},'if':'','features':['bad name']}",
        "{'struct':'Str','data':{},'features':['bad name'],'if':''}",
        "{'command':'cmd','if':'','features':['bad name']}",
    ] {
        rules(&[
            ((4, 2, 0), text, invalid.clone()),
            ((5, 0, 0), text, condition.clone()),
            ((6, 1, 0), text, condition.clone()),
        ]);
    }
}

#[test]
fn four_one_checks_a_structs_features_after_its_members() {
    let member_clash = "{'struct':'Str','data':{'a-b':'int','a_b':'int'},'features':['f-x','f_x']}";
    let uppercase = "{'struct':'Str','data':{'aB':'int'},'features':['f-x','f_x']}";
    rules(&[
        ((4, 1, 0), member_clash, clash("a_b", "a-b")),
        (
            (4, 2, 0),
            member_clash,
            Some(Kind::FeatureClash {
                name: "f_x".into(),
                previous: "f-x".into(),
            }),
        ),
        ((4, 1, 0), uppercase, case("aB", NameRule::NoUppercase)),
    ]);
}

#[test]
fn each_rule_ends_where_its_release_drops_it() {
    // The uppercase check of the members' clashes ends with 4.1; from 4.2
    // the pragma exempts an inline base by its union's name.
    let inline_base = "{'pragma':{'name-case-whitelist':['Uni']}} {'enum':'Enm','data':['a']}
        {'struct':'Str','data':{'x':'int'}}
        {'union':'Uni','base':{'kind':'Enm','aB':'int'},'discriminator':'kind','data':{'a':'Str'}}";
    let one_branch = "{'alternate':'Alt','data':{'i':'int'}}";
    let empty_simple = "{'union':'Uni','data':{}}";
    let empty_base = "{'struct':'Str','base':'q_empty','data':{}}";
    rules(&[
        ((4, 1, 0), inline_base, case("aB", NameRule::NoUppercase)),
        ((4, 2, 0), inline_base, None),
        ((5, 2, 0), inline_base, None),
        (
            (4, 1, 0),
            one_branch,
            Some(Kind::AlternateBranches { minimum: 2 }),
        ),
        ((4, 2, 0), one_branch, None),
        ((4, 1, 0), empty_simple, Some(Kind::NoBranches)),
        ((4, 2, 0), empty_simple, Some(Kind::NoBranches)),
        ((5, 0, 0), empty_simple, Some(Kind::NoBranches)),
        (
            (4, 1, 0),
            empty_base,
            Some(Kind::UnknownType {
                name: "q_empty".into(),
            }),
        ),
        ((4, 2, 0), empty_base, None),
    ]);
}

#[test]
fn two_four_takes_untyped_members_only_by_name() {
    let untyped = Some(Kind::UntypedWithoutGen);
    rules(&[
        (
            (2, 4, 0),
            "{'command':'cmd','gen':false,'data':{'a':'**'},'returns':'**'}",
            None,
        ),
        (
            (2, 4, 0),
            "{'command':'cmd','gen':false,'data':{'a':['**']}}",
            untyped.clone(),
        ),
        (
            (2, 4, 0),
            "{'command':'cmd','gen':false,'returns':['**']}",
            untyped,
        ),
    ]);
}

#[test]
fn the_tag_enum_of_an_alternate_is_a_type_in_2_4_and_2_5() {
    let text =
        "{'alternate':'Alt','data':{'a':'int','b':'str'}} {'struct':'Str','data':{'k':'AltKind'}}";
    rules(&[((2, 4, 0), text, None), ((2, 5, 0), text, None)]);
}

#[test]
fn names_may_end_in_a_newline_until_4_1() {
    // QEMU's name pattern ends in `$`, which matches before a final
    // newline; from 4.2 no name can hold one.
    for text in [
        "{'enum':'Enm','data':['a\\n']}",
        "{'struct':'Str\\n','data':{}}",
        "{'struct':'Str','data':{'m\\n':'int'}}",
    ] {
        rules(&[((2, 4, 0), text, None), ((4, 1, 0), text, None)]);
    }
    rules(&[(
        (4, 1, 0),
        "{'enum':'Enm','data':['a\\n\\n']}",
        Some(Kind::InvalidName {
            name: "a\n\n".into(),
        }),
    )]);
}

#[test]
fn keys_are_checked_in_the_order_of_each_release() {
    let missing = |key: &str| Some(Kind::MissingKey { key: key.into() });
    let flag = |flag, value| Some(Kind::FlagValue { flag, value });
    let invalid = |name: &str| Some(Kind::InvalidName { name: name.into() });
    rules(&[
        // Keys that no release knows are the validation's.
        (
            (2, 4, 0),
            "{'command':'cc','bogus':'x'}",
            unknown_key("bogus"),
        ),
        (
            (11, 1, 3),
            "{'event':'EV','bogus':{'x':['y',{}]}}",
            unknown_key("bogus"),
        ),
        // Until 3.1 key by key in the source's order, then missing keys.
        (
            (3, 1, 0),
            "{'command':'cc','gen':true,'bogus':'x'}",
            flag("gen", false),
        ),
        (
            (3, 1, 0),
            "{'union':'Uu','bogus':'x'}",
            unknown_key("bogus"),
        ),
        (
            (2, 7, 0),
            "{'command':'cc','boxed':false,'if':''}",
            flag("boxed", true),
        ),
        (
            (2, 7, 0),
            "{'command':'cc','if':'','boxed':false}",
            unknown_key("if"),
        ),
        (
            (3, 0, 0),
            "{'command':'cc','if':'','boxed':false}",
            Some(Kind::EmptyCondition {
                condition: "''".into(),
            }),
        ),
        ((2, 4, 0), "{'enum':'Ee'}", missing("data")),
        // From 4.0 the missing keys, then the unknown ones, sorted.
        (
            (4, 0, 0),
            "{'command':'cc','gen':true,'bogus':'x'}",
            unknown_key("bogus"),
        ),
        ((4, 0, 0), "{'union':'Uu','bogus':'x'}", missing("data")),
        (
            (4, 0, 0),
            "{'command':'cc','zz':'x','aa':'y'}",
            unknown_key("aa"),
        ),
        ((6, 2, 0), "{'union':'Uu','data':{}}", missing("base")),
        // A second kind's key is unknown, from 6.2 a fault of its own.
        (
            (6, 1, 0),
            "{'struct':'Ss','enum':'Ee','data':[]}",
            unknown_key("struct"),
        ),
        (
            (6, 2, 0),
            "{'struct':'Ss','enum':'Ee','data':[]}",
            Some(Kind::Metatypes),
        ),
        ((2, 4, 0), "{'bogus':'x'}", Some(Kind::MissingMetatype)),
        ((6, 2, 0), "{'bogus':'x'}", Some(Kind::Metatypes)),
        // Members, branches, enum values and features as dictionaries,
        // checked with each, in their own order.
        (
            (4, 0, 0),
            "{'struct':'Ss','data':{'a':{'type':'int','bogus':'x'}}}",
            unknown_key("bogus"),
        ),
        (
            (11, 1, 3),
            "{'struct':'Ss','data':{'a':{'if':'X'}}}",
            missing("type"),
        ),
        ((11, 1, 3), "{'enum':'Ee','data':[{}]}", missing("name")),
        (
            (4, 1, 0),
            "{'struct':'Ss','data':{},'features':[{'name':'f','bogus':'x'}]}",
            unknown_key("bogus"),
        ),
        (
            (4, 0, 0),
            "{'struct':'Tt','data':{}} {'union':'Uu','data':{'a':{'type':'Tt','bogus':'x'}}}",
            unknown_key("bogus"),
        ),
        (
            (4, 0, 0),
            "{'struct':'Ss','data':{'Bad!':'int','b':{'type':'int','features':['f']}}}",
            invalid("Bad!"),
        ),
        (
            (4, 2, 0),
            "{'enum':'Enm','data':['Bad!',{'name':'b','features':['f']}]}",
            invalid("Bad!"),
        ),
        // 4.0 and 4.1 check a member's keys with its expression.
        (
            (4, 1, 0),
            "{'struct':'Ss','data':{'a':{'type':'int','features':['f']}}} {'struct':'bad name','data':{}}",
            invalid("bad name"),
        ),
        (
            (4, 1, 0),
            "{'struct':'Ss','data':{'a':{'type':'int','features':['f']}}}",
            unknown_key("features"),
        ),
    ]);
    // From 4.2 the documentation is checked before the keys.
    let error = rejected(
        (4, 2, 0),
        "##\n# @Other:\n##\n{ 'alternate': 'Alt', 'base': 'Aa', 'data': { 'a': 'int' } }",
    );
    assert!(
        matches!(error.kind(), Kind::DocumentationSymbol { .. }),
        "{error}"
    );
}

fn crash(crash: crate::unified::Crash) -> Option<Kind> {
    Some(Kind::Crash { crash })
}

#[test]
fn schemas_that_qemu_fails_on_are_refused_with_a_crash_where_it_fails() {
    use crate::unified::Crash;
    let conflict = "{'alternate':'Alt','data':{'a':'bool','b':'str'}}";
    let any = "{'alternate':'Alt','data':{'a':'int','b':'any'}}";
    let tag = "{'struct':'Ss','data':{}} {'alternate':'Alt','data':{'a':'int','b':'Ss'}}
        {'struct':'Str','data':{'k':'AltKind'}}";
    let tag_after_case = tag.replace("'k':'AltKind'", "'A':'int','k':'AltKind'");
    let tag_branch = "{'struct':'Ss','data':{}} {'alternate':'Alt','data':{'a':'int','b':'Ss'}}
        {'alternate':'Two','data':{'a':'AltKind','b':'Ss'}}";
    let array = "{'enum':'E','data':['a']} {'struct':'B','data':{'kind':['E']}}
        {'struct':'S','data':{'x':'int'}}
        {'union':'U','base':'B','discriminator':'kind','data':{'a':'S'}}";
    let missing_enum = "{'type':'Ss','data':{}} {'enum':'Aa'}";
    let missing_union = "{'union':'Uu'}";
    let missing_base = "{'enum':'Ee','data':['a']} {'type':'Bb'}
        {'union':'Uu','base':'Bb','discriminator':'d','data':{'a':'Ss'}}";
    let missing = |name: &str| crash(Crash::MissingData { name: name.into() });
    rules(&[
        ((2, 9, 0), conflict, None),
        (
            (2, 10, 0),
            conflict,
            crash(Crash::AlternateConflict { branch: "b".into() }),
        ),
        (
            (2, 11, 0),
            conflict,
            Some(Kind::AmbiguousAlternate {
                branch: "b".into(),
                previous: "a".into(),
            }),
        ),
        (
            (2, 4, 0),
            any,
            Some(Kind::UnknownType { name: "any".into() }),
        ),
        (
            (2, 5, 0),
            any,
            crash(Crash::AlternateAny { branch: "b".into() }),
        ),
        ((2, 6, 0), any, not_allowed("built-in", "'any'")),
        ((2, 5, 0), tag, None),
        (
            (2, 6, 0),
            tag,
            crash(Crash::AlternateTag {
                name: "AltKind".into(),
            }),
        ),
        (
            (4, 1, 0),
            tag,
            crash(Crash::AlternateTag {
                name: "AltKind".into(),
            }),
        ),
        (
            (4, 2, 0),
            tag,
            Some(Kind::UnknownType {
                name: "AltKind".into(),
            }),
        ),
        // The members in turn, each with its clashes.
        ((2, 6, 0), &tag_after_case, case("A", NameRule::NoUppercase)),
        // From 2.10 the alternate looks for the tag enum's values first.
        (
            (2, 10, 0),
            tag_branch,
            crash(Crash::AlternateTag {
                name: "AltKind".into(),
            }),
        ),
        (
            (2, 7, 0),
            array,
            Some(Kind::DiscriminatorNotEnum {
                discriminator: "kind".into(),
                discriminator_type: "['E']".into(),
            }),
        ),
        (
            (2, 9, 0),
            array,
            crash(Crash::UnhashableDiscriminator {
                discriminator: "kind".into(),
            }),
        ),
        (
            (4, 1, 0),
            array,
            crash(Crash::UnhashableDiscriminator {
                discriminator: "kind".into(),
            }),
        ),
        ((1, 7, 0), missing_enum, None),
        ((2, 0, 0), missing_enum, missing("Aa")),
        ((2, 3, 0), missing_union, missing("Uu")),
        ((2, 0, 0), missing_base, missing("Bb")),
        (
            (2, 4, 0),
            missing_union,
            Some(Kind::MissingKey { key: "data".into() }),
        ),
    ]);
}

fn shape(expected: crate::unified::Shape) -> Option<Kind> {
    Some(Kind::ValueShape { expected })
}

#[test]
fn values_of_other_shapes_are_refused_as_each_release_checks_them() {
    use crate::unified::{Crash, Shape};
    let value = |key: &str| {
        Some(Kind::Unsupported {
            construct: crate::unified::Construct::Value { key: key.into() },
        })
    };
    let unchecked = |key: &str| crash(Crash::UncheckedValue { key: key.into() });
    rules(&[
        (
            (2, 4, 0),
            "{'struct':'Ss','data':{},'base':true}",
            shape(Shape::Object),
        ),
        (
            (2, 5, 0),
            "{'struct':'Ss','data':{},'base':true}",
            shape(Shape::TypeName),
        ),
        (
            (8, 0, 0),
            "{'struct':'Ss','data':{},'base':[]}",
            Some(Kind::ArrayNotAllowed),
        ),
        (
            (8, 1, 0),
            "{'struct':'Ss','data':{},'base':[]}",
            shape(Shape::TypeName),
        ),
        (
            (2, 5, 0),
            "{'command':'c','data':true}",
            shape(Shape::ObjectOrTypeName),
        ),
        (
            (8, 1, 0),
            "{'command':'c','returns':true}",
            shape(Shape::TypeNameOrArray),
        ),
        (
            (4, 1, 0),
            "{'command':'c','returns':['a','b']}",
            shape(Shape::ArrayOfOneTypeName),
        ),
        ((11, 1, 3), "{'enum':'Ee','data':'x'}", shape(Shape::Array)),
        (
            (4, 1, 0),
            "{'enum':'Ee','data':[],'prefix':[]}",
            shape(Shape::String),
        ),
        (
            (11, 1, 3),
            "{'enum':'Ee','data':[true]}",
            Some(Kind::NameNotString),
        ),
        (
            (2, 4, 0),
            "{'struct':true,'data':{}}",
            Some(Kind::NameNotString),
        ),
        (
            (11, 1, 3),
            "{'struct':true,'data':{}}",
            Some(Kind::NameNotString),
        ),
        (
            (11, 1, 3),
            "{'struct':'Ss','data':{},'features':'f'}",
            shape(Shape::Array),
        ),
        (
            (11, 1, 3),
            "{'struct':'Ss','data':{},'features':[[]]}",
            Some(Kind::NameNotString),
        ),
        (
            (11, 1, 3),
            "{'command':'c','gen':'no'}",
            Some(Kind::FlagValue {
                flag: "gen",
                value: false,
            }),
        ),
        // `null` is no value from 4.2, and until 4.1 an absent one where
        // QEMU looks a key up with `get`.
        (
            (4, 1, 0),
            "{'command':'c','data':null,'returns':null}",
            None,
        ),
        (
            (4, 1, 0),
            "{'struct':'Ss','data':{},'base':null,'if':null}",
            None,
        ),
        ((2, 4, 0), "{'struct':'Ss','data':null}", value("data")),
        ((2, 5, 0), "{'struct':'Ss','data':null}", unchecked("data")),
        (
            (4, 1, 0),
            "{'struct':'Ss','data':{'a':null}}",
            unchecked("type"),
        ),
        // 4.0 and 4.1 do not check a member's condition.
        (
            (4, 1, 0),
            "{'struct':'Ss','data':{'a':{'type':'int','if':{}}}}",
            value("if"),
        ),
        (
            (4, 2, 0),
            "{'struct':'Ss','data':{'a':{'type':'int','if':{}}}}",
            shape(Shape::Condition),
        ),
        // Conditions.
        (
            (4, 1, 0),
            "{'struct':'Ss','data':{},'if':{}}",
            shape(Shape::Condition),
        ),
        (
            (4, 1, 0),
            "{'struct':'Ss','data':{},'if':['A',true]}",
            shape(Shape::Condition),
        ),
        (
            (6, 2, 0),
            "{'struct':'Ss','data':{},'if':''}",
            Some(Kind::InvalidCondition {
                condition: "''".into(),
            }),
        ),
        (
            (6, 2, 0),
            "{'struct':'Ss','data':{},'if':{'all':['A'],'not':'B'}}",
            Some(Kind::ConditionKeys),
        ),
        (
            (6, 2, 0),
            "{'struct':'Ss','data':{},'if':{'any':'A'}}",
            shape(Shape::Array),
        ),
        // The branches of a union or alternate.
        (
            (2, 5, 0),
            "{'alternate':'Al','data':['a','b']}",
            unchecked("data"),
        ),
        (
            (2, 6, 0),
            "{'alternate':'Al','data':['a']}",
            Some(Kind::AlternateBranches { minimum: 2 }),
        ),
        (
            (4, 2, 0),
            "{'alternate':'Al','data':['a']}",
            unchecked("data"),
        ),
        (
            (6, 1, 0),
            "{'alternate':'Al','data':[]}",
            Some(Kind::AlternateBranches { minimum: 1 }),
        ),
        (
            (6, 1, 0),
            "{'alternate':'Al','data':'x'}",
            shape(Shape::Object),
        ),
        (
            (2, 6, 0),
            "{'alternate':'Al','data':{'a':null,'b':'int'}}",
            Some(Kind::TypeNotAllowed {
                meta: "null",
                reference: "null".into(),
            }),
        ),
        // Names that may not be optional until 4.1.
        (
            (2, 4, 0),
            "{'enum':'*Ee','data':[]}",
            Some(Kind::OptionalName { name: "*Ee".into() }),
        ),
        (
            (4, 2, 0),
            "{'enum':'*Ee','data':[]}",
            Some(Kind::InvalidName { name: "*Ee".into() }),
        ),
        // 2.4 takes only a type name for a flat union's base, and a
        // legacy alternate's `{}` is no discriminator name.
        (
            (2, 4, 0),
            "{'enum':'Ee','data':['a']} {'union':'Uu','base':{'k':'Ee'},'discriminator':'k','data':{}}",
            Some(Kind::StringBase),
        ),
        (
            (2, 6, 0),
            "{'struct':'Bb','data':{'k':'int'}} {'union':'Uu','base':'Bb','discriminator':{},'data':{}}",
            Some(Kind::NameNotString),
        ),
        // From 2.6 an empty enum value fails QEMU's look at its first
        // character.
        ((2, 5, 0), "{'enum':'Ee','data':['']}", None),
        ((2, 6, 0), "{'enum':'Ee','data':['']}", unchecked("name")),
    ]);
}

#[test]
fn values_of_other_shapes_keep_qemus_order() {
    // An earlier definition's error comes first, in each phase.
    let later = "{'struct':'a b','data':{}}\n{'enum':'Ee','data':'x'}";
    for version in [(2, 4, 0), (4, 2, 0), (11, 1, 3)] {
        assert_eq!(
            error_line(version, later),
            (Kind::InvalidName { name: "a b".into() }, 1),
            "{version:?}"
        );
    }
    // Within a definition, its arguments before its return type.
    let returns = "{'command':'cmd','data':{'Bad!':'int'},'returns':{'a':'int'}}";
    for version in [(4, 0, 0), (11, 1, 3)] {
        assert_eq!(
            error_line(version, returns),
            (
                Kind::InvalidName {
                    name: "Bad!".into()
                },
                1
            ),
            "{version:?}"
        );
    }
    // A documented expression without a metatype is refused for that.
    let documented = "##\n# @Target:\n##\n{'type':'Target','data':{}}";
    assert_eq!(
        error_line((2, 9, 0), documented),
        (Kind::MissingMetatype, 4)
    );
    assert_eq!(error_line((6, 2, 0), documented), (Kind::Metatypes, 4));
}

#[test]
fn release_boundaries_of_the_parser_and_entities_hold() {
    // From 2.9 a pragma is a directive, whose grammar refuses it.
    let pragma = "{ 'pragma': { 'doc-required': 'x' } }";
    assert!(matches!(rejected((2, 9, 0), pragma).kind(), Kind::Parse(_)));
    assert_eq!(rejected((2, 7, 0), pragma).kind(), &Kind::MissingMetatype);
    // From 7.1 an alternate's array branch makes an array type, which
    // QEMU checks with the alternate.
    let array = "{ 'alternate': 'Alt', 'data': { 'a': [ 'Nope' ], 'b': 'str' } }";
    let error = rejected((7, 1, 0), array);
    assert_eq!(
        error.kind(),
        &Kind::UnknownType {
            name: "Nope".into()
        }
    );
    assert!(error.context().is_empty(), "{error}");
    assert_eq!(rejected((7, 0, 0), array).kind(), &Kind::ArrayNotAllowed);
    // QEMU 2.9 and later read a documentation comment with an empty line,
    // also at the end of a file.
    for version in [(4, 2, 0), (5, 0, 0), (11, 1, 3)] {
        check(version, "##\n\n# text\n##\n").unwrap();
    }
}
