//! The emitted Rust: what it says, that it compiles, its documentation in
//! both modes, its module layout, and how generation and writing fail.
//!
//! `tests/emitted` holds two synthetic histories, `qmp` and `qga`: the schema
//! of each release, and the exact output that the generator must emit for
//! them. Set `QAPI_CODEGEN_BLESS=1` to rewrite the expected output after a
//! deliberate change, and review the difference.

use qapi_codegen::unified::Docs::{Reference, Schema};
use qapi_codegen::unified::Protocol::{Qga, Qmp};
use qapi_codegen::unified::{
    Construct, Docs, GenerateError, GeneratedOutput, Protocol, RegistryError, Releases,
    SchemaErrorKind, build_qga_registry, build_qmp_registry, emit_registry, generate,
};
use qapi_codegen::versions::PINNED_VERSIONS;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn emitted() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/emitted")
}

/// The generated modules of the pinned releases, generated once per test run.
fn pinned(protocol: Protocol, docs: Docs) -> &'static GeneratedOutput {
    static OUTPUTS: [OnceLock<GeneratedOutput>; 4] = [const { OnceLock::new() }; 4];
    let index = usize::from(protocol == Qga) * 2 + usize::from(docs == Schema);
    OUTPUTS[index].get_or_init(|| {
        generate(
            &workspace().join("vendor/qemu"),
            Releases::Pinned(PINNED_VERSIONS),
            protocol,
            docs,
        )
        .unwrap()
    })
}

fn git(repo: &Path, args: &[&str]) {
    let result = Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
        .args(["-c", "core.hooksPath=/dev/null", "-c", "user.name=Test"])
        .args(["-c", "user.email=test@example.invalid"])
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

/// A git repository with a commit and tag for each release, holding its
/// schema as the QMP or QGA entry point.
fn repository(qga: bool, releases: &[(&str, &str)]) -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let schema = repo
        .path()
        .join(if qga { "qga" } else { "qapi" })
        .join("qapi-schema.json");
    std::fs::create_dir_all(schema.parent().unwrap()).unwrap();
    git(repo.path(), &["init", "-q"]);
    for (tag, text) in releases {
        std::fs::write(&schema, text).unwrap();
        git(repo.path(), &["add", "."]);
        git(repo.path(), &["commit", "-q", "--allow-empty", "-m", tag]);
        git(repo.path(), &["tag", tag]);
    }
    repo
}

/// The releases of a synthetic history of `tests/emitted`, a tag and schema
/// per `v*.json` file there, in release order.
fn history_releases(name: &str) -> Vec<(String, String)> {
    let dir = emitted().join(name);
    let mut releases: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let tag = path.file_stem()?.to_str()?.to_owned();
            (path.extension()? == "json").then(|| (tag, std::fs::read_to_string(&path).unwrap()))
        })
        .collect();
    releases.sort_by_key(|(tag, _)| qapi_codegen::versions::parse_version_tag(tag).unwrap());
    releases
}

/// The output of a synthetic history of `tests/emitted`.
fn history(name: &str, docs: Docs) -> GeneratedOutput {
    let releases = history_releases(name);
    let releases: Vec<_> = releases
        .iter()
        .map(|(tag, text)| (tag.as_str(), text.as_str()))
        .collect();
    let tags: Vec<_> = releases.iter().map(|(tag, _)| *tag).collect();
    let repo = repository(name == "qga", &releases);
    let protocol = if name == "qga" { Qga } else { Qmp };
    generate(repo.path(), Releases::Tags(&tags), protocol, docs).unwrap()
}

/// Require `actual` to be the content of `path`, or with `QAPI_CODEGEN_BLESS`
/// set, write it there.
#[track_caller]
fn expect_file(path: &Path, actual: &str) {
    if std::env::var_os("QAPI_CODEGEN_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, actual).unwrap();
        return;
    }
    let expected =
        std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    if expected != actual {
        let line = expected
            .lines()
            .zip(actual.lines())
            .position(|(expected, actual)| expected != actual)
            .unwrap_or_else(|| expected.lines().count().min(actual.lines().count()));
        panic!(
            "{} differs from the output from line {}:\n  expected: {:?}\n  actual:   {:?}\n\
             Set QAPI_CODEGEN_BLESS=1 to accept the output.",
            path.display(),
            line + 1,
            expected.lines().nth(line),
            actual.lines().nth(line),
        );
    }
}

/// The metadata after the type definitions that start every metadata file.
fn tables(metadata: &str) -> &str {
    let start = metadata
        .find("/// Every type that a field")
        .expect("metadata tables");
    &metadata[start..]
}

/// A registry that the public builders build emits as generate() emits the
/// releases.
#[test]
fn emit_registry_emits_a_registry_of_the_public_builders() {
    for name in ["qmp", "qga"] {
        let releases = history_releases(name);
        let releases: Vec<_> = releases
            .iter()
            .map(|(tag, text)| (tag.as_str(), text.as_str()))
            .collect();
        let tags: Vec<_> = releases.iter().map(|(tag, _)| *tag).collect();
        let repo = repository(name == "qga", &releases);
        let (registry, protocol) = if name == "qga" {
            (build_qga_registry(repo.path(), &tags).unwrap(), Qga)
        } else {
            (build_qmp_registry(repo.path(), &tags).unwrap(), Qmp)
        };
        for docs in [Reference, Schema] {
            let emitted = emit_registry(&registry, protocol, docs).unwrap();
            let generated = generate(repo.path(), Releases::Tags(&tags), protocol, docs).unwrap();
            assert_eq!(emitted.types, generated.types, "{name}");
            assert_eq!(emitted.commands, generated.commands, "{name}");
            assert_eq!(emitted.events, generated.events, "{name}");
            assert_eq!(emitted.metadata, generated.metadata, "{name}");
        }
    }
}

#[test]
fn synthetic_histories_emit_exactly_the_expected_modules() {
    for name in ["qmp", "qga"] {
        let output = history(name, Reference);
        let dir = emitted().join(name).join("out");
        expect_file(&dir.join("mod.rs"), &output.types);
        expect_file(&dir.join("commands.rs"), &output.commands);
        expect_file(&dir.join("events.rs"), &output.events);
        expect_file(&dir.join("tables.rs"), tables(&output.metadata));

        let docs = history(name, Schema);
        let dir = emitted().join(name).join("docs");
        expect_file(&dir.join("mod.rs"), &docs.types);
        expect_file(&dir.join("commands.rs"), &docs.commands);
        expect_file(&dir.join("events.rs"), &docs.events);
        assert_eq!(docs.metadata, output.metadata, "{name}");
    }
}

/// A generated file without its `#[qapi]` attributes and their import, the
/// attribute macro of qrab-qapi-macros, which would expand them.
///
/// With `unindented`, every item, field and variant also gets a last doc
/// line that does not start with a space, as the macro's line for a build
/// condition does. rustdoc then keeps the space after each `///`, which
/// adds a column to the indentation of the generated documentation.
fn without_qapi_attributes(source: &str, unindented: bool) -> String {
    let mut file = syn::parse_file(source).unwrap();
    let mut removed = 0;
    let mut strip = |attrs: &mut Vec<syn::Attribute>| {
        let before = attrs.len();
        attrs.retain(|attr| !attr.path().is_ident("qapi"));
        removed += before - attrs.len();
        if unindented {
            attrs.push(syn::parse_quote! { #[doc = ""] });
            attrs.push(syn::parse_quote! { #[doc = "Unindented."] });
        }
    };
    file.items.retain(|item| {
        !matches!(item, syn::Item::Use(item)
            if quote::quote!(#item).to_string() == "use qapi_macros :: qapi ;")
    });
    for item in &mut file.items {
        match item {
            syn::Item::Struct(item) => {
                strip(&mut item.attrs);
                item.fields
                    .iter_mut()
                    .for_each(|field| strip(&mut field.attrs));
            }
            syn::Item::Enum(item) => {
                strip(&mut item.attrs);
                for variant in &mut item.variants {
                    strip(&mut variant.attrs);
                }
            }
            _ => {}
        }
    }
    assert!(
        removed > 0
            || file
                .items
                .iter()
                .all(|item| matches!(item, syn::Item::Mod(_))),
        "no #[qapi] attributes in {source:.200}"
    );
    prettyplease::unparse(&file)
}

/// Run cargo on the fixture crate; arguments after `--` go to the tool.
fn cargo(fixture: &Path, args: &[&str], rustdocflags: Option<&str>) -> String {
    let (args, tool) = args.split_at(
        args.iter()
            .position(|arg| *arg == "--")
            .unwrap_or(args.len()),
    );
    let mut command = Command::new(env!("CARGO"));
    command
        .args(args)
        .arg("--offline")
        .arg("--manifest-path")
        .arg(fixture.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(Path::new(env!("CARGO_TARGET_TMPDIR")).join("generated-code"))
        .args(tool)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS");
    if let Some(flags) = rustdocflags {
        command.env("RUSTDOCFLAGS", flags);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "cargo {args:?} failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Generated code compiles, and what its metadata says is true.
///
/// qrab-qapi-macros, which expands the `#[qapi]` attributes, and the
/// qrab-qapi runtime that its expansion uses come later, so this builds a
/// crate of the generated modules with those attributes removed and a
/// stand-in for `serde_json::Value`. That proves what the items, fields,
/// references and modules are: every type exists in a module that the code
/// naming it can see, code outside the modules reaches each command under
/// `commands` and each event under `events`, no two items of a module share
/// a name, recursive
/// fields are boxed and the metadata tables type-check against their
/// definitions. It cannot prove that the attributes expand. The crate's
/// `main`, `tests/emitted/check.rs`, then checks the metadata of the pinned
/// releases and of the synthetic histories. Its documentation, the reference
/// lines of the default mode and the schema prose of the schema-docs mode,
/// must build with every rustdoc lint denied and contain no doctest, also
/// where rustdoc keeps the space after `///` (`without_qapi_attributes`),
/// and the crate must pass Clippy with its warnings denied where Clippy is
/// installed.
#[test]
fn generated_code_compiles_and_its_metadata_is_true() {
    let fixture = Path::new(env!("CARGO_TARGET_TMPDIR")).join("generated-code-crate");
    let _ = std::fs::remove_dir_all(&fixture);
    std::fs::create_dir_all(fixture.join("serde_json/src")).unwrap();
    std::fs::write(
        fixture.join("Cargo.toml"),
        "[package]\nname = \"generated-check\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
         [workspace]\n[dependencies]\nserde_json = { path = \"serde_json\" }\n",
    )
    .unwrap();
    std::fs::write(
        fixture.join("serde_json/Cargo.toml"),
        "[package]\nname = \"serde_json\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::write(
        fixture.join("serde_json/src/lib.rs"),
        "//! A stand-in for the JSON value type.\n/// Any JSON value.\npub struct Value;\n",
    )
    .unwrap();
    let modules = [
        ("qmp", pinned(Qmp, Reference)),
        ("qga", pinned(Qga, Reference)),
        ("qmp_docs", pinned(Qmp, Schema)),
        ("qga_docs", pinned(Qga, Schema)),
        ("history", &history("qmp", Reference)),
        ("history_docs", &history("qmp", Schema)),
        ("guest", &history("qga", Reference)),
        ("guest_docs", &history("qga", Schema)),
        ("qmp_docs_unindented", pinned(Qmp, Schema)),
        ("qga_docs_unindented", pinned(Qga, Schema)),
        ("history_docs_unindented", &history("qmp", Schema)),
    ];
    let mut lib = String::from(
        "//! The generated modules without their `#[qapi]` attributes.\n\
         #![allow(dead_code, unused_imports)]\n",
    );
    for (name, output) in modules {
        let dir = fixture.join("src").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        for (file, source) in [
            ("mod.rs", &output.types),
            ("commands.rs", &output.commands),
            ("events.rs", &output.events),
        ] {
            let unindented = name.ends_with("_unindented");
            std::fs::write(dir.join(file), without_qapi_attributes(source, unindented)).unwrap();
        }
        std::fs::write(dir.join("metadata.rs"), &output.metadata).unwrap();
        lib.push_str(&format!("pub mod {name};\n"));
        // Each command of `COMMANDS` and each event, named from outside.
        for command in metadata_commands(output) {
            lib.push_str(&format!(
                "const _: Option<&{name}::commands::{command}> = None;\n"
            ));
        }
        for (_, event, _) in command_and_event_structs(output)
            .into_iter()
            .filter(|(_, _, kind)| *kind == "event")
        {
            lib.push_str(&format!(
                "const _: Option<&{name}::events::{event}> = None;\n"
            ));
        }
    }
    std::fs::write(fixture.join("src/lib.rs"), lib).unwrap();
    std::fs::copy(emitted().join("check.rs"), fixture.join("src/main.rs")).unwrap();

    cargo(&fixture, &["run", "--quiet"], None);
    cargo(
        &fixture,
        &["doc", "--quiet", "--no-deps", "--lib"],
        Some("-D warnings -W rustdoc::all"),
    );
    let doctests = cargo(&fixture, &["test", "--quiet", "--doc"], None);
    assert!(
        doctests.contains("0 passed; 0 failed; 0 ignored"),
        "{doctests}"
    );
    // Clippy's documentation lints, such as `doc_lazy_continuation`, read
    // the rendered prose of the schema-docs modules.
    let clippy = Command::new(env!("CARGO"))
        .args(["clippy", "--version"])
        .output()
        .is_ok_and(|output| output.status.success());
    if clippy {
        cargo(
            &fixture,
            &["clippy", "--quiet", "--lib", "--", "-D", "warnings"],
            None,
        );
    } else {
        // Past the test harness's capture, so that a passing run shows it.
        use std::io::Write;
        let _ = writeln!(
            std::io::stderr(),
            "note: Clippy is not installed, so the generated code is not linted"
        );
    }
}

#[test]
fn schema_docs_mode_changes_only_documentation() {
    let strip = |code: &str| {
        code.lines()
            .filter(|line| !line.trim_start().starts_with("///"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    for (protocol, prose) in [
        (
            Qmp,
            "/// Allow client connections for VNC, Spice and socket based character",
        ),
        (
            Qga,
            "/// Ping the guest agent, a non-error return implies success",
        ),
    ] {
        let plain = pinned(protocol, Reference);
        let docs = pinned(protocol, Schema);
        assert!(
            docs.commands.contains(prose),
            "{protocol:?}: missing schema prose"
        );
        assert!(
            !plain.commands.contains(prose),
            "{protocol:?}: default has prose"
        );
        for (plain, docs) in [
            (&plain.types, &docs.types),
            (&plain.commands, &docs.commands),
            (&plain.events, &docs.events),
            (&plain.metadata, &docs.metadata),
        ] {
            assert_eq!(strip(plain), strip(docs), "{protocol:?}");
        }
    }
}

/// The documentation of each item of a module, by name.
fn item_docs(source: &str) -> std::collections::BTreeMap<String, Vec<String>> {
    let docs = |attrs: &[syn::Attribute]| {
        attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .map(|attr| quote::quote!(#attr).to_string())
            .collect()
    };
    syn::parse_file(source)
        .unwrap()
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Struct(item) => Some((item.ident.to_string(), docs(&item.attrs))),
            syn::Item::Enum(item) => Some((item.ident.to_string(), docs(&item.attrs))),
            _ => None,
        })
        .collect()
}

#[test]
fn schema_docs_mode_keeps_the_reference_line_of_items_without_prose() {
    for protocol in [Qmp, Qga] {
        let mut reference_only = 0;
        for (plain, docs) in [
            (
                &pinned(protocol, Reference).types,
                &pinned(protocol, Schema).types,
            ),
            (
                &pinned(protocol, Reference).commands,
                &pinned(protocol, Schema).commands,
            ),
            (
                &pinned(protocol, Reference).events,
                &pinned(protocol, Schema).events,
            ),
        ] {
            let docs = item_docs(docs);
            for (name, plain) in item_docs(plain) {
                if !plain.is_empty() && docs[&name] == plain {
                    reference_only += 1;
                }
                assert!(
                    plain.is_empty() || !docs[&name].is_empty(),
                    "{protocol:?}: {name} has no documentation in schema-docs mode"
                );
            }
        }
        // `AcpiSlotType` and `GuestDeviceInfo` have no schema prose.
        assert!(reference_only > 10, "{protocol:?}: {reference_only}");
    }
}

#[test]
fn default_output_carries_no_schema_prose() {
    // QAPI schema documentation is GPL-2.0-or-later QEMU text; generated
    // items only name themselves and point at QEMU's reference manuals.
    for (output, protocol, reference) in [
        (pinned(Qmp, Reference), "QMP", "QEMU QMP Reference Manual"),
        (
            pinned(Qga, Reference),
            "QEMU Guest Agent",
            "QEMU Guest Agent Protocol Reference",
        ),
    ] {
        let mut item_docs = 0;
        for code in [&output.types, &output.commands, &output.events] {
            assert!(!code.contains("/**"), "{protocol}: block doc comment");
            for line in code
                .lines()
                .filter_map(|line| line.trim().strip_prefix("///"))
            {
                if line == " Additional command arguments not modeled by this schema." {
                    continue;
                }
                let (kind, rest) = line
                    .strip_prefix(&format!(" {protocol} "))
                    .and_then(|rest| rest.split_once(" `"))
                    .unwrap_or_else(|| panic!("{protocol}: unexpected doc line {line:?}"));
                let (name, tail) = rest.split_once('`').unwrap();
                assert!(
                    ["command", "event", "struct", "enum", "union", "alternate"].contains(&kind)
                        && !name.is_empty()
                        && !name.contains(char::is_whitespace)
                        && tail == format!("; see the {reference}."),
                    "{protocol}: unexpected doc line {line:?}"
                );
                item_docs += 1;
            }
        }
        assert!(item_docs > 30, "{protocol}: only {item_docs} item docs");
        // The metadata documents only its own types and tables, the same
        // for every registry.
        let docs = |metadata: &str| -> Vec<String> {
            metadata
                .lines()
                .filter(|line| line.trim_start().starts_with("//"))
                .map(String::from)
                .collect()
        };
        assert_eq!(
            docs(&output.metadata),
            docs(&history("qmp", Reference).metadata),
            "{protocol}: metadata carries schema documentation"
        );
    }
}

/// The `#[qapi(...)]` attributes of the struct `name` of a module, as text.
fn struct_attributes(source: &str, name: &str) -> Vec<String> {
    let file = syn::parse_file(source).unwrap();
    let item = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Struct(item) if item.ident == name => Some(item),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no struct {name}"));
    item.attrs
        .iter()
        .filter(|attr| attr.path().is_ident("qapi"))
        .map(|attr| quote::quote!(#attr).to_string())
        .collect()
}

#[test]
fn spice_types_arrive_in_2_1() {
    for name in ["SpiceServerInfo", "SpiceBasicInfo"] {
        let attributes = struct_attributes(&pinned(Qmp, Reference).types, name);
        assert!(
            attributes.contains(&"# [qapi (added = \"2.1.0\")]".to_owned()),
            "{name}: {attributes:?}"
        );
    }
}

#[test]
fn netdev_add_is_a_current_command() {
    let attributes = struct_attributes(&pinned(Qmp, Reference).commands, "NetdevAdd");
    assert!(
        attributes.contains(&"# [qapi (command)]".to_owned()),
        "{attributes:?}"
    );
    assert!(
        !attributes
            .iter()
            .any(|attribute| attribute.contains("removed")),
        "{attributes:?}"
    );
}

/// Boxed commands with different names should have their data type's fields
/// inlined (duplicated) directly into the command struct, rather than emitting
/// a synthetic `data` field annotated with `#[qapi(flatten)]`.
#[test]
fn boxed_commands_inline_their_data() {
    let output = pinned(Qmp, Reference);
    for (name, field, data) in [
        ("NetdevAdd", "r#type", "Netdev"),
        ("BlockdevAdd", "driver", "BlockdevOptions"),
    ] {
        let file = syn::parse_file(&output.commands).unwrap();
        let fields: Vec<_> = file
            .items
            .iter()
            .find_map(|item| match item {
                syn::Item::Struct(item) if item.ident == name => Some(item),
                _ => None,
            })
            .unwrap()
            .fields
            .iter()
            .map(|field| {
                let ty = &field.ty;
                (
                    field.ident.as_ref().unwrap().to_string(),
                    quote::quote!(#ty).to_string(),
                )
            })
            .collect();
        assert!(
            fields.iter().any(|(name, _)| name == field),
            "{name}: {fields:?}"
        );
        assert!(
            !fields
                .iter()
                .any(|(name, ty)| name == "data" && ty.contains(data)),
            "{name}: {fields:?}"
        );
    }

    // Flat unions keep their branch fields at the same JSON object level.
    // Reject synthetic data-ref wrappers while allowing these branches and
    // the gen:false catch-all argument maps.
    let mut flattened = 0;
    for code in [&output.types, &output.commands, &output.events] {
        for item in syn::parse_file(code).unwrap().items {
            let syn::Item::Struct(item) = item else {
                continue;
            };
            for field in item.fields {
                let marker = |name: &str| {
                    field.attrs.iter().any(|attr| {
                        attr.path().is_ident("qapi")
                            && matches!(&attr.meta,
                        syn::Meta::List(list) if list.tokens.to_string() == name)
                    })
                };
                if marker("flatten") {
                    let ty = &field.ty;
                    let ty = quote::quote!(#ty).to_string();
                    assert!(
                        marker("union") || ty.contains("HashMap < String , serde_json :: Value >"),
                        "Unexpected flatten field in {}: {:?}",
                        item.ident,
                        field.ident
                    );
                    flattened += 1;
                }
            }
        }
    }
    assert!(flattened > 50, "only {flattened} flattened fields");
}

#[test]
fn boxed_commands_have_structs_of_their_own() {
    let file = syn::parse_file(&pinned(Qmp, Reference).commands).unwrap();
    let structs: Vec<_> = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Struct(item) => Some(item.ident.to_string()),
            _ => None,
        })
        .collect();
    for expected in [
        "NetdevAdd",
        "BlockdevAdd",
        "ObjectAdd",
        "SetPassword",
        "ExpirePassword",
        "NbdServerAdd",
        "BlockSetIoThrottle",
        "BlockExportAdd",
        "QueryStats",
        "AnnounceSelf",
        "SetNumaNode",
        "BlockJobChange",
    ] {
        assert!(structs.iter().any(|name| name == expected), "{expected}");
    }
}

/// The command and event structs of an output, by the module they are in:
/// `(module, name, kind)` for each struct with `#[qapi(command)]` or
/// `#[qapi(event)]`.
fn command_and_event_structs(
    output: &GeneratedOutput,
) -> Vec<(&'static str, String, &'static str)> {
    let mut found = Vec::new();
    for (module, source) in [
        ("mod.rs", &output.types),
        ("commands.rs", &output.commands),
        ("events.rs", &output.events),
    ] {
        for item in syn::parse_file(source).unwrap().items {
            let syn::Item::Struct(item) = item else {
                continue;
            };
            for attr in &item.attrs {
                match quote::quote!(#attr).to_string().as_str() {
                    "# [qapi (command)]" => found.push((module, item.ident.to_string(), "command")),
                    "# [qapi (event)]" => found.push((module, item.ident.to_string(), "event")),
                    _ => {}
                }
            }
        }
    }
    found
}

/// The Rust names of the entries of an output's `COMMANDS` table.
fn metadata_commands(output: &GeneratedOutput) -> Vec<String> {
    output
        .metadata
        .split("CommandMeta { name: \"")
        .skip(1)
        .map(|entry| entry[..entry.find('"').unwrap()].to_owned())
        .collect()
}

/// Every command struct is in `commands.rs` and every event struct in
/// `events.rs`, also those that data types name, which `mod.rs` imports
/// from there; every entry of `COMMANDS` is one of those commands.
#[test]
fn modules_separate_types_commands_and_events() {
    let outputs = [
        ("qmp", pinned(Qmp, Reference)),
        ("qga", pinned(Qga, Reference)),
        ("history", &history("qmp", Reference)),
        ("guest", &history("qga", Reference)),
    ];
    for (name, output) in outputs {
        let structs = command_and_event_structs(output);
        for (module, item, kind) in &structs {
            let expected = if *kind == "command" {
                "commands.rs"
            } else {
                "events.rs"
            };
            assert_eq!(module, &expected, "{name}: {kind} {item}");
        }
        let mut commands: Vec<_> = structs
            .iter()
            .filter(|(_, _, kind)| *kind == "command")
            .map(|(_, item, _)| item.clone())
            .collect();
        let mut listed = metadata_commands(output);
        commands.sort();
        listed.sort();
        assert_eq!(commands, listed, "{name}");

        let types = syn::parse_file(&output.types).unwrap();
        let modules: Vec<_> = types
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Mod(item) if item.content.is_none() => Some(item.ident.to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(modules, ["commands", "events", "metadata"], "{name}");
        let imports: Vec<_> = types
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Use(item) => Some(quote::quote!(#item).to_string()),
                _ => None,
            })
            .collect();
        // Private: the crate root re-exports only the data types.
        let globs: &[&str] = if matches!(name, "qga" | "guest") {
            &[]
        } else {
            &["use self :: commands :: * ;"]
        };
        assert_eq!(
            imports,
            [&["use crate as qapi ;", "use qapi_macros :: qapi ;"], globs].concat(),
            "{name}"
        );
    }
    let output = pinned(Qmp, Reference);
    let structs = command_and_event_structs(output);
    let count = |kind| structs.iter().filter(|item| item.2 == kind).count();
    assert!(count("command") > 200 && count("event") > 30);
    // Commands whose structs data types name, as the transaction actions do.
    for command in [
        "BlockdevBackup",
        "BlockdevSnapshotSync",
        "BlockDirtyBitmapAdd",
        "DriveBackup",
    ] {
        assert!(
            struct_attributes(&output.commands, command).contains(&"# [qapi (command)]".to_owned()),
            "{command}"
        );
        assert!(
            output.types.contains(&format!("data: {command},")),
            "{command}"
        );
    }
    assert!(structs.contains(&("events.rs", "ShutdownEvent".to_owned(), "event")));
}

#[test]
fn generation_reports_invalid_pins_trees_and_schemas() {
    let repo = repository(false, &[("v9.0.0", "{ 'command': 'ping' }")]);
    let error = generate(repo.path(), Releases::Pinned("not a pin\n"), Qmp, Reference).unwrap_err();
    assert!(matches!(error, GenerateError::Pins(_)), "{error:?}");

    let pin = format!("v9.0.0 {}\n", "0".repeat(40));
    let error = generate(repo.path(), Releases::Pinned(&pin), Qmp, Reference).unwrap_err();
    assert!(
        matches!(&error, GenerateError::Tree(source) if source.to_string().contains("moved")),
        "{error:?}"
    );

    let repo = repository(
        false,
        &[("v9.0.0", "{ 'command': 'ping' }\n{ 'command': 'ping' }")],
    );
    let error = generate(repo.path(), Releases::Tags(&["v9.0.0"]), Qmp, Reference).unwrap_err();
    assert!(matches!(error, GenerateError::Registry(_)), "{error:?}");

    // QEMU 1.0 does not check the types that a schema refers to; the
    // lowering refuses an undefined one as unsupported.
    let repo = repository(
        false,
        &[("v1.0.0", "{ 'command': 'c', 'data': { 'x': 'Missing' } }")],
    );
    let error = generate(repo.path(), Releases::Tags(&["v1.0.0"]), Qmp, Reference).unwrap_err();
    assert!(
        matches!(&error, GenerateError::Registry(RegistryError::Schema { source, .. })
            if matches!(source.kind(), SchemaErrorKind::Unsupported {
                construct: Construct::UndefinedType { name },
            } if name == "Missing")),
        "{error:?}"
    );
}

#[test]
fn generated_output_keeps_the_registry_warnings() {
    let schema = "##\n# @Wrong:\n#\n# Since: 2.8\n##\n{ 'struct': 'Right', 'data': {} }\n";
    let repo = repository(false, &[("v2.8.0", schema)]);
    let output = generate(repo.path(), Releases::Tags(&["v2.8.0"]), Qmp, Reference).unwrap();
    let registry = qapi_codegen::unified::build_qmp_registry(repo.path(), &["v2.8.0"]).unwrap();
    assert!(!output.warnings.is_empty());
    assert_eq!(output.warnings, registry.warnings);
}
