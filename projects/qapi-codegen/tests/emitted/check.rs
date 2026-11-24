//! Check the metadata of the generated modules: what holds for every table,
//! and facts of the pinned releases and the synthetic histories.

/// What holds for the tables of every metadata module.
macro_rules! check_tables {
    ($module:ident) => {{
        use generated_check::$module::metadata::*;
        // Whether every run lies within a lifetime from `since` to `until`,
        // ending with it at the latest.
        let within = |since: Ver, until: Option<Ver>, runs: &[Releases]| {
            runs.iter().all(|run| {
                run.since >= since
                    && run.until.is_none_or(|end| end > run.since)
                    && until.is_none_or(|until| run.until.is_some_and(|end| end <= until))
            })
        };
        // `Table::get` searches by name, so it must find every entry.
        for (name, info) in STRUCTS.iter() {
            assert_eq!(info.name, name);
            assert!(std::ptr::eq(STRUCTS.get(name).unwrap(), info), "{name}");
        }
        for (name, info) in ENUMS.iter() {
            assert_eq!(info.name, name);
            assert!(std::ptr::eq(ENUMS.get(name).unwrap(), info), "{name}");
        }
        assert!(STRUCTS.keys().is_sorted() && ENUMS.keys().is_sorted());
        assert!(STRUCTS.get("").is_none() && !ENUMS.contains_key("~"));
        assert!(COMMANDS.is_sorted_by(|a, b| a.wire_name < b.wire_name));
        for command in COMMANDS {
            assert!(std::ptr::eq(command.args, STRUCTS.get(command.name).unwrap()));
            assert_eq!(command.args.wire_name, command.wire_name);
            assert!(command.returns.is_none_or(|ty| (ty as usize) < TYPE_REFS.len()));
            for runs in [
                command.allow_oob,
                command.allow_preconfig,
                command.success_response,
                command.deprecated,
                command.unstable,
            ] {
                assert!(within(command.since, command.until, runs), "{}: {runs:?}", command.wire_name);
            }
            assert_eq!(command.deprecated, command.args.deprecated);
            assert_eq!(command.unstable, command.args.unstable);
        }
        for (index, ty) in TYPE_REFS.iter().enumerate() {
            assert_eq!(ty.prim.is_some(), ty.kind == TypeKind::Primitive, "{ty:?}");
            assert_eq!(ty.inner.is_some(), ty.kind == TypeKind::List, "{ty:?}");
            match ty.kind {
                TypeKind::Struct => assert!(STRUCTS.contains_key(ty.name), "{ty:?}"),
                TypeKind::Enum => assert!(ENUMS.contains_key(ty.name), "{ty:?}"),
                TypeKind::List => {
                    let element = ty.inner.unwrap() as usize;
                    assert!(element < index);
                    assert_eq!(ty.name, format!("List<{}>", TYPE_REFS[element].name));
                }
                TypeKind::Primitive => {}
            }
        }
        for info in STRUCTS.values() {
            for field in info.fields {
                assert!((field.ty as usize) < TYPE_REFS.len());
            }
            assert_eq!(info.is_union, info.discriminator_field.is_some());
            assert_eq!(info.is_union, info.branch_enum.is_some());
            if let Some(branches) = info.branch_enum {
                assert_eq!(ENUMS.get(branches).unwrap().kind, EnumKind::UnionBranch);
            }
            assert!(info.is_union || info.simple_union.is_empty(), "{}", info.name);
            for runs in [info.simple_union, info.free_form, info.deprecated, info.unstable] {
                assert!(within(info.since, info.until, runs), "{}: {runs:?}", info.name);
            }
            for field in info.fields {
                let lifetime = [Releases { since: field.since, until: field.until }];
                assert!(within(info.since, info.until, &lifetime), "{}.{}", info.name, field.name);
                for runs in [field.deprecated, field.unstable, field.discriminator] {
                    assert!(within(field.since, field.until, runs), "{}.{}", info.name, field.name);
                }
                assert!(info.is_union || field.discriminator.is_empty(), "{}.{}", info.name, field.name);
            }
            // At most one field discriminates in each release.
            let runs: Vec<&Releases> = info.fields.iter().flat_map(|field| field.discriminator).collect();
            for (index, run) in runs.iter().enumerate() {
                for other in &runs[..index] {
                    let before = |a: &Releases, b: &Releases| a.until.is_some_and(|until| until <= b.since);
                    assert!(before(run, other) || before(other, run), "{}: {runs:?}", info.name);
                }
            }
        }
        for info in ENUMS.values() {
            for runs in [info.deprecated, info.unstable] {
                assert!(within(info.since, info.until, runs), "{}: {runs:?}", info.name);
            }
            for variant in info.variants {
                let lifetime = [Releases { since: variant.since, until: variant.until }];
                assert!(within(info.since, info.until, &lifetime), "{}.{}", info.name, variant.name);
                for runs in [variant.deprecated, variant.unstable] {
                    assert!(within(variant.since, variant.until, runs), "{}.{}", info.name, variant.name);
                }
                match variant.payload_type {
                    Some(payload) => {
                        let ty = &TYPE_REFS[payload as usize];
                        assert_eq!(
                            variant.payload_struct,
                            (ty.kind == TypeKind::Struct).then_some(ty.name)
                        );
                    }
                    None => assert!(variant.payload_struct.is_none()),
                }
                if info.kind == EnumKind::Unit {
                    assert!(variant.payload_type.is_none());
                }
            }
        }
    }};
}

/// The entry of `COMMANDS` for a wire name.
macro_rules! command {
    ($module:ident, $wire:literal) => {
        generated_check::$module::metadata::COMMANDS
            .binary_search_by(|command| command.wire_name.cmp($wire))
            .map(|index| &generated_check::$module::metadata::COMMANDS[index])
            .unwrap()
    };
}

/// The field of a struct by its wire name.
macro_rules! field {
    ($module:ident, $struct:literal, $wire:literal) => {
        generated_check::$module::metadata::STRUCTS
            .get($struct)
            .unwrap()
            .fields
            .iter()
            .find(|field| field.wire_name == $wire)
            .unwrap()
    };
}

/// The type that a command returns.
macro_rules! returns {
    ($module:ident, $wire:literal) => {
        &generated_check::$module::metadata::TYPE_REFS[command!($module, $wire).returns.unwrap() as usize]
    };
}

fn pinned() {
    use generated_check::qmp::metadata::*;
    check_tables!(qmp);
    check_tables!(qga);

    // Renamed types are spelled as the newest release spells them.
    assert_eq!(STRUCTS.get("SgxInfo").unwrap().wire_name, "SgxInfo");
    // section-size was deprecated in 7.0 and removed in 8.0.
    let section_size = field!(qmp, "SgxInfo", "section-size");
    assert!(!section_size.is_deprecated((6, 2, 0)) && section_size.is_deprecated((7, 2, 0)));
    assert_eq!(section_size.until, Some((8, 0, 0)));
    assert!(!field!(qmp, "SgxInfo", "flc").is_deprecated((7, 0, 0)));
    // MEM_UNPLUG_ERROR was deprecated in 6.2 and removed in 9.1.
    let unplug = STRUCTS.get("MemUnplugErrorEvent").unwrap();
    assert_eq!(
        unplug.deprecated,
        [Releases { since: (6, 2, 0), until: Some((9, 1, 0)) }].as_slice()
    );
    assert!(!unplug.is_deprecated((6, 1, 0)) && unplug.is_deprecated((9, 0, 0)));
    // CpuInfoFast's discriminator is arch in 2.12 and target from 3.0, which
    // keeps arch until 6.0.
    let cpu = STRUCTS.get("CpuInfoFast").unwrap();
    assert_eq!(cpu.discriminator((2, 12, 0)).map(|field| field.wire_name), Some("arch"));
    assert_eq!(cpu.discriminator((5, 2, 0)).map(|field| field.wire_name), Some("target"));
    assert_eq!(field!(qmp, "CpuInfoFast", "arch").until, Some((6, 0, 0)));

    // QEMU 6.2 made the simple unions flat.
    let chardev = STRUCTS.get("ChardevBackend").unwrap();
    assert!(chardev.is_simple_union((6, 1, 0)) && !chardev.is_simple_union((6, 2, 0)));
    assert!(!STRUCTS.get("BlockdevOptions").unwrap().is_simple_union((6, 1, 0)));
    assert_eq!(
        ENUMS.get(chardev.branch_enum.unwrap()).unwrap().kind,
        EnumKind::UnionBranch
    );

    // gen:false arguments, as each release has them.
    let netdev_add = command!(qmp, "netdev_add").args;
    assert!(netdev_add.is_free_form((4, 2, 0)) && !netdev_add.is_free_form((5, 0, 0)));
    assert!(command!(qmp, "device_add").args.is_free_form((11, 1, 3)));
    let object_add = command!(qmp, "object-add").args;
    for (version, free_form) in [
        ((2, 0, 0), true),
        ((2, 5, 0), false),
        ((5, 0, 0), true),
        ((6, 0, 0), false),
    ] {
        assert_eq!(object_add.is_free_form(version), free_form, "{version:?}");
    }
    assert!(command!(qmp, "query-status").args.free_form.is_empty());

    assert_eq!(returns!(qmp, "query-status").name, "StatusInfo");
    assert_eq!(returns!(qmp, "query-status").kind, TypeKind::Struct);
    assert_eq!(returns!(qmp, "query-commands").name, "List<CommandInfo>");
    assert!(command!(qmp, "query-status").returns.is_some());
    assert!(command!(qmp, "migrate-pause").allows_oob((11, 1, 3)));
    assert!(!command!(qmp, "query-status").allows_oob((11, 1, 3)));
    assert!(command!(qmp, "qmp_capabilities").allows_preconfig((11, 1, 3)));
    assert!(!command!(qmp, "migrate-pause").allows_preconfig((11, 1, 3)));
    assert!(command!(qmp, "query-status").sends_success_response((11, 1, 3)));
    assert!(field!(qmp, "BlockdevOptionsBlkdebug", "image").boxed);
    assert!(!field!(qmp, "BlockdevOptionsBlkdebug", "config").boxed);

    assert!(!command!(qga, "guest-shutdown").sends_success_response((11, 1, 3)));
    assert!(command!(qga, "guest-ping").sends_success_response((11, 1, 3)));
    // A builtin that a command returns is spelled as the newest release
    // spells it, and so is a field's: qom-set's value was a visitor before
    // QEMU 2.4.
    assert_eq!(returns!(qga, "guest-get-time").name, "int");
    let value = &TYPE_REFS[field!(qmp, "QomSet", "value").ty as usize];
    assert_eq!((value.name, value.prim), ("any", Some(Prim::Any)));
    assert_eq!(
        returns!(qga, "guest-get-time").prim,
        Some(generated_check::qga::metadata::Prim::I64)
    );
}

fn histories() {
    use generated_check::history::metadata::*;
    check_tables!(history);
    check_tables!(guest);

    assert_eq!(STRUCTS.get("FooInfo").unwrap().wire_name, "FooInfo");
    let shape = STRUCTS.get("Shape").unwrap();
    assert_eq!(
        shape.simple_union,
        [Releases {
            since: (6, 1, 0),
            until: Some((6, 2, 0)),
        }]
        .as_slice()
    );
    // loose takes undeclared arguments in 6.1, declares them in 6.2 and
    // takes undeclared ones again from 7.0.
    assert_eq!(
        command!(history, "loose").args.free_form,
        [
            Releases {
                since: (6, 1, 0),
                until: Some((6, 2, 0)),
            },
            Releases {
                since: (7, 0, 0),
                until: None,
            },
        ]
        .as_slice()
    );
    // Every release's flags: flip allows out-of-band execution in 6.1 and
    // from 7.0, and preconfig from 6.2.
    let flip = command!(history, "flip");
    let since = |since, until| Releases { since, until };
    assert_eq!(
        flip.allow_oob,
        [since((6, 1, 0), Some((6, 2, 0))), since((7, 0, 0), None)].as_slice()
    );
    assert_eq!(flip.allow_preconfig, [since((6, 2, 0), None)].as_slice());
    assert_eq!(flip.success_response, [since((6, 1, 0), None)].as_slice());
    assert!(!flip.allows_oob((6, 2, 0)) && flip.allows_preconfig((6, 2, 0)));
    // quiet does not reply in 6.2 only.
    let quiet = command!(history, "quiet");
    assert_eq!(
        quiet.success_response,
        [since((6, 1, 0), Some((6, 2, 0))), since((7, 0, 0), None)].as_slice()
    );
    assert!(!quiet.sends_success_response((6, 2, 0)) && quiet.sends_success_response((7, 0, 0)));
    // gone's flag and deprecation end with it in 7.0, as do OLD_EVT's.
    let gone = command!(history, "gone");
    assert_eq!(gone.until, Some((7, 0, 0)));
    assert_eq!(gone.allow_oob, [since((6, 1, 0), Some((7, 0, 0)))].as_slice());
    assert_eq!(gone.deprecated, [since((6, 2, 0), Some((7, 0, 0)))].as_slice());
    let old_event = STRUCTS.get("OldEvtEvent").unwrap();
    assert_eq!(old_event.deprecated, [since((6, 2, 0), Some((7, 0, 0)))].as_slice());
    assert!(!old_event.is_deprecated((6, 1, 0)) && old_event.is_deprecated((6, 2, 0)));
    // Types are unstable until 6.2: Mode in 6.1, and Lab in its only release.
    let mode = ENUMS.get("Mode").unwrap();
    assert_eq!(mode.unstable, [since((6, 1, 0), Some((6, 2, 0)))].as_slice());
    assert!(mode.is_unstable((6, 1, 0)) && !mode.is_unstable((6, 2, 0)) && mode.deprecated.is_empty());
    let lab = STRUCTS.get("Lab").unwrap();
    assert_eq!(lab.unstable, [since((6, 1, 0), Some((6, 2, 0)))].as_slice());
    assert!(lab.is_unstable((6, 1, 0)) && !lab.is_deprecated((6, 1, 0)));
    // Cpu's discriminator is arch in 6.1 and target from 6.2, which keeps
    // arch.
    let cpu = STRUCTS.get("Cpu").unwrap();
    assert_eq!(field!(history, "Cpu", "arch").discriminator, [since((6, 1, 0), Some((6, 2, 0)))].as_slice());
    assert_eq!(field!(history, "Cpu", "target").discriminator, [since((6, 2, 0), None)].as_slice());
    assert_eq!(cpu.discriminator((6, 1, 0)).map(|field| field.name), Some("arch"));
    assert_eq!(cpu.discriminator((7, 0, 0)).map(|field| field.name), Some("target"));
    assert!(!field!(history, "Cpu", "arch").is_discriminator((6, 2, 0)));
    assert_eq!(cpu.discriminator_field, Some("target"));
    // Probe's instability ends with it in 6.2.
    let probe = ENUMS.get("Probe").unwrap();
    assert_eq!(probe.unstable, [since((6, 1, 0), Some((6, 2, 0)))].as_slice());
    // backup has its own features, not those of its argument struct, which
    // is unstable in 6.1: deprecated from 6.2, and unstable in 6.2 only.
    let backup = command!(history, "backup");
    assert_eq!(backup.deprecated, [since((6, 2, 0), None)].as_slice());
    assert_eq!(backup.unstable, [since((6, 2, 0), Some((7, 0, 0)))].as_slice());
    // A field's Rust name keeps the prefix of a raw identifier.
    assert_eq!(shape.discriminator_field, Some("r#type"));
    assert!(shape.fields.iter().any(|field| field.name == "r#type" && field.wire_name == "type"));
    assert_eq!(returns!(history, "query-kind").name, "List<KindInfo>");
    // b is deprecated until it goes in 7.0, c unstable in every release.
    let b = field!(history, "Thing", "b");
    assert_eq!(b.deprecated, [since((6, 1, 0), Some((7, 0, 0)))].as_slice());
    assert!(b.is_deprecated((6, 2, 0)) && !b.is_deprecated((7, 0, 0)));
    assert_eq!(field!(history, "Thing", "c").unstable, [since((6, 1, 0), None)].as_slice());
    let a = field!(history, "Thing", "a");
    assert!(a.deprecated.is_empty() && a.unstable.is_empty());
    // e stays, but 7.0 no longer deprecates it.
    let e = field!(history, "Thing", "e");
    assert_eq!(e.deprecated, [since((6, 1, 0), Some((7, 0, 0)))].as_slice());
    assert_eq!(e.until, None);
    assert!(field!(history, "Node", "next").boxed);
    assert!(!field!(history, "Node", "children").boxed);
    let color = ENUMS.get("Color").unwrap();
    let blue = color.variants.iter().find(|variant| variant.wire_name == "blue").unwrap();
    assert_eq!(blue.deprecated, [since((6, 2, 0), None)].as_slice());
    assert!(!blue.is_deprecated((6, 1, 0)) && blue.is_deprecated((7, 0, 0)));

    assert!(command!(guest, "guest-shutdown").success_response.is_empty());
    assert_eq!(command!(guest, "guest-shutdown").since, (6, 2, 0));
    assert_eq!(returns!(guest, "guest-info").name, "GuestInfo");
    assert_eq!(command!(guest, "guest-info").since, (7, 0, 0));
    let sync = command!(guest, "guest-sync");
    assert_eq!((sync.since, sync.until), ((6, 2, 0), Some((7, 0, 0))));
    assert_eq!(returns!(guest, "guest-sync").name, "int");
}

fn main() {
    pinned();
    histories();
}
