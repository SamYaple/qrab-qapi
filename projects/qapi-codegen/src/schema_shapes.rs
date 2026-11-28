//! The shapes of the values that each release's QAPI generator checks.
//!
//! From 2.4 a value that the grammar of its place refuses is kept as a
//! [`RawValue`] of the dictionary that holds it, and the validation refuses
//! it where, and as, the release's QEMU does, in the order of its checks.
//! A value that QEMU uses without checking it fails there with
//! [`Crash::UncheckedValue`]; one that it takes without checking it is
//! [`Construct::Value`] once nothing else is wrong.

use crate::{
    parsers::common::{Malformed, Raw, RawValue},
    schema_validation::since,
    unified::{
        Construct, Crash, SchemaError, SchemaErrorKind, SchemaResult, Shape,
        require,
    },
};

/// The value of `key` that the grammar refused.
pub(crate) fn bad<'t, 'i>(
    malformed: &'t Malformed<'i>,
    key: &str,
) -> Option<&'t RawValue<'i>> {
    crate::parsers::common::malformed(malformed, key)
}

fn shape(expected: Shape) -> SchemaError {
    SchemaErrorKind::ValueShape { expected }.into()
}

/// QEMU's failure on the value of `key`, which it uses without checking.
pub(crate) fn unchecked(key: &str) -> SchemaError {
    SchemaErrorKind::Crash {
        crash: Crash::UncheckedValue { key: key.to_owned() },
    }
    .into()
}

/// A value that QEMU takes without checking it, which the IR cannot
/// represent.
pub(crate) fn unsupported(key: &str) -> SchemaError {
    SchemaErrorKind::Unsupported {
        construct: Construct::Value { key: key.to_owned() },
    }
    .into()
}

/// A value where QEMU takes a type, `array` where it takes an array of
/// one too, `object` where it takes members written out in place: its
/// `check_type` until 8.1, then `check_type_name`, `check_type_name_or_array`
/// and `check_type_implicit`. `null`, which QEMU reads until 4.1, is no
/// type to check.
pub(crate) fn type_value(
    value: &RawValue<'_>,
    array: bool,
    object: bool,
) -> SchemaResult {
    let modern = since((8, 1, 0));
    Err(match value.value {
        Raw::Null => return Ok(()),
        Raw::Array(_) if array => shape(Shape::ArrayOfOneTypeName),
        Raw::Array(_) if !modern => SchemaErrorKind::ArrayNotAllowed.into(),
        Raw::Object(_) if array && modern => shape(Shape::TypeNameOrArray),
        Raw::Object(_) if !object => SchemaErrorKind::AnonymousStruct.into(),
        // 2.4 asks for an object first.
        _ if !since((2, 5, 0)) => shape(Shape::Object),
        _ if object => shape(Shape::ObjectOrTypeName),
        _ if array && modern => shape(Shape::TypeNameOrArray),
        _ => shape(Shape::TypeName),
    })
}

/// A name that is no string.
pub(crate) fn name(_: &RawValue<'_>) -> SchemaResult {
    Err(SchemaErrorKind::NameNotString.into())
}

/// A value of a key that must be an array, an enum's values or features.
pub(crate) fn array(value: &RawValue<'_>) -> SchemaResult {
    require(matches!(value.value, Raw::Array(_)), || {
        SchemaErrorKind::ValueShape { expected: Shape::Array }
    })
}

/// An enum's prefix, which must be a string from 2.5.
pub(crate) fn string(value: &RawValue<'_>) -> SchemaResult {
    require(matches!(value.value, Raw::String(_)), || {
        SchemaErrorKind::ValueShape { expected: Shape::String }
    })
}

/// Python's `len` of a value, which only strings, arrays and objects have.
pub(crate) fn len(value: &RawValue<'_>) -> Option<usize> {
    match &value.value {
        Raw::String(text) => Some(text.chars().count()),
        Raw::Array(values) => Some(values.len()),
        Raw::Object(entries) => Some(entries.len()),
        Raw::Bool(_) | Raw::Null => None,
    }
}

/// A condition that the grammar refused: from 3.0 to 6.1 one that is no
/// string or list of strings, or a list with one that is no string, from
/// 6.2 any condition that is not well formed, checked as QEMU's `check_if`
/// does.
pub(crate) fn condition(value: &RawValue<'_>) -> SchemaResult {
    if since((6, 2, 0)) {
        return expression(value);
    }
    let Raw::Array(values) = &value.value else {
        return Err(shape(Shape::Condition));
    };
    require(!values.is_empty(), || SchemaErrorKind::EmptyCondition {
        condition: "[]".into(),
    })?;
    for value in values {
        let Raw::String(name) = &value.value else {
            return Err(shape(Shape::Condition));
        };
        let blank = if since((4, 2, 0)) {
            name.trim().is_empty()
        } else {
            name.is_empty()
        };
        require(!blank, || SchemaErrorKind::EmptyCondition {
            condition: format!("'{name}'"),
        })?;
    }
    Ok(())
}

/// A condition from 6.2: a configuration name, or an object of one of
/// `all`, `any` and `not`.
fn expression(value: &RawValue<'_>) -> SchemaResult {
    match &value.value {
        Raw::String(name) => {
            let bytes = name.as_bytes();
            require(
                bytes.first().is_some_and(u8::is_ascii_uppercase)
                    && bytes.iter().all(|b| {
                        b.is_ascii_uppercase()
                            || b.is_ascii_digit()
                            || *b == b'_'
                    }),
                || SchemaErrorKind::InvalidCondition {
                    condition: format!("'{name}'"),
                },
            )
        }
        Raw::Object(entries) => {
            let mut unknown: Vec<&str> = entries
                .iter()
                .map(|(key, _)| key.as_ref())
                .filter(|key| !matches!(*key, "all" | "any" | "not"))
                .collect();
            unknown.sort();
            if let Some(key) = unknown.first() {
                return Err(SchemaErrorKind::UnknownKey {
                    key: (*key).to_owned(),
                }
                .into());
            }
            let [(operator, operand)] = &entries[..] else {
                return Err(SchemaErrorKind::ConditionKeys.into());
            };
            if operator == "not" {
                return expression(operand);
            }
            let Raw::Array(operands) = &operand.value else {
                return Err(shape(Shape::Array));
            };
            require(!operands.is_empty(), || {
                SchemaErrorKind::EmptyCondition { condition: "[]".into() }
            })?;
            operands.iter().try_for_each(expression)
        }
        _ => Err(shape(Shape::Condition)),
    }
}

/// The `data` of a union or alternate that is no object, which QEMU 2.4 to
/// 6.0 iterate as one: first, from 2.6, an alternate's count of branches
/// (`minimum`) or a union's emptiness, which Python takes from the length
/// of a string or array, from 5.0 an alternate's from its truth; from 6.1
/// QEMU refuses it.
pub(crate) fn branches(
    value: &RawValue<'_>,
    minimum: Option<usize>,
) -> SchemaResult {
    if since((6, 1, 0)) {
        // An alternate's emptiness first.
        return Err(match minimum {
            Some(minimum) if !value.truthy() => {
                SchemaErrorKind::AlternateBranches { minimum }.into()
            }
            _ => shape(Shape::Object),
        });
    }
    let count = if since((5, 0, 0)) && minimum.is_some() {
        Some(usize::from(value.truthy()))
    } else if since((2, 6, 0)) && (minimum.is_some() || !since((4, 2, 0))) {
        Some(len(value).ok_or_else(|| unchecked("data"))?)
    } else {
        None
    };
    match (count, minimum) {
        (Some(count), Some(minimum)) if count < minimum => {
            Err(SchemaErrorKind::AlternateBranches { minimum }.into())
        }
        (Some(0), None) => Err(SchemaErrorKind::NoBranches.into()),
        _ => Err(unchecked("data")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::common::raw_value;

    fn value(text: &str) -> RawValue<'_> {
        raw_value(text).unwrap().1
    }

    fn kind(result: SchemaResult) -> Option<SchemaErrorKind> {
        result.err().map(|error| error.kind().clone())
    }

    #[test]
    fn types_follow_the_checks_of_each_release() {
        let expected =
            |expected| Some(SchemaErrorKind::ValueShape { expected });
        for (version, text, array, object, outcome) in [
            ((2, 4, 0), "true", false, false, expected(Shape::Object)),
            ((2, 5, 0), "true", false, false, expected(Shape::TypeName)),
            ((2, 5, 0), "true", false, true, expected(Shape::ObjectOrTypeName)),
            ((4, 1, 0), "null", false, false, None),
            (
                (4, 1, 0),
                "['a', 'b']",
                true,
                false,
                expected(Shape::ArrayOfOneTypeName),
            ),
            (
                (8, 0, 0),
                "[]",
                false,
                false,
                Some(SchemaErrorKind::ArrayNotAllowed),
            ),
            ((8, 1, 0), "[]", false, false, expected(Shape::TypeName)),
            ((8, 1, 0), "[]", false, true, expected(Shape::ObjectOrTypeName)),
            ((8, 1, 0), "{}", true, false, expected(Shape::TypeNameOrArray)),
            (
                (8, 0, 0),
                "{}",
                true,
                false,
                Some(SchemaErrorKind::AnonymousStruct),
            ),
            ((8, 1, 0), "false", true, false, expected(Shape::TypeNameOrArray)),
        ] {
            crate::set_parser_version(version);
            assert_eq!(
                kind(type_value(&value(text), array, object)),
                outcome,
                "{version:?} {text}"
            );
        }
    }

    #[test]
    fn conditions_follow_the_checks_of_each_release() {
        let empty = |condition: &str| {
            Some(SchemaErrorKind::EmptyCondition {
                condition: condition.into(),
            })
        };
        for (version, text, outcome) in [
            (
                (4, 1, 0),
                "{}",
                Some(SchemaErrorKind::ValueShape {
                    expected: Shape::Condition,
                }),
            ),
            (
                (4, 1, 0),
                "[' ', true]",
                Some(SchemaErrorKind::ValueShape {
                    expected: Shape::Condition,
                }),
            ),
            ((4, 2, 0), "[' ', true]", empty("' '")),
            (
                (6, 2, 0),
                "'x'",
                Some(SchemaErrorKind::InvalidCondition {
                    condition: "'x'".into(),
                }),
            ),
            (
                (6, 2, 0),
                "{ 'b': 'X', 'a': 'Y' }",
                Some(SchemaErrorKind::UnknownKey { key: "a".into() }),
            ),
            (
                (6, 2, 0),
                "{ 'all': ['X'], 'not': 'Y' }",
                Some(SchemaErrorKind::ConditionKeys),
            ),
            ((6, 2, 0), "{ 'any': [] }", empty("[]")),
            (
                (6, 2, 0),
                "{ 'not': { 'all': 'X' } }",
                Some(SchemaErrorKind::ValueShape { expected: Shape::Array }),
            ),
            ((6, 2, 0), "{ 'not': { 'all': ['X', 'Y_1'] } }", None),
        ] {
            crate::set_parser_version(version);
            assert_eq!(
                kind(condition(&value(text))),
                outcome,
                "{version:?} {text}"
            );
        }
    }

    #[test]
    fn branches_that_are_no_object_fail_where_qemu_fails() {
        let crash = Some(SchemaErrorKind::Crash {
            crash: Crash::UncheckedValue { key: "data".into() },
        });
        for (version, text, minimum, outcome) in [
            ((2, 5, 0), "[]", Some(2), crash.clone()),
            (
                (2, 6, 0),
                "['a']",
                Some(2),
                Some(SchemaErrorKind::AlternateBranches { minimum: 2 }),
            ),
            ((2, 6, 0), "true", Some(2), crash.clone()),
            ((2, 6, 0), "[]", None, Some(SchemaErrorKind::NoBranches)),
            ((4, 2, 0), "[]", None, crash.clone()),
            (
                (5, 0, 0),
                "false",
                Some(1),
                Some(SchemaErrorKind::AlternateBranches { minimum: 1 }),
            ),
            ((5, 0, 0), "true", Some(1), crash),
            (
                (6, 1, 0),
                "'x'",
                None,
                Some(SchemaErrorKind::ValueShape { expected: Shape::Object }),
            ),
        ] {
            crate::set_parser_version(version);
            assert_eq!(
                kind(branches(&value(text), minimum)),
                outcome,
                "{version:?} {text}"
            );
        }
    }
}
