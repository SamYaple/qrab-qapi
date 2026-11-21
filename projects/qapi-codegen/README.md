# qrab-qapi-codegen

Parses QEMU's QAPI schema language for every pinned stable QEMU release,
lowers each release to a Rust IR, merges the releases into one registry in
which every type, field, variant and command flag carries its QEMU version
lifetime, and emits the QMP and QGA modules of `qrab-qapi`.

## License

Apache-2.0. QEMU's QAPI schemas are GPL-2.0-or-later; the generator reads
them only as input. Output generated with `--schema-docs` contains QEMU's
GPL-2.0-or-later documentation text and must not be published or shipped.
