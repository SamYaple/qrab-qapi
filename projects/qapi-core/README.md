# qrab-qapi-core

The runtime of the QEMU Machine Protocol (QMP) and guest agent (QGA) types of
`qrab-qapi`: the traits of QAPI commands and events, and the version
contexts against which their serialization checks a peer's QEMU release.

`qrab-qapi` re-exports all of it at the same paths, such as
`qapi::QapiCommand` and `qapi::with_version`, and most code names it there.
Depend on this crate to use these items without the generated types, which
change with QEMU's releases, or to implement a command or a version-gated
value by hand.

The package is `qrab-qapi-core`; the library is `qapi_core`:

```toml
[dependencies]
qapi-core = { package = "qrab-qapi-core", version = "0.1" }
```

```rust
assert_eq!(qapi_core::parse_qga_version("11.1.0"), Ok((11, 1, 0)));
assert_eq!(qapi_core::with_version((11, 1, 3), qapi_core::qemu_version), (11, 1, 3));
```

## License

Apache-2.0.
