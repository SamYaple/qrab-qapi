//! Lowering and merging releases given as schema text.
use super::lower::{VersionResult, VersionSources, lower_versions, parse_versions};
use super::merge::{has_feature, merge_struct};
use super::registry::merge_releases;
use super::*;

fn lowered(releases: &[(&str, &str)]) -> Result<Vec<VersionResult>, RegistryError> {
    let tags: Vec<_> = releases.iter().map(|(tag, _)| *tag).collect();
    let sources = releases
        .iter()
        .zip(0..)
        .map(|((tag, text), hash)| {
            let order = parse_version_tag_full(tag).unwrap();
            VersionSources {
                tag: tag.to_string(),
                version: (order.0, order.1, order.2),
                order,
                schema_hash: hash,
                sources: vec![("schema.json".into(), text.to_string())],
            }
        })
        .collect();
    lower_versions(&tags, sources)
}

fn registry(releases: &[(&str, &str)]) -> Result<UnifiedRegistry, RegistryError> {
    merge_releases("QMP", lowered(releases)?)
}

fn schema_error(tag: &str, text: &str) -> SchemaError {
    match registry(&[(tag, text)]).unwrap_err() {
        RegistryError::Schema {
            tag: failed,
            source,
        } => {
            assert_eq!(failed, tag);
            source
        }
        error => panic!("{error:?}"),
    }
}

fn field_names(value: &Struct) -> Vec<&str> {
    value
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect()
}

#[test]
fn workers_are_all_joined_and_the_first_failure_in_tag_order_is_returned() {
    let tag = |tag: &str| RegistryError::Tag { tag: tag.into() };
    let result = parse_versions(&["a", "b", "c"], |name| match name {
        "b" => Err(tag(name)),
        "c" => panic!("worker for c"),
        _ => Ok(()),
    });
    assert!(
        matches!(&result, Err(RegistryError::Tag { tag }) if tag == "b"),
        "{result:?}"
    );
    let result = parse_versions(&["a", "b", "c"], |name| match name {
        "b" => panic!("worker for b"),
        "c" => Err(tag(name)),
        _ => Ok(()),
    });
    assert!(
        matches!(&result, Err(RegistryError::WorkerPanicked { tag }) if tag == "b"),
        "{result:?}"
    );
    assert_eq!(
        parse_versions(&["a", "bb"], |name| Ok(name.len())).unwrap(),
        [1, 2]
    );
}

#[test]
fn a_used_predefined_type_spans_every_release_of_its_era() {
    let registry = registry(&[
        ("v2.6.0", "{'struct':'Holder','data':{'kind':'QType'}}"),
        ("v2.7.0", "{'struct':'Holder','data':{}}"),
        ("v2.10.0", "{'struct':'Holder','data':{}}"),
    ])
    .unwrap();
    let UnifiedType::Enum(qtype) = &registry.types["QType"] else {
        panic!("QType is an enum")
    };
    assert_eq!(
        (qtype.versions.since, qtype.versions.until),
        ((2, 6, 0), None)
    );
    assert_eq!(qtype.variant_versions["qint"].until, Some((2, 10, 0)));
    assert_eq!(qtype.variant_versions["qnum"].since, (2, 10, 0));
    assert!(!registry.types.contains_key("QEmpty"));
    let unused = self::registry(&[("v2.6.0", "{'struct':'Holder','data':{}}")]).unwrap();
    assert!(!unused.types.contains_key("QType"));
}

#[test]
fn colliding_member_names_get_alternatives_with_free_rust_names() {
    let releases = lowered(&[(
        "v2.3.0",
        "{'type':'Record','data':{'type':'str','Type':'int','type_alt':'bool'}}",
    )])
    .unwrap();
    let record = &releases[0].structs["Record"];
    assert_eq!(field_names(record), ["type", "type_alt2", "type_alt"]);
    assert_eq!(
        record.fields[1].meta.attributes,
        [Attribute::with_value("name", "Type")]
    );
}

#[test]
fn declared_members_keep_their_names_before_virtual_ones() {
    let base = |members: &str| {
        format!(
            "{{'enum':'Choice','data':['one']}} {{'struct':'Payload','data':{{}}}}
             {{'struct':'Base','data':{{'kind':'Choice'{members}}}}}
             {{'union':'Owner','base':'Base','discriminator':'kind','data':{{'one':'Payload'}}}}"
        )
    };
    let releases = lowered(&[("v2.3.0", &base(",'U':'str'")), ("v2.4.0", &base(""))]).unwrap();
    for release in &releases {
        let owner = &release.structs["Owner"];
        let virtual_field = owner
            .fields
            .iter()
            .find(|field| field.meta.attributes.contains(&Attribute::new("flatten")))
            .unwrap();
        assert_eq!(virtual_field.name, "u_alt", "{}", release.tag);
    }
    assert!(field_names(&releases[0].structs["Owner"]).contains(&"U"));
}

#[test]
fn merged_fields_lose_the_blank_lines_around_their_documentation() {
    let field = |name: &str, doc: &str| StructField {
        name: name.into(),
        r#type: "str".into(),
        optional: false,
        array: false,
        meta: Metadata {
            schema_doc: Some(doc.into()),
            ..Default::default()
        },
    };
    let record = |fields| Struct {
        name: "Record".into(),
        fields,
        meta: Metadata::default(),
    };
    let mut registry = UnifiedRegistry::default();
    merge_struct(
        &mut registry,
        "Record",
        &record(vec![field("a-b", "Old")]),
        (1, 0, 0),
    );
    merge_struct(
        &mut registry,
        "Record",
        &record(vec![
            field("a_b", "\nRenamed\n"),
            field("added", "\n\nAdded\n"),
        ]),
        (2, 0, 0),
    );
    let UnifiedType::Struct(value) = &registry.types["Record"] else {
        panic!("struct")
    };
    let docs: Vec<_> = value
        .ir
        .fields
        .iter()
        .map(|field| field.meta.schema_doc.as_deref())
        .collect();
    assert_eq!(docs, [Some("Renamed"), Some("Added")]);
}

#[test]
fn a_dropped_deprecation_ends_its_range() {
    let deprecated = Attribute::List(vec![Attribute::with_value("feature", "deprecated")]);
    let mut registry = UnifiedRegistry::default();
    for (major, present) in [(1, true), (2, false), (3, true)] {
        let mut value = Struct {
            name: "Record".into(),
            fields: vec![StructField {
                name: "value".into(),
                r#type: "str".into(),
                optional: false,
                array: false,
                meta: Metadata::default(),
            }],
            meta: Metadata::default(),
        };
        if present {
            value.meta.attributes.push(deprecated.clone());
            value.fields[0].meta.attributes.push(deprecated.clone());
        }
        merge_struct(&mut registry, "Record", &value, (major, 0, 0));
        let UnifiedType::Struct(merged) = &registry.types["Record"] else {
            panic!("struct")
        };
        assert_eq!(has_feature(&merged.ir.meta, "deprecated"), present);
        assert_eq!(
            has_feature(&merged.ir.fields[0].meta, "deprecated"),
            present
        );
    }
    let UnifiedType::Struct(merged) = &registry.types["Record"] else {
        panic!("struct")
    };
    for range in [&merged.versions, &merged.field_versions["value"]] {
        assert_eq!(range.deprecated, Some((1, 0, 0)));
        assert_eq!(
            range.deprecated_ranges,
            [((1, 0, 0), Some((2, 0, 0))), ((3, 0, 0), None)]
        );
        assert!(range.is_deprecated((1, 0, 0)));
        assert!(!range.is_deprecated((2, 0, 0)));
        assert!(range.is_deprecated((3, 0, 0)));
    }
}

#[test]
fn declarations_that_share_a_c_symbol_are_refused() {
    for (tag, text, symbol) in [
        (
            "v11.1.3",
            "{'command':'foo-bar'} {'command':'foo_bar'}",
            "foo_bar",
        ),
        (
            "v2.3.0",
            "{'union':'Choice','data':{'one':'str'}} {'enum':'ChoiceKind','data':['one']}",
            "ChoiceKind",
        ),
    ] {
        let error = schema_error(tag, text);
        assert!(
            matches!(error.kind(), SchemaErrorKind::SymbolClash { symbol: s, .. } if s == symbol),
            "{error}"
        );
        assert_eq!(error.file(), Some(Path::new("schema.json")));
    }
}

#[test]
fn a_base_cycle_is_refused_where_it_starts() {
    let error = schema_error(
        "v11.1.3",
        "{'struct':'First','base':'Second','data':{}} {'struct':'Second','base':'First','data':{}}",
    );
    assert_eq!(error.file(), Some(Path::new("schema.json")));
    assert_eq!(error.definition(), Some("struct 'First'"));
    assert!(error.to_string().contains("cycle"), "{error}");
}

#[test]
fn discriminators_must_be_required_scalar_enums_with_known_branches() {
    let union = |base: &str, branches: &str| {
        format!(
            "{{'enum':'Choice','data':['known']}} {{'enum':'Empty','data':[]}}
             {{'struct':'Payload','data':{{}}}}
             {{'union':'Invalid','base':{base},'discriminator':'kind','data':{branches}}}"
        )
    };
    for (tag, base, branches, kind) in [
        (
            "v11.1.3",
            "{'*kind':'Choice'}",
            "{}",
            SchemaErrorKind::DiscriminatorShape {
                discriminator: "kind".into(),
            },
        ),
        (
            "v11.1.3",
            "{'kind':'Empty'}",
            "{}",
            SchemaErrorKind::NoBranches,
        ),
        (
            "v11.1.3",
            "{'kind':'Choice'}",
            "{'unknown':'Payload'}",
            SchemaErrorKind::UnknownBranch {
                branch: "unknown".into(),
                discriminator_type: "Choice".into(),
            },
        ),
        (
            "v6.2.0",
            "{'kind':'str'}",
            "{}",
            SchemaErrorKind::DiscriminatorNotEnum {
                discriminator: "kind".into(),
                discriminator_type: "str".into(),
            },
        ),
    ] {
        let error = schema_error(tag, &union(base, branches));
        assert_eq!(error.kind(), &kind, "{error}");
        assert_eq!(error.definition(), Some("union 'Invalid'"));
        assert_eq!(error.file(), Some(Path::new("schema.json")));
    }
    // Before 6.2, a discriminator of another type is a tolerated quirk.
    let registry = registry(&[("v6.1.0", &union("{'kind':'str'}", "{}"))]).unwrap();
    assert!(
        registry.warnings.iter().any(|warning| matches!(
            &warning.kind,
            WarningKind::NonEnumDiscriminator { union, .. } if union == "Invalid"
        )),
        "{:?}",
        registry.warnings
    );
}

#[test]
fn a_command_named_like_its_response_gets_a_command_type() {
    let registry = registry(&[(
        "v11.1.3",
        "{'struct':'GuestExec','data':{'pid':'int'}}
         {'command':'guest-exec','data':{'path':'str'},'returns':'GuestExec'}",
    )])
    .unwrap();
    let UnifiedType::Struct(command) = &registry.types["GuestExecCommand"] else {
        panic!("the command is a struct")
    };
    assert!(
        command
            .ir
            .meta
            .attributes
            .contains(&Attribute::new("command"))
    );
    assert!(
        command
            .ir
            .meta
            .attributes
            .contains(&Attribute::with_value("name", "guest-exec"))
    );
    assert_eq!(command.returns(), Some("GuestExec"));
    let UnifiedType::Struct(response) = &registry.types["GuestExec"] else {
        panic!("the response is a struct")
    };
    assert_eq!(field_names(&response.ir), ["pid"]);
}

#[test]
fn unparsable_and_unlowerable_schemas_are_refused() {
    let error = schema_error("v11.1.3", "{ 'struct': 'Broken'");
    assert!(matches!(error.kind(), SchemaErrorKind::Parse(_)), "{error}");
    assert_eq!(error.file(), Some(Path::new("schema.json")));
    let error = schema_error(
        "v11.1.3",
        "{'struct':'Base','data':{}} {'struct':'Payload','data':{}}
         {'union':'Invalid','base':'Base','discriminator':'kind','data':{'one':'Payload'}}",
    );
    assert!(
        matches!(
            error.kind(),
            SchemaErrorKind::Lower(crate::qapi_ir::LowerError::MissingDiscriminator { .. })
        ),
        "{error}"
    );
    assert_eq!(error.definition(), Some("union 'Invalid'"));
}
