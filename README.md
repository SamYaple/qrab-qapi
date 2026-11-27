# qrab-qapi

Rust types and async clients for QEMU's QAPI protocols: the QEMU Machine
Protocol (QMP) and the QEMU Guest Agent protocol (QGA). The types are
generated from QEMU's own QAPI schemas across every stable release, so each
command, event, type, field and variant carries the QEMU version that
introduced, changed or removed it, and serialization checks those lifetimes
against the version of the peer you are talking to.

## The crates

| Package | Library | Contents |
| --- | --- | --- |
| [`qrab-qapi`](projects/qapi/README.md) | `qapi` | Generated QMP and QGA types, with the runtime of `qrab-qapi-core` re-exported |
| [`qrab-qapi-core`](projects/qapi-core/README.md) | `qapi_core` | The runtime of the types: the `QapiCommand`, `QapiEvent` and `PreconfigSafe` traits and the version contexts |
| [`qrab-qapi-protocol`](projects/qapi-protocol/README.md) | `qapi_protocol` | Async Unix-socket QMP and QGA clients on Tokio, plus bounded mock servers for tests |
| [`qrab-qapi-macros`](projects/qapi-macros/README.md) | `qapi_macros` | The `#[qapi(...)]` attribute: version-aware Serde and trait implementations |
| [`qrab-qapi-codegen`](projects/qapi-codegen/README.md) | `qapi_codegen` | Parses QEMU's QAPI schema language and generates the unified, version-annotated Rust types |

How they fit together:

- `qrab-qapi-codegen` reads the QAPI schemas of every pinned QEMU release,
  merges them into one version-annotated registry and writes the Rust
  sources of `qrab-qapi`. Those sources are checked in, so ordinary builds
  neither run the generator nor need a QEMU source tree.
- `qrab-qapi` contains the generated types. Each one carries a
  `#[qapi(...)]` attribute that `qrab-qapi-macros` expands into Serde and
  trait implementations that check QEMU version lifetimes. `qrab-qapi`
  re-exports the attribute, so you do not depend on the macros crate
  directly.
- `qrab-qapi-core` holds the runtime of those types: the traits of
  commands and events, and the version contexts that their serialization
  checks. `qrab-qapi` re-exports it at the same paths, such as
  `qapi::QapiCommand`.
- `qrab-qapi-protocol` connects to a QEMU monitor (QMP) socket, learns the
  peer's QEMU version during the handshake, and sends and decodes
  `qrab-qapi` types checked against that version.
  Its guest agent (QGA) client does the same with the agent's own version.

Most users want `qrab-qapi-protocol` together with `qrab-qapi`.

## QEMU version support

- **Schema history.** The generated types cover every stable QEMU release
  pinned in
  [`projects/qapi-codegen/qemu-versions.lock`](projects/qapi-codegen/qemu-versions.lock):
  QMP from QEMU 1.0, where its schema first appears, and QGA from 0.15,
  through the current baseline, **QEMU 11.1.3**.
- **Maintained matrix.** The release series **9.2, 10.0, 10.1, 10.2, 11.0
  and 11.1** are maintained: the generator's tests compare the latest pinned
  patch of each series with that release's own QAPI parser.
- **Protocol clients.** `qrab-qapi-protocol` targets the QEMU 11.1.3
  protocol. Each connection checks requests and replies against its own
  peer's version.
  A guest agent's version can differ from QEMU's.
  The guest agent client needs qemu-ga 1.1 or later.
- **Older QEMU** is untested, not unsupported. Historical schemas stay
  generation inputs and regression fixtures, but schema coverage does not
  mean every release has been tested against a live QEMU.

Every build has the complete history, and the clients refuse at run time
what a peer's release lacks. What the newest pinned release removed or
deprecates is Rust `#[deprecated]`.

[docs/codegen.md](docs/codegen.md#compatibility-policy) has the full
compatibility policy and [docs/protocol.md](docs/protocol.md) the client
behaviour.

## Usage

The packages carry a `qrab-` prefix, while their library names stay `qapi`,
`qapi_protocol`, `qapi_macros` and `qapi_codegen`: code imports `qapi::...`
from a dependency on `qrab-qapi`. These snippets also rename the
dependencies to the library names, so that the `[dependencies]` keys match:

```toml
[dependencies]
qapi = { package = "qrab-qapi", version = "0.1" }
qapi-protocol = { package = "qrab-qapi-protocol", version = "0.1" }
```

Query a running QEMU over its QMP socket, inside an async function:

```rust
let (qmp, _events) = qapi_protocol::Qmp::new(std::path::Path::new("/run/vm/qmp.sock")).await?;
let status = qmp.execute(&qapi::commands::QueryStatus::default()).await?;
println!("running: {}", status.running);
qmp.shutdown().await;
```

Each crate's README has more:

- [qrab-qapi](projects/qapi/README.md): the generated types, their QEMU
  lifetimes and the `schema-docs` feature;
- [qrab-qapi-protocol](projects/qapi-protocol/README.md): connecting to
  QEMU;
- [qrab-qapi-macros](projects/qapi-macros/README.md): the `#[qapi(...)]`
  attribute;
- [qrab-qapi-codegen](projects/qapi-codegen/README.md): the generator and its
  command line.

## QEMU's schema documentation is GPL

QEMU's QAPI schema files, including their documentation comments, are
licensed GPL-2.0-or-later. The generator reads them only as input. The
checked-in generated code keeps the facts needed for the wire protocol
(names, types and version information) and no schema prose; each generated
item points to QEMU's own QMP or guest agent reference instead.

The generator's `--schema-docs` flag, and the `qrab-qapi` build with both the
opt-in `schema-docs` feature and the `QRAB_QAPI_SCHEMA_DOCS` environment
variable, render QEMU's schema prose as rustdoc so you can read the full
documentation locally. The feature only compiles the generator into the build
script; the variable, the absolute path of a QEMU git tree with the pinned
release tags, switches generation on. A dependency can enable the feature but
cannot set the variable, so `--all-features`, docs.rs and dependents that
enable the feature get the ordinary, prose-free build; so does the variable
without the feature. Either half alone makes cargo print a warning, for local
packages only. Build the documentation into a target directory of its own, so
that the GPL-2.0-or-later text never mixes with the documentation in
`target/doc`, which may be published.
[`contrib/schema-docs.sh`](contrib/schema-docs.sh) sets all three:

```sh
contrib/schema-docs.sh vendor/qemu --open
# or by hand:
QRAB_QAPI_SCHEMA_DOCS="$PWD/vendor/qemu" \
  cargo doc -p qrab-qapi --features schema-docs --target-dir target/schema-docs --open
```

**Output built with `schema-docs` and `QRAB_QAPI_SCHEMA_DOCS`, or with
`--schema-docs`, includes QEMU's GPL-2.0-or-later documentation text. It is
for local reading only: do not publish or ship it.** That includes generated
sources, built crates and binaries, and rendered documentation, whose front
page says so. See [NOTICE](NOTICE).

## Building, testing and regenerating

You need a Rust toolchain from [rustup](https://rustup.rs/), Rust 1.88 or
newer; CI uses a pinned stable toolchain, currently 1.97.1. The default
workspace members (`qrab-qapi-core`, `qrab-qapi-macros`, `qrab-qapi` and
`qrab-qapi-protocol`)
build from the checked-in generated sources and need no QEMU checkout,
Python, KVM, root privileges or network access:

```sh
cargo test --locked
```

The generated sources under `projects/qapi/src/generated` (QMP) and
`projects/qapi/src/qga` (QGA) are outputs of `qrab-qapi-codegen`. Generator
work and regeneration need the QEMU source as the `vendor/qemu` git
submodule, which points at the QEMU repository URL in
[.gitmodules](.gitmodules), with the release tags pinned in
`projects/qapi-codegen/qemu-versions.lock`. Regeneration also needs Git, and
the generator's tests, which compare with QEMU's own parser, Python 3.14 or
newer:

```sh
git submodule update --init vendor/qemu
git -C vendor/qemu fetch origin --tags
bash contrib/check-qapi.sh
```

[`contrib/check-qapi.sh`](contrib/check-qapi.sh) runs strict Clippy on all
five crates, and on `qrab-qapi-protocol` without its default features, tests
the generator against QEMU's own parser and tests the other crates,
regenerates QMP and QGA and compares the output byte for byte with the
checked-in sources, and builds the API documentation with warnings treated
as errors. The submodule is used only by
the generator, its tests and the `schema-docs` build; the published crates
do not include it.

Tests against real QEMU binaries are explicit and opt-in.
[docs/testing.md](docs/testing.md) describes them, and
[docs/codegen.md](docs/codegen.md) describes the generator.
[CONTRIBUTING.md](CONTRIBUTING.md) lists every check CI runs.

## Relationship to qrab

These crates were developed as part of
[qrab](https://github.com/qrab-dev/qrab), a Kubernetes platform for QEMU
virtual machines, which uses them to drive QEMU and its guest agent. They do
not depend on qrab, Kubernetes or any particular way of launching QEMU, and
are maintained as a general-purpose library. qrab-qapi is a subproject of
qrab and is governed by the qrab project; see [GOVERNANCE.md](GOVERNANCE.md).
The project site is <https://qrab.dev>.

## Status

qrab-qapi is pre-1.0. The APIs may change between minor releases, and each
crate is versioned separately: [docs/versioning.md](docs/versioning.md) has
the policy, [docs/releases.md](docs/releases.md) the release procedure, and
each crate's `CHANGELOG.md` its releases.
Report bugs and propose features in this repository's GitHub issues, and
security problems privately as described in [SECURITY.md](SECURITY.md).

## Contributing

Contributions are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) covers the
checks, the pull request flow, signed commits and the Developer Certificate
of Origin sign-off. Everyone who takes part follows the
[Code of Conduct](CODE_OF_CONDUCT.md).

## License

The crates are licensed under the Apache License, Version 2.0
([LICENSE](LICENSE)). The documentation is licensed under CC-BY-4.0
([LICENSE-docs](LICENSE-docs)). [NOTICE](NOTICE) describes the third-party
QEMU material and the `schema-docs` restriction.
