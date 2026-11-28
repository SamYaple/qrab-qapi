/// Names the QEMU git tree to regenerate the modules from with QEMU's schema
/// documentation. It takes effect only together with the `schema-docs`
/// feature, which compiles the generator into this build script.
const SCHEMA_DOCS: &str = "QRAB_QAPI_SCHEMA_DOCS";

fn main() {
    println!("cargo::rerun-if-env-changed={SCHEMA_DOCS}");
    println!("cargo::rustc-check-cfg=cfg(qrab_qapi_schema_docs)");
    // An empty value counts as unset, so `QRAB_QAPI_SCHEMA_DOCS=` turns the
    // mode off for one command.
    let qemu = std::env::var_os(SCHEMA_DOCS).filter(|path| !path.is_empty());
    #[cfg(feature = "schema-docs")]
    match qemu {
        Some(qemu) => schema_docs::generate(qemu.into()),
        None => println!(
            "cargo::warning=the `schema-docs` feature is enabled but {SCHEMA_DOCS} is not set, \
             so qrab-qapi builds its checked-in modules without QEMU's schema documentation; \
             set {SCHEMA_DOCS} to the absolute path of a QEMU git tree to include it"
        ),
    }
    #[cfg(not(feature = "schema-docs"))]
    if qemu.is_some() {
        println!(
            "cargo::warning={SCHEMA_DOCS} is set but the `schema-docs` feature of qrab-qapi is \
             not enabled, so the build carries no QEMU schema documentation; enable the feature \
             as well to include it"
        );
    }
}

/// With the `schema-docs` feature and `QRAB_QAPI_SCHEMA_DOCS` set, build.rs
/// regenerates the QMP and QGA modules into `OUT_DIR` with QEMU's
/// GPL-2.0-or-later schema documentation, for local reading only, and sets
/// `cfg(qrab_qapi_schema_docs)`. `src/lib.rs` then includes the module
/// declarations written here in place of the checked-in `src/generated` and
/// `src/qga` trees.
#[cfg(feature = "schema-docs")]
mod schema_docs {
    use super::SCHEMA_DOCS;
    use std::path::PathBuf;

    fn fail(message: std::fmt::Arguments) -> ! {
        eprintln!("error: qrab-qapi {SCHEMA_DOCS}: {message}");
        std::process::exit(1);
    }

    pub fn generate(qemu: PathBuf) {
        // A build script runs in its package directory, so a relative path
        // would resolve against projects/qapi or the registry copy.
        if !qemu.is_absolute() {
            fail(format_args!(
                "{} is a relative path. Set {SCHEMA_DOCS} to the absolute path of a QEMU git \
                 tree.",
                qemu.display()
            ));
        }
        if !qemu.join("qapi").is_dir() {
            fail(format_args!(
                "no QEMU source tree at {}. Set {SCHEMA_DOCS} to a QEMU git checkout that has \
                 the release tags pinned in projects/qapi-codegen/qemu-versions.lock (for \
                 example the initialized vendor/qemu submodule), or unset it.",
                qemu.display()
            ));
        }
        let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));
        use qapi_codegen::unified::{Docs, Protocol, Releases, generate};
        for (dir, protocol) in [("qmp", Protocol::Qmp), ("qga", Protocol::Qga)]
        {
            generate(
                &qemu,
                Releases::Pinned(qapi_codegen::versions::PINNED_VERSIONS),
                protocol,
                Docs::Schema,
            )
            .and_then(|output| output.write_to(&out.join(dir)))
            .unwrap_or_else(|error| {
                let mut message = error.to_string();
                let mut cause = std::error::Error::source(&error);
                while let Some(error) = cause {
                    message = format!("{message}: {error}");
                    cause = error.source();
                }
                fail(format_args!(
                    "generation from {} failed: {message}",
                    qemu.display()
                ))
            });
        }
        // `#[path]` needs a literal, so declare the modules in a file that
        // `src/lib.rs` includes; module paths match the default build.
        let path = |dir: &str| {
            format!("{:?}", out.join(dir).join("mod.rs").display().to_string())
        };
        let modules = format!(
            "#[rustfmt::skip]\n#[path = {}]\nmod generated;\n\n\
             /// QEMU Guest Agent (QGA) types.\n#[rustfmt::skip]\n#[path = {}]\npub mod qga;\n",
            path("qmp"),
            path("qga"),
        );
        std::fs::write(out.join("schema_docs_modules.rs"), modules)
            .unwrap_or_else(|error| {
                fail(format_args!("writing module declarations: {error}"))
            });
        println!("cargo::rustc-cfg=qrab_qapi_schema_docs");
    }
}
