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
    ClientOptions, Error, InvalidRequestKind, PeerCommands, RemoteError, Required, Result,
    TimeoutPhase,
    framing::{self, JsonReader, SyncLine},
    wire,
};
use qapi_core::QapiCommand;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
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

/// The error reply of an agent that lacks or blocks `guest-sync-delimited`:
/// QEMU reports both as `CommandNotFound`, naming the command.
fn sync_refusal(line: &[u8]) -> Option<RemoteError> {
    let envelope = wire::envelope(line).ok()?;
    match wire::response(&envelope) {
        Err(Error::Remote(error))
            if error.class == "CommandNotFound" && error.desc.contains("guest-sync-delimited") =>
        {
            Some(error)
        }
        _ => None,
    }
}

/// The part of a `guest-info` reply that the handshake reads. The reply's
/// shape has not changed since qemu-ga 1.1, so the client decodes it
/// itself rather than through the generated types.
#[derive(serde::Deserialize)]
struct GuestInfo {
    version: String,
    /// qemu-ga 1.1 and later list their commands; 0.15 sends only the version.
    #[serde(default)]
    supported_commands: Option<Vec<GuestCommand>>,
}

/// One entry of `guest-info`'s `supported_commands`.
#[derive(serde::Deserialize)]
struct GuestCommand {
    name: String,
    enabled: bool,
}

/// The commands of a `guest-info` reply, enabled and disabled. A list that
/// does not name `guest-info` itself, which every agent that answers it
/// has, is no command list, as from a mock that sends none.
fn agent_commands(info: &GuestInfo) -> Option<PeerCommands> {
    let listed = info.supported_commands.as_deref()?;
    if !listed
        .iter()
        .any(|command| command.name == "guest-info" && command.enabled)
    {
        return None;
    }
    let (enabled, disabled): (Vec<_>, Vec<_>) = listed.iter().partition(|command| command.enabled);
    Some(PeerCommands::new(
        enabled.into_iter().map(|command| command.name.clone()),
        disabled.into_iter().map(|command| command.name.clone()),
    ))
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

    pub async fn connect_with_options(path: &Path, options: ClientOptions) -> Result<Self> {
        options.validate()?;
        let stream = timeout(options.connect_timeout, UnixStream::connect(path))
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
            let mut nonce = [0_u8; 8];
            getrandom::fill(&mut nonce).map_err(|error| {
                Error::SyncNonce(error.raw_os_error().map_or_else(
                    || std::io::Error::other(error.to_string()),
                    std::io::Error::from_raw_os_error,
                ))
            })?;
            let nonce = i64::from_ne_bytes(nonce);
            let command = wire::request("guest-sync-delimited", json!({"id": nonce}), false)?;
            let mut bytes = vec![0xff];
            bytes.extend(framing::encode(&command, options.max_message_bytes)?);
            session.writer.write_all(&bytes).await?;
            loop {
                match session.reader.sync_message().await? {
                    SyncLine::Sentineled(candidate) => {
                        let synchronized = wire::envelope(&candidate)
                            .and_then(|envelope| {
                                wire::response(&envelope).and_then(wire::decode::<i64>)
                            })
                            .is_ok_and(|reply| reply == nonce);
                        if synchronized {
                            break;
                        }
                    }
                    // Only the command running sends a sentinel, so an
                    // agent that cannot run it answers without one.
                    SyncLine::Bare(line) => {
                        if let Some(error) = sync_refusal(&line) {
                            return Err(Error::SyncUnsupported(error));
                        }
                    }
                }
            }
            let info = wire::request("guest-info", json!({}), false)?;
            framing::write_json(&mut session.writer, &info, options.max_message_bytes).await?;
            let reply = session.reader.next().await?;
            let info: GuestInfo = wire::decode(wire::response(&wire::envelope(&reply)?)?)?;
            let version =
                qapi_core::parse_qga_version(&info.version).map_err(Error::AgentVersion)?;
            let commands = options
                .probe_commands
                .then(|| agent_commands(&info))
                .flatten();
            Ok((version, commands))
        })
        .await
        .map_err(|_| Error::Timeout {
            phase: TimeoutPhase::Handshake,
        })??;
        Ok(Self {
            version,
            commands,
            options,
            session: Some(session),
        })
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
        Required::of::<C>()
            .refusal(self.version, self.commands())
            .is_none()
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
    pub async fn execute<C: QapiCommand>(&mut self, cmd: &C) -> Result<C::Returns> {
        self.execute_with_reply_limit(cmd, self.options.max_message_bytes)
            .await
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
        if max_reply_bytes == 0 {
            return Err(Error::InvalidRequest(InvalidRequestKind::ZeroReplyLimit));
        }
        let bytes = framing::encode(&request, self.options.max_message_bytes)?;
        // Ownership leaves self before any I/O await. Cancellation drops the
        // session, so a late reply cannot be mistaken for the next command.
        let mut session = self.session.take().ok_or(Error::Closed)?;
        let mut sent = false;
        let operation = async {
            framing::write_frame(&mut session.writer, &bytes, &mut sent).await?;
            session.reader.next_within(max_reply_bytes).await
        };
        let reply = timeout(self.options.command_timeout, operation).await;
        let reply = match reply {
            Ok(reply) => reply?,
            Err(_) if !sent => {
                // Nothing was written, so the session is still aligned.
                self.session = Some(session);
                return Err(Error::Timeout {
                    phase: TimeoutPhase::BeforeDispatch,
                });
            }
            Err(_) => {
                return Err(Error::Timeout {
                    phase: TimeoutPhase::AfterDispatch,
                });
            }
        };
        // A complete, well-formed reply leaves the session aligned, whether it
        // is an error reply or fails to decode into the return type.
        let raw = wire::response(&wire::envelope(&reply)?);
        if matches!(raw, Ok(_) | Err(Error::Remote(_))) {
            self.session = Some(session);
        }
        qapi_core::with_version(self.version, || wire::decode(raw?))
    }

    /// Send an explicitly response-less command and close the connection.
    /// Success only means the bytes were written. The agent can still fail the
    /// command or shut down later, and no remote success acknowledgement exists.
    pub async fn send_no_response<C: QapiCommand<Returns = ()>>(&mut self, cmd: &C) -> Result<()> {
        if C::sends_success_response(self.version) {
            return Err(Error::InvalidRequest(InvalidRequestKind::HasResponse {
                command: C::NAME.into(),
            }));
        }
        let request = wire::command(cmd, self.version, self.commands(), false)?;
        let bytes = framing::encode(&request, self.options.max_message_bytes)?;
        let mut session = self.session.take().ok_or(Error::Closed)?;
        let mut sent = false;
        let written = timeout(
            self.options.command_timeout,
            framing::write_frame(&mut session.writer, &bytes, &mut sent),
        )
        .await;
        match written {
            Ok(result) => result,
            Err(_) if !sent => {
                self.session = Some(session);
                Err(Error::Timeout {
                    phase: TimeoutPhase::BeforeDispatch,
                })
            }
            Err(_) => Err(Error::Timeout {
                phase: TimeoutPhase::AfterDispatch,
            }),
        }
    }
}
