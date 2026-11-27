# Changelog

All notable changes to `qrab-qapi` are recorded here, newest first. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the version numbers follow the policy in the repository's
`docs/versioning.md`. Each entry names the QEMU baseline, the newest QEMU
release that its types are generated from.

## 0.1.1

QEMU baseline: 11.1.3.

- The runtime moves to the new crate `qrab-qapi-core`: the traits
  `QapiCommand`, `PreconfigSafe` and `QapiEvent`, the version contexts and
  `parse_qga_version`. This crate requires it and re-exports every item at
  the same path, so code that names them through `qapi::` does not change.

## 0.1.0

The first release. QEMU baseline: 11.1.3.

- QMP and guest agent (QGA) types generated from the QAPI schemas of every
  stable QEMU release that `qrab-qapi-codegen` pins: QMP from QEMU 1.0, where
  its schema first appears, and QGA from 0.15, through 11.1.3. Every command,
  event, type, field and variant records the releases that have it, and
  serialization for a peer refuses what the peer's release lacks.
- Rust `#[deprecated]` follows the baseline: what QEMU 11.1.3 no longer has
  is marked "removed in QEMU X", and what it deprecates "deprecated since
  QEMU X".
- Every generated struct and enum is `#[non_exhaustive]`. A struct whose
  fields are all `Option`, or that has none, implements `Default`; any other
  struct has `new`, which takes its non-`Option` fields in declaration order.
- The `QapiCommand`, `QapiEvent`, `PreconfigSafe` and `QapiFlatUnion` traits,
  and the version contexts of serialization: `with_version`, `to_value` with
  its `EncodeError`, `set_qemu_version` and `parse_qga_version`.
- The opt-in `schema-docs` feature: with `QRAB_QAPI_SCHEMA_DOCS` set as well,
  the build regenerates the types with QEMU's GPL-2.0-or-later schema
  documentation, for local reading only.
