fn main() {
    println!("cargo:rerun-if-env-changed=QEMU_MIN_VERSION");
    println!("cargo:rerun-if-env-changed=QEMU_MAX_VERSION");
    #[cfg(feature = "schema-docs")]
    schema_docs::generate();
}

/// The `schema-docs` feature regenerates the QMP and QGA modules into
/// `OUT_DIR` with QEMU's GPL-2.0-or-later schema documentation, for local
/// reading only. `src/lib.rs` includes the module declarations written here in
/// place of the checked-in `src/generated` and `src/qga` trees.
#[cfg(feature = "schema-docs")]
mod schema_docs {
    use std::path::{Path, PathBuf};

    const QEMU_SRC: &str = "QRAB_QAPI_QEMU_SRC";

    fn fail(message: std::fmt::Arguments) -> ! {
        eprintln!("error: qapi feature `schema-docs`: {message}");
        std::process::exit(1);
    }

    pub fn generate() {
        println!("cargo:rerun-if-env-changed={QEMU_SRC}");
        let qemu = match std::env::var_os(QEMU_SRC) {
            Some(path) => PathBuf::from(path),
            None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/qemu"),
        };
        if !qemu.join("qapi").is_dir() {
            fail(format_args!(
                "no QEMU source tree at {}. Set {QEMU_SRC} to a QEMU git checkout that has \
                 the release tags pinned in projects/qapi-codegen/qemu-versions.lock (for \
                 example the initialized vendor/qemu submodule), or build without the feature.",
                qemu.display()
            ));
        }
        let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));
        use qapi_codegen::unified::{Docs, Protocol, Releases, generate};
        for (dir, protocol) in [("qmp", Protocol::Qmp), ("qga", Protocol::Qga)] {
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
        let path = |dir: &str| format!("{:?}", out.join(dir).join("mod.rs").display().to_string());
        let modules = format!(
            "#[rustfmt::skip]\n#[path = {}]\nmod generated;\n\n\
             /// QEMU Guest Agent (QGA) types.\n#[rustfmt::skip]\n#[path = {}]\npub mod qga;\n",
            path("qmp"),
            path("qga"),
        );
        std::fs::write(out.join("schema_docs_modules.rs"), modules)
            .unwrap_or_else(|error| fail(format_args!("writing module declarations: {error}")));
    }
}
