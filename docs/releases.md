# Releases

A maintainer publishes the crates to crates.io from a commit on `main`, by
hand, with the steps below. [versioning.md](versioning.md) says which
crates a change releases, their version numbers and their tags.

The crates are published with a crates.io API token. crates.io's Trusted
Publishing cannot create a crate, so at least a crate's first release
needs one. Only the maintainer holds the token, and only in the shell
that publishes. Create a token for each release, with a short expiry,
and revoke it afterwards. It needs both publish scopes, `publish-new` for
a crate's first version and `publish-update` for a new version of a
crate on crates.io, and is limited to the crates `qrab-qapi*`, a pattern
that covers `qrab-qapi` itself, unlike `qrab-qapi-*`. crates.io refuses
an upload that the token does not cover with 403, "this token does not
have the required permissions to perform this action".

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
  -p qrab-qapi-core -p qrab-qapi-codegen -p qrab-qapi-macros -p qrab-qapi -p qrab-qapi-protocol
```

The dry run packages each crate and builds the package the way crates.io
receives it, without a token. Name every crate that the release publishes
in the one command: Cargo then resolves the crates that depend on each
other among themselves, even before they are on crates.io, and handles them
in dependency order. For a later release, name only the crates it
publishes. Dev-dependencies count too: `qrab-qapi-protocol` builds
without `qrab-qapi`, but its tests use it, so Cargo also needs the
`qrab-qapi` version that they require, from crates.io or from the same
command.

## 3. Sign the tags

Sign a tag on C for each crate that the release publishes,
`<package>-v<version>`, with the maintainer's SSH signing key under the
public name and address of their commits, and check it, but do not push
the tags yet. Give the identity on the command line: otherwise the tag
records the clone's own `user.name` and `user.email`, and a published tag
can only be replaced by relaxing the tag rules. For example, for
`qrab-qapi` 0.1.1:

```sh
git -c user.name=<name> -c user.email=<public address> \
    -c gpg.format=ssh -c user.signingkey=<key> \
    tag -s qrab-qapi-v0.1.1 -m "qrab-qapi 0.1.1" C
git cat-file tag qrab-qapi-v0.1.1 | grep '^tagger'
git -c gpg.ssh.allowedSignersFile=<allowed signers> tag -v qrab-qapi-v0.1.1
```

## 4. Publish

```sh
read -rs CARGO_REGISTRY_TOKEN && export CARGO_REGISTRY_TOKEN
cargo publish --locked \
  -p qrab-qapi-core -p qrab-qapi-codegen -p qrab-qapi-macros -p qrab-qapi -p qrab-qapi-protocol
unset CARGO_REGISTRY_TOKEN
```

The token reaches Cargo through this shell's environment only, never on a
command line or through `cargo login`. Cargo uploads the crates in
dependency order and waits for each one to reach the index before the next
that depends on it. A published version can be yanked but never replaced:
if the command stops partway, fix the cause and run it again, naming every
crate that is not published yet, so that none waits for another that the
command leaves out.

## 5. Push the tags and check

```sh
git push origin <tag>...
```

Then check each crate's page on crates.io and its documentation build on
docs.rs, and build a scratch crate against the released versions:

```sh
cargo new --lib release-check && cd release-check
cargo add qrab-qapi@<version> qrab-qapi-protocol@<version>
cargo check
```
