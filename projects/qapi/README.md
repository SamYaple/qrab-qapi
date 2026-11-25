# qrab-qapi

Rust types for QEMU's QAPI protocols: the QEMU Machine Protocol (QMP) and the
QEMU Guest Agent protocol (QGA). The types are generated from QEMU's own QAPI
schemas across every stable release, so each command, event, type, field and
variant records the QEMU version that introduced, deprecated or removed it.
Serialization checks those lifetimes against the peer's version.

The package is `qrab-qapi`; the library is `qapi`:

```toml
[dependencies]
qapi = { package = "qrab-qapi", version = "0.1" }
```

```rust
use qapi::QapiCommand;
use qapi::commands::QueryStatus;

assert_eq!(QueryStatus::NAME, "query-status");
let request = qapi::to_value((11, 1, 0), &QueryStatus {}).unwrap();
assert_eq!(request, serde_json::json!({}));
```

`qrab-qapi-protocol` provides async QMP and QGA clients for these types.

## Version bounds

Set `QEMU_MIN_VERSION` and `QEMU_MAX_VERSION` (for example `9.2.0` and
`11.1.3`) at build time to compile only the representations that exist in
that range of QEMU releases. Without them the complete history is available.

## The `schema-docs` feature

The generated sources carry no QEMU schema documentation. The opt-in
`schema-docs` feature regenerates them during the build with QEMU's schema
prose as rustdoc, for reading the full documentation locally. It needs a QEMU
git tree with the pinned release tags, named by `QRAB_QAPI_QEMU_SRC`.

## License

Apache-2.0. QEMU's QAPI schemas and their documentation comments are
GPL-2.0-or-later; the generated code keeps only the names, types and version
facts needed for the wire protocol. Output built with the `schema-docs`
feature includes QEMU's GPL-2.0-or-later documentation text: it is for local
reading, so do not publish or ship artifacts built with it.
