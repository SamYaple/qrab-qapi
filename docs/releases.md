# Releases

A maintainer publishes the crates to crates.io from a commit on `main`, by
hand, with the steps below. [versioning.md](versioning.md) says which
crates a change releases, their version numbers and their tags.

crates.io's Trusted Publishing cannot create a crate, so the first release,
0.1.0 of all four crates, is published with a crates.io API token. Only the
maintainer holds the token, and only in the shell that publishes.

## 1. Prepare the release commit

On `main`, through a pull request like any other change:

- the new `version` in the `Cargo.toml` of each crate that the release
  publishes;
- when `qrab-qapi-macros` or `qrab-qapi-codegen` is released, the exact
  requirements on them in the root `Cargo.toml`;
- an entry for the new version at the top of each released crate's
  `CHANGELOG.md`, which for `qrab-qapi` names the QEMU baseline.

## 2. Check a clean checkout

The release commit, C below, must have passed CI on `main`. In a fresh
clone:

```sh
git clone https://github.com/qrab-dev/qrab-qapi && cd qrab-qapi
git checkout --detach C
git status --porcelain     # must print nothing
cargo publish --dry-run --locked \
  -p qrab-qapi-codegen -p qrab-qapi-macros -p qrab-qapi -p qrab-qapi-protocol
```

The dry run packages each crate and builds the package the way crates.io
receives it, without a token. Name every crate that the release publishes
in the one command: Cargo then resolves the crates that depend on each
other among themselves, even before they are on crates.io, and handles them
in dependency order. For a later release, name only the crates it
publishes.

## 3. Sign the tag

Sign the tag on C with the maintainer's SSH signing key, and check the
signature, but do not push the tag yet:

```sh
git -c gpg.format=ssh -c user.signingkey=<key> tag -s v0.1.0 -m "qrab-qapi 0.1.0" C
git -c gpg.ssh.allowedSignersFile=<allowed signers> tag -v v0.1.0
```

## 4. Publish

```sh
read -rs CARGO_REGISTRY_TOKEN && export CARGO_REGISTRY_TOKEN
cargo publish --locked \
  -p qrab-qapi-codegen -p qrab-qapi-macros -p qrab-qapi -p qrab-qapi-protocol
unset CARGO_REGISTRY_TOKEN
```

The token reaches Cargo through this shell's environment only, never on a
command line or through `cargo login`. Cargo uploads the crates in
dependency order and waits for each one to reach the index before the next
that depends on it. A published version can be yanked but never replaced:
if the command stops partway, fix the cause and run it again, naming only
the crates that are not published yet.

## 5. Push the tag and check

```sh
git push origin v0.1.0
```

Then check each crate's page on crates.io and its documentation build on
docs.rs, and build a scratch crate against the released versions:

```sh
cargo new --lib release-check && cd release-check
cargo add qrab-qapi@0.1.0 qrab-qapi-protocol@0.1.0
cargo check
```
