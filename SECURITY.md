# Security Policy

qrab-qapi is a subproject of [qrab](https://github.com/qrab-dev/qrab) and
follows the qrab project's security process, described here for these
crates.

## Supported versions

qrab-qapi is pre-1.0. Security fixes go to `main` and to the latest release
of each affected crate only. There are no long-term support branches; upgrade
to the latest release to get a fix.

| Version | Supported |
| --- | --- |
| `main` | Yes |
| Latest release of each crate | Yes |
| Older releases | No |

## Reporting a vulnerability

**Do not open a public issue, pull request or discussion for a security
problem.** Report it privately in one of these ways:

1. **GitHub private vulnerability reporting (preferred).** On the
   [qrab-qapi repository](https://github.com/qrab-dev/qrab-qapi), open the
   **Security** tab and choose **Report a vulnerability**, or go directly to
   <https://github.com/qrab-dev/qrab-qapi/security/advisories/new>. The report
   is visible only to you and the maintainers, and the fix can be prepared in
   a private fork attached to the advisory.
2. **Email** [security@qrab.dev](mailto:security@qrab.dev).

Please include:

- the affected crate (`qrab-qapi`, `qrab-qapi-protocol`, `qrab-qapi-macros`
  or `qrab-qapi-codegen`) and its version or commit;
- the QEMU or `qemu-ga` version of the peer involved, if any;
- a description of the problem and its impact;
- steps or a proof of concept to reproduce it, such as the bytes a peer sends
  or the schema input that triggers it;
- whether you want to be credited, and under what name.

## What to expect

| Step | Target |
| --- | --- |
| Acknowledge the report | Within 3 business days |
| Initial assessment: confirm or reject, and a severity | Within 10 business days |
| Fix and release | Depends on severity and complexity; we keep you updated at least every 2 weeks |

These are targets, not guarantees. If you get no acknowledgement within 3
business days, send a reminder by the other channel.

## Disclosure policy

qrab-qapi follows coordinated disclosure:

1. The maintainers confirm the problem, determine the affected versions and
   prepare a fix in private.
2. A disclosure date is agreed with the reporter. The default is when a fixed
   release is available, and no later than 90 days after the report unless
   both sides agree to extend it, for example while an upstream fix is
   pending.
3. The fix is released and a GitHub security advisory is published, with a
   CVE where one applies, crediting the reporter unless they ask otherwise.

If a vulnerability is already public or actively exploited, the maintainers
release a fix and advisory as soon as possible.

## Scope

In scope is the code in this repository and the crates published from it:

- `qrab-qapi-protocol`, the QMP and QGA clients and mock servers. A guest
  agent runs inside the guest, so a QGA peer may be hostile; so may a QMP
  peer the caller does not control. Examples: a peer that can make the client
  exceed its documented size bounds, hang past its configured timeouts,
  panic, or deliver a reply to the wrong request.
- `qrab-qapi` and `qrab-qapi-macros`: the generated types and their
  serialization and decoding, for example a reply that passes version checks
  it should fail or a crafted value that causes a panic.
- `qrab-qapi-codegen`, the schema parser and generator, including the
  generated Rust it emits from the pinned QEMU schemas.
- The build and CI scripts in this repository.

Out of scope, and best reported to the upstream project:

- **QEMU** itself, including `qemu-ga` and the QAPI schemas, unless the
  problem comes from how these crates handle them. Report QEMU bugs through
  the [QEMU security process](https://www.qemu.org/contribute/security-process/).
- **qrab** and other software that uses these crates. Report problems in qrab
  through [qrab's security policy](https://github.com/qrab-dev/qrab/blob/main/SECURITY.md).
- The Rust crates qrab-qapi depends on, such as Tokio and Serde, unless the
  problem comes from how qrab-qapi uses them.
- Resource limits a caller raises deliberately through `ClientOptions`, and
  problems that need control of the local host or of the socket path the
  caller chose.

If you are unsure whether something is in scope, report it privately anyway.
We would rather triage a report that turns out to be out of scope than miss
one.
