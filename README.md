# qrab-qapi

Rust types and async clients for QEMU's QAPI protocols: the QEMU Machine
Protocol (QMP) and the QEMU Guest Agent protocol (QGA). The types are
generated from QEMU's own QAPI schemas across every stable release, so each
command, event, type, field and variant carries the QEMU version that
introduced, changed or removed it, and serialization checks those lifetimes
against the version of the peer you are talking to.

## The crates

| Package | Library | Contents |
| --- | --- | --- |
| [`qrab-qapi`](projects/qapi/README.md) | `qapi` | Generated QMP and QGA types plus the `QapiCommand`, `QapiEvent` and `PreconfigSafe` traits |
| [`qrab-qapi-protocol`](projects/qapi-protocol/README.md) | `qapi_protocol` | Async Unix-socket QMP and QGA clients on Tokio, plus bounded mock servers for tests |
| [`qrab-qapi-macros`](projects/qapi-macros/README.md) | `qapi_macros` | The `#[qapi(...)]` attribute: version-aware Serde and trait implementations |
| [`qrab-qapi-codegen`](projects/qapi-codegen/README.md) | `qapi_codegen` | Parses QEMU's QAPI schema language and generates the unified, version-annotated Rust types |

## Relationship to qrab

These crates were developed as part of
[qrab](https://github.com/qrab-dev/qrab), a Kubernetes platform for QEMU
virtual machines, which uses them to drive QEMU and its guest agent. They do
not depend on qrab, Kubernetes or any particular way of launching QEMU, and
are maintained as a general-purpose library. qrab-qapi is a subproject of
qrab and is governed by the qrab project; see [GOVERNANCE.md](GOVERNANCE.md).
The project site is <https://qrab.dev>.

## Contributing

Contributions are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) covers the
checks, the pull request flow, signed commits and the Developer Certificate
of Origin sign-off. Everyone who takes part follows the
[Code of Conduct](CODE_OF_CONDUCT.md).

## License

The crates are licensed under the Apache License, Version 2.0
([LICENSE](LICENSE)). The documentation is licensed under CC-BY-4.0
([LICENSE-docs](LICENSE-docs)). [NOTICE](NOTICE) describes the third-party
QEMU material and the `schema-docs` restriction.
