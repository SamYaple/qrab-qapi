//! Resolve QEMU's implicit `ElementList` names to ordinary array references.
use crate::unified::{SchemaContext, SchemaErrorKind, SchemaResult, definition, require};
use crate::{MembersOrRef, QapiMembers, QapiSchema, QapiSchemaToken, QapiString, QapiTypeRef};
use std::{collections::HashMap, path::Path};

fn visit_type<'a>(
    value: &mut QapiTypeRef<'a>,
    visitor: &mut impl FnMut(&mut QapiTypeRef<'a>) -> SchemaResult,
) -> SchemaResult {
    visitor(value)?;
    if let QapiTypeRef::InlineStruct(fields) = value {
        visit_members(fields, visitor)?;
    }
    Ok(())
}

fn visit_members<'a>(
    fields: &mut QapiMembers<'a>,
    visitor: &mut impl FnMut(&mut QapiTypeRef<'a>) -> SchemaResult,
) -> SchemaResult {
    for field in &mut fields.0 {
        visit_type(&mut field.r#type, visitor).within(|| format!("member '{}'", field.name))?;
    }
    Ok(())
}

fn visit_object<'a>(
    value: &mut MembersOrRef<'a>,
    visitor: &mut impl FnMut(&mut QapiTypeRef<'a>) -> SchemaResult,
) -> SchemaResult {
    if let MembersOrRef::Members(fields) = value {
        visit_members(fields, visitor)?;
    }
    Ok(())
}

fn visit<'a>(
    schema: &mut QapiSchema<'a>,
    mut visitor: impl FnMut(&mut QapiTypeRef<'a>) -> SchemaResult,
) -> SchemaResult {
    for token in &mut schema.0 {
        let label = definition(token);
        let mut visit_token = || -> SchemaResult {
            match token {
                QapiSchemaToken::Struct(value) => visit_members(&mut value.data, &mut visitor)?,
                QapiSchemaToken::Union(value) => {
                    visit_object(&mut value.base, &mut visitor)?;
                    for branch in &mut value.data.0 {
                        visit_type(&mut branch.r#type, &mut visitor)
                            .within(|| format!("branch '{}'", branch.name))?;
                    }
                }
                QapiSchemaToken::Alternate(value) => {
                    for branch in &mut value.data.0 {
                        visit_type(&mut branch.r#type, &mut visitor)
                            .within(|| format!("branch '{}'", branch.name))?;
                    }
                }
                QapiSchemaToken::Command(value) => {
                    if let Some(data) = &mut value.data {
                        visit_object(data, &mut visitor)?;
                    }
                    if let Some(returns) = &mut value.returns {
                        visit_type(returns, &mut visitor).within(|| "return type".into())?;
                    }
                }
                QapiSchemaToken::Event(value) => {
                    if let Some(data) = &mut value.data {
                        visit_object(data, &mut visitor)?;
                    }
                }
                _ => {}
            }
            Ok(())
        };
        visit_token().in_definition(|| label)?;
    }
    Ok(())
}

/// Resolve references to the `ElementList` names of arrays, which QEMU
/// accepts from 4.2: before, every array is spelled `[Element]`. The arrays of
/// the builtin types are predefined; another type's exists only where the
/// release spells it `[Element]` somewhere.
pub(crate) fn normalize<'a>(schemas: &mut [(&Path, QapiSchema<'a>)]) -> SchemaResult {
    if crate::parser_version() < (4, 2, 0) {
        return Ok(());
    }
    let mut aliases = HashMap::new();
    for element in crate::builtins::Builtins::current().types() {
        aliases.insert(format!("{element}List"), QapiString::Borrowed(element));
    }
    for (_, schema) in schemas.iter_mut() {
        visit(schema, |value| {
            if let QapiTypeRef::ArrayRef(element) = value {
                aliases
                    .entry(format!("{element}List"))
                    .or_insert_with(|| element.clone());
            }
            Ok(())
        })?;
    }
    for (path, schema) in schemas {
        visit(schema, |value| {
            let (name, array) = match &*value {
                QapiTypeRef::Ref(name) => (name.as_ref(), false),
                QapiTypeRef::ArrayRef(name) => (name.as_ref(), true),
                _ => return Ok(()),
            };
            if let Some(element) = aliases.get(name) {
                require(!array && !aliases.contains_key(element.as_ref()), || {
                    SchemaErrorKind::NestedArray {
                        name: name.to_owned(),
                    }
                })?;
                *value = QapiTypeRef::ArrayRef(element.clone());
            }
            Ok(())
        })
        .in_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn members(version: (u32, u32, u32), text: &str) -> SchemaResult<Vec<(String, String)>> {
        crate::set_parser_version(version);
        let mut schemas = vec![(Path::new("a.json"), crate::parse_schema(text).unwrap())];
        normalize(&mut schemas)?;
        let mut members = Vec::new();
        for token in &schemas[0].1.0 {
            if let QapiSchemaToken::Struct(value) = token {
                for field in &value.data.0 {
                    let shape = match &field.r#type {
                        QapiTypeRef::Ref(name) => name.to_string(),
                        QapiTypeRef::ArrayRef(name) => format!("[{name}]"),
                        _ => String::new(),
                    };
                    members.push((field.name.to_string(), shape));
                }
            }
        }
        Ok(members)
    }

    const LISTS: &str = "{'struct':'Item','data':{}}
        {'struct':'Holder','data':{'ints':'intList','nulls':'nullList','items':'ItemList',
         'direct':['Item']}}";

    #[test]
    fn element_list_names_are_arrays_from_4_2() {
        let shapes = |version| {
            members(version, LISTS)
                .unwrap()
                .into_iter()
                .map(|(_, shape)| shape)
                .collect::<Vec<_>>()
        };
        assert_eq!(shapes((4, 2, 0)), ["[int]", "[null]", "[Item]", "[Item]"]);
        assert_eq!(
            shapes((4, 1, 0)),
            ["intList", "nullList", "ItemList", "[Item]"]
        );
        let undeclared = members((4, 2, 0), "{'struct':'Holder','data':{'a':'ItemList'}}");
        assert_eq!(undeclared.unwrap(), [("a".into(), "ItemList".into())]);
    }

    #[test]
    fn arrays_of_arrays_are_refused() {
        for text in [
            "{'struct':'Holder','data':{'values':['intList']}}",
            "{'struct':'Holder','data':{'direct':['intList'],'alias':'intListList'}}",
        ] {
            let error = members((4, 2, 0), text).unwrap_err();
            assert!(
                matches!(error.kind(), SchemaErrorKind::NestedArray { .. }),
                "{error}"
            );
            assert_eq!(error.file(), Some(Path::new("a.json")));
            assert_eq!(error.definition(), Some("struct 'Holder'"));
        }
    }
}
