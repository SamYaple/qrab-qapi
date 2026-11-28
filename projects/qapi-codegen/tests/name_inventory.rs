//! Every Rust name that the generator makes from the pinned releases, in
//! the namespace that declares it: no two of one namespace collide, in the
//! full history or in any pinned release alone, and the names that the
//! generator synthesizes or escapes are pinned in `tests/names/`.

use qapi_codegen::{
    namespace::{Name, Pattern, inventory},
    unified::{UnifiedRegistry, build_qga_registry, build_qmp_registry},
    versions::{PINNED_VERSIONS, parse_pinned_versions},
};
use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

fn qemu() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/qemu")
}

fn pinned_tags() -> Vec<String> {
    parse_pinned_versions(PINNED_VERSIONS)
        .unwrap()
        .into_iter()
        .map(|pin| pin.tag)
        .collect()
}

type Build =
    fn(
        &Path,
        &[&str],
    ) -> Result<UnifiedRegistry, qapi_codegen::unified::RegistryError>;

const PROTOCOLS: [(&str, &str, Build); 2] =
    [("QMP", "qmp", build_qmp_registry), ("QGA", "qga", build_qga_registry)];

/// The listing of the names that the generator does not take unchanged
/// from QAPI, after a count of each pattern.
fn listing(names: &[Name]) -> String {
    let mut counts = std::collections::BTreeMap::<Pattern, usize>::new();
    for name in names {
        *counts.entry(name.pattern).or_default() += 1;
    }
    let mut text = String::from(
        "# The Rust names of the generated code by pattern, then every name\n",
    );
    text.push_str(
        "# but those taken unchanged from QAPI and the metadata statics, as\n",
    );
    text.push_str("# pattern, namespace, name: what it stands for.\n");
    for (pattern, count) in &counts {
        writeln!(text, "# {pattern:?}: {count}").unwrap();
    }
    for name in names {
        if matches!(name.pattern, Pattern::Schema | Pattern::Metadata) {
            continue;
        }
        writeln!(
            text,
            "{:?}\t{}\t{}: {}",
            name.pattern, name.namespace, name.name, name.source
        )
        .unwrap();
    }
    text
}

fn expect_file(path: &Path, actual: &str) {
    if std::env::var_os("QAPI_CODEGEN_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let first = expected
        .lines()
        .zip(actual.lines())
        .position(|(expected, actual)| expected != actual);
    assert!(
        expected == actual,
        "{} differs from the inventory at line {}:\n  expected: {:?}\n  actual:   {:?}\n\
         Set QAPI_CODEGEN_BLESS=1 to accept it.",
        path.display(),
        first.map_or(0, |line| line + 1),
        first.and_then(|line| expected.lines().nth(line)),
        first.and_then(|line| actual.lines().nth(line)),
    );
}

/// The full history of both protocols has no two items of a namespace with
/// one name, and its synthesized and escaped names are those listed.
#[test]
fn the_names_of_the_full_history_are_distinct_and_listed() {
    let tags = pinned_tags();
    let tags: Vec<_> = tags.iter().map(String::as_str).collect();
    for (protocol, file, build) in PROTOCOLS {
        let registry = build(&qemu(), &tags).unwrap();
        let names = inventory(&registry, protocol).unwrap();
        // Every item of a namespace has a name of its own.
        let mut seen = std::collections::HashSet::new();
        for name in &names {
            assert!(
                seen.insert((&name.namespace, &name.name)),
                "{protocol} {}: {}",
                name.namespace,
                name.name
            );
        }
        // The types, members and enums of return types are all listed.
        let types = names
            .iter()
            .filter(|name| {
                name.namespace == "items" && name.pattern != Pattern::Reserved
            })
            .count();
        let returns = names
            .iter()
            .filter(|name| name.pattern == Pattern::Returns)
            .count();
        assert_eq!(types, registry.types.len() + returns, "{protocol}");
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/names")
            .join(format!("{file}.txt"));
        expect_file(&path, &listing(&names));
    }
}

/// Each pinned release alone generates without a collision of names.
#[test]
fn every_pinned_release_alone_has_distinct_names() {
    let tags = pinned_tags();
    let workers =
        std::thread::available_parallelism().map_or(1, |n| n.get()).min(4);
    let chunk = tags.len().div_ceil(workers);
    let checked: usize = std::thread::scope(|scope| {
        let handles: Vec<_> = tags
            .chunks(chunk)
            .map(|tags| {
                scope.spawn(move || {
                    let mut checked = 0;
                    for tag in tags {
                        for (protocol, _, build) in PROTOCOLS {
                            let registry = build(&qemu(), &[tag.as_str()])
                                .unwrap_or_else(|error| {
                                    panic!("{protocol} {tag}: {error}")
                                });
                            let names = inventory(&registry, protocol)
                                .unwrap_or_else(|error| {
                                    panic!("{protocol} {tag}: {error}")
                                });
                            checked += names.len();
                        }
                    }
                    checked
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().unwrap()).sum()
    });
    assert!(checked > 0);
}
