# Changelog

All notable changes to `qrab-qapi-core` are recorded here, newest first.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the version numbers follow the policy in the repository's
`docs/versioning.md`.

## Unreleased

The first release, with the runtime that `qrab-qapi` 0.1.0 held itself.
`qrab-qapi` re-exports it at the same paths.

- The traits `QapiCommand`, `PreconfigSafe` and `QapiEvent`.
- The version contexts: `with_version`, `set_qemu_version`,
  `scoped_qemu_version`, `qemu_version`, and `to_value` with its
  `EncodeError`.
- `unavailable_error` and `unavailable_type_message`, which a hand-written
  `Serialize` or `Deserialize` of a version-gated value reports. In
  `qrab-qapi` 0.1.0 they were hidden helpers of its `#[qapi]` attribute.
- `parse_qga_version` and its `VersionParseError`.
