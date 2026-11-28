# Changelog

All notable changes to `qrab-qapi-protocol` are recorded here, newest first.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the version numbers follow the policy in the repository's
`docs/versioning.md`.

## 0.2.1

- The client sends file descriptors with rustix instead of nix, which is
  no longer a dependency.
- `Qmp::handshake` returns the client, its events and the connection's
  `QmpDriver`, a future that does the connection's I/O and deadlines, for
  code that runs it itself, such as on a `LocalSet`. `from_stream` and the
  connecting constructors still spawn it on tokio. The new `Socket` trait,
  which is sealed, names the sockets it takes: tokio's `UnixStream`.
- Out-of-band requests are sent before queued in-band ones.
- The QMP client's admission, deadlines and routing run in a sans-IO engine,
  which the coming blocking and async-io clients are to share. The `tokio`
  feature no longer enables tokio's `macros`.

## 0.2.0

- Breaking: the crate depends on `qrab-qapi-core` instead of `qrab-qapi`.
  The typed calls take any `qapi_core::QapiCommand`, and `Error::Encode`
  wraps `qapi_core::EncodeError`. `qrab-qapi` re-exports both from 0.1.1
  on, so its types work as before. With `qrab-qapi` 0.1.0, which defined
  them itself, typed calls do not compile: update it to 0.1.1.
- A breaking release of `qrab-qapi`, such as a QEMU refresh with a removal,
  no longer needs a release of this crate.
- Breaking: `RemoteError::known_class` returns the new `ErrorClass`, the
  error classes of QEMU and its guest agent, instead of
  `qapi::QapiErrorClass`. `RemoteError::class` still holds the string.
- The client decodes the handshake's `guest-info` reply itself.

## 0.1.0

The first release, requiring `qrab-qapi` 0.1.

- `Qmp` and `qga::Qga`, async clients for QEMU's QMP monitor and the QEMU
  guest agent over Unix sockets, on Tokio. Each connection negotiates its
  peer's version and checks typed requests and replies against it. The
  clients also send raw commands by name, and the QMP client receives events
  through `QmpEvents`, runs out-of-band commands when the peer offers them
  and passes file descriptors with a command.
- While connecting, a client learns the commands that its peer offers: QMP
  from one `query-commands`, the guest agent from the `guest-info` reply
  that the handshake already reads. Any other command is refused before
  anything is sent, as `Error::Unavailable` with its `Unavailability`
  reason. `commands`, `supports` and `require` read the list, and
  `ClientOptions::probe_commands` turns the query off.
- The clients are behind the `tokio` feature, which is on by default.
  Without it, the crate builds only the types that need no runtime, such as
  `Error`, `ClientOptions` and `PeerCommands`, and depends on neither Tokio,
  nix nor getrandom.
- Bounded mock QMP and guest agent servers for tests, in the `server` module
  behind the `mock` feature, which enables `tokio`.
- `Error` says which failures leave a connection usable.
