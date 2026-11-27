# Changelog

All notable changes to `qrab-qapi-macros` are recorded here, newest first.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the version numbers follow the policy in the repository's
`docs/versioning.md`.

## 0.1.0

The first release.

- The `#[qapi(...)]` attribute of the types that `qrab-qapi-codegen`
  generates for `qrab-qapi`. It expands string enums, alternates, the enums
  of union branches, structs with their flattened bases and unions, and
  command and event types into Serde implementations that check the QEMU
  version lifetime of each type and member against the peer's version, and
  implements `QapiCommand` and `QapiEvent` for commands and events.
- A type or member that the newest pinned release no longer has, or
  deprecates, gets Rust `#[deprecated]` with a note naming the release.
