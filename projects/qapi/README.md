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
let request = qapi::to_value((11, 1, 0), &QueryStatus::default()).unwrap();
assert_eq!(request, serde_json::json!({}));
```

## Building values

Every generated struct and enum is `#[non_exhaustive]`, so that a new QEMU
release can add fields and variants in a compatible release of this crate.
Outside the crate, a struct is therefore built with a constructor rather
than a struct literal, and a `match` on an enum has a wildcard arm:

- A struct whose fields are all `Option`, or that has none, implements
  `Default`, with every field `None`; set the fields you need afterwards.
- Any other struct has `new`, which takes its non-`Option` fields in their
  declaration order (the order of the struct's documentation) and leaves the
  optional ones `None`.
- Enum variants, union branches included, are constructed as usual.

```rust
use qapi::commands::DeviceAdd;

let mut device = DeviceAdd::new("virtio-rng-pci".into());
device.id = Some("rng0".into());
assert_eq!(
    qapi::to_value((11, 1, 0), &device).unwrap(),
    serde_json::json!({"driver": "virtio-rng-pci", "id": "rng0"}),
);

fn describe(state: &qapi::RunState) -> &'static str {
    match state {
        qapi::RunState::Running => "running",
        qapi::RunState::Paused => "paused",
        _ => "another state",
    }
}
assert_eq!(describe(&qapi::RunState::Paused), "paused");
```

A struct field that newer QEMU releases require is still an `Option`, so
`Default` leaves it `None`: serialization for such a peer reports the
missing field as an error.

`qrab-qapi-protocol` provides async QMP and QGA clients for these types.

## Version bounds

Set `QEMU_MIN_VERSION` and `QEMU_MAX_VERSION` (for example `9.2.0` and
`11.1.3`) at build time to compile only the representations that exist in
that range of QEMU releases. Without them the complete history is available.

## The `schema-docs` feature

The generated sources carry no QEMU schema documentation. To read the full
documentation locally, enable the opt-in `schema-docs` feature and set
`QRAB_QAPI_SCHEMA_DOCS` to the absolute path of a QEMU git tree with the
pinned release tags: the build then regenerates the modules with QEMU's schema
prose as rustdoc. The feature alone only compiles the generator and builds the
checked-in sources, with a warning, so it is safe under `--all-features` and
for a dependent that enables it. The variable alone does nothing; cargo shows
the build script's warning about it only for local packages, not for
registry dependencies.

## License

Apache-2.0. QEMU's QAPI schemas and their documentation comments are
GPL-2.0-or-later; the generated code keeps only the names, types and version
facts needed for the wire protocol. Output built with the `schema-docs`
feature and `QRAB_QAPI_SCHEMA_DOCS` includes QEMU's GPL-2.0-or-later
documentation text: it is for local reading, so do not publish or ship
artifacts built with it.
