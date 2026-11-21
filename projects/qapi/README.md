# qrab-qapi

Rust types for QEMU's QAPI protocols: the QEMU Machine Protocol (QMP) and the
QEMU Guest Agent protocol (QGA). The types are generated from QEMU's own QAPI
schemas across every stable release, so each command, event, type, field and
variant records the QEMU version that introduced, deprecated or removed it.
Serialization checks those lifetimes against the peer's version.

## License

Apache-2.0. QEMU's QAPI schemas and their documentation comments are
GPL-2.0-or-later; the generated code keeps only the names, types and version
facts needed for the wire protocol. Output built with the `schema-docs`
feature includes QEMU's GPL-2.0-or-later documentation text: it is for local
reading, so do not publish or ship artifacts built with it.
