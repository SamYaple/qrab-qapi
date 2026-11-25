#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
//!
//! Each connection owns its version and lifecycle. [`Qmp`] describes the
//! QMP client's cancellation and timeout semantics.
//! [`Error`] says which failures leave a connection usable. The
//! repository's `docs/protocol.md` describes the client lifecycle, the mock
//! server API and the internals, and `docs/testing.md` the tests.

mod framing;
mod qmp;
#[cfg(feature = "mock")]
pub mod server;
mod wire;

pub use qmp::{Greeting, QMP_QUEUE_CAPACITY, QemuVersion, Qmp, VersionInfo};
use std::{fmt, sync::Arc, time::Duration};

/// Why a connection or a call failed.
///
/// [`Remote`](Error::Remote), [`Decode`](Error::Decode) and the failures
/// detected before any byte is sent ([`Unavailable`](Error::Unavailable),
/// [`Encode`](Error::Encode), [`InvalidRequest`](Error::InvalidRequest) and
/// a [`Timeout`](Error::Timeout) in [`TimeoutPhase::BeforeDispatch`]) leave
/// the connection usable. Every other failure closes it.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The peer ran the command and answered with an error reply.
    #[error("command failed: {0}")]
    Remote(#[from] RemoteError),
    /// The command, or a type, variant or field it uses, does not exist in
    /// the peer's release. Nothing was sent.
    #[error("{name} is not available in QEMU {}.{}.{}", .version.0, .version.1, .version.2)]
    Unavailable {
        /// The QAPI command name, or the Rust name of the type, variant or
        /// field, as [`qapi::EncodeError::Unavailable`] names it.
        name: String,
        /// The peer's version.
        version: (u32, u32, u32),
    },
    /// The request could not be serialized for another reason than
    /// [`Unavailable`](Error::Unavailable). Nothing was sent.
    #[error("cannot encode the request: {0}")]
    Encode(#[source] qapi::EncodeError),
    /// A well-formed reply does not match the expected return type.
    #[error("cannot decode the reply: {0}")]
    Decode(#[source] serde_json::Error),
    /// The peer sent bytes that are not valid JSON.
    #[error("malformed JSON from the peer: {0}")]
    Malformed(#[source] serde_json::Error),
    /// The peer sent a message larger than the limit in effect. The client
    /// stops reading it at the limit.
    #[error("message from the peer exceeds {limit} bytes")]
    MessageTooLarge {
        /// The limit, in JSON bytes.
        limit: usize,
    },
    /// The peer sent valid JSON that breaks the protocol.
    #[error("protocol violation: {0}")]
    Protocol(String),
    /// A deadline elapsed; the phase says whether the request reached the peer.
    #[error("timed out {phase}")]
    Timeout { phase: TimeoutPhase },
    /// Connecting, reading or writing failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The request or the options are invalid. Nothing was sent.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// The connection was already closed.
    #[error("connection closed")]
    Closed,
    /// The connection failed while the call was pending. The original
    /// failure is shared with every pending caller.
    #[error("connection lost: {0}")]
    Disconnected(#[source] Arc<Error>),
}

pub type Result<T> = std::result::Result<T, Error>;

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
}

impl Default for ClientOptions {
    /// 5 seconds to connect, 10 for the handshake and 30 per command;
    /// 8 MiB per message, 1 MiB per event and 128 buffered events.
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            handshake_timeout: Duration::from_secs(10),
            command_timeout: Duration::from_secs(30),
            max_message_bytes: 8 * 1024 * 1024,
            max_event_bytes: 1024 * 1024,
            event_capacity: 128,
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

    pub(crate) fn validate(&self) -> Result<()> {
        if self.connect_timeout.is_zero()
            || self.handshake_timeout.is_zero()
            || self.command_timeout.is_zero()
            || self.max_message_bytes == 0
            || self.max_event_bytes == 0
            || self.event_capacity == 0
            || self.event_capacity > tokio::sync::Semaphore::MAX_PERMITS
            || [
                self.connect_timeout,
                self.handshake_timeout,
                self.command_timeout,
            ]
            .iter()
            .any(|duration| tokio::time::Instant::now().checked_add(*duration).is_none())
        {
            return Err(Error::InvalidRequest(
                "limits and timeouts must be nonzero and capacities valid".into(),
            ));
        }
        Ok(())
    }
}

/// An error reply from the peer. Unknown error classes from newer or patched
/// peers are kept verbatim.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
#[error("{class}: {desc}")]
pub struct RemoteError {
    pub class: String,
    pub desc: String,
}

impl RemoteError {
    pub fn known_class(&self) -> Option<qapi::QapiErrorClass> {
        serde_json::from_value(serde_json::Value::String(self.class.clone())).ok()
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QmpEvent {
    pub event: String,
    pub timestamp: Timestamp,
    #[serde(default)]
    pub data: serde_json::Value,
}

/// QEMU reports both components as -1 when host time is unavailable.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Timestamp {
    pub seconds: i64,
    pub microseconds: i64,
}
