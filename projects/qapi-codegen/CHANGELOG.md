# Changelog

All notable changes to `qrab-qapi-codegen` are recorded here, newest first.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the version numbers follow the policy in the repository's
`docs/versioning.md`.

## 0.1.0

The first release.

- A parser of QEMU's QAPI schema language for every stable QEMU release
  pinned in the embedded `qemu-versions.lock`, from 0.15.0 through 11.1.3,
  following the grammar of each release.
- The generator: it lowers each release to a Rust IR, merges the releases
  into one registry in which every type, field, variant and command flag
  carries its QEMU version lifetime, and writes the QMP and QGA modules of
  `qrab-qapi`. The library's API is `unified::generate`, with the release
  pins of `versions` and the errors they report.
- The `qapi-codegen` command line: `--qga` for the guest agent schema,
  `--versions-file` for another set of releases and `--schema-docs` for
  QEMU's schema documentation as rustdoc, whose output must not be
  published.
