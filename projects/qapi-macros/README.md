# qrab-qapi-macros

The `#[qapi(...)]` attribute macro behind the generated types of `qrab-qapi`.
Its expansion implements Serde for QAPI types as `qrab-qapi-codegen`
generates them, checking the QEMU version lifetime of each type and member
against the peer's version. The crate documentation lists the kinds of
items that it expands.

The package is `qrab-qapi-macros`; the library is `qapi_macros`. The
expansion refers to items of `qrab-qapi` by the absolute path `::qapi`, so
it builds only where the crate `qapi` is that library, as in a crate that
depends on `qrab-qapi` under its library name.

## License

Apache-2.0.
