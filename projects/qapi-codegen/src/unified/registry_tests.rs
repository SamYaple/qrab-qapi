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
    // Before 2.4 a struct is a 'type'.
    let base = |members: &str| match members {
        "" => base(members),
        _ => base(members).replace("'struct'", "'type'"),
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
fn two_types_of_one_name_before_2_4_are_unsupported() {
    for text in [
        "{'type':'Choice','data':{}} {'enum':'Choice','data':[]}",
        "{'union':'Choice','data':{'one':'str'}} {'enum':'ChoiceKind','data':['one']}",
    ] {
        let error = schema_error("v2.3.0", text);
        assert!(
            matches!(
                error.kind(),
                SchemaErrorKind::Unsupported {
                    construct: Construct::SharedTypeName { symbol, .. }
                } if symbol.starts_with("Choice")
            ),
            "{error}"
        );
        assert_eq!(error.file(), Some(Path::new("schema.json")));
    }
    // Two commands or events of one name, which the IR cannot hold apart.
    for text in [
        "{'command':'foo'} {'command':'foo','data':{'a':'int'}}",
        "{'event':'FOO'} {'event':'FOO'}",
    ] {
        let error = schema_error("v2.3.0", text);
        assert!(
            matches!(
                error.kind(),
                SchemaErrorKind::Unsupported {
                    construct: Construct::SharedName { name }
                } if name.eq_ignore_ascii_case("foo")
            ),
            "{error}"
        );
        assert!(
            matches!(
                schema_error("v2.4.0", text).kind(),
                SchemaErrorKind::Duplicate { .. }
            ),
            "{text}"
        );
    }
    // From 2.4 QEMU refuses a second type of one name.
    let error = schema_error(
        "v2.4.0",
        "{'struct':'Choice','data':{}} {'enum':'Choice','data':[]}",
    );
    assert!(
        matches!(error.kind(), SchemaErrorKind::Duplicate { name, .. } if name == "Choice"),
        "{error}"
    );
}

#[test]
fn pre_2_4_expressions_are_read_as_their_generators_read_them() {
    // A flat union, which 1.6 reads and 1.5 reads as a simple union.
    let flat = "{'enum':'Kind','data':['a']} {'type':'Payload','data':{'kind':'Kind'}}
         {'type':'A','data':{'x':'int'}}
         {'union':'U','base':'Payload','discriminator':'kind','data':{'a':'A'}}";
    let tag = |tag: &str| {
        registry(&[(tag, flat)])
            .unwrap()
            .types
            .contains_key("UKind")
    };
    assert!(tag("v1.5.0"));
    assert!(!tag("v1.6.0"));
    // The keys that the generators do not read are ignored, whatever
    // they hold.
    let command = registry(&[(
        "v2.3.0",
        "{'command':'c','data':{'a':'int'},'if':'CONFIG_X','bogus':['x',{}]}",
    )])
    .unwrap();
    assert!(command.types.contains_key("C"));
    // Before 2.1 the generators have no events and no includes.
    let skipped = registry(&[(
        "v2.0.0",
        "{'event':'EVT','data':{'a':'int'}} {'include':'missing.json'} {'type':'T','data':{}}",
    )])
    .unwrap();
    assert!(skipped.types.contains_key("T"));
    assert!(!skipped.types.keys().any(|name| name.contains("Evt")));
    // An expression whose values they read loosely is unsupported.
    let error = schema_error(
        "v2.3.0",
        "{'type':'T','data':{}}\n{'enum':'E','data':[{'name':'a','bogus':'x'}]}",
    );
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::Unsupported {
            construct: Construct::Expression
        }
    );
    assert_eq!((error.line(), error.column()), (Some(2), Some(1)));
    // Expressions without a metatype that 2.3 reads are skipped.
    let skipped = registry(&[(
        "v2.3.0",
        "{'pragma':{'doc-required':'yes'}} {'alternate':'A','data':{'b':'bool'}} {'type':'T','data':{}}",
    )])
    .unwrap();
    assert!(skipped.types.contains_key("T"));
    // Before 2.4 `true` is a stray character.
    let error = schema_error("v2.3.0", "{'command':'c','gen':true}");
    assert!(matches!(error.kind(), SchemaErrorKind::Parse(_)), "{error}");
    assert_eq!((error.line(), error.column()), (Some(1), Some(22)));
}

#[test]
fn two_members_of_one_name_are_unsupported_where_qemu_accepts_them() {
    for (tag, text) in [
        ("v1.0", "{'type':'T','data':{'a':'int','*a':'str'}}"),
        (
            "v2.3.0",
            "{'type':'T','data':{'b':'int','c':{'a':'int','*a':'str'}}}",
        ),
        ("v2.4.0", "{'command':'c','data':{'a':'int','*a':'str'}}"),
    ] {
        let error = schema_error(tag, text);
        assert_eq!(
            error.kind(),
            &SchemaErrorKind::Unsupported {
                construct: Construct::DuplicateMember { name: "a".into() }
            },
            "{tag} {text}"
        );
        assert_eq!((error.line(), error.column()), (Some(1), Some(1)));
    }
    // 2.5 fails an assertion on them, and from 2.6 they clash.
    let twins = "{'struct':'S','data':{'a':'int','*a':'str'}}";
    assert_eq!(
        schema_error("v2.5.0", twins).kind(),
        &SchemaErrorKind::Crash {
            crash: Crash::OptionalTwin { name: "a".into() }
        }
    );
    assert!(matches!(
        schema_error("v2.6.0", twins).kind(),
        SchemaErrorKind::MemberClash { .. }
    ));
    // Before 2.0 a repeated key is one member, of the last value.
    let lowered = lowered(&[("v1.7.0", "{'type':'T','data':{'a':'int','a':'str'}}")]).unwrap();
    let fields = &lowered[0].structs["T"].fields;
    assert_eq!(
        fields
            .iter()
            .map(|field| (field.name.as_str(), field.r#type.as_str()))
            .collect::<Vec<_>>(),
        [("a", "str")]
    );
}

#[test]
fn a_parse_error_names_its_position_once() {
    let error = schema_error(
        "v11.1.3",
        "{'enum':'E','data':[]}\n{'enum':'F','data':'x' 'y'}",
    );
    let message = error.to_string();
    assert!(
        message.contains("schema.json:2:24: QAPI parse error (Char)"),
        "{message}"
    );
    assert!(!message.contains("line 2"), "{message}");
}

#[test]
fn schemas_on_which_qemu_fails_are_refused_in_those_releases() {
    // 2.5 checks an alternate's branches against its tag member, `type`.
    let alternate = "{'alternate':'Aa','data':{'type':'int','s':'bool'}}";
    assert_eq!(
        schema_error("v2.5.0", alternate).kind(),
        &SchemaErrorKind::Crash {
            crash: Crash::AlternateTypeBranch
        }
    );
    for tag in ["v2.4.0", "v2.6.0"] {
        registry(&[(tag, alternate)]).unwrap();
    }
    // A cycle of bases: accepted before 2.4, a recursion without end in
    // 2.4 and 2.5, refused from 2.6.
    let cycle = "{'struct':'A','base':'B','data':{}} {'struct':'B','base':'A','data':{}}";
    let legacy = cycle.replace("'struct'", "'type'");
    assert!(matches!(
        schema_error("v2.3.0", &legacy).kind(),
        SchemaErrorKind::Unsupported {
            construct: Construct::BaseCycle { .. }
        }
    ));
    for tag in ["v2.4.0", "v2.5.0"] {
        assert!(
            matches!(
                schema_error(tag, cycle).kind(),
                SchemaErrorKind::Crash {
                    crash: Crash::BaseCycle { .. }
                }
            ),
            "{tag}"
        );
    }
    assert!(matches!(
        schema_error("v2.6.0", cycle).kind(),
        SchemaErrorKind::InheritanceCycle { .. }
    ));
}

#[test]
fn constructs_beyond_the_ir_are_unsupported_only_where_qemu_accepts_them() {
    let construct = |tag: &str, text: &str| match schema_error(tag, text).kind() {
        SchemaErrorKind::Unsupported { construct } => Some(construct.clone()),
        _ => None,
    };
    let boxed = "{'struct':'Bar','data':{'x':'int'}} {'alternate':'Alt','data':{'n':'int','b':'Bar'}} {'command':'c','data':'Alt','boxed':true}";
    for tag in ["v2.7.0", "v4.1.0"] {
        assert_eq!(
            construct(tag, boxed),
            Some(Construct::BoxedAlternate { name: "Alt".into() }),
            "{tag}"
        );
    }
    assert_eq!(construct("v4.2.0", boxed), None);
    assert_eq!(construct("v2.6.0", boxed), None);
    // QEMU refuses an enum as boxed arguments.
    let boxed_enum = "{'enum':'E','data':['a']} {'command':'c','data':'E','boxed':true}";
    assert_eq!(construct("v2.7.0", boxed_enum), None);
    // 2.4 takes an anonymous struct as a command's return type, and no
    // other place outside struct members.
    let returns = "{'command':'c','returns':{'a':'int'}}";
    assert_eq!(
        construct("v2.4.0", returns),
        Some(Construct::AnonymousStruct {
            member: "returns".into()
        })
    );
    assert_eq!(construct("v2.5.0", returns), None);
    assert_eq!(
        construct("v2.4.0", "{'command':'c','data':{'a':{'b':'int'}}}"),
        None
    );
    // Events, from 2.1, refuse one in their data.
    let event = "{'event':'E','data':{'a':{'b':'int'}}}";
    assert_eq!(construct("v2.1.0", event), None);
}
#[test]
fn an_undefined_type_is_unsupported_only_before_2_4() {
    let undefined = "{'command':'c','data':'Missing'}";
    let error = schema_error("v2.3.0", undefined);
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::Unsupported {
            construct: Construct::UndefinedType {
                name: "Missing".into()
            }
        }
    );
    let error = schema_error("v2.4.0", undefined);
    assert!(
        !matches!(error.kind(), SchemaErrorKind::Unsupported { .. }),
        "{error}"
    );
}

#[test]
fn commands_and_events_whose_c_names_clash_get_rust_names_of_their_own() {
    let registry = registry(&[(
        "v5.2.0",
        "{'command':'foo-bar','data':{'a':'int'}} {'command':'foo_bar','data':{'b':'str'}}
         {'event':'NOTICE-BAR'} {'event':'NOTICE_BAR','data':{'c':'int'}}",
    )])
    .unwrap();
    let wire = |name: &str| match &registry.types[name] {
        UnifiedType::Struct(value) => value
            .ir
            .meta
            .attributes
            .iter()
            .find_map(|attr| match attr {
                Attribute::Item { name, value } if name == "name" => Some(value.clone()),
                _ => None,
            })
            .unwrap_or_else(|| value.ir.name.clone()),
        UnifiedType::Enum(_) => panic!("{name} is a struct"),
    };
    assert_eq!(wire("FooBar"), "foo-bar");
    assert_eq!(wire("FooBarGenerated"), "foo_bar");
    assert_eq!(wire("NoticeBarEvent"), "NOTICE-BAR");
    assert_eq!(wire("NoticeBarEventGenerated"), "NOTICE_BAR");
}

#[test]
fn unions_and_alternates_without_branches_lower_before_2_6() {
    let registry = registry(&[(
        "v2.5.0",
        "{'union':'Simple','data':{}} {'alternate':'Choice','data':{}}",
    )])
    .unwrap();
    assert!(registry.types.contains_key("Simple"));
    assert!(registry.types.contains_key("Choice"));
    // From 2.6 QEMU refuses them.
    assert_eq!(
        schema_error("v2.6.0", "{'union':'Simple','data':{}}").kind(),
        &SchemaErrorKind::NoBranches
    );
    assert!(matches!(
        schema_error("v2.6.0", "{'alternate':'Choice','data':{}}").kind(),
        SchemaErrorKind::AlternateBranches { .. }
    ));
}

#[test]
fn pre_2_0_keys_are_read_in_the_release_that_reads_them() {
    // A struct's base from 1.7.
    let based = "{'type':'Foo','base':'Missing','data':{}}";
    registry(&[("v1.6.0", based)]).unwrap();
    assert!(matches!(
        schema_error("v1.7.0", based).kind(),
        SchemaErrorKind::Unsupported {
            construct: Construct::UndefinedBases { .. }
        }
    ));
    // An optional discriminator, which QEMU accepts before 2.0.
    let optional = "{'enum':'Kind','data':['a']} {'type':'Base','data':{'*kind':'Kind'}}
         {'type':'A','data':{}} {'union':'U','base':'Base','discriminator':'kind','data':{'a':'A'}}";
    assert_eq!(
        schema_error("v1.7.0", optional).kind(),
        &SchemaErrorKind::Unsupported {
            construct: Construct::DiscriminatorShape {
                discriminator: "kind".into()
            }
        }
    );
}

#[test]
fn a_base_cycle_is_unsupported_before_2_6_where_it_starts() {
    let error = schema_error(
        "v2.3.0",
        "{'type':'Other','base':'Second','data':{}}
         {'type':'First','base':'Second','data':{}} {'type':'Second','base':'First','data':{}}",
    );
    assert_eq!(error.file(), Some(Path::new("schema.json")));
    assert_eq!(error.definition(), Some("struct 'First'"));
    assert_eq!((error.line(), error.column()), (Some(2), Some(10)));
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::Unsupported {
            construct: Construct::BaseCycle {
                name: "First".into()
            }
        }
    );
    let error = schema_error("v2.3.0", "{'type':'Child','base':'Missing','data':{}}");
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::Unsupported {
            construct: Construct::UndefinedBases {
                pending: vec![("Child".into(), "Missing".into())]
            }
        }
    );
}

#[test]
fn discriminators_must_be_required_scalar_enums_with_known_branches() {
    let union = |base: &str, branches: &str| {
        format!(
            "{{'enum':'Choice','data':['known']}} {{'enum':'Empty','data':[]}}
             {{'type':'Payload','data':{{}}}} {{'type':'Base','data':{base}}}
             {{'union':'Invalid','base':'Base','discriminator':'kind','data':{branches}}}"
        )
    };
    for (tag, base, branches, kind) in [
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
            "v2.6.0",
            "{'kind':'Empty'}",
            "{}",
            SchemaErrorKind::NoBranches,
        ),
        (
            "v2.0.0",
            "{'kind':'Choice'}",
            "{'unknown':'Payload'}",
            SchemaErrorKind::UnknownBranch {
                branch: "unknown".into(),
                discriminator_type: "Choice".into(),
            },
        ),
        (
            "v2.0.0",
            "{'kind':'str'}",
            "{}",
            SchemaErrorKind::DiscriminatorNotEnum {
                discriminator: "kind".into(),
                discriminator_type: "str".into(),
            },
        ),
    ] {
        let text = union(base, branches);
        // The struct keyword arrives in 2.4.
        let text = if !matches!(tag, "v1.7.0" | "v2.0.0") {
            text.replace("'type'", "'struct'")
        } else {
            text
        };
        let error = schema_error(tag, &text);
        assert_eq!(error.kind(), &kind, "{error}");
        assert_eq!(error.definition(), Some("union 'Invalid'"));
        assert_eq!(error.file(), Some(Path::new("schema.json")));
        assert_eq!((error.line(), error.column()), (Some(3), Some(14)));
    }
    // QEMU checks no branches before 2.6, nor any branch before 2.0.
    registry(&[(
        "v2.5.0",
        &union("{'kind':'Empty'}", "{}").replace("'type'", "'struct'"),
    )])
    .unwrap();
    registry(&[(
        "v1.7.0",
        &union("{'kind':'Choice'}", "{'unknown':'Payload'}"),
    )])
    .unwrap();
    // Before 2.0, a discriminator of another type is a tolerated quirk.
    let registry = registry(&[("v1.7.0", &union("{'kind':'str'}", "{}"))]).unwrap();
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
    // QEMU 1.7 does not check the discriminator, but the IR needs its type:
    // the IR cannot represent such a union.
    let error = schema_error(
        "v1.7.0",
        "{'type':'Base','data':{}} {'type':'Payload','data':{}}
         {'union':'Invalid','base':'Base','discriminator':'kind','data':{'one':'Payload'}}",
    );
    assert!(
        matches!(
            error.kind(),
            SchemaErrorKind::Unsupported {
                construct: Construct::UndefinedDiscriminator { .. }
            }
        ),
        "{error}"
    );
    assert_eq!(error.definition(), Some("union 'Invalid'"));
}

#[test]
fn an_anonymous_return_type_of_2_4_is_unsupported() {
    let error = schema_error("v2.4.0", "{'command':'c','returns':{'a':'int'}}");
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::Unsupported {
            construct: Construct::AnonymousStruct {
                member: "returns".into()
            }
        }
    );
    assert_eq!((error.line(), error.column()), (Some(1), Some(1)));
}

#[test]
fn renamed_types_take_the_newest_spelling() {
    let registry = registry(&[
        (
            "v10.0.0",
            "{'struct':'SGXInfo','data':{}} {'enum':'FOOKind','data':['a']} {'struct':'SameValue','data':{}} {'struct':'ABCInfo','data':{}}",
        ),
        (
            "v10.1.0",
            "{'struct':'SgxInfo','data':{}} {'enum':'FooKind','data':['a']} {'struct':'SAMEValue','data':{}} {'struct':'AbcINFO','data':{}}",
        ),
    ])
    .unwrap();
    let spelling = |name: &str| {
        let (ir_name, meta) = match &registry.types[name] {
            UnifiedType::Struct(value) => (&value.ir.name, &value.ir.meta),
            UnifiedType::Enum(value) => (&value.ir.name, &value.ir.meta),
        };
        let wire = meta.attributes.iter().find_map(|attr| match attr {
            Attribute::Item { name, value } if name == "name" => Some(value.as_str()),
            _ => None,
        });
        (ir_name.as_str(), wire)
    };
    assert_eq!(spelling("SgxInfo"), ("SgxInfo", None));
    assert_eq!(spelling("FooKind"), ("FooKind", None));
    // A spelling that needs a wire name gets one, or replaces another's.
    assert_eq!(spelling("SameValue"), ("SAMEValue", Some("SAMEValue")));
    assert_eq!(spelling("AbcInfo"), ("AbcINFO", Some("AbcINFO")));
    let UnifiedType::Struct(value) = &registry.types["SgxInfo"] else {
        panic!("struct")
    };
    assert_eq!(value.versions.since, (10, 0, 0));
}

#[test]
fn definitions_are_checked_where_their_files_are_included() {
    let dir = std::env::temp_dir().join(format!("qapi-include-order-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let unknown = |files: &[(&str, &str)]| {
        for (name, text) in files {
            std::fs::write(dir.join(name), text).unwrap();
        }
        let sources =
            crate::read_schema_with_version(&dir.join(files[0].0), Some((11, 1, 3))).unwrap();
        match super::lower::process_version_schema(&sources)
            .unwrap_err()
            .kind()
        {
            SchemaErrorKind::UnknownKey { key } => key.clone(),
            other => panic!("{other:?}"),
        }
    };
    // A definition before an include comes before the included ones.
    assert_eq!(
        unknown(&[
            (
                "main.json",
                "{'struct':'Main','data':{},'first':'x'}\n{'include':'a.json'}"
            ),
            ("a.json", "{'struct':'Aaa','data':{},'second':'x'}"),
        ]),
        "first"
    );
    // A file included by an included file comes in its place.
    assert_eq!(
        unknown(&[
            ("main.json", "{'include':'a.json'}"),
            (
                "a.json",
                "{'include':'b.json'}\n{'struct':'Aaa','data':{},'second':'x'}"
            ),
            ("b.json", "{'struct':'Bbb','data':{},'first':'x'}"),
        ]),
        "first"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_moved_comment_keeps_a_definition_of_the_same_name_documented() {
    // Before 2.4 a type and a command may share a name; the comment that
    // names it before a helper is the undocumented command's.
    let text = "##\n# @Foo:\n# Own doc of the type Foo.\n##\n{ 'type': 'Foo', 'data': {} }\n##\n# @Foo:\n# Doc naming Foo.\n##\n{ 'type': 'Helper', 'data': {} }\n{ 'command': 'Foo', 'data': {} }";
    let lowered = lowered(&[("v2.3.0", text)]).unwrap();
    let docs: Vec<_> = lowered[0]
        .structs
        .values()
        .filter_map(|value| value.meta.schema_doc.as_deref())
        .collect();
    assert!(docs.contains(&"Own doc of the type Foo."), "{docs:?}");
    assert!(docs.contains(&"Doc naming Foo."), "{docs:?}");
}

#[test]
fn early_union_bases_and_discriminators_follow_each_release() {
    // An empty discriminator names none before 2.4: a simple union.
    let empty = "{'type':'S','data':{'x':'int'}} {'union':'U','discriminator':'','data':{'a':'S'}}";
    for tag in ["v1.6.0", "v2.0.0"] {
        registry(&[(tag, empty)]).unwrap();
    }
    // A legacy alternate's base is ignored, and from 2.0 checked.
    let legacy = |base: &str| {
        format!(
            "{{'type':'S','data':{{'x':'int'}}}} {{'union':'U','base':'{base}','discriminator':{{}},'data':{{'a':'int','b':'S'}}}}"
        )
    };
    registry(&[("v1.7.0", &legacy("Nope"))]).unwrap();
    registry(&[("v2.0.0", &legacy("S"))]).unwrap();
    assert_eq!(
        schema_error("v2.0.0", &legacy("Nope")).kind(),
        &SchemaErrorKind::UnknownType {
            name: "Nope".into()
        }
    );
    // A union's base that is a union, which QEMU resolves from 2.0.
    let union_base = "{'type':'S','data':{'x':'int'}} {'union':'V','data':{'a':'S'}} {'union':'U','base':'V','data':{'a':'S'}}";
    assert_eq!(
        schema_error("v1.7.0", union_base).kind(),
        &SchemaErrorKind::Unsupported {
            construct: Construct::UnionBase { base: "V".into() }
        }
    );
    // A simple union's tag enum as a discriminator: QEMU fails on it
    // until 4.1 and accepts it from 4.2.
    let tag = "{'struct':'Str','data':{'x':'int'}} {'union':'Simple','data':{'a':'Str'}}
         {'struct':'Bas','data':{'tag':'SimpleKind'}} {'union':'Outer','base':'Bas','discriminator':'tag','data':{'a':'Str'}}";
    assert!(matches!(
        schema_error("v4.1.0", tag).kind(),
        SchemaErrorKind::Crash {
            crash: Crash::ImplicitTagDiscriminator { .. }
        }
    ));
    assert!(matches!(
        schema_error("v4.2.0", tag).kind(),
        SchemaErrorKind::Unsupported {
            construct: Construct::ImplicitTagDiscriminator { .. }
        }
    ));
}

#[test]
fn the_lowering_meets_none_of_what_the_validation_refuses() {
    // Each would trip a debug assertion of the lowering if the validation
    // let it through.
    let kind = |tag: &str, text: &str| schema_error(tag, text).kind().clone();
    let commands = "{'command':'c'} {'command':'c'}";
    let types = "{'struct':'Ss','data':{}} {'enum':'Ss','data':[]}";
    for tag in ["v2.4.0", "v4.2.0", "v11.1.0"] {
        assert!(
            matches!(kind(tag, commands), SchemaErrorKind::Duplicate { .. }),
            "{tag}"
        );
        assert!(
            matches!(kind(tag, types), SchemaErrorKind::Duplicate { .. }),
            "{tag}"
        );
    }
    let twins = "{'struct':'Ss','data':{'a':'int','*a':'int'}}";
    assert!(matches!(
        kind("v2.6.0", twins),
        SchemaErrorKind::MemberClash { .. }
    ));
    let optional = "{'enum':'Kind','data':['a']} {'type':'Base','data':{'*kind':'Kind','x':'int'}}
         {'type':'A','data':{}} {'union':'U','base':'Base','discriminator':'kind','data':{'a':'A'}}";
    assert!(matches!(
        kind("v2.0.0", optional),
        SchemaErrorKind::DiscriminatorNotMember { .. }
    ));
    let boxed =
        "{'struct':'Bar','data':{'x':'int'}} {'alternate':'Alt','data':{'n':'int','b':'Bar'}}
         {'command':'c','data':'Alt','boxed':true}";
    assert!(matches!(
        kind("v4.2.0", boxed),
        SchemaErrorKind::TypeNotAllowed { .. }
    ));
}

#[test]
fn undefined_types_are_unsupported_where_qemu_accepts_them() {
    let undefined = |tag: &str, text: &str| match schema_error(tag, text).kind() {
        SchemaErrorKind::Unsupported {
            construct: Construct::UndefinedType { name },
        } => Some(name.clone()),
        _ => None,
    };
    // An argument, a return type, a member and a branch: QEMU checks none
    // of them before 2.4.
    for (text, name) in [
        ("{'command':'c','data':{'a':['Nope']}}", "Nope"),
        ("{'command':'c','returns':'Nope'}", "Nope"),
        ("{'type':'Ss','data':{'a':{'b':'Nope'}}}", "Nope"),
        ("{'union':'Uu','data':{'a':'Nope'}}", "Nope"),
    ] {
        for tag in ["v1.0", "v2.3.0"] {
            assert_eq!(undefined(tag, text).as_deref(), Some(name), "{tag} {text}");
        }
        let defined = format!("{text} {{'enum':'Nope','data':[]}}");
        registry(&[("v2.3.0", &defined)]).unwrap();
    }
    // From 2.0 QEMU checks a flat union's base, and from 2.4 every type.
    let flat = "{'enum':'Ee','data':['a']} {'type':'Bb','data':{'k':'Ee'}}
        {'union':'Uu','base':'Bb','discriminator':'k','data':{'a':'Nope'}}";
    assert_eq!(undefined("v1.7.0", flat).as_deref(), Some("Nope"));
    let command = "{'command':'c','returns':'Nope'}";
    assert_eq!(
        schema_error("v2.4.0", command).kind(),
        &SchemaErrorKind::UnknownType {
            name: "Nope".into()
        }
    );
}
