//! Sequential guest-agent transport. A timed-out or cancelled operation closes
//! its socket: the next connection must perform guest-sync-delimited again.
//! That command needs qemu-ga 1.1 or later; an agent that lacks or disables
//! it fails the handshake with [`Error::SyncUnsupported`].
//!
//! A guest agent runs inside the guest and is not trusted. Replies are decoded
//! straight from their bytes into the command's return type, and are limited
//! to [`ClientOptions::max_message_bytes`]; raise the limit for a single call
//! with [`Qga::execute_with_reply_limit`], such as a large `guest-file-read`.
//! Requests are limited to it too, so write a large file in `guest-file-write`
//! chunks: under the default limit, a chunk of up to 6 MiB less 1 KiB fits.
//!
//! The `guest-info` that the handshake reads also lists the agent's commands,
//! with those it has disabled; a command it disables or does not list fails
//! before anything is sent ([`Qga::commands`]).
use crate::{
    ClientOptions, Error, InvalidRequestKind, PeerCommands, Required, Result,
    TimeoutPhase,
    engine::{
        self,
        qga::{QgaCall, QgaHandshake},
    },
    framing::{self, JsonReader},
    wire,
};
use qapi_core::QapiCommand;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::path::Path;
use tokio::{
    io::AsyncWriteExt,
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
    time::timeout,
};

/// The largest reply of `guest-file-read`: the base64 of its 48 MiB maximum
/// `count`, plus the envelope. Pass it to [`Qga::execute_with_reply_limit`]
/// to read files in chunks of any size the agent allows.
pub const FILE_READ_REPLY_BYTES: usize = 64 * 1024 * 1024 + 1024;

/// The random nonce that the handshake synchronizes with.
fn sync_nonce() -> Result<i64> {
    let mut nonce = [0_u8; 8];
    getrandom::fill(&mut nonce).map_err(|error| {
        Error::SyncNonce(error.raw_os_error().map_or_else(
            || std::io::Error::other(error.to_string()),
            std::io::Error::from_raw_os_error,
        ))
    })?;
    Ok(i64::from_ne_bytes(nonce))
}

struct Session {
    reader: JsonReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

pub struct Qga {
    version: (u32, u32, u32),
    commands: Option<PeerCommands>,
    options: ClientOptions,
    session: Option<Session>,
}

impl std::fmt::Debug for Qga {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Qga")
            .field("version", &self.version)
            .field("closed", &self.is_closed())
            .finish()
    }
}

impl Qga {
    pub async fn new(path: &Path) -> Result<Self> {
        Self::connect_with_options(path, ClientOptions::default()).await
    }

    pub async fn connect_with_options(
        path: &Path,
        options: ClientOptions,
    ) -> Result<Self> {
        options.validate()?;
        let stream =
            timeout(options.connect_timeout, UnixStream::connect(path))
                .await
                .map_err(|_| Error::Timeout {
                    phase: TimeoutPhase::Connect,
                })??;
        Self::from_stream_with_options(stream, options).await
    }

    pub async fn from_stream(stream: UnixStream) -> Result<Self> {
        Self::from_stream_with_options(stream, ClientOptions::default()).await
    }

    pub async fn from_stream_with_options(
        stream: UnixStream,
        options: ClientOptions,
    ) -> Result<Self> {
        options.validate()?;
        let (reader, writer) = stream.into_split();
        let mut session = Session {
            reader: JsonReader::new(reader, options.max_message_bytes),
            writer,
        };
        let (version, commands) = timeout(options.handshake_timeout, async {
            let mut handshake = QgaHandshake::new(sync_nonce()?, &options)?;
            loop {
                if let Some(frame) = handshake.poll_transmit() {
                    session.writer.write_all(&frame).await?;
                }
                if handshake.is_done() {
                    break;
                }
                if handshake.is_syncing() {
                    handshake.handle_sync_line(
                        session.reader.sync_message().await?,
                    )?;
                } else {
                    handshake.handle_frame(&session.reader.next().await?)?;
                }
            }
            Ok::<_, Error>(handshake.finish())
        })
        .await
        .map_err(|_| Error::Timeout { phase: TimeoutPhase::Handshake })??;
        Ok(Self { version, commands, options, session: Some(session) })
    }

    pub fn version(&self) -> (u32, u32, u32) {
        self.version
    }

    /// The commands the agent listed in the `guest-info` that the client
    /// read while connecting, with those it has disabled. `None` when
    /// [`ClientOptions::probe_commands`] was off or the reply listed no
    /// commands; the calls are then checked against the agent's release
    /// alone.
    pub fn commands(&self) -> Option<&PeerCommands> {
        self.commands.as_ref()
    }

    /// Whether the agent can run `C`: its release has the command and, when
    /// the agent listed its commands, it lists it as enabled.
    pub fn supports<C: QapiCommand>(&self) -> bool {
        Required::of::<C>().refusal(self.version, self.commands()).is_none()
    }

    /// Check that the agent can run every command in `required`, as
    /// [`supports`](Self::supports) does, without sending anything. The
    /// error lists all those it cannot run.
    pub fn require(&self, required: &[Required]) -> Result<()> {
        crate::require(required, self.version, self.commands())
    }
    pub fn is_closed(&self) -> bool {
        self.session.is_none()
    }
    /// Close the socket. Later calls fail with [`Error::Closed`].
    ///
    /// It is synchronous and takes `&mut self`, unlike the shared QMP
    /// client's async [`Qmp::shutdown`](crate::Qmp::shutdown): a guest agent
    /// client is owned and sequential, with no task or pending caller to
    /// wait for, so dropping its socket is the whole shutdown.
    pub fn shutdown(&mut self) {
        self.session.take();
    }

    /// Commands with success-response:false must use send_no_response, which
    /// explicitly acknowledges delivery rather than claiming remote success.
    pub async fn execute<C: QapiCommand>(
        &mut self,
        cmd: &C,
    ) -> Result<C::Returns> {
        self.execute_with_reply_limit(cmd, self.options.max_message_bytes).await
    }

    /// [`execute`](Self::execute), accepting a reply of up to
    /// `max_reply_bytes` for this call only, such as
    /// [`FILE_READ_REPLY_BYTES`] for `guest-file-read`.
    pub async fn execute_with_reply_limit<C: QapiCommand>(
        &mut self,
        cmd: &C,
        max_reply_bytes: usize,
    ) -> Result<C::Returns> {
        if !C::sends_success_response(self.version) {
            return Err(crate::qmp::no_success_response::<C>());
        }
        let request = wire::command(cmd, self.version, self.commands(), false)?;
        self.call(request, max_reply_bytes).await
    }

    /// Run the command `name` with `args`, a JSON object (`{}` for none),
    /// and decode its reply into `T`, such as a `serde_json::Value`, in the
    /// agent's version scope, as [`Qmp::execute_raw`](crate::Qmp::execute_raw)
    /// does. A name that the agent's [`commands`](Self::commands) do not
    /// list as enabled fails before anything is sent.
    pub async fn execute_raw<T: DeserializeOwned + 'static>(
        &mut self,
        name: &str,
        args: Value,
    ) -> Result<T> {
        let request = wire::request(name, args, false)?;
        wire::offered(name, self.version, self.commands())?;
        self.call(request, self.options.max_message_bytes).await
    }

    async fn call<T: DeserializeOwned + 'static>(
        &mut self,
        request: Value,
        max_reply_bytes: usize,
    ) -> Result<T> {
        let mut call = QgaCall::new(
            &request,
            self.options.max_message_bytes,
            max_reply_bytes,
        )?;
        // Ownership leaves self before any I/O await. Cancellation drops the
        // session, so a late reply cannot be mistaken for the next command.
        let mut session = self.session.take().ok_or(Error::Closed)?;
        let operation = async {
            framing::write_call(&mut session.writer, &mut call).await?;
            session.reader.next_within(max_reply_bytes).await
        };
        let reply = timeout(self.options.command_timeout, operation).await;
        let reply = match reply {
            Ok(reply) => reply?,
            Err(_) => {
                let (error, aligned) = call.timeout();
                if aligned {
                    self.session = Some(session);
                }
                return Err(error);
            }
        };
        let (raw, aligned) = engine::qga::reply(&reply);
        if aligned {
            self.session = Some(session);
        }
        qapi_core::with_version(self.version, || wire::decode(raw?))
    }

    /// Send an explicitly response-less command and close the connection.
    /// Success only means the bytes were written. The agent can still fail the
    /// command or shut down later, and no remote success acknowledgement
    /// exists.
    pub async fn send_no_response<C: QapiCommand<Returns = ()>>(
        &mut self,
        cmd: &C,
    ) -> Result<()> {
        if C::sends_success_response(self.version) {
            return Err(Error::InvalidRequest(
                InvalidRequestKind::HasResponse { command: C::NAME.into() },
            ));
        }
        let request = wire::command(cmd, self.version, self.commands(), false)?;
        let mut call =
            QgaCall::without_reply(&request, self.options.max_message_bytes)?;
        let mut session = self.session.take().ok_or(Error::Closed)?;
        let written = timeout(
            self.options.command_timeout,
            framing::write_call(&mut session.writer, &mut call),
        )
        .await;
        match written {
            Ok(result) => result,
            Err(_) => {
                let (error, aligned) = call.timeout();
                if aligned {
                    self.session = Some(session);
                }
                Err(error)
            }
        }
    }
}
