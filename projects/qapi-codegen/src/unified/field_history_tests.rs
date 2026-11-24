use super::merge::{
    TRACKED_BOOL_ATTRS, fix_recursive_types, has_feature, materialize_field_history, merge_enum,
    merge_struct,
};
use super::*;

fn field(name: &str, ty: &str, array: bool) -> StructField {
    StructField {
        name: name.into(),
        r#type: ty.into(),
        array,
        optional: false,
        meta: Metadata::default(),
    }
}

fn record(fields: Vec<StructField>) -> Struct {
    Struct {
        name: "Record".into(),
        fields,
        meta: Metadata::default(),
    }
}

#[test]
fn initially_deprecated_members_record_their_first_release() {
    let mut registry = UnifiedRegistry::default();
    let feature = Attribute::List(vec![Attribute::with_value("feature", "deprecated")]);
    let mut member = field("value", "str", false);
    member.meta.attributes.push(feature.clone());
    let record = record(vec![member]);
    let enumeration = Enum {
        name: "Kind".into(),
        variants: vec![EnumVariant {
            name: "value".into(),
            kind: EnumVariantKind::Unit,
            array: false,
            meta: Metadata {
                doc: None,
                schema_doc: None,
                schema_item: None,
                attributes: vec![feature],
            },
        }],
        meta: Metadata::default(),
    };
    for version in [(6, 1, 0), (8, 0, 0)] {
        merge_struct(&mut registry, "Record", &record, version);
        merge_enum(&mut registry, "Kind", &enumeration, version);
        let UnifiedType::Struct(value) = &registry.types["Record"] else {
            panic!()
        };
        assert_eq!(value.field_versions["value"].deprecated, Some((6, 1, 0)));
        let UnifiedType::Enum(value) = &registry.types["Kind"] else {
            panic!()
        };
        assert_eq!(value.variant_versions["value"].deprecated, Some((6, 1, 0)));
    }
}

#[test]
fn stabilization_clears_latest_type_and_field_feature_metadata() {
    let mut registry = UnifiedRegistry::default();
    for (major, unstable) in [(6, true), (8, false), (11, true), (12, false)] {
        let mut value = record(vec![field("value", "str", false)]);
        if unstable {
            let feature = Attribute::List(vec![Attribute::with_value("feature", "unstable")]);
            value.meta.attributes.push(feature.clone());
            value.fields[0].meta.attributes.push(feature);
        }
        merge_struct(&mut registry, "Record", &value, (major, 0, 0));
    }
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!()
    };
    assert!(!has_feature(&value.ir.meta, "unstable"));
    assert!(!has_feature(&value.ir.fields[0].meta, "unstable"));
    let expected = vec![((6, 0, 0), Some((8, 0, 0))), ((11, 0, 0), Some((12, 0, 0)))];
    assert_eq!(value.versions.unstable_ranges, expected);
    assert_eq!(value.field_versions["value"].unstable_ranges, expected);
}

#[test]
fn optional_fields_keep_one_lifetime_when_conditions_change() {
    let mut registry = UnifiedRegistry::default();
    let mut value = field("value", "str", false);
    value.optional = true;
    value
        .meta
        .attributes
        .push(Attribute::with_value("condition", "CONFIG_A"));
    merge_struct(
        &mut registry,
        "Record",
        &record(vec![value.clone()]),
        (1, 0, 0),
    );
    value.meta.attributes.clear();
    merge_struct(&mut registry, "Record", &record(vec![value]), (2, 0, 0));
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!()
    };
    assert!(value.field_history.is_empty());
    assert_eq!(value.field_versions["value"].since, (1, 0, 0));
    assert_eq!(condition(&value.ir.fields[0].meta), None);
}

#[test]
fn conditional_field_presence_preserves_requiredness_and_latest_conditions() {
    let mut registry = UnifiedRegistry::default();
    for (major, condition) in [(1, Some("CONFIG_A")), (2, None), (3, Some("CONFIG_B"))] {
        let mut value = field("value", "str", false);
        if let Some(condition) = condition {
            value
                .meta
                .attributes
                .push(Attribute::with_value("condition", condition));
        }
        merge_struct(&mut registry, "Record", &record(vec![value]), (major, 0, 0));
    }
    materialize_field_history(&mut registry);
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!()
    };
    assert_eq!(value.field_history.len(), 2);
    assert!(
        value.ir.fields[0]
            .meta
            .attributes
            .contains(&Attribute::with_value("condition", "CONFIG_B"))
    );
    assert!(
        !value.field_history[1]
            .0
            .meta
            .attributes
            .iter()
            .any(|attr| matches!(attr, Attribute::Item {name,..} if name == "condition"))
    );
}

#[test]
fn reintroduced_fields_keep_the_gap_between_lifetimes() {
    let mut registry = UnifiedRegistry::default();
    let present = record(vec![field("returning", "str", false)]);
    let absent = record(vec![]);
    for (major, value) in [(1, &present), (2, &absent), (3, &present)] {
        merge_struct(&mut registry, "Record", value, (major, 0, 0));
    }
    materialize_field_history(&mut registry);
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!()
    };
    assert_eq!(value.field_versions["returning"].since, (3, 0, 0));
    assert_eq!(value.field_history.len(), 1);
    assert_eq!(value.field_history[0].1.until, Some((2, 0, 0)));
}

#[test]
fn field_optionality_changes_preserve_each_wire_contract() {
    let mut registry = UnifiedRegistry::default();
    for (major, optional) in [(1, true), (2, false), (3, true)] {
        let mut value = field("value", "str", false);
        value.optional = optional;
        merge_struct(&mut registry, "Record", &record(vec![value]), (major, 0, 0));
    }
    materialize_field_history(&mut registry);
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!()
    };
    assert_eq!(value.field_history.len(), 2);
    assert_eq!(value.field_versions["value"].since, (3, 0, 0));
    assert!(value.field_history[0].0.optional);
    assert!(!value.field_history[1].0.optional);
}

#[test]
fn field_type_history_preserves_arrays_and_does_not_collide_with_real_fields() {
    let mut registry = UnifiedRegistry::default();
    for (major, ty, array) in [
        (1, "int", false),
        (2, "str", false),
        (3, "int", true),
        (4, "int64", true),
    ] {
        let value = record(vec![
            field("value", ty, array),
            field("value_legacy", "bool", false),
        ]);
        merge_struct(&mut registry, "Record", &value, (major, 0, 0));
    }
    materialize_field_history(&mut registry);
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!()
    };
    assert_eq!(value.field_history.len(), 2);
    assert_eq!(value.field_versions["value"].since, (3, 0, 0));
    assert_eq!(value.field_versions["value_legacy2"].until, Some((2, 0, 0)));
    assert_eq!(value.field_versions["value_legacy3"].until, Some((3, 0, 0)));
    assert_eq!(value.ir.fields.len(), 4);
}

#[test]
fn the_earlier_shape_of_a_keyword_field_is_named_without_r_hash() {
    let mut registry = UnifiedRegistry::default();
    for (major, ty) in [(1, "int"), (2, "str")] {
        merge_struct(
            &mut registry,
            "Record",
            &record(vec![field("type", ty, false)]),
            (major, 0, 0),
        );
    }
    materialize_field_history(&mut registry);
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!()
    };
    let legacy = &value.ir.fields[1];
    assert_eq!(legacy.name, "type_legacy");
    assert_eq!(legacy.r#type, "int");
    assert!(
        legacy
            .meta
            .attributes
            .contains(&Attribute::with_value("name", "type"))
    );
    assert_eq!(value.field_versions["type_legacy"].until, Some((2, 0, 0)));
}

#[test]
fn field_wire_renames_preserve_original_keys() {
    let mut registry = UnifiedRegistry::default();
    merge_struct(
        &mut registry,
        "Record",
        &record(vec![field("foo-bar", "str", false)]),
        (1, 0, 0),
    );
    merge_struct(
        &mut registry,
        "Record",
        &record(vec![field("foo_bar", "str", false)]),
        (2, 0, 0),
    );
    materialize_field_history(&mut registry);
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!()
    };
    assert_eq!(value.field_versions["foo_bar"].since, (2, 0, 0));
    let old = value
        .ir
        .fields
        .iter()
        .find(|field| field.name == "foo_bar_legacy")
        .unwrap();
    assert!(
        old.meta
            .attributes
            .contains(&Attribute::with_value("name", "foo-bar"))
    );
}

#[test]
fn recursive_vectors_break_size_cycles_but_inline_references_require_boxing() {
    for array in [false, true] {
        let mut registry = UnifiedRegistry::default();
        let value = Struct {
            name: "Node".into(),
            fields: vec![field("children", "Node", array)],
            meta: Metadata::default(),
        };
        merge_struct(&mut registry, "Node", &value, (1, 0, 0));
        fix_recursive_types(&mut registry);
        let UnifiedType::Struct(value) = &registry.types["Node"] else {
            panic!()
        };
        assert_eq!(
            value.ir.fields[0]
                .meta
                .attributes
                .contains(&Attribute::new("boxed")),
            !array
        );
    }
    let mut registry = UnifiedRegistry::default();
    let value = Struct {
        name: "Node".into(),
        fields: vec![field("value", "Branch", false)],
        meta: Metadata::default(),
    };
    merge_struct(&mut registry, "Node", &value, (1, 0, 0));
    merge_enum(
        &mut registry,
        "Branch",
        &Enum {
            name: "Branch".into(),
            meta: Metadata::default(),
            variants: vec![EnumVariant {
                name: "nodes".into(),
                kind: EnumVariantKind::Tuple("Node".into()),
                array: true,
                meta: Metadata::default(),
            }],
        },
        (1, 0, 0),
    );
    fix_recursive_types(&mut registry);
    let UnifiedType::Struct(value) = &registry.types["Node"] else {
        panic!()
    };
    assert!(
        !value.ir.fields[0]
            .meta
            .attributes
            .contains(&Attribute::new("boxed"))
    );
}

fn release(major: u32, structs: &[&str], enums: &[&str]) -> super::lower::VersionResult {
    super::lower::VersionResult {
        tag: format!("v{major}.0.0"),
        version: (major, 0, 0),
        order: (major, 0, 0, 0),
        schema_hash: major.into(),
        enums: enums
            .iter()
            .map(|name| {
                let value = Enum {
                    name: (*name).into(),
                    meta: Metadata::default(),
                    variants: Vec::new(),
                };
                ((*name).to_owned(), value)
            })
            .collect(),
        structs: structs
            .iter()
            .map(|name| ((*name).to_owned(), record(Vec::new())))
            .collect(),
        warnings: Vec::new(),
    }
}

#[test]
fn types_that_return_or_change_kind_are_errors() {
    let registry = super::registry::merge_releases(
        "QMP",
        vec![release(1, &["Record"], &[]), release(2, &["Record"], &[])],
    )
    .unwrap();
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!("struct")
    };
    assert_eq!(
        (value.versions.since, value.versions.until),
        ((1, 0, 0), None)
    );

    let error = super::registry::merge_releases(
        "QGA",
        vec![
            release(1, &["Record"], &[]),
            release(2, &[], &[]),
            release(3, &["Record"], &[]),
        ],
    )
    .unwrap_err();
    assert!(
        matches!(&error, RegistryError::Reappeared { protocol: "QGA", name, removed, tag }
            if name == "Record" && removed == "v2.0.0" && tag == "v3.0.0"),
        "{error:?}"
    );
    assert_eq!(
        error.to_string(),
        "QGA type Record returns in v3.0.0 after its removal in v2.0.0"
    );

    let error = super::registry::merge_releases(
        "QMP",
        vec![release(1, &["Record"], &[]), release(2, &[], &["Record"])],
    )
    .unwrap_err();
    assert!(
        matches!(
            &error,
            RegistryError::KindChanged { protocol: "QMP", name, previous: "struct", kind: "enum", tag }
                if name == "Record" && tag == "v2.0.0"
        ),
        "{error:?}"
    );
    assert_eq!(
        error.to_string(),
        "QMP type Record changes from struct to enum in v2.0.0"
    );
    // Of several refused types, the first by name is reported.
    let error = super::registry::merge_releases(
        "QMP",
        vec![
            release(
                1,
                &["Alpha", "Beta", "Gamma", "Delta", "Omega", "Sigma"],
                &[],
            ),
            release(
                2,
                &[],
                &["Omega", "Sigma", "Delta", "Gamma", "Beta", "Alpha"],
            ),
        ],
    )
    .unwrap_err();
    assert!(
        matches!(&error, RegistryError::KindChanged { name, .. } if name == "Alpha"),
        "{error:?}"
    );
}

#[test]
fn reintroduced_flags_start_a_new_lifetime() {
    for flag in TRACKED_BOOL_ATTRS {
        let mut registry = UnifiedRegistry::default();
        for (major, present) in [(1, true), (2, false), (3, true)] {
            let mut value = Struct {
                name: "Example".into(),
                fields: vec![],
                meta: Metadata::default(),
            };
            if present {
                value.meta.attributes.push(Attribute::new(flag));
            }
            merge_struct(&mut registry, "Example", &value, (major, 0, 0));
        }
        let UnifiedType::Struct(value) = &registry.types["Example"] else {
            panic!()
        };
        assert_eq!(value.attr_versions[*flag].since, (3, 0, 0), "{flag}");
        assert_eq!(value.attr_versions[*flag].until, None, "{flag}");
        let [(name, removed)] = &value.attr_history[..] else {
            panic!("{flag}: {:?}", value.attr_history)
        };
        assert_eq!(name, flag);
        assert_eq!((removed.since, removed.until), ((1, 0, 0), Some((2, 0, 0))));
        assert!(value.flag_enabled(flag, (1, 0, 0)));
        assert!(!value.flag_enabled(flag, (2, 0, 0)));
        assert!(value.flag_enabled(flag, (3, 0, 0)));
    }
}
