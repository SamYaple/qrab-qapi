# Versioning

The crates follow [Semantic Versioning](https://semver.org/) as Cargo
applies it
([Cargo's SemVer guide](https://doc.rust-lang.org/cargo/reference/semver.html)):
before 1.0, in `0.y.z`, a new `y` is a breaking release and a new `z` a
compatible one. This page says what decides a release and its number, how
the crates require each other, and how releases are tagged.
[releases.md](releases.md) has the release procedure.

## Each crate has its own version

`qrab-qapi-codegen`, `qrab-qapi-macros`, `qrab-qapi-core`, `qrab-qapi` and
`qrab-qapi-protocol` are versioned and released separately. A crate gets a
release when its own public API or behaviour changes, and not otherwise: a
generator change that leaves the generated sources of `qrab-qapi` as they
are releases `qrab-qapi-codegen` alone, and a fix in the clients releases
`qrab-qapi-protocol` alone. The first release, 0.1.0 of the four crates that existed then, is the one
release they share.

Versions carry no build metadata such as `0.2.0+qemu.11.2.0`. The changelog
entry of each `qrab-qapi` release names its QEMU baseline instead: the
newest release pinned in `projects/qapi-codegen/qemu-versions.lock`, which
its sources were generated from.

## The bump rule

The public API decides, whatever caused the change: a new QEMU pin, a
generator or macro change, or a change to the runtime or the clients.

- **No change** to the public API or the behaviour: no release. The change
  waits for the next release that has one.
- **Compatible** changes are a patch release before 1.0, `0.y.z+1`: new
  types, commands, events, enum variants and optional fields, a new
  "deprecated since QEMU X" mark (Cargo counts a new `#[deprecated]` as a
  compatible change), new runtime or client API, and fixes. This rests on
  every generated struct and enum being `#[non_exhaustive]`, so that code
  outside the crate neither matches them exhaustively nor builds them with
  struct literals. A field that newer QEMU releases require is still an
  `Option` in Rust, so adding one is compatible too.
- **Breaking** changes are a minor release before 1.0, `0.y+1.0`:
  - a new "removed in QEMU X" mark, which a command, event, type, field or
    variant gets when the newest pinned release no longer has it. The item
    stays, for the peers that still have it, and code that uses it still
    compiles, but it no longer works against the new baseline;
  - a field, variant payload or return type that changes type, including a
    command whose single return type becomes a per-release enum;
  - anything removed, renamed or changed incompatibly in the runtime or the
    clients;
  - a raised minimum Rust version.

A field changes type when QEMU changes a member's shape. The generated code
keeps the old shape under a `_legacy` (or `Legacy`) name and gives the plain
name to the newest shape, which breaks code that reads the field as well as
code that writes it.

Enum values that a newer peer sends and the build does not know remain
decoding errors: `#[non_exhaustive]` changes nothing at run time.

## How the crates require each other

| From | To | Requirement | Why |
| --- | --- | --- | --- |
| `qrab-qapi` | `qrab-qapi-macros` | `=0.y.z`, exact | The expansion calls run-time items of `qrab-qapi` that are not public API (`qapi::__private`), and the generated sources use attribute keys that only some macro versions accept. |
| `qrab-qapi` | `qrab-qapi-codegen`, the optional build dependency of `schema-docs` | `=0.y.z`, exact | `schema-docs` must regenerate the same API with the generator that wrote the checked-in sources. |
| `qrab-qapi` | `qrab-qapi-core` | `0.y.z`, a caret on its minor | The runtime, which `qrab-qapi` re-exports at the same paths. The expansion uses only its public API. |
| `qrab-qapi-protocol` | `qrab-qapi-core` | `0.y.z`, a caret on its minor | Its API takes the runtime's traits and errors, not the generated types, below. |

The exact requirements follow serde, which requires its derive crate
exactly because the derived code uses serde items that SemVer does not
cover. Use the macros only through `qrab-qapi`'s re-export, `qapi::qapi`.

Cargo links at most one version of a crate in each compatible range, such
as `0.1.x`, into one build. Two `qrab-qapi` minors in one dependency graph
must therefore not require different versions of the same minor of a
tooling crate. A `qrab-qapi-macros` or `qrab-qapi-codegen` release that a
new `qrab-qapi` release adopts is a new minor of that crate, even for a
fix. A tooling change that no `qrab-qapi` release needs, such as a fix of
the generator's command line, or a parser fix that changes no output, is a
patch of that crate alone.

`qrab-qapi-protocol` depends on `qrab-qapi-core`, not on `qrab-qapi`.
`Qmp::execute` and `Qga::execute` take any `qapi_core::QapiCommand`, its
`Error` wraps `qapi_core::EncodeError`, and `RemoteError::known_class`
returns the protocol's own `ErrorClass`. Every `qrab-qapi` release from
0.1.1 on re-exports those items and implements the traits for its types,
so a breaking `qrab-qapi` release, such as a QEMU refresh with a removal,
needs no protocol release. Two `qrab-qapi` minors in one dependency graph
can even use the same protocol release. Only a breaking release of
`qrab-qapi-core` moves both: `qrab-qapi` and the protocol then each take a
minor that requires it.

## Examples

Starting from `qrab-qapi-core` 0.1.0, `qrab-qapi` 0.1.1, `qrab-qapi-protocol`
0.2.0 and the tooling crates at 0.1.0, on QEMU 11.1.3:

| Change | Releases |
| --- | --- |
| QEMU 11.1.4 is pinned, and the regenerated sources are identical, the usual case for a QEMU stable release | None. The pin waits in the repository, and the next `qrab-qapi` release names 11.1.4 in its changelog. |
| A QEMU release adds types, variants and optional fields, and deprecates some | `qrab-qapi` 0.1.2 alone. |
| A QEMU release no longer has a command that 11.1.3 has, so the command is marked "removed in QEMU X" | `qrab-qapi` 0.2.0 alone; the protocol works with it unchanged. |
| A QEMU release changes a field's type, as 11.0.0 changed `BlockExportAdd.iothread` from `Option<String>` to `Option<BlockExportIothreads>` | `qrab-qapi` 0.2.0 alone. |
| A parser fix that changes no generated output | `qrab-qapi-codegen` 0.1.1 alone. |
| A macro fix, such as a wrong serialization in the expansion | `qrab-qapi-macros` 0.2.0, then `qrab-qapi` 0.1.2 requiring `=0.2.0` of it, or 0.2.0 if the fix changes the API of `qrab-qapi` incompatibly. |
| A bug fix in the clients | `qrab-qapi-protocol` 0.2.1 alone. |
| A breaking change of the runtime, such as a new item of `QapiCommand` without a default | `qrab-qapi-core` 0.2.0, then a `qrab-qapi` minor and a `qrab-qapi-protocol` minor that require it. |

## Minimum Rust version

All the crates declare one minimum Rust version, `rust-version` in the
root `Cargo.toml`, currently 1.88, which the `msrv` CI job checks. It is
raised only in a breaking release of each crate it affects, bundled with
other breaking changes where possible, and only to a stable Rust at least
six months old.

## Backports

Fixes are not backported to older minors, except security fixes, and those
only into the previous minor.

## Classifying and tagging a release

Before tagging, compare each crate with its last release. The `semver` CI
job, [`contrib/check-semver.sh`](../contrib/check-semver.sh), runs
cargo-semver-checks on each library crate against that crate's last
release tag and fails a breaking change without the minor bump. It cannot
read `qrab-qapi-macros`, whose changes reach users through the API of
`qrab-qapi`, and it misses what only the generated sources show: a field
that changes type, and a new "removed in QEMU X" mark. Review the diff of
`projects/qapi/src/generated` and `projects/qapi/src/qga` since the last
release for those, and review behaviour, which neither tool sees.

Every release is an SSH-signed tag on `main`. The first release, 0.1.0 of
all four crates, is the tag `v0.1.0`. After it, each crate that a release
publishes gets a tag of its own, `<package>-v<version>`, for example
`qrab-qapi-codegen-v0.1.1`.
