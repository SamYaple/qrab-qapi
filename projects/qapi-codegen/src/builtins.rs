//! QAPI definitions supplied implicitly by QEMU rather than schema files.
use crate::{
    MembersOrRef, QapiEnum, QapiEnumValue, QapiMembers, QapiSchema,
    QapiSchemaToken, QapiStruct, QapiTypeRef,
    unified::{
        SchemaContext, SchemaError, SchemaErrorKind, SchemaResult, Version,
    },
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

/// The definitions that QEMU predefines from 2.6, in every release.
pub(crate) const PREDEFINED: [&str; 2] = ["QType", "q_empty"];

/// The types that QEMU's generator predefines, which changed in four
/// releases: `builtin_types` and `_def_predefineds` in its `scripts/qapi.py`,
/// later in `scripts/qapi/schema.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Builtins {
    /// Before 2.4, which checks no names: the scalar types, and `visitor`,
    /// `dict` and `**` for untyped values.
    Untyped,
    /// 2.4: the scalar types, and `**`, the untyped value of a command whose
    /// code QEMU does not generate.
    Scalars,
    /// 2.5: `any` replaces `**`.
    Any,
    /// 2.6 to 2.9: also the enum `QType`, with distinct `qint` and `qfloat`
    /// numbers, and the empty struct `q_empty`.
    QType,
    /// From 2.10: also `null`, and `QType` has one `qnum` for all numbers.
    Null,
}

/// The builtin types, with the first era that has each and the first that no
/// longer does.
const TYPES: &[(&str, Builtins, Option<Builtins>)] = &[
    ("str", Builtins::Untyped, None),
    ("number", Builtins::Untyped, None),
    ("int", Builtins::Untyped, None),
    ("int8", Builtins::Untyped, None),
    ("int16", Builtins::Untyped, None),
    ("int32", Builtins::Untyped, None),
    ("int64", Builtins::Untyped, None),
    ("uint8", Builtins::Untyped, None),
    ("uint16", Builtins::Untyped, None),
    ("uint32", Builtins::Untyped, None),
    ("uint64", Builtins::Untyped, None),
    ("size", Builtins::Untyped, None),
    ("bool", Builtins::Untyped, None),
    ("visitor", Builtins::Untyped, Some(Builtins::Scalars)),
    ("dict", Builtins::Untyped, Some(Builtins::Scalars)),
    ("**", Builtins::Untyped, Some(Builtins::Any)),
    ("any", Builtins::Any, None),
    ("null", Builtins::Null, None),
];

impl Builtins {
    pub(crate) fn of(version: Version) -> Self {
        if version < (2, 4, 0) {
            Self::Untyped
        } else if version < (2, 5, 0) {
            Self::Scalars
        } else if version < (2, 6, 0) {
            Self::Any
        } else if version < (2, 10, 0) {
            Self::QType
        } else {
            Self::Null
        }
    }

    /// The era of the release being lowered.
    pub(crate) fn current() -> Self {
        Self::of(crate::parser_version())
    }

    /// The names of the builtin types of this era.
    pub(crate) fn types(self) -> impl Iterator<Item = &'static str> {
        TYPES
            .iter()
            .filter(move |(_, since, until)| {
                *since <= self && until.is_none_or(|until| self < until)
            })
            .map(|(name, _, _)| *name)
    }

    /// Whether `name` is a builtin type of this era.
    pub(crate) fn defines(self, name: &str) -> bool {
        self.types().any(|builtin| builtin == name)
    }

    /// Whether no definition may take `name`: a builtin type, or `QType`.
    /// QEMU checks no names before 2.4, but there too a definition of a
    /// builtin's name could not be told apart from the builtin.
    pub(crate) fn reserves(self, name: &str) -> bool {
        self.defines(name) || self >= Self::QType && name == "QType"
    }

    /// The definitions of `names` that QEMU makes in every release of this
    /// era.
    fn predefined(
        self,
        names: &BTreeSet<&str>,
    ) -> Vec<QapiSchemaToken<'static>> {
        if self < Self::QType {
            return Vec::new();
        }
        let values: &[&'static str] = if self == Self::QType {
            &[
                "none", "qnull", "qint", "qstring", "qdict", "qlist", "qfloat",
                "qbool",
            ]
        } else {
            &["none", "qnull", "qnum", "qstring", "qdict", "qlist", "qbool"]
        };
        let tokens = [
            QapiSchemaToken::Enum(QapiEnum {
                name: "QType".into(),
                prefix: Some("QTYPE".into()),
                data: values
                    .iter()
                    .map(|&name| QapiEnumValue {
                        name: name.into(),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }),
            QapiSchemaToken::Struct(QapiStruct {
                name: "q_empty".into(),
                ..Default::default()
            }),
        ];
        tokens
            .into_iter()
            .filter(|token| {
                token.definition_name().is_some_and(|name| names.contains(name))
            })
            .collect()
    }
}

fn type_references<'a>(
    value: &'a QapiTypeRef<'_>,
    names: &mut BTreeSet<&'a str>,
) {
    match value {
        QapiTypeRef::Ref(name) | QapiTypeRef::ArrayRef(name) => {
            names.insert(name);
        }
        QapiTypeRef::InlineStruct(fields) => member_references(fields, names),
        QapiTypeRef::Unset => {}
    }
}

fn member_references<'a>(
    fields: &'a QapiMembers<'_>,
    names: &mut BTreeSet<&'a str>,
) {
    for field in &fields.0 {
        type_references(&field.r#type, names);
    }
}

fn object_references<'a>(
    value: &'a MembersOrRef<'_>,
    names: &mut BTreeSet<&'a str>,
) {
    match value {
        MembersOrRef::Ref(name) => {
            names.insert(name);
        }
        MembersOrRef::Members(fields) => member_references(fields, names),
        MembersOrRef::Unset => {}
    }
}

/// The [`PREDEFINED`] definitions that the schemas refer to.
pub(crate) fn used_predefined(
    schemas: &[(&Path, QapiSchema<'_>)],
) -> BTreeSet<&'static str> {
    let mut names = BTreeSet::new();
    for token in schemas.iter().flat_map(|(_, schema)| &schema.0) {
        match token {
            QapiSchemaToken::Struct(value) => {
                if let Some(base) = &value.base {
                    names.insert(base.as_ref());
                }
                member_references(&value.data, &mut names);
            }
            QapiSchemaToken::Union(value) => {
                object_references(&value.base, &mut names);
                for branch in &value.data.0 {
                    type_references(&branch.r#type, &mut names);
                }
            }
            QapiSchemaToken::Alternate(value) => {
                for branch in &value.data.0 {
                    type_references(&branch.r#type, &mut names);
                }
            }
            QapiSchemaToken::Command(value) => {
                if let Some(data) = &value.data {
                    object_references(data, &mut names);
                }
                if let Some(returns) = &value.returns {
                    type_references(returns, &mut names);
                }
            }
            QapiSchemaToken::Event(value) => {
                if let Some(data) = &value.data {
                    object_references(data, &mut names);
                }
            }
            _ => {}
        }
    }
    PREDEFINED.into_iter().filter(|name| names.contains(name)).collect()
}

/// Parse a release's schema files and add the predefined definitions of
/// `predefined` that the release's generator makes. A selection of releases
/// defines a predefined type in every release of its era once one release
/// uses it, as QEMU does, rather than only in the releases that use it. The
/// validation checks the definitions' names.
pub(crate) fn parse_sources<'s>(
    sources: &'s [(PathBuf, String)],
    predefined: &BTreeSet<&str>,
) -> SchemaResult<Vec<(&'s Path, QapiSchema<'s>)>> {
    let mut schemas = sources
        .iter()
        .map(|(path, content)| {
            let schema = crate::parse_schema(content)
                .map_err(|error| {
                    let position = (error.line, error.column);
                    SchemaError::from(SchemaErrorKind::Parse(error))
                        .at(Some(position))
                })
                .in_file(path)?;
            Ok((path.as_path(), schema))
        })
        .collect::<SchemaResult<Vec<_>>>()?;
    crate::schema_docs::resolve_legacy(&mut schemas);
    crate::arrays::normalize(&mut schemas, sources)?;
    let predefined = Builtins::current().predefined(predefined);
    if !predefined.is_empty() {
        schemas.push((
            Path::new("<predefined QAPI types>"),
            QapiSchema(predefined),
        ));
    }
    Ok(schemas)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builtin_types_follow_each_era() {
        for (version, builtins) in [
            ((2, 3, 0), &["visitor", "dict", "**"][..]),
            ((2, 4, 0), &["size", "**"]),
            ((2, 5, 0), &["any"]),
            ((2, 10, 0), &["any", "null"]),
        ] {
            let era = Builtins::of(version);
            for builtin in builtins {
                assert!(era.defines(builtin), "{version:?} {builtin}");
            }
        }
        for (version, other) in [
            ((2, 4, 0), "dict"),
            ((2, 4, 0), "any"),
            ((2, 5, 0), "**"),
            ((2, 9, 0), "null"),
        ] {
            assert!(
                !Builtins::of(version).defines(other),
                "{version:?} {other}"
            );
        }
    }

    #[test]
    fn qtype_and_the_empty_struct_are_predefined_from_2_6() {
        let predefined = |version| -> (Vec<String>, Vec<String>) {
            crate::set_parser_version(version);
            let schemas = parse_sources(&[], &PREDEFINED.into()).unwrap();
            let tokens: Vec<_> =
                schemas.iter().flat_map(|(_, schema)| &schema.0).collect();
            let names = tokens
                .iter()
                .filter_map(|token| token.definition_name().map(str::to_owned))
                .collect();
            let values = tokens
                .iter()
                .find_map(|token| match token {
                    QapiSchemaToken::Enum(value) if value.name == "QType" => {
                        Some(
                            value
                                .data
                                .iter()
                                .map(|v| v.name.to_string())
                                .collect(),
                        )
                    }
                    _ => None,
                })
                .unwrap_or_default();
            (names, values)
        };
        assert_eq!(predefined((2, 5, 0)), (vec![], vec![]));
        let (names, values) = predefined((2, 6, 0));
        assert_eq!(names, ["QType", "q_empty"]);
        assert_eq!(
            values,
            [
                "none", "qnull", "qint", "qstring", "qdict", "qlist", "qfloat",
                "qbool"
            ]
        );
        let (names, values) = predefined((2, 10, 0));
        assert_eq!(names, ["QType", "q_empty"]);
        assert_eq!(
            values,
            ["none", "qnull", "qnum", "qstring", "qdict", "qlist", "qbool"]
        );
    }
}
