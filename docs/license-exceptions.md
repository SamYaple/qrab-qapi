# Third-party license exceptions

qrab-qapi's code is Apache-2.0. Third-party components must be licensed under
the [CNCF allowed third-party license list][policy] or covered by an approved
CNCF exception ([exceptions.cncf.io](https://exceptions.cncf.io); the source
data is `license-exceptions/exceptions.json` in [cncf/foundation][foundation]).

`cargo deny --locked check licenses` enforces this using
[`deny.toml`](../deny.toml): its `allow` list is the CNCF list, and every crate
outside it has a per-crate exception that names one of the requests below. CI
runs the check in the `cargo-deny` job.

None of the licenses below has an approved CNCF record that covers this use, so
each has a request ready to file with the [license exception request
template][template] in cncf/foundation, one issue per license. When one is
approved, replace the `pending:` comment in `deny.toml` with the exception id
and the issue URL.

Every crate is unmodified and fetched from crates.io at build time; no
third-party Rust source is stored in the repository.

[policy]: https://github.com/cncf/foundation/blob/main/policies-guidance/allowed-third-party-license-policy.md
[foundation]: https://github.com/cncf/foundation/tree/main/license-exceptions
[template]: https://github.com/cncf/foundation/issues/new?template=license-exception-request.yaml

## Unicode-3.0

**Request: blanket exception for Unicode-3.0, or exception for qrab-qapi.**

| Component | Upstream URL | License |
| --- | --- | --- |
| unicode-ident | https://github.com/dtolnay/unicode-ident | (MIT OR Apache-2.0) AND Unicode-3.0 |

- **How qrab-qapi uses it:** unicode-ident supplies the Unicode identifier
  tables for `proc-macro2`, which procedural macros such as the `thiserror`
  derive use at compile time.
- **Runtime use:** the qrab-qapi-codegen library also links
  `proc-macro2` and `syn`, which build and parse the Rust it generates,
  so programs that generate code with it contain unicode-ident.
  The other crates use it only at compile time.
- **Modified:** no.
- **Distribution and integration model:** User-Fetched Dependency. The project
  publishes source crates; Cargo fetches unicode-ident when they are built.
- **Structural separation:** Separated Component (a separate crate fetched at
  build time). **Communication:** Static Linking. **Data exchange:** Tightly
  Coupled (Rust library calls).
- **Notes for reviewers:** Unicode-3.0 is the OSI-approved successor of
  Unicode-DFS-2016, which the CNCF already approved for all projects
  (exc-2026-02-18-001, [cncf/foundation#1242][textseg]). It is a permissive
  notice-only license. Other CNCF projects have requested the same exception
  for unicode-ident ([cncf/foundation#1518][openshell],
  [cncf/foundation#1581][wasmcloud]).

[textseg]: https://github.com/cncf/foundation/issues/1242
[openshell]: https://github.com/cncf/foundation/issues/1518
[wasmcloud]: https://github.com/cncf/foundation/issues/1581
