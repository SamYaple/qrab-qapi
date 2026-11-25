# qrab-qapi-codegen

Parses QEMU's QAPI schema language for every pinned stable QEMU release,
lowers each release to a Rust IR, merges the releases into one registry in
which every type, field, variant and command flag carries its QEMU version
lifetime, and emits the QMP and QGA modules of `qrab-qapi`.

The package is `qrab-qapi-codegen`; the library is `qapi_codegen` and the
binary `qapi-codegen`. Given a QEMU git tree with the release tags of the
embedded `qemu-versions.lock`, it writes `mod.rs`, `commands.rs`,
`events.rs` and `metadata.rs`:

```sh
qapi-codegen path/to/qemu --output-dir out/qmp
qapi-codegen path/to/qemu --qga --output-dir out/qga
```

`--versions-file` selects another set of release tags and commits, and
`--schema-docs` renders QEMU's schema documentation as rustdoc;
`qapi-codegen --help` lists the options. Nothing is written unless every
pinned release generates Rust that parses. The quirks of old schemas that
the generator tolerates, with a warning each, are in `KNOWN_ISSUES.md`.

## License

Apache-2.0. QEMU's QAPI schemas are GPL-2.0-or-later; the generator reads
them only as input. Output generated with `--schema-docs` contains QEMU's
GPL-2.0-or-later documentation text and must not be published or shipped.
