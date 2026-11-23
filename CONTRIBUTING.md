# Contributing to qrab-qapi

Thank you for your interest in qrab-qapi. This guide covers the checks to
run, the rules for generated code, the pull request flow, signed commits and
the Developer Certificate of Origin (DCO) sign-off. qrab-qapi is a subproject of
[qrab](https://github.com/qrab-dev/qrab) and follows its governance and
contribution policies; this file adapts them to these crates. Everyone who
takes part follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Reporting bugs and requesting features

Use this repository's GitHub issues and pick the bug report or feature
request template.

- For a bug, include the crate and its version or commit, the QEMU (or
  `qemu-ga`) version of the peer, what you did, what you expected and what
  happened. A minimal program or a QMP/QGA transcript helps most. Remove
  secrets, tokens and private addresses first.
- For a feature, describe the problem you want solved before the solution
  you have in mind. For a larger change, such as a new public API or a change
  to the generated representation, open an issue to agree on the design
  before writing the code.

**Do not report security vulnerabilities in public issues.** Follow
[SECURITY.md](SECURITY.md).

## Build and test

You need:

- A Rust toolchain from [rustup](https://rustup.rs/), Rust 1.88 or newer,
  with the `rustfmt` and `clippy` components. CI pins a stable toolchain,
  currently 1.97.1, in [.github/workflows/ci.yml](.github/workflows/ci.yml).
- Optionally [cargo-deny](https://github.com/EmbarkStudios/cargo-deny), for
  the dependency license check.
- For generator work: Git, Python 3.14 or newer, and the `vendor/qemu`
  submodule with its release tags (see below).

The default workspace members (`qrab-qapi-macros`, `qrab-qapi` and
`qrab-qapi-protocol`) build from the checked-in generated sources and need
no QEMU source, Python, KVM, root privileges or network access. From the
repository root, these are the checks CI runs:

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --no-deps -- -D warnings
cargo test --locked
cargo deny --locked check licenses
bash contrib/check-qapi.sh
```

`contrib/check-qapi.sh` runs the checks that need the QEMU source:

- the generator tests

It needs the QEMU source with its release tags:

```sh
git submodule update --init vendor/qemu
git -C vendor/qemu fetch origin --tags
```

`vendor/qemu` is a git submodule of the QEMU repository named in
[.gitmodules](.gitmodules). The tags it must hold are the ones pinned in
[`projects/qapi-codegen/qemu-versions.lock`](projects/qapi-codegen/qemu-versions.lock).

## Dependencies and licensing

The crates are licensed under the [Apache License 2.0](LICENSE) and the
documentation under [CC-BY-4.0](LICENSE-docs). Contributions are accepted
under the same licenses.

New Cargo dependencies must use a license on the
[CNCF allowed third-party license list](https://github.com/cncf/foundation/blob/main/policies-guidance/allowed-third-party-license-policy.md).
`cargo deny --locked check licenses` enforces this with [deny.toml](deny.toml);
do not widen its allow list. A dependency that needs a license exception is
discussed in an issue first.

## Pull request flow

Every change to `main` goes through a pull request, including the
maintainers' own changes. The rules are the qrab project's; its
[review policy](https://github.com/qrab-dev/qrab/blob/main/docs/contributing-review.md)
has the details and the reasoning.

1. **Fork and branch.** Fork `qrab-dev/qrab-qapi` and make your change on a
   branch of your fork, based on the current `main`. Every change comes from
   a fork, the maintainers' own included: no branch other than `main` is
   pushed to this repository.
2. **Sign off every commit** (see below), and sign it if you can (see
   [Signed commits](#signed-commits)).
3. **Open a pull request** against `main`. Fill in the template: what the
   change does, why, and how you tested it. Keep a pull request to one
   logical change; split unrelated fixes into separate pull requests.
4. **Pass CI.** The `ci` workflow runs the jobs `fmt`, `clippy`, `test`,
   `cargo-deny` and `check-qapi`, and the `DCO` check verifies every commit's
   sign-off. `check-qapi` needs the QEMU source and takes longer than the
   others. The branch must sit on top of the current `main`, so that
   `main` can be fast-forwarded to it; rebase it when `main` has moved.
5. **Get a code owner's review.** A maintainer listed in
   [.github/CODEOWNERS](.github/CODEOWNERS) must approve the latest push, and
   every review thread must be resolved. Pushing new commits dismisses
   earlier approvals.
6. **Land.** GitHub's merge button is not used. Once the pull request is
   approved and every check is green, a maintainer fast-forwards `main` to
   its head with a push authenticated by their hardware security key.
   GitHub marks the pull request merged, and `main` holds exactly the
   commits that were reviewed and tested, with the same SHAs. `main` keeps a
   linear history: merge commits, force pushes and deletion are refused for
   everyone, maintainers included.

Because `main` is fast-forwarded to your branch, each commit lands on `main`
as it is. Make every commit build and pass the checks, and tidy the series
(squash fixups, reword messages) before it lands. Address review feedback
with new commits or by amending; either is fine, as long as every commit is
signed off.

## Signed commits

Every commit on `main` carries a verified signature: the repository's rules
require signed commits for everyone. Maintainers sign with a hardware
security key, so every signature and every push to `main` needs a physical
touch; no automation can sign or land a change on its own.

Your commits can meet this rule in one of two ways:

- **Sign them yourself** with an SSH or GPG key that you have added to your
  GitHub account as a signing key (`git commit -S`, or set
  `commit.gpgsign`), so that GitHub shows each commit as Verified. GitHub's
  guide to
  [commit signature verification](https://docs.github.com/en/authentication/managing-commit-signature-verification)
  explains the setup. The maintainer then lands your commits unchanged.
- **Leave them unsigned**, and the maintainer re-signs them as the committer
  before landing them. Your authorship, message and sign-off are kept; the
  committer and the SHAs change, and the re-signed commits go through CI
  again before `main` is fast-forwarded to them.

A signature records who committed a change; the `Signed-off-by` line is your
DCO certification. They are separate, and the DCO sign-off is required either
way.

## Developer Certificate of Origin

qrab-qapi uses the [Developer Certificate of Origin](https://developercertificate.org/)
(DCO) instead of a contributor license agreement. By signing off a commit you
certify that you wrote the change or otherwise have the right to submit it
under the project's licenses.

Sign off by adding a `Signed-off-by` line with your real name and an email
address you can be reached at:

```text
Signed-off-by: Jane Doe <jane@example.com>
```

`git commit -s` adds the line for you. To sign off commits you already made:

```sh
git commit --amend -s --no-edit        # the last commit
git rebase --signoff main              # every commit on your branch
```

Commits made in the GitHub web editor must also be signed off. The `DCO`
check fails on any commit without a sign-off, and a pull request cannot merge
until it passes.

## Commit messages

Commit messages follow the 50/72 rules. The maintainers hold their own
commits to them strictly; for contributions they are recommended, and a
maintainer may reword a message before landing it.

- Write the subject in the imperative mood ("Add guest-ssh to the QGA
  client", not "Added" or "Adds"), at most 50 characters, starting with an
  uppercase letter, with no trailing period and no `area:` prefix.
- Leave a blank line, then a body that explains what changed and why. The
  diff shows how; the body records the reasoning a reviewer or a later
  reader needs. For a generator change, say what changed in the generated
  output.
- Wrap the body at 72 columns. Only a line that holds a single unbreakable
  token, such as a long URL, path or command, may be longer; put it on a
  line of its own, optionally indented or after a `- ` bullet.
- Use no tabs, trailing whitespace or Markdown `#` headings (git drops lines
  starting with `#` as comments when you edit a message).
- Reference related issues in the body (`Fixes #123`).
- End with your `Signed-off-by` line. The trailers, the `Key: value` lines
  of the last paragraph, are exempt from the 72-column limit.

## AI-assisted contributions

Much of qrab and qrab-qapi is written with coding agents, and AI-assisted
contributions are welcome under the same rules as any other:

- The person who submits a change is responsible for it. Review what a tool
  generated, test it, and make sure you have the right to submit it before
  you sign off. The DCO sign-off is yours, not the tool's.
- Mention significant AI assistance in the pull request description.
- The qrab project's own agents push to a fork and open pull requests as the
  `qrab-dev-agent[bot]` GitHub App, never with a maintainer's credentials.
  Their pull requests take the same path as yours: CI, the DCO check and
  approval by a human code owner. Their commits are signed, and landed, only
  by a maintainer's security key.
- Automated reviewers may leave advisory comments, but they are not code
  owners and never replace the human approval.

The reasons for this policy are in qrab's
[review policy](https://github.com/qrab-dev/qrab/blob/main/docs/contributing-review.md#reviewing-agent-authored-changes).

## Governance

qrab-qapi is governed by the qrab project. Roles, decision making and how to
become a reviewer or maintainer are described in qrab's
[GOVERNANCE.md](https://github.com/qrab-dev/qrab/blob/main/GOVERNANCE.md);
this repository's [GOVERNANCE.md](GOVERNANCE.md) and
[MAINTAINERS.md](MAINTAINERS.md) point there and list who maintains these
crates.
