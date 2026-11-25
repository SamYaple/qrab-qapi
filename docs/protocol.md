# Protocol clients

`qapi-protocol` targets the vendored QEMU **11.1.3** protocol. It
provides Tokio Unix-socket clients and bounded mock sessions, without
depending on systemd, Kubernetes or guest hardware. Each connection speaks
its own peer's release, older ones included, as their schemas describe them
(see [Types and errors](#types-and-errors)). The tests cover older releases
through their schemas and the mock sessions; the opt-in live tests run against
the binaries you give them ([testing.md](testing.md)).

## Connecting and configuration

`Qmp::new(path)` returns a QMP client and its event receiver.
`Qga::new(path)` returns a sequential guest-agent client.
`from_stream(stream)` takes an already-connected Unix socket instead, and the
`*_with_options` constructors take `ClientOptions`. Options set connect,
handshake and command timeouts, maximum JSON message size, and the QMP event
buffer capacity. `ClientOptions` is `#[non_exhaustive]`: start from
`ClientOptions::default()` and set its fields, or chain its `with_*` methods.

Defaults are 5 seconds for connect, 10 seconds for the complete handshake,
30 seconds for commands, 128 buffered events with a 1 MiB per-event bound, and
8 MiB per JSON message. JSON size excludes the final line terminator. Both
compact and pretty-printed QMP objects are accepted, with LF or CRLF
termination. Invalid UTF-8, malformed envelopes, truncated EOF and oversized
frames close the connection; the client never drains an endless oversized
frame waiting for a newline.

The client never parses a message's envelope into a `serde_json::Value` tree,
which can take many times its JSON's size in memory: it reads the envelope as
borrowed raw JSON and gives the payload's bytes to the return type to decode.
Plain structs, enums and lists decode straight from those bytes. Unions,
alternates, structs with flattened bases and members whose type changed between
releases are buffered as a `Value` while they decode, as in the replies of
`query-qmp-schema` and `query-cpus-fast`; the message size limit bounds them.
The raw calls return a `Value`, since the caller asks for one.

QMP validates the typed greeting and sends a correlated `qmp_capabilities`
request. It enables OOB only if advertised. A missing, malformed or incorrectly
correlated capabilities acknowledgement fails the handshake. Events arriving
during negotiation are retained within the configured buffer. `greeting()`
exposes the peer's version, package string and advertised capabilities.

## QMP requests and lifecycle

One task owns the socket and routing table. Reads and partial writes progress
independently, so a large upload cannot prevent reading an earlier reply.
Request IDs belong to the connection. Typed calls use `qapi::commands` and
decode the reply into `QapiCommand::Returns` in the peer's version scope, which
selects the return type of the peer's release for a command whose return type
changed. `execute_raw` accepts an `execute` envelope with optional
object arguments, while `execute_raw_oob` requires `exec-oob`. Caller IDs,
conflicting verbs and malformed arguments fail before any socket I/O. Unknown
response IDs of any JSON type are dropped as required by QMP. An ID-less server
parser error closes the connection while preserving its error class and description.

`execute_raw_with_fds` (and its `getfd(name, fd)` shorthand) attaches file
descriptors to an in-band command as SCM_RIGHTS, the way QEMU's `getfd` and
`add-fd` read them: the driver sends them with the frame's first bytes in one
`sendmsg`, then the rest with ordinary writes. One message carries at most 253
descriptors (Linux's `SCM_MAX_FD`); a request with more is refused before
anything is sent. The descriptors are moved in and closed once sent, or when
the request is refused or expires.

Admission is bounded to eight unanswered in-band commands
(`QMP_QUEUE_CAPACITY`), following QEMU's recommendation for keeping OOB
responsive. A separate eight-command bound limits OOB requests. Permits belong
to dispatched requests, not their callers: cancelling a future cannot free a
slot while QEMU may still be executing it. Requests cancelled while still
queued are skipped. `queue_depth()` counts dispatched requests, including OOB
and requests whose callers were cancelled.

Command deadlines cover admission, queued writes and waiting for a reply. They
can be configured per connection, or per call: each call has a `_with_timeout`
form (`execute_with_timeout`, `execute_oob_with_timeout`,
`execute_raw_with_timeout`, `execute_raw_oob_with_timeout`) and
`execute_raw_with_fds_and_timeout` that of `execute_raw_with_fds`. A deadline that elapses before the request is
dispatched fails with `TimeoutPhase::BeforeDispatch` and leaves the connection
healthy. If a dispatched command expires, the call fails with
`TimeoutPhase::AfterDispatch` and the connection closes, because the remote
outcome and queue occupancy are unknown. The call whose deadline elapsed
always gets that timeout, never the disconnect it caused; pending callers
still within their deadlines get `Error::Disconnected` holding it. No
automatic retry occurs: cancellation, timeout or disconnect never proves that
a command did not execute.

`shutdown(&self)` works through `Arc<Qmp>`, wakes both admitted callers and
callers waiting for capacity, closes the socket and ends the driver. Dropping
the owner aborts the driver with the same cleanup. `is_closed()`,
`close_reason()` and `closed().await` expose connection lifecycle. Pending
callers share the original failure via `Error::Disconnected`.

Events within `max_event_bytes` are parsed once and delivered without blocking
replies. Dropping the event receiver does not disconnect QMP. Buffer overflow
or an event exceeding `max_event_bytes` drops that event, unparsed;
`dropped_events()` counts these losses, so consumers can detect loss and refresh
state. QEMU itself also rate-limits some events: this stream is not a durable
lifecycle journal. Event timestamps support QEMU's `-1` unavailable-time values.

An event's `data` stays raw JSON, since only the caller knows its type, and the
client's version scope does not reach the caller's decode. Decode it inside
`qapi::with_version(qmp.version(), || ...)`, so that the peer's release selects
the payload layout and checks it; outside any scope a decode takes the newest
layout unchecked.

## QGA requests and lifecycle

QGA sends an OS-random 64-bit guest-sync-delimited nonce. It flushes parser state
at every `0xFF` sentinel and ignores stale, malformed or mismatched replies until
the nonce matches. The same handshake deadline also bounds guest-info discovery.
A guest-info version that `qapi::parse_qga_version` refuses fails connection
establishment with `Error::AgentVersion`.

The client needs qemu-ga 1.1 or later, the first with guest-sync-delimited.
An older agent, or one started with the command disabled (`--block-rpcs`,
`--allow-rpcs`), answers it with a `CommandNotFound` error and no sentinel;
the handshake fails at once with `Error::SyncUnsupported` holding that reply,
instead of waiting for its deadline.

QGA calls use `&mut self` and `qapi::qga::commands`. The session is taken out of
the client before socket I/O and restored only after a complete valid response,
or after a timeout before any byte of the request was written. A timeout after
that, cancellation, malformed JSON or I/O failure therefore closes the socket
instead of letting a late reply satisfy the next command. Reconnect to perform
synchronization again. Closing a socket does not roll back guest-side
resources or effects; a known file handle may require explicit cleanup through
a newly synchronized connection. A valid agent error or a typed payload decoding
error leaves an aligned session usable. `is_closed()` and `shutdown()` expose
this state explicitly.

The guest agent runs inside the guest and is not trusted. Its replies are
limited to `max_message_bytes` and decoded straight into their return type.
`execute_with_reply_limit` raises the limit for one call:
`qga::FILE_READ_REPLY_BYTES` admits the base64 reply of a `guest-file-read` of
the agent's 48 MiB maximum count. Requests are limited to `max_message_bytes`
as well, and a larger one fails with `Error::InvalidRequest` before anything
is sent. Write a large file in several `guest-file-write` calls on its
handle: under the default limit, a chunk of up to 6 MiB less 1 KiB fits its
base64 request. Raising `max_message_bytes` instead admits larger requests,
but also larger replies from the guest for every call.

Commands with `success-response: false`, such as guest-shutdown and guest-suspend,
use `send_no_response` instead of `execute`. Success means only that the request
bytes were written; no remote success acknowledgement exists. This operation
closes the socket so a possible error reply cannot contaminate a later call.
Typed QMP and QGA calls evaluate OOB and reply flags for their connected peer's
release, including historical additions and removals of those flags.

## Types and errors

Each connection serializes requests and decodes typed replies using its own
peer's `version()`.
For a guest agent, that is the agent's own version, which can differ from QEMU's.
Direct serialization can use the synchronous
`qapi::with_version(version, || ...)` scope or `qapi::to_value(version, &value)`;
outside any scope a version-gated type fails with an error naming
`qapi::with_version`. `qapi::to_value` reports a type, variant or field that
the release lacks as `qapi::EncodeError::Unavailable`, which the clients
return as `Error::Unavailable`.

Responses require exactly one of `return` or a well-formed `error`. Missing
return values are never success. An empty object maps to Rust `()` only when
that is the actual return type; other failed decodes are not retried as null.

`Error` is `#[non_exhaustive]`. Its variants tell the kind of failure apart,
and with it whether the request reached the peer and the connection survived:

| Variant | Meaning | Connection |
| --- | --- | --- |
| `Remote` | The peer ran the command and returned an error reply | usable |
| `Unavailable` | The command, or a type, variant or field it uses, is not in the peer's release; nothing was sent | usable |
| `Encode` | The request could not be serialized; nothing was sent | usable |
| `InvalidRequest` | The request or the options are invalid; nothing was sent | usable |
| `Decode` | A well-formed reply does not match the return type | usable |
| `Timeout { phase }` | A deadline elapsed in the `Connect`, `Handshake`, `BeforeDispatch` or `AfterDispatch` phase | usable after `BeforeDispatch` |
| `Malformed` | The peer sent invalid JSON | closed |
| `MessageTooLarge { limit }` | A message from the peer exceeds the limit in effect | closed |
| `Protocol` | The peer sent valid JSON that breaks the protocol | closed |
| `AgentVersion` | The guest agent reported a version that is not a release number | closed |
| `SyncUnsupported` | The guest agent refused guest-sync-delimited; it predates qemu-ga 1.1 or has the command disabled | closed |
| `Io` | Connecting, reading or writing failed | closed |
| `Closed` | The connection was already closed | closed |
| `Disconnected` | The connection failed while the call was pending; it holds the original failure | closed |

`RemoteError` retains the exact string `class` and `desc` of an error reply,
including new error classes; `known_class()` optionally maps it to the
generated enum.

These API choices are deliberate: `RemoteError.class` is a string,
`Timestamp.microseconds` is signed, deserialized return types are owned and
`'static`, and `Qmp::shutdown` takes `&self`. Consumers should use the
connection's version and lifecycle rather than a process-global snapshot.

See [testing.md](testing.md) for fault tests, the core checks and smoke tests
against real QEMU binaries. The mock `server` sessions need the `mock` feature.
They share the bounded framing and validation helpers, including explicit
response IDs for out-of-order tests.
