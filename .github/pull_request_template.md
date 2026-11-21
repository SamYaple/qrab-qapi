## What this changes and why

<!-- What the change does, the problem it solves, and anything a reviewer
should look at closely. Link related issues (Fixes #123). -->

## How it was tested

<!-- Commands you ran and their results. For protocol changes, the QEMU or
qemu-ga version of any live test (docs/testing.md). -->

## Checklist

- [ ] Every commit is signed off (`git commit -s`) under the
      [DCO](https://developercertificate.org/); the `DCO` check verifies this.
- [ ] `cargo fmt --all --check`, the strict Clippy command and
      `cargo test --locked` pass locally; for parser, generator or macro
      changes, `bash contrib/check-qapi.sh` passes too.
- [ ] Generated sources under `projects/qapi/src/generated` and
      `projects/qapi/src/qga` were regenerated, not edited by hand, and no
      `schema-docs` output is included.
- [ ] Commit subjects are in the imperative mood, bodies say what changed and
      why, and each commit builds on its own.
- [ ] Docs are updated for any user-visible change.
- [ ] Significant AI assistance, if any, is mentioned above, and I have
      reviewed and tested the generated code.

See [CONTRIBUTING.md](https://github.com/qrab-dev/qrab-qapi/blob/main/CONTRIBUTING.md) for the full flow.
