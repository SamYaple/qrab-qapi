use super::{
    merge::{
        has_feature, materialize_field_history, materialize_variant_history,
        merge_enum, merge_struct,
    },
    *,
};

fn enumeration(wire: &str, payload: EnumVariantKind, array: bool) -> Enum {
    Enum {
        name: "Example".into(),
        meta: Metadata::default(),
        variants: vec![EnumVariant {
            name: wire.into(),
            kind: payload,
            array,
            meta: Metadata {
                attributes: vec![Attribute::with_value("name", wire)],
                ..Default::default()
            },
        }],
    }
}

#[test]
fn enum_and_variant_instability_retains_stabilization_gaps() {
    let mut registry = UnifiedRegistry::default();
    for (major, unstable) in [(6, true), (8, false), (11, true), (12, false)] {
        let mut value = enumeration("value", EnumVariantKind::Unit, false);
        if unstable {
            let feature = Attribute::List(vec![Attribute::with_value(
                "feature", "unstable",
            )]);
            value.meta.attributes.push(feature.clone());
            value.variants[0].meta.attributes.push(feature);
        }
        merge_enum(&mut registry, "Example", &value, (major, 0, 0));
    }
    let UnifiedType::Enum(value) = &registry.types["Example"] else { panic!() };
    assert!(!has_feature(&value.ir.meta, "unstable"));
    assert!(!has_feature(&value.ir.variants[0].meta, "unstable"));
    for (version, expected) in [
        ((6, 0, 0), true),
        ((8, 0, 0), false),
        ((11, 0, 0), true),
        ((12, 0, 0), false),
    ] {
        assert_eq!(value.versions.is_unstable(version), expected);
        assert_eq!(
            value.variant_versions["value"].is_unstable(version),
            expected
        );
    }
}

#[test]
fn latest_type_and_variant_conditions_replace_or_clear_old_conditions() {
    let mut registry = UnifiedRegistry::default();
    let mut value = enumeration("value", EnumVariantKind::Unit, false);
    value.meta.attributes.push(Attribute::with_value("condition", "CONFIG_A"));
    value.variants[0]
        .meta
        .attributes
        .push(Attribute::with_value("condition", "CONFIG_A"));
    merge_enum(&mut registry, "Example", &value, (1, 0, 0));
    value.meta.attributes =
        vec![Attribute::with_value("condition", "CONFIG_B")];
    value.variants[0]
        .meta
        .attributes
        .retain(|attr| !matches!(attr, Attribute::Item {name,..} if name == "condition"));
    merge_enum(&mut registry, "Example", &value, (2, 0, 0));
    let UnifiedType::Enum(value) = &registry.types["Example"] else { panic!() };
    assert_eq!(condition(&value.ir.meta), Some("CONFIG_B"));
    assert_eq!(condition(&value.ir.variants[0].meta), None);
    let mut value = Struct {
        name: "Record".into(),
        fields: vec![],
        meta: Metadata::default(),
    };
    value.meta.attributes.push(Attribute::with_value("condition", "CONFIG_A"));
    merge_struct(&mut registry, "Record", &value, (1, 0, 0));
    value.meta.attributes.clear();
    merge_struct(&mut registry, "Record", &value, (2, 0, 0));
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!()
    };
    assert_eq!(condition(&value.ir.meta), None);
}

#[test]
fn reintroduced_variants_keep_the_gap_between_lifetimes() {
    let mut registry = UnifiedRegistry::default();
    let present = enumeration("returning", EnumVariantKind::Unit, false);
    let mut absent = present.clone();
    absent.variants.clear();
    for (major, value) in [(1, &present), (2, &absent), (3, &present)] {
        merge_enum(&mut registry, "Example", value, (major, 0, 0));
    }
    materialize_variant_history(&mut registry, "QMP").unwrap();
    let UnifiedType::Enum(value) = &registry.types["Example"] else { panic!() };
    assert_eq!(value.variant_versions["returning"].since, (3, 0, 0));
    assert_eq!(value.variant_history.len(), 1);
    assert_eq!(value.variant_history[0].1.until, Some((2, 0, 0)));
}

#[test]
fn payload_history_keeps_each_shape_and_avoids_real_variant_name_collisions() {
    let mut registry = UnifiedRegistry::default();
    for (major, payload, array) in [
        (1, "int", false),
        (2, "str", false),
        (3, "int", true),
        (4, "int64", true),
    ] {
        let mut input =
            enumeration("value", EnumVariantKind::Tuple(payload.into()), array);
        input.variants.push(EnumVariant {
            name: "value-legacy".into(),
            kind: EnumVariantKind::Unit,
            array: false,
            meta: Metadata {
                attributes: vec![Attribute::with_value("name", "value-legacy")],
                ..Default::default()
            },
        });
        merge_enum(&mut registry, "Example", &input, (major, 0, 0));
    }
    materialize_variant_history(&mut registry, "QMP").unwrap();
    materialize_field_history(&mut registry, "QMP").unwrap();
    let UnifiedType::Enum(value) = &registry.types["Example"] else { panic!() };
    assert_eq!(
        value.variant_history.len(),
        2,
        "Rust-equivalent int/int64 must not create another representation"
    );
    assert_eq!(value.ir.variants.len(), 4);
    assert_eq!(value.variant_versions["value"].since, (3, 0, 0));
    assert_eq!(
        value.variant_versions["ValueLegacyV1p0"].until,
        Some((2, 0, 0))
    );
    assert_eq!(
        value.variant_versions["ValueLegacyV2p0"].until,
        Some((3, 0, 0))
    );
    assert!(
        value
            .ir
            .variants
            .iter()
            .filter(|variant| variant.name.starts_with("ValueLegacy"))
            .all(|variant| variant
                .meta
                .attributes
                .contains(&Attribute::with_value("name", "value")))
    );
}

#[test]
fn wire_renames_with_the_same_rust_name_keep_both_spellings() {
    let mut registry = UnifiedRegistry::default();
    merge_enum(
        &mut registry,
        "Example",
        &enumeration("foo-bar", EnumVariantKind::Unit, false),
        (1, 0, 0),
    );
    merge_enum(
        &mut registry,
        "Example",
        &enumeration("foo_bar", EnumVariantKind::Unit, false),
        (2, 0, 0),
    );
    materialize_variant_history(&mut registry, "QMP").unwrap();
    materialize_field_history(&mut registry, "QMP").unwrap();
    let UnifiedType::Enum(value) = &registry.types["Example"] else { panic!() };
    assert_eq!(value.variant_versions["foo_bar"].since, (2, 0, 0));
    assert_eq!(value.variant_versions["FooBarLegacy"].until, Some((2, 0, 0)));
    let old = value
        .ir
        .variants
        .iter()
        .find(|variant| variant.name == "FooBarLegacy")
        .unwrap();
    assert!(
        old.meta.attributes.contains(&Attribute::with_value("name", "foo-bar"))
    );
}
