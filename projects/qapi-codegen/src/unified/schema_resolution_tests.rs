use super::lower::process_version_schema;
use super::merge::has_feature;
use super::*;

/// Read a fixture under the 11.1.3 grammar or that of an older `version`,
/// restoring the caller's grammar.
fn read_fixture(
    path: &std::path::Path,
    version: Version,
) -> Result<Vec<(std::path::PathBuf, String)>, crate::ReadError> {
    crate::read_schema_with_version(path, Some(version))
}

#[test]
fn boxed_union_arguments_lower() {
    let definitions = concat!(
        "{'enum':'Kind','data':['one']}\n",
        "{'struct':'Payload','data':{'value':'str'}}\n",
        "{'union':'Choice','base':{'kind':'Kind'},'discriminator':'kind','data':{'one':'Payload'}}\n",
    );
    process(&format!("{definitions} {{'command':'good','boxed':true,'data':'Choice'}} {{'event':'GOOD','boxed':true,'data':'Choice'}} {{'command':'plain','data':'Payload'}}")).unwrap();
    let error = process(&format!("{definitions} {{'event':'BAD','data':'Choice'}}")).unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::UnboxedUnion {
            name: "Choice".into()
        }
    );
}

#[test]
fn boxed_inherited_conditions_and_older_unboxed_forms_remain_valid() {
    let definitions = concat!(
        "{'struct':'Base','data':{'optional-build':{'type':'str','if':'CONFIG_TEST'}}}\n",
        "{'struct':'Args','base':'Base','data':{'value':'str'}}\n",
    );
    process(&format!("{definitions} {{'command':'good','boxed':true,'data':'Args'}} {{'event':'GOOD','boxed':true,'data':'Args'}}")).unwrap();
    let bad = format!("{definitions} {{'command':'bad','data':'Args'}}");
    assert_eq!(
        process(&bad).unwrap_err().kind(),
        &SchemaErrorKind::ConditionalArguments
    );
    crate::set_parser_version((8, 0, 0));
    process_version_schema(&[("old.json".into(), bad)]).unwrap();
}

#[test]
fn command_flags_reject_invalid_values_and_incompatible_execution_modes() {
    for input in [
        "{'command':'bad','allow-oob':false}",
        "{'command':'bad','allow-preconfig':false}",
        "{'command':'bad','coroutine':false}",
        "{'command':'bad','gen':true}",
        "{'command':'bad','success-response':true}",
        "{'command':'bad','allow-oob':true,'coroutine':true}",
    ] {
        let error = format!("{:#}", process(input).unwrap_err());
        assert!(error.contains("fixture.json"), "{error}");
        assert!(
            error.contains("Flag") || error.contains("incompatible"),
            "{error}"
        );
    }
    process(concat!(
        "{'command':'oob','allow-oob':true}\n",
        "{'command':'preconfig','allow-preconfig':true}\n",
        "{'command':'coroutine','coroutine':true}\n",
        "{'command':'manual','gen':false,'success-response':false}",
    ))
    .unwrap();
    crate::set_parser_version((1, 0, 0));
    process_version_schema(&[("old.json".into(), "{'command':'manual','gen':'no'}".into())])
        .unwrap();
}

#[test]
fn boxed_commands_and_events_keep_named_object_fields_flat() {
    let (_, structs) = process(concat!(
        "{'struct':'Payload','data':{'value':'str'}}\n",
        "{'command':'boxed-command','boxed':true,'data':'Payload'}\n",
        "{'event':'BOXED_EVENT','boxed':true,'data':'Payload'}\n",
        "{'command':'boxed-empty','boxed':true,'data':'q_empty'}",
    ))
    .unwrap();
    assert_eq!(structs["BoxedCommand"].fields[0].name, "value");
    assert_eq!(structs["BoxedEventEvent"].fields[0].name, "value");
    assert!(structs["BoxedEmpty"].fields.is_empty());
}

#[test]
fn alternate_scalar_conflicts_follow_the_historical_qobject_model() {
    for input in [
        "{'alternate':'Choice','data':{'integer':'int','floating':'number'}}",
        "{'alternate':'Choice','data':{'text':'str','boolean':'bool'}}",
    ] {
        crate::set_parser_version((2, 9, 0));
        process_version_schema(&[("old.json".into(), input.into())]).unwrap();
        crate::set_parser_version((2, 10, 0));
        let error = process_version_schema(&[("new.json".into(), input.into())]).unwrap_err();
        assert!(format!("{error:#}").contains("cannot be distinguished"));
    }
    process(concat!(
        "{'enum':'Kind','data':['yes','no']}\n",
        "{'struct':'Payload','data':{'value':'str'}}\n",
        "{'alternate':'Choice','data':{'kind':'Kind','number':'int','object':'Payload','empty':'null'}}",
    )).unwrap();
}

#[test]
fn implicit_arrays_require_creation_and_reject_nested_arrays() {
    for (input, expected) in [
        (
            "{'struct':'Holder','data':{'values':'QTypeList'}}",
            SchemaErrorKind::UnknownType {
                name: "QTypeList".into(),
            },
        ),
        (
            "{'struct':'Item','data':{}} {'struct':'Holder','data':{'values':'ItemList'}}",
            SchemaErrorKind::UnknownType {
                name: "ItemList".into(),
            },
        ),
        (
            "{'struct':'Holder','data':{'values':['intList']}}",
            SchemaErrorKind::NestedArray {
                name: "intList".into(),
            },
        ),
        (
            "{'struct':'Holder','data':{'direct':['intList'],'alias':'intListList'}}",
            SchemaErrorKind::NestedArray {
                name: "intList".into(),
            },
        ),
        (
            "{'struct':'ItemList','data':{}}",
            SchemaErrorKind::ReservedSuffix {
                name: "ItemList".into(),
                suffix: "List",
            },
        ),
    ] {
        let error = process(input).unwrap_err();
        assert_eq!(error.kind(), &expected, "{input}");
        assert_eq!(error.file(), Some(Path::new("fixture.json")));
    }
}

#[test]
fn implicit_arrays_resolve_across_source_files() {
    crate::set_parser_version((11, 1, 3));
    let (_, structs) = process_version_schema(&[
        (
            "first.json".into(),
            "{'struct':'Holder','data':{'values':'ItemList'}}".into(),
        ),
        (
            "second.json".into(),
            "{'struct':'Item','data':{'value':'str'}} {'struct':'Seed','data':{'direct':['Item']}}"
                .into(),
        ),
    ])
    .unwrap();
    let field = &structs["Holder"].fields[0];
    assert!(field.array);
    assert_eq!(field.r#type, "Item");
}

#[test]
fn predefined_types_are_synthesized_only_when_used() {
    let (enums, structs) = process("{'struct':'Plain','data':{}}").unwrap();
    assert!(!enums.contains_key("QType"));
    assert!(!structs.contains_key("QEmpty"));
    crate::set_parser_version((4, 2, 0));
    let (enums, structs) = process_version_schema(&[(
        "builtin.json".into(),
        "{'struct':'Holder','data':{'kind':'QType','empty':'q_empty'}}".into(),
    )])
    .unwrap();
    assert!(
        enums["QType"]
            .variants
            .iter()
            .any(|variant| variant.name == "qnum")
    );
    assert!(structs.contains_key("QEmpty"));
    crate::set_parser_version((2, 6, 0));
    let (enums, _) = process_version_schema(&[(
        "builtin.json".into(),
        "{'struct':'Holder','data':{'kind':'QType'}}".into(),
    )])
    .unwrap();
    assert!(
        enums["QType"]
            .variants
            .iter()
            .any(|variant| variant.name == "qint")
    );
    crate::set_parser_version((2, 5, 0));
    let error = process_version_schema(&[(
        "old.json".into(),
        "{'struct':'Holder','data':{'kind':'QType'}}".into(),
    )])
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::UnknownType {
            name: "QType".into()
        }
    );
}

#[test]
fn duplicate_definitions_fail_across_kinds_and_source_files() {
    let sources = [
        (
            "first.json".into(),
            "{'struct':'Repeated','data':{}}".into(),
        ),
        (
            "included.json".into(),
            "{'enum':'Repeated','data':['one']}".into(),
        ),
    ];
    let error = format!("{:#}", process_version_schema(&sources).unwrap_err());
    for expected in ["Repeated", "first.json", "included.json"] {
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn primitive_type_names_cannot_be_redefined() {
    for name in [
        "str", "number", "int", "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32",
        "uint64", "size", "bool", "any", "null",
    ] {
        let error = process(&format!("{{'command':'{name}'}}")).unwrap_err();
        assert_eq!(error.file(), Some(Path::new("fixture.json")));
        assert_eq!(
            error.kind(),
            &SchemaErrorKind::Predefined { name: name.into() }
        );
    }
    let (_, structs) = process("{'struct':'Str','data':{'value':'str'}}").unwrap();
    assert!(structs.contains_key("Str"));
    // Before 2.4 QEMU checks no names, but the builtin could not be told
    // apart; a command is no type.
    crate::set_parser_version((1, 0, 0));
    let old = |text: &str| process_version_schema(&[("old.json".into(), text.into())]);
    let error = old("{'type':'str','data':{}}").unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::Unsupported {
            construct: Construct::BuiltinName { name: "str".into() }
        }
    );
    old("{'command':'str'} {'type':'any','data':{}}").unwrap();
    let error = old("{'enum':'dict','data':[]}").unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::Unsupported {
            construct: Construct::BuiltinName {
                name: "dict".into()
            }
        }
    );
}

#[test]
fn documentation_links_follow_the_declared_symbol_and_members() {
    crate::set_parser_version((2, 8, 0));
    process_version_schema(&[(
        "legacy.json".into(),
        "##\n# @Other:\n##\n{'struct':'Example','data':{}}".into(),
    )])
    .unwrap();
    crate::set_parser_version((2, 9, 0));
    let (_, structs) = process_version_schema(&[("simple.json".into(), "{'struct':'Leaf','data':{}}\n##\n# @Simple:\n# @type: Select branch.\n##\n{'union':'Simple','data':{'leaf':'Leaf'}}".into())]).unwrap();
    let field = structs["Simple"].fields.iter().find(|field| field.meta.attributes.iter().any(|attr| matches!(attr, Attribute::Item {name,value} if name == "name" && value == "type"))).unwrap();
    assert_eq!(field.meta.schema_doc.as_deref(), Some("Select branch."));
    assert_eq!(structs["Simple"].meta.schema_item, Some(SchemaItem::Union));
}

#[test]
fn inheritance_requires_plain_struct_bases() {
    for (base, meta) in [
        ("str", "built-in"),
        ("Kind", "enum"),
        ("Choice", "alternate"),
    ] {
        let definitions = concat!(
            "{'enum':'Kind','data':['one']}\n",
            "{'alternate':'Choice','data':{'flag':'bool','integer':'int'}}\n",
        );
        for definition in [
            format!("{{'struct':'Derived','base':'{base}','data':{{}}}}"),
            format!("{{'union':'Derived','base':'{base}','discriminator':'kind','data':{{}}}}"),
        ] {
            let error = process(&format!("{definitions}{definition}")).unwrap_err();
            assert_eq!(error.file(), Some(Path::new("fixture.json")));
            assert_eq!(error.context(), ["'base'"]);
            assert_eq!(
                error.kind(),
                &SchemaErrorKind::TypeNotAllowed {
                    meta,
                    reference: format!("'{base}'")
                }
            );
        }
    }
    let (_, structs) = process(concat!(
        "{'struct':'Derived','base':'Base','data':{'extra':'bool'}}\n",
        "{'struct':'Base','data':{'value':'str'}}",
    ))
    .unwrap();
    assert_eq!(structs["Derived"].fields.len(), 2);
}

#[test]
fn invalid_nested_union_layouts_report_source_context() {
    let error = process(concat!(
        "{'enum':'Kind','data':['one']}\n",
        "{'struct':'Base','data':{'kind':{'type':'Kind','if':'CONFIG_A'}}}\n",
        "{'struct':'Derived','base':'Base','data':{}}\n",
        "{'struct':'Payload','data':{}}\n",
        "{'union':'Choice','base':'Derived','discriminator':'kind','data':{'one':'Payload'}}",
    ))
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::ConditionalDiscriminator {
            discriminator: "kind".into()
        }
    );
    assert_eq!(error.line(), Some(5));
    let error = process(concat!(
        "{'enum':'Kind','data':['one']}\n",
        "{'union':'Loop','base':{'kind':'Kind'},'discriminator':'kind','data':{'one':'Loop'}}",
    ))
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::InheritanceCycle {
            name: "Loop".into()
        }
    );
    // A union may be the branch type of two branches.
    process(concat!(
        "{'enum':'Kind','data':['one','two']} {'enum':'Inner','data':['a']}\n",
        "{'struct':'Payload','data':{'p':'int'}}\n",
        "{'union':'Nested','base':{'inner':'Inner'},'discriminator':'inner','data':{'a':'Payload'}}\n",
        "{'union':'Outer','base':{'kind':'Kind'},'discriminator':'kind','data':{'one':'Nested','two':'Nested'}}",
    ))
    .unwrap();
}

#[test]
fn separate_union_branches_can_reuse_member_names() {
    let (_, structs) = process(concat!(
        "{'enum':'Kind','data':['left','right']}\n",
        "{'struct':'Base','data':{'kind':'Kind'}}\n",
        "{'struct':'Left','data':{'value':'str'}}\n",
        "{'struct':'Right','data':{'value':'int'}}\n",
        "{'union':'Choice','base':'Base','discriminator':'kind','data':{'left':'Left','right':'Right'}}",
    )).unwrap();
    assert!(structs.contains_key("Choice"));
}

#[test]
#[ignore = "audits QEMU's negative fixtures under the grammar of every pinned release series"]
fn upstream_negative_fixtures_never_panic_during_lowering_or_emission() {
    /// What generating a fixture under the grammar of `version` gives: the
    /// kind of its error, as the fixture table of the validation tests
    /// writes it, or `None` when it lowers, merges and emits in both modes.
    /// A fixture that cannot be read at all panics.
    fn outcome(path: &Path, version: Version) -> Result<Option<String>, GenerateError> {
        let kind = crate::schema_validation::tests::kind_name;
        let sources = match read_fixture(path, version) {
            Ok(sources) => sources,
            Err(crate::ReadError::Parse { source, .. }) => {
                return Ok(Some(crate::schema_validation::tests::parse_kind_name(
                    &source.kind,
                )));
            }
            Err(crate::ReadError::Include { kind: fault, .. }) => {
                return Ok(Some(format!("Include:{}", kind(format!("{fault:?}")))));
            }
            Err(error) => panic!("{} cannot be read: {error}", path.display()),
        };
        crate::with_parser_version(version, || {
            let (enums, structs) = match process_version_schema(&sources) {
                Ok(lowered) => lowered,
                Err(error) => return Ok(Some(kind(format!("{:?}", error.kind())))),
            };
            let release = super::lower::VersionResult {
                tag: format!("v{}", version_str(version)),
                version,
                order: (version.0, version.1, version.2, 0),
                schema_hash: 0,
                enums,
                structs,
                warnings: Vec::new(),
            };
            let registry = super::registry::merge_releases("QMP", vec![release])?;
            for docs in [Docs::Reference, Docs::Schema] {
                super::emit::emit_registry(&registry, Protocol::Qmp, docs)?;
            }
            Ok(None)
        })
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/qemu/tests/qapi-schema");
    let pins = crate::versions::parse_pinned_versions(crate::versions::PINNED_VERSIONS).unwrap();
    // The first release of each series: the grammar and the rules change
    // with the series.
    let mut versions: Vec<_> = pins
        .iter()
        .filter_map(|pin| crate::versions::parse_version_tag(&pin.tag))
        .collect();
    let newest = *versions.last().expect("the lockfile pins a release");
    versions.dedup_by_key(|version| (version.0, version.1));
    if versions.last() != Some(&newest) {
        versions.push(newest);
    }
    let mut accepted = Vec::new();
    let mut failed = Vec::new();
    for (name, expected) in crate::schema_validation::tests::FIXTURES {
        let path = root.join(format!("{name}.json"));
        let mut first_accepted = None;
        for &version in &versions {
            match std::panic::catch_unwind(|| outcome(&path, version)) {
                // At the pinned release, the error that the validation
                // table gives. Older releases have no such table, and their
                // rules are tested where the validation implements them, so
                // there any error of the schema will do, and a schema that
                // they accept must emit.
                Ok(Ok(kind)) if version == newest => {
                    if kind.as_deref() != Some(*expected) {
                        failed.push(format!(
                            "{name}: {kind:?} at {newest:?}, expected {expected}"
                        ));
                    }
                }
                Ok(Ok(None)) => {
                    first_accepted.get_or_insert(version);
                }
                Ok(Ok(Some(_))) => {}
                Ok(Err(error)) => failed.push(format!("{name} at {version:?}: {error}")),
                Err(_) => failed.push(format!("{name} panics at {version:?}")),
            }
        }
        accepted.extend(first_accepted.map(|version| format!("{name} (from {version:?})")));
    }
    eprintln!("Negative fixtures that older releases accept: {accepted:#?}");
    assert!(failed.is_empty(), "Failed on:\n{}", failed.join("\n"));
    assert!(
        !accepted.is_empty(),
        "no older release accepts a negative fixture"
    );
}

fn process(input: &str) -> SchemaResult<(HashMap<String, Enum>, HashMap<String, Struct>)> {
    crate::set_parser_version((11, 1, 3));
    process_version_schema(&[("fixture.json".into(), input.into())])
}

#[test]
fn undefined_type_references_report_the_source_and_definition() {
    for (input, definition, member) in [
        (
            "{'struct':'Record','data':{'value':'Missing'}}",
            "Record",
            "value",
        ),
        // QEMU names no member for the element type of an array.
        (
            "{'struct':'Record','data':{'values':['Missing']}}",
            "Record",
            "Record",
        ),
        (
            "{'command':'query-record','returns':'Missing'}",
            "query-record",
            "return",
        ),
        (
            "{'event':'RECORD','data':{'value':'Missing'}}",
            "RECORD",
            "value",
        ),
        (
            "{'alternate':'Choice','data':{'value':'Missing'}}",
            "Choice",
            "value",
        ),
        (
            "{'command':'query-record','data':{'value':'Missing'}}",
            "query-record",
            "value",
        ),
    ] {
        let error = format!("{:#}", process(input).unwrap_err());
        for expected in ["fixture.json", definition, member, "Missing"] {
            assert!(error.contains(expected), "{expected}: {error}");
        }
    }
}

#[test]
fn schema_errors_carry_their_kind_and_location() {
    for (input, definition, context, kind) in [
        (
            "{'struct':'Record','data':{'value':'Missing'}}",
            "struct 'Record'",
            &["member 'value'"][..],
            SchemaErrorKind::UnknownType {
                name: "Missing".into(),
            },
        ),
        (
            "{'command':'query-record','returns':'Missing'}",
            "command 'query-record'",
            &["'returns'"][..],
            SchemaErrorKind::UnknownType {
                name: "Missing".into(),
            },
        ),
        (
            "{'struct':'Base','data':{}} {'struct':'Base','data':{}}",
            "struct 'Base'",
            &[][..],
            SchemaErrorKind::Duplicate {
                name: "Base".into(),
                previous: "fixture.json".into(),
            },
        ),
    ] {
        let error = process(input).unwrap_err();
        assert_eq!(error.kind(), &kind, "{input}");
        assert_eq!(error.file(), Some(Path::new("fixture.json")));
        assert_eq!(error.definition(), Some(definition));
        assert_eq!(error.context(), context);
    }
}

#[test]
fn forward_member_references_resolve_across_source_files() {
    crate::set_parser_version((11, 1, 3));
    let (_, structs) = process_version_schema(&[
        (
            "first.json".into(),
            "{'struct':'First','data':{'value':'Second'}}".into(),
        ),
        (
            "second.json".into(),
            "{'struct':'Second','data':{'value':'str'}}".into(),
        ),
    ])
    .unwrap();
    assert_eq!(structs["First"].fields[0].r#type, "Second");
}

#[test]
fn historical_inline_members_lower_and_undefined_types_are_unsupported() {
    // QEMU checks no type references before 2.4, and the IR has no
    // undefined types.
    crate::set_parser_version((1, 0, 0));
    let record = "{'type':'Record','data':{'outer':{'inner':'Missing'}}}";
    let error = process_version_schema(&[("old.json".into(), record.into())]).unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::Unsupported {
            construct: crate::unified::Construct::UndefinedType {
                name: "Missing".into()
            }
        }
    );
    assert_eq!(
        (error.line(), error.definition()),
        (Some(1), Some("struct 'Record'"))
    );
    let defined = format!("{record} {{'type':'Missing','data':{{}}}}");
    let (_, structs) = process_version_schema(&[("old.json".into(), defined)]).unwrap();
    assert!(structs.contains_key("Record"));
}

#[test]
fn inline_event_members_are_refused_from_2_1_to_2_3() {
    let input = "{'event':'BAD','data':{'object':{'value':'str'}}}";
    crate::set_parser_version((2, 3, 0));
    let error = process_version_schema(&[("old.json".into(), input.into())]).unwrap_err();
    assert_eq!(error.kind(), &SchemaErrorKind::NestedEventData);
    assert_eq!(error.context(), ["member 'object'"]);
    assert_eq!(error.file(), Some(Path::new("old.json")));
}

#[test]
fn legacy_alternate_lowers_to_a_bare_value_enum_without_the_ignored_features() {
    crate::set_parser_version((1, 7, 0));
    let input = "{ 'union': 'Ref', 'discriminator': {}, 'data': {'reference':'str'}, 'features':['deprecated'] }";
    let (enums, structs) =
        process_version_schema(&[("fixture.json".into(), input.into())]).unwrap();
    assert!(!structs.contains_key("Ref"));
    let value = &enums["Ref"];
    assert!(!has_feature(&value.meta, "deprecated"));
    assert!(
        !value
            .meta
            .attributes
            .contains(&Attribute::new("union_branch"))
    );
}

#[test]
fn all_implicit_union_branches_are_preserved_as_branch_enums() {
    let (enums, structs) = process(concat!(
        "{ 'enum': 'Kind', 'data': ['first', 'second'] }\n",
        "{ 'union': 'EmptyPayload', 'base': {'kind': 'Kind'}, 'discriminator': 'kind', 'data': {} }",
    )).unwrap();
    assert!(structs.contains_key("EmptyPayload"));
    let branches = &enums["EmptyPayloadBranch"];
    assert_eq!(branches.variants.len(), 2);
    assert!(
        branches
            .variants
            .iter()
            .all(|branch| branch.kind == EnumVariantKind::Unit)
    );
    assert!(
        branches
            .meta
            .attributes
            .iter()
            .any(|attr| matches!(attr, Attribute::Unit(name) if name == "union_branch"))
    );
    let code = branches.generate().to_string();
    assert!(code.contains("union_branch"));
}

#[test]
fn conflicting_schema_definitions_do_not_silently_overwrite_types() {
    let error = process("{ 'struct': 'Example', 'data': {} }\n{ 'struct': 'Example', 'data': {} }")
        .unwrap_err();
    assert!(
        matches!(error.kind(), SchemaErrorKind::Duplicate { name, .. } if name == "Example"),
        "{error}"
    );
}

#[test]
fn distinct_raw_type_names_keep_separate_rust_identities() {
    let (enums, structs) =
        process("{ 'struct':'FooBAR', 'data':{} } { 'enum':'FooBar', 'data':['one'] }").unwrap();
    assert!(enums.contains_key("FooBar"));
    assert!(structs.contains_key("FooBarGenerated"));
}

#[test]
fn inherited_structs_resolve_forward_base_chains() {
    let (_, structs) = process(concat!(
        "{ 'struct': 'Leaf', 'base': 'Middle', 'data': { 'leaf': 'int' } }\n",
        "{ 'struct': 'Middle', 'base': 'Root', 'data': { 'middle': 'int' } }\n",
        "{ 'struct': 'Root', 'data': { 'root': 'int' } }\n",
    ))
    .unwrap();
    let fields: Vec<_> = structs["Leaf"]
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect();
    assert_eq!(fields, ["root", "middle", "leaf"]);
}

#[test]
fn unresolved_inheritance_returns_an_error() {
    let error = process("{ 'struct': 'Leaf', 'base': 'Missing', 'data': {} }").unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::UnknownType {
            name: "Missing".into()
        }
    );
    let error = process(
        "{ 'struct': 'First', 'base': 'Second', 'data': {} }\n{ 'struct': 'Second', 'base': 'First', 'data': {} }",
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::InheritanceCycle {
            name: "First".into()
        }
    );
}

#[test]
fn invalid_union_bases_return_an_error() {
    let error = process(
        "{ 'union': 'ExampleUnion', 'base': 'Missing', 'discriminator': 'kind', 'data': { 'one': 'Missing' } }",
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::UnknownType {
            name: "Missing".into()
        }
    );
    let error = process(
        "{ 'struct': 'Base', 'data': {} }\n{ 'union': 'ExampleUnion', 'base': 'Base', 'discriminator': 'kind', 'data': { 'one': 'Base' } }",
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &SchemaErrorKind::DiscriminatorNotMember {
            discriminator: "kind".into()
        }
    );
}
