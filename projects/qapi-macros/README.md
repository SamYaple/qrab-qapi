# qrab-qapi-macros

The `#[qapi(...)]` attribute macro behind the generated types of `qrab-qapi`.
It expands QAPI structs, enums, alternates and unions into Serde
implementations that check each type's, field's and variant's QEMU version
lifetime against the peer version, and into the `QapiCommand`, `QapiEvent`
and union-branch trait implementations.

The package is `qrab-qapi-macros`; the library is `qapi_macros`. Depend on
`qrab-qapi`, which re-exports the attribute as `qapi::qapi`; the expansion
refers to items of the `qapi` crate.

## License

Apache-2.0.
