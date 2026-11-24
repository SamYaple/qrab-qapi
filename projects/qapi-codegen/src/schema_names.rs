//! Version-specific QName syntax, independent of casing/pragma style rules.
use crate::QapiSchemaToken;
use crate::unified::{SchemaErrorKind, SchemaResult, require};

pub(crate) fn c_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Whether the release's generator accepts `name`: no checks before 2.4,
/// letters, digits, `_`, `-` and `.` until 2.5, then QEMU's downstream
/// `__RFQDN_` prefix before a letter, letters, digits, `-` and `_`. From
/// 2.5 the `q_` namespace of C names is QEMU's own. QEMU's pattern ends
/// in `$`, which also matches before a final newline; only until 4.1 can
/// a name hold one, through the `\n` escape.
fn valid(name: &str) -> bool {
    let version = crate::parser_version();
    if version < (2, 4, 0) {
        return true;
    }
    let name = name.strip_suffix('\n').unwrap_or(name);
    let reserved = version >= (2, 5, 0) && c_name(name).starts_with("q_");
    let valid = if version < (2, 6, 0) {
        name.as_bytes()
            .first()
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
    } else {
        let stem = if let Some(rest) = name.strip_prefix("__") {
            rest.split_once('_').and_then(|(prefix, stem)| {
                (!prefix.is_empty()
                    && prefix
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-')))
                .then_some(stem)
            })
        } else {
            Some(name)
        };
        stem.is_some_and(|stem| {
            stem.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
                && stem
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        })
    };
    valid && !reserved
}

/// The suffixes that the release reserves for the types its generator names
/// itself: `Kind` for simple unions' tag enums from 2.4 to 6.1, `List` for
/// arrays from 2.5. Both apply to commands and events too until 6.0.
fn reserved_suffixes(command_or_event: bool) -> &'static [&'static str] {
    let version = crate::parser_version();
    if version < (2, 4, 0) {
        &[]
    } else if version < (2, 5, 0) {
        &["Kind"]
    } else if version < (6, 1, 0) || version < (6, 2, 0) && !command_or_event {
        &["Kind", "List"]
    } else if command_or_event {
        &[]
    } else {
        &["List"]
    }
}

/// Check a definition's name as the release's generator does, before its
/// reserved suffixes.
pub(crate) fn validate_definition(token: &QapiSchemaToken<'_>) -> SchemaResult {
    token.definition_name().map_or(Ok(()), validate_mandatory)
}

/// Refuse a definition name with a suffix that the release reserves.
pub(crate) fn reserved_suffix(token: &QapiSchemaToken<'_>) -> SchemaResult {
    let Some(name) = token.definition_name() else {
        return Ok(());
    };
    let command_or_event = matches!(
        token,
        QapiSchemaToken::Command(_) | QapiSchemaToken::Event(_)
    );
    for suffix in reserved_suffixes(command_or_event) {
        require(!name.ends_with(suffix), || {
            SchemaErrorKind::ReservedSuffix {
                name: name.to_owned(),
                suffix,
            }
        })?;
    }
    Ok(())
}

/// Check the name of a member, branch or feature.
pub(crate) fn validate(name: &str) -> SchemaResult {
    require(valid(name), || SchemaErrorKind::InvalidName {
        name: name.to_owned(),
    })
}

/// Check a name that QEMU 2.4 to 4.1 do not let be optional, a
/// definition's, branch's, feature's or discriminator: they refuse a `*`
/// first, and from 4.2 the name is invalid.
pub(crate) fn validate_mandatory(name: &str) -> SchemaResult {
    optional(name)?;
    validate(name)
}

fn optional(name: &str) -> SchemaResult {
    let version = crate::parser_version();
    require(
        !(name.starts_with('*') && ((2, 4, 0)..(4, 2, 0)).contains(&version)),
        || SchemaErrorKind::OptionalName {
            name: name.to_owned(),
        },
    )
}

/// Check the name of an enum member, which may also start with a digit,
/// since the C name of a member has the enum's name before it: QEMU checks
/// `_` and the name until 2.5, then a digit prefixed with `D`.
pub(crate) fn validate_enum_member(name: &str) -> SchemaResult {
    optional(name)?;
    let checked = if crate::parser_version() < (2, 6, 0) {
        format!("_{name}")
    } else if name.starts_with(|c: char| c.is_ascii_digit()) {
        format!("D{name}")
    } else {
        name.to_owned()
    };
    require(valid(&checked), || SchemaErrorKind::InvalidName {
        name: name.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition(version: (u32, u32, u32), text: &str) -> SchemaResult {
        crate::set_parser_version(version);
        let schema = crate::parse_schema(text).unwrap();
        validate_definition(&schema.0[0]).and_then(|()| reserved_suffix(&schema.0[0]))
    }

    #[test]
    fn definition_names_follow_historical_namespace_changes() {
        for (version, name, valid) in [
            ((2, 3, 0), "q-private", true),
            ((2, 3, 0), "1Thing", true),
            ((2, 4, 0), "_private", true),
            ((2, 4, 0), "dot.name", true),
            ((2, 4, 0), "q-private", true),
            ((2, 4, 0), "1Thing", false),
            ((2, 5, 0), "q-private", false),
            ((2, 5, 0), "q.private", false),
            ((2, 5, 0), "q_private", false),
            ((2, 5, 0), "dot.name", true),
            ((2, 6, 0), "_private", false),
            ((2, 6, 0), "dot.name", false),
            ((11, 1, 3), "__org.qemu_Name", true),
            ((11, 1, 3), "__org.qemu_x-name", true),
            ((11, 1, 3), "__org.qemu_q-name", true),
            ((11, 1, 3), "x-name", true),
            ((11, 1, 3), "Self", true),
            ((11, 1, 3), "___Name", false),
            ((11, 1, 3), "__org.qemu_1", false),
        ] {
            let result = definition(version, &format!("{{'struct':'{name}','data':{{}}}}"));
            assert_eq!(result.is_ok(), valid, "{version:?} {name}");
            if !valid {
                assert_eq!(
                    result.unwrap_err().kind(),
                    &SchemaErrorKind::InvalidName { name: name.into() }
                );
            }
        }
    }

    #[test]
    fn reserved_suffixes_follow_each_era_and_kind_of_definition() {
        for (version, text, suffix) in [
            ((2, 3, 0), "{'enum':'FooKind','data':[]}", None),
            ((2, 3, 0), "{'enum':'FooList','data':[]}", None),
            ((2, 4, 0), "{'enum':'FooKind','data':[]}", Some("Kind")),
            ((2, 4, 0), "{'command':'foo-Kind'}", Some("Kind")),
            ((2, 4, 0), "{'enum':'FooList','data':[]}", None),
            ((2, 5, 0), "{'struct':'FooList','data':{}}", Some("List")),
            ((2, 5, 0), "{'event':'FOOList'}", Some("List")),
            ((6, 0, 0), "{'struct':'FooList','data':{}}", Some("List")),
            (
                (6, 0, 0),
                "{'union':'FooKind','data':{'a':'str'}}",
                Some("Kind"),
            ),
            (
                (6, 1, 0),
                "{'alternate':'FooKind','data':{'a':'str'}}",
                Some("Kind"),
            ),
            ((6, 1, 0), "{'command':'foo-List'}", None),
            ((6, 1, 0), "{'event':'FOOKind'}", None),
            ((6, 2, 0), "{'enum':'FooKind','data':[]}", None),
            ((6, 2, 0), "{'enum':'FooList','data':[]}", Some("List")),
            ((11, 1, 3), "{'command':'foo-List'}", None),
        ] {
            let result = definition(version, text);
            match suffix {
                None => result.unwrap(),
                Some(suffix) => {
                    let error = result.unwrap_err();
                    assert!(
                        matches!(error.kind(), SchemaErrorKind::ReservedSuffix { suffix: s, .. } if *s == suffix),
                        "{version:?} {text}: {error}"
                    );
                }
            }
        }
    }

    #[test]
    fn enum_members_may_start_with_a_digit() {
        for (version, name, valid) in [
            ((2, 4, 0), "1-name", true),
            ((2, 5, 0), "q-private", true),
            ((2, 5, 0), "", true),
            ((2, 6, 0), "q-private", false),
            ((2, 6, 0), "", false),
            ((11, 1, 3), "1-name", true),
            ((11, 1, 3), "__org.qemu_1", false),
        ] {
            crate::set_parser_version(version);
            assert_eq!(
                validate_enum_member(name).is_ok(),
                valid,
                "{version:?} {name}"
            );
        }
        crate::set_parser_version((11, 1, 3));
        assert_eq!(
            validate_enum_member("q-x").unwrap_err().kind(),
            &SchemaErrorKind::InvalidName { name: "q-x".into() }
        );
        assert!(validate("1-name").is_err());
        validate("x-name").unwrap();
    }
}
