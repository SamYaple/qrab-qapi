#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
//!
//! Each connection owns its version and lifecycle. [`Qmp`] describes the
//! QMP client's cancellation and timeout semantics.
//! [`qga::Qga`] describes the guest agent client's.
//! [`Error`] says which failures leave a connection usable. The
//! repository's `docs/protocol.md` describes the client lifecycle, the mock
//! server API and the internals, and `docs/testing.md` the tests.

#[cfg(feature = "tokio")]
mod engine;
#[cfg(feature = "tokio")]
mod framing;
#[cfg(feature = "tokio")]
pub mod qga;
#[cfg(feature = "tokio")]
mod qmp;
#[cfg(feature = "mock")]
pub mod server;
#[cfg(feature = "tokio")]
mod wire;

#[cfg(feature = "tokio")]
pub use engine::{Greeting, QMP_QUEUE_CAPACITY, QemuVersion, VersionInfo};
use qapi_core::QapiCommand;
#[cfg(feature = "tokio")]
pub use qmp::{Qmp, QmpEvents};
use std::{collections::BTreeSet, fmt, sync::Arc, time::Duration};

/// Why a connection or a call failed.
///
/// [`Remote`](Error::Remote), [`Decode`](Error::Decode) and the failures
/// detected before any byte is sent ([`Unavailable`](Error::Unavailable),
/// [`MissingCommands`](Error::MissingCommands), [`Encode`](Error::Encode),
/// [`InvalidRequest`](Error::InvalidRequest) and a
/// [`Timeout`](Error::Timeout) in [`TimeoutPhase::BeforeDispatch`]) leave
/// the connection usable. Every other failure closes it.
///
/// The clients and mock sessions of this crate create these errors; match
/// on them, but do not build them. Variants with named fields cannot be
/// built outside the crate, and new variants and fields may be added.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The peer ran the command and answered with an error reply.
    #[error("command failed: {0}")]
    Remote(#[from] RemoteError),
    /// The peer cannot run the command: its release lacks the command or a
    /// type, variant or field the command uses, or the peer did not list
    /// the command when the client connected. Nothing was sent.
    #[error("{name} is not available in QEMU {}.{}.{}: {reason}", .version.0, .version.1, .version.2)]
    #[non_exhaustive]
    Unavailable {
        /// The QAPI command name, or the Rust name of the type, variant or
        /// field, as [`qapi_core::EncodeError::Unavailable`] names it.
        name: String,
        /// The peer's version.
        version: (u32, u32, u32),
        /// Why the peer cannot run it.
        reason: Unavailability,
    },
    /// [`Qmp::require`] or [`Qga::require`](qga::Qga::require) found
    /// commands that the peer cannot run, all of them, each with the reason.
    /// Nothing was sent.
    #[error("QEMU {}.{}.{} cannot run {}", .version.0, .version.1, .version.2, list_missing(.missing))]
    #[non_exhaustive]
    MissingCommands {
        /// The peer's version.
        version: (u32, u32, u32),
        /// The commands, in the order they were required.
        missing: Vec<MissingCommand>,
    },
    /// The request could not be serialized for another reason than
    /// [`Unavailable`](Error::Unavailable). Nothing was sent.
    #[error("cannot encode the request: {0}")]
    Encode(#[source] qapi_core::EncodeError),
    /// A well-formed reply does not match the expected return type.
    #[error("cannot decode the reply: {0}")]
    Decode(#[source] serde_json::Error),
    /// The peer sent bytes that are not valid JSON.
    #[error("malformed JSON from the peer: {0}")]
    Malformed(#[source] serde_json::Error),
    /// The peer sent a message larger than the limit in effect. The client
    /// stops reading it at the limit.
    #[error("message from the peer exceeds {limit} bytes")]
    #[non_exhaustive]
    MessageTooLarge {
        /// The limit, in JSON bytes.
        limit: usize,
    },
    /// The peer sent valid JSON that breaks the protocol, or a mock session
    /// received a request that breaks it. The violation names the rule.
    #[error("protocol violation: {0}")]
    Protocol(#[source] ProtocolViolation),
    /// The guest agent's `guest-info` reports a version that is not a
    /// release number, so no client was returned.
    #[error(transparent)]
    AgentVersion(#[from] qapi_core::VersionParseError),
    /// The guest agent answered `guest-sync-delimited`, which the client
    /// synchronizes with, with this error reply instead of its nonce: an
    /// agent before qemu-ga 1.1 lacks the command, and one started with it
    /// disabled (`--block-rpcs`, `--allow-rpcs`) refuses it. No client was
    /// returned.
    #[error(
        "the guest agent cannot synchronize; it needs guest-sync-delimited (qemu-ga 1.1 or later): {0}"
    )]
    SyncUnsupported(#[source] RemoteError),
    /// A deadline elapsed; the phase says whether the request reached the peer.
    #[error("timed out {phase}")]
    #[non_exhaustive]
    Timeout { phase: TimeoutPhase },
    /// Connecting, reading or writing failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The operating system could not supply the random nonce that the
    /// guest agent handshake synchronizes with. No client was returned.
    #[error("cannot generate the guest agent sync nonce: {0}")]
    SyncNonce(#[source] std::io::Error),
    /// The request or the options are invalid. Nothing was sent. The kind
    /// names the rule it breaks.
    #[error("invalid request: {0}")]
    InvalidRequest(#[source] InvalidRequestKind),
    /// The connection was already closed.
    #[error("connection closed")]
    Closed,
    /// The connection failed while the call was pending. The original
    /// failure is shared with every pending caller.
    #[error("connection lost: {0}")]
    Disconnected(#[source] Arc<Error>),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Why [`Error::InvalidRequest`] refused a request or the client options.
/// Nothing was sent.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum InvalidRequestKind {
    /// A [`ClientOptions`] field is zero or out of range: a timeout that
    /// cannot be added to the current time, or an event capacity above
    /// `usize::MAX >> 3`.
    #[error("the {option} option must be nonzero and in range")]
    #[non_exhaustive]
    Options {
        /// The field's name, such as `"command_timeout"`.
        option: &'static str,
    },
    /// A raw command has an empty name.
    #[error("the command name is empty")]
    EmptyCommandName,
    /// A raw command's arguments are not a JSON object.
    #[error("the arguments of {command} must be a JSON object")]
    #[non_exhaustive]
    ArgumentsNotObject {
        /// The command name.
        command: String,
    },
    /// The encoded request exceeds
    /// [`ClientOptions::max_message_bytes`].
    #[error("JSON request of {size} bytes exceeds {limit} bytes")]
    #[non_exhaustive]
    RequestTooLarge {
        /// The request's JSON bytes, excluding the line terminator.
        size: usize,
        /// The limit in effect.
        limit: usize,
    },
    /// The command sends no success response in the peer's release, so
    /// waiting for one would never end. A guest agent sends such a command
    /// with [`Qga::send_no_response`](qga::Qga::send_no_response).
    #[error("{command} does not send a success response; send it with send_no_response")]
    #[non_exhaustive]
    NoSuccessResponse {
        /// The command name.
        command: String,
    },
    /// [`Qga::send_no_response`](qga::Qga::send_no_response) was given a
    /// command that sends a response in the agent's release.
    #[error("{command} has a response; use execute")]
    #[non_exhaustive]
    HasResponse {
        /// The command name.
        command: String,
    },
    /// The command does not allow out-of-band execution in the peer's
    /// release.
    #[error("{command} does not allow OOB execution")]
    #[non_exhaustive]
    OobNotAllowed {
        /// The command name.
        command: String,
    },
    /// An out-of-band call on a connection whose peer did not advertise
    /// the `oob` capability.
    #[error("OOB was not negotiated")]
    OobNotNegotiated,
    /// More file descriptors than one message carries.
    #[error("at most {max} file descriptors fit in one message, not {count}")]
    #[non_exhaustive]
    TooManyFds {
        /// The descriptors passed.
        count: usize,
        /// Linux's `SCM_MAX_FD`.
        max: usize,
    },
    /// The call's timeout cannot be added to the current time.
    #[error("the timeout of {timeout:?} is too large")]
    #[non_exhaustive]
    TimeoutTooLarge {
        /// The timeout passed.
        timeout: Duration,
    },
    /// A per-call reply limit of zero.
    #[error("the reply limit must be nonzero")]
    ZeroReplyLimit,
}

/// The protocol rule an [`Error::Protocol`] broke.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProtocolViolation {
    /// A message is valid JSON but not an object. For a scalar, the source
    /// says what was found instead.
    #[error("message must be a JSON object{}", .0.as_ref().map(|error| format!(": {error}")).unwrap_or_default())]
    NotAnObject(#[source] Option<serde_json::Error>),
    /// The input ended inside a message.
    #[error("truncated JSON message at EOF")]
    Truncated,
    /// A response has both or neither of `return` and `error`, or also
    /// event or greeting members.
    #[error("response must contain exactly one of return or error")]
    ResponseShape,
    /// A response's `error` is not an object of string `class` and `desc`.
    #[error("invalid error response: {0}")]
    InvalidErrorResponse(#[source] serde_json::Error),
    /// A QMP response without an ID that is not an error reply; an ID-less
    /// error reply is returned as [`Error::Remote`] instead.
    #[error("QMP response has no ID")]
    MissingResponseId,
    /// A QMP reply arrived before the client finished writing its request.
    #[error("response preceded completion of its request")]
    EarlyResponse,
    /// The connection used up its request IDs.
    #[error("QMP request ID space exhausted")]
    IdsExhausted,
    /// The first QMP message is not the greeting.
    #[error("expected the QMP greeting")]
    MissingGreeting,
    /// The QMP greeting does not have the expected members.
    #[error("invalid QMP greeting: {0}")]
    InvalidGreeting(#[source] serde_json::Error),
    /// The reply to `qmp_capabilities` has another ID than the request's.
    #[error("capabilities reply has wrong or missing ID")]
    CapabilitiesReplyId,
    /// The reply to `qmp_capabilities` is not an empty object.
    #[error("capabilities reply must return an empty object")]
    CapabilitiesReply,
    /// The reply to the `query-commands` that the client sends after
    /// `qmp_capabilities` has another ID than the request's.
    #[error("query-commands reply has wrong or missing ID")]
    CommandsReplyId,
    /// An event also has response members (`return`, `error` or `id`).
    #[error("event mixed with response fields")]
    EventWithResponseFields,
    /// An event's `data` is not an object.
    #[error("event data must be an object")]
    EventDataNotObject,
    /// An event does not have the expected members.
    #[error("invalid event: {0}")]
    InvalidEvent(#[source] serde_json::Error),
    /// A mock session received a request without a nonempty `execute` or
    /// `exec-oob` command name, or with another member than that verb,
    /// `arguments` and `id`, or with arguments that are not an object.
    #[error("expected {verb} with a command name and optional object arguments")]
    #[non_exhaustive]
    InvalidCommand {
        /// The verb of the request, `execute` or `exec-oob`.
        verb: &'static str,
    },
    /// A mock session received `exec-oob` before the client enabled OOB.
    #[error("exec-oob before the client enabled OOB")]
    OobNotEnabled,
    /// A mock session expected one command and received another.
    #[error("expected in-band {expected}, got {}{found}", if *.oob { "exec-oob " } else { "" })]
    #[non_exhaustive]
    UnexpectedCommand {
        /// The command the session expected, in-band.
        expected: &'static str,
        /// The command it received.
        found: String,
        /// Whether that command was out-of-band.
        oob: bool,
    },
    /// A mock QMP session received `qmp_capabilities` enabling another
    /// capability than `oob`.
    #[error("unsupported capabilities")]
    UnsupportedCapabilities,
    /// A mock guest agent session received `guest-sync-delimited` without
    /// an integer `id`.
    #[error("missing sync nonce")]
    MissingSyncNonce,
    /// A mock guest agent session received `exec-oob`.
    #[error("QGA does not support OOB")]
    GuestAgentOob,
}

/// Why [`Error::Unavailable`] refused a command, or why
/// [`Error::MissingCommands`] lists one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Unavailability {
    /// The peer's release does not have it: the command, or a type,
    /// variant or field its arguments use, is outside its lifetime there.
    Release,
    /// The peer did not list the command when the client connected: this
    /// QEMU or guest agent was built without it.
    NotOffered,
    /// The guest agent lists the command as disabled, as `--block-rpcs`
    /// and `--allow-rpcs` leave it.
    Disabled,
}

impl fmt::Display for Unavailability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Release => "that release does not have it",
            Self::NotOffered => "the peer does not offer it",
            Self::Disabled => "the guest agent has it disabled",
        })
    }
}

/// A command that [`Error::MissingCommands`] lists.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MissingCommand {
    /// The command's name on the wire.
    pub name: String,
    /// Why the peer cannot run it.
    pub reason: Unavailability,
}

fn list_missing(missing: &[MissingCommand]) -> String {
    let names: Vec<String> = missing
        .iter()
        .map(|command| format!("{} ({})", command.name, command.reason))
        .collect();
    names.join(", ")
}

/// The commands a peer listed when the client connected: the reply to
/// `query-commands` for QMP, or the commands of a guest agent's
/// `guest-info`, with the ones it has disabled. [`Qmp::commands`] and
/// [`Qga::commands`](qga::Qga::commands) return it.
///
/// The list can only refuse a command: one that the peer's release has
/// but this peer does not list fails with [`Unavailability::NotOffered`]
/// or [`Unavailability::Disabled`]. A typed command that the release does
/// not have stays refused even if the peer lists it, as a backport would.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PeerCommands {
    enabled: BTreeSet<String>,
    disabled: BTreeSet<String>,
}

impl PeerCommands {
    #[cfg(feature = "tokio")]
    pub(crate) fn new(
        enabled: impl IntoIterator<Item = String>,
        disabled: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            enabled: enabled.into_iter().collect(),
            disabled: disabled.into_iter().collect(),
        }
    }

    /// Whether the peer listed `name` as a command it runs.
    pub fn offers(&self, name: &str) -> bool {
        self.enabled.contains(name)
    }

    /// Whether a guest agent listed `name` as disabled.
    pub fn is_disabled(&self, name: &str) -> bool {
        self.disabled.contains(name)
    }

    /// The commands the peer runs, in name order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.enabled.iter().map(String::as_str)
    }

    /// Why the peer cannot run `name`, if it cannot.
    #[cfg(feature = "tokio")]
    pub(crate) fn refusal(&self, name: &str) -> Option<Unavailability> {
        if self.offers(name) {
            None
        } else if self.is_disabled(name) {
            Some(Unavailability::Disabled)
        } else {
            Some(Unavailability::NotOffered)
        }
    }
}

/// A command that an application needs, for [`Qmp::require`] and
/// [`Qga::require`](qga::Qga::require):
///
/// ```
/// use qapi::commands::{QueryBlock, QueryStatus};
/// use qapi_protocol::Required;
/// let needed = [Required::of::<QueryStatus>(), Required::of::<QueryBlock>()];
/// assert_eq!(needed[0].name(), "query-status");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Required {
    name: &'static str,
    since: (u32, u32, u32),
    removed: Option<(u32, u32, u32)>,
}

impl Required {
    /// The typed command `C`, with its lifetime.
    pub fn of<C: QapiCommand>() -> Self {
        Self {
            name: C::NAME,
            since: C::AVAILABLE_SINCE,
            removed: C::REMOVED_IN,
        }
    }

    /// The command's name on the wire.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Why a peer of `version`, with the command list `peer` if it gave
    /// one, cannot run the command: its release first, then its list.
    #[cfg(feature = "tokio")]
    pub(crate) fn refusal(
        &self,
        version: (u32, u32, u32),
        peer: Option<&PeerCommands>,
    ) -> Option<Unavailability> {
        if version < self.since || self.removed.is_some_and(|removed| version >= removed) {
            return Some(Unavailability::Release);
        }
        peer.and_then(|peer| peer.refusal(self.name))
    }
}

/// Check every command of `required` against a peer, and list all those it
/// cannot run.
#[cfg(feature = "tokio")]
pub(crate) fn require(
    required: &[Required],
    version: (u32, u32, u32),
    peer: Option<&PeerCommands>,
) -> Result<()> {
    let missing: Vec<MissingCommand> = required
        .iter()
        .filter_map(|command| {
            command.refusal(version, peer).map(|reason| MissingCommand {
                name: command.name.into(),
                reason,
            })
        })
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(Error::MissingCommands { version, missing })
    }
}

/// Where an [`Error::Timeout`] happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimeoutPhase {
    /// Connecting to the socket.
    Connect,
    /// The handshake, which completes before a client is returned.
    Handshake,
    /// Waiting for admission or for the socket, before the client began
    /// writing the request. The peer never saw it, and the connection stays
    /// usable.
    BeforeDispatch,
    /// After the client began writing the request. The command may have run,
    /// and the connection is closed, since its reply could still arrive.
    AfterDispatch,
}

impl fmt::Display for TimeoutPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Connect => "connecting",
            Self::Handshake => "during the handshake",
            Self::BeforeDispatch => "before the request was sent",
            Self::AfterDispatch => "after the request was sent; it may have run",
        })
    }
}

/// Connection limits and deadlines. Start from the defaults and change
/// what you need:
///
/// ```
/// use std::time::Duration;
/// let options = qapi_protocol::ClientOptions::default()
///     .with_command_timeout(Duration::from_secs(5))
///     .with_max_message_bytes(1024 * 1024);
/// assert_eq!(options.command_timeout, Duration::from_secs(5));
/// ```
///
/// Timeouts cover the entire operation, including admission and socket writes.
/// Message size counts JSON bytes, excluding the line terminator.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ClientOptions {
    pub connect_timeout: Duration,
    pub handshake_timeout: Duration,
    pub command_timeout: Duration,
    /// The largest message accepted from the peer, and the largest request.
    pub max_message_bytes: usize,
    /// Large events are dropped instead of filling the event queue with blobs.
    pub max_event_bytes: usize,
    /// QMP events are bounded and never block command responses.
    pub event_capacity: usize,
    /// Learn the peer's commands while connecting, so that a command it
    /// does not list fails before anything is sent: QMP sends one
    /// `query-commands` after `qmp_capabilities`, and a guest agent's
    /// commands come with the `guest-info` that the client reads anyway.
    /// Turning it off saves that one QMP round trip and keeps only the
    /// checks against the peer's release.
    pub probe_commands: bool,
}

impl Default for ClientOptions {
    /// 5 seconds to connect, 10 for the handshake and 30 per command;
    /// 8 MiB per message, 1 MiB per event and 128 buffered events; the
    /// peer's commands are learned while connecting.
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            handshake_timeout: Duration::from_secs(10),
            command_timeout: Duration::from_secs(30),
            max_message_bytes: 8 * 1024 * 1024,
            max_event_bytes: 1024 * 1024,
            event_capacity: 128,
            probe_commands: true,
        }
    }
}

impl ClientOptions {
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }
    pub fn with_handshake_timeout(mut self, timeout: Duration) -> Self {
        self.handshake_timeout = timeout;
        self
    }
    pub fn with_command_timeout(mut self, timeout: Duration) -> Self {
        self.command_timeout = timeout;
        self
    }
    pub fn with_max_message_bytes(mut self, bytes: usize) -> Self {
        self.max_message_bytes = bytes;
        self
    }
    pub fn with_max_event_bytes(mut self, bytes: usize) -> Self {
        self.max_event_bytes = bytes;
        self
    }
    pub fn with_event_capacity(mut self, events: usize) -> Self {
        self.event_capacity = events;
        self
    }
    pub fn with_probe_commands(mut self, probe: bool) -> Self {
        self.probe_commands = probe;
        self
    }

    #[cfg(feature = "tokio")]
    pub(crate) fn validate(&self) -> Result<()> {
        let now = tokio::time::Instant::now();
        let bad_timeout =
            |timeout: Duration| timeout.is_zero() || now.checked_add(timeout).is_none();
        let invalid = [
            ("connect_timeout", bad_timeout(self.connect_timeout)),
            ("handshake_timeout", bad_timeout(self.handshake_timeout)),
            ("command_timeout", bad_timeout(self.command_timeout)),
            ("max_message_bytes", self.max_message_bytes == 0),
            ("max_event_bytes", self.max_event_bytes == 0),
            (
                "event_capacity",
                self.event_capacity == 0 || self.event_capacity > MAX_EVENT_CAPACITY,
            ),
        ];
        match invalid
            .into_iter()
            .find_map(|(option, bad)| bad.then_some(option))
        {
            Some(option) => Err(Error::InvalidRequest(InvalidRequestKind::Options {
                option,
            })),
            None => Ok(()),
        }
    }
}

/// The largest event capacity that [`ClientOptions`] accepts.
#[cfg(feature = "tokio")]
const MAX_EVENT_CAPACITY: usize = usize::MAX >> 3;
// The event queue is a Tokio channel, which holds at most that many.
#[cfg(feature = "tokio")]
const _: () = assert!(MAX_EVENT_CAPACITY <= tokio::sync::Semaphore::MAX_PERMITS);

/// An error reply from the peer. Unknown error classes from newer or patched
/// peers are kept verbatim.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
#[error("{class}: {desc}")]
#[non_exhaustive]
pub struct RemoteError {
    pub class: String,
    pub desc: String,
}

impl RemoteError {
    pub fn new(class: impl Into<String>, desc: impl Into<String>) -> Self {
        Self {
            class: class.into(),
            desc: desc.into(),
        }
    }

    /// The class as an [`ErrorClass`], or `None` for a class that this
    /// crate does not know, which [`class`](Self::class) keeps verbatim.
    pub fn known_class(&self) -> Option<ErrorClass> {
        Some(match self.class.as_str() {
            "CommandNotFound" => ErrorClass::CommandNotFound,
            "DeviceEncrypted" => ErrorClass::DeviceEncrypted,
            "DeviceNotActive" => ErrorClass::DeviceNotActive,
            "DeviceNotFound" => ErrorClass::DeviceNotFound,
            "GenericError" => ErrorClass::GenericError,
            "KVMMissingCap" => ErrorClass::KvmMissingCap,
            _ => return None,
        })
    }
}

/// The error classes that QEMU and its guest agent report in an error reply,
/// [`RemoteError::class`] as a type.
///
/// Most errors are [`GenericError`](Self::GenericError); the others mark the
/// few conditions that a client may want to tell apart.
///
/// ```
/// use qapi_protocol::{ErrorClass, RemoteError};
///
/// let error = RemoteError::new("KVMMissingCap", "this host has no KVM");
/// assert_eq!(error.known_class(), Some(ErrorClass::KvmMissingCap));
/// assert_eq!(ErrorClass::KvmMissingCap.as_str(), "KVMMissingCap");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorClass {
    /// The peer has no such command, or a guest agent has it disabled.
    CommandNotFound,
    /// The device is encrypted. QEMU stopped reporting this class in 2.10.
    DeviceEncrypted,
    /// The device is not active.
    DeviceNotActive,
    /// The device does not exist.
    DeviceNotFound,
    /// Any other error.
    GenericError,
    /// KVM lacks a capability that the command needs (`KVMMissingCap`).
    KvmMissingCap,
}

impl ErrorClass {
    /// The class as it appears on the wire.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CommandNotFound => "CommandNotFound",
            Self::DeviceEncrypted => "DeviceEncrypted",
            Self::DeviceNotActive => "DeviceNotActive",
            Self::DeviceNotFound => "DeviceNotFound",
            Self::GenericError => "GenericError",
            Self::KvmMissingCap => "KVMMissingCap",
        }
    }
}

/// A QMP event as the peer sent it, with its payload as raw JSON.
///
/// The client cannot know the payload's type, so decoding `data` is left
/// to you, outside the client's version scope. Decode it inside
/// `qapi::with_version(qmp.version(), ...)`: the peer's release then selects
/// the payload layout and checks it, as for replies. Outside any scope a
/// decode takes the newest layout and checks nothing. QEMU 2.8 to 2.11, for
/// example, always sent `BLOCK_IO_ERROR`'s `node-name`, which later releases
/// may omit, and the generated type keeps the two layouts in two fields:
///
/// ```
/// use qapi::events::BlockIoErrorEvent;
/// let event: qapi_protocol::QmpEvent = serde_json::from_value(serde_json::json!({
///     "event": "BLOCK_IO_ERROR",
///     "data": {"device": "", "node-name": "disk0", "operation": "read",
///              "action": "report", "reason": "Input/output error"},
///     "timestamp": {"seconds": 1, "microseconds": 0},
/// }))?;
/// let decode = || serde_json::from_value::<BlockIoErrorEvent>(event.data.clone());
/// let scoped = qapi::with_version((2, 11, 0), decode)?;
/// assert_eq!(scoped.node_name_legacy.as_deref(), Some("disk0"));
/// let unscoped = decode()?;
/// assert_eq!(unscoped.node_name.as_deref(), Some("disk0"));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// [`QmpEvent::new`] builds one, such as an event of the caller's own:
///
/// ```
/// use qapi_protocol::{QmpEvent, Timestamp};
/// let event = QmpEvent::new("POWER", Timestamp::new(-1, -1), serde_json::json!({}));
/// assert_eq!(event.event, "POWER");
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct QmpEvent {
    pub event: String,
    pub timestamp: Timestamp,
    #[serde(default)]
    pub data: serde_json::Value,
}

impl QmpEvent {
    pub fn new(event: impl Into<String>, timestamp: Timestamp, data: serde_json::Value) -> Self {
        Self {
            event: event.into(),
            timestamp,
            data,
        }
    }
}

/// QEMU reports both components as -1 when host time is unavailable.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Timestamp {
    pub seconds: i64,
    pub microseconds: i64,
}

impl Timestamp {
    pub const fn new(seconds: i64, microseconds: i64) -> Self {
        Self {
            seconds,
            microseconds,
        }
    }
}
