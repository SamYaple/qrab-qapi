//! Bounded mock sessions for tests, using the same JSON framing as the
//! clients. Enabled by the `mock` feature; add it to the dev-dependency:
//!
//! ```toml
//! [dev-dependencies.qapi-protocol]
//! package = "qrab-qapi-protocol"
//! version = "0.1"
//! features = ["mock"]
//! ```
use crate::{
    Error, ProtocolViolation, Result, engine,
    framing::{self, JsonReader},
};
use serde_json::{Value, json};
use tokio::{
    io::AsyncWriteExt,
    net::{
        UnixListener, UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
};

/// A command a mock session received.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ParsedCommand {
    pub name: String,
    pub arguments: Value,
    pub oob: bool,
    pub id: Option<Value>,
}

/// A request, without its ID, has the verb, a nonempty command name and
/// at most object arguments.
fn validate(value: &Value, oob: bool) -> Result<()> {
    let verb = if oob { "exec-oob" } else { "execute" };
    let object = value
        .as_object()
        .ok_or(Error::Protocol(ProtocolViolation::NotAnObject(None)))?;
    if !object.get(verb).and_then(Value::as_str).is_some_and(|s| !s.is_empty())
        || object.keys().any(|k| k != verb && k != "arguments")
        || object.get("arguments").is_some_and(|v| !v.is_object())
    {
        return Err(Error::Protocol(ProtocolViolation::InvalidCommand {
            verb,
        }));
    }
    Ok(())
}

/// Refuse any other command than an in-band `expected`.
fn expect(command: &ParsedCommand, expected: &'static str) -> Result<()> {
    if command.name != expected || command.oob {
        return Err(Error::Protocol(ProtocolViolation::UnexpectedCommand {
            expected,
            found: command.name.clone(),
            oob: command.oob,
        }));
    }
    Ok(())
}

/// The largest message a mock session reads or writes.
const LIMIT: usize = 64 * 1024 * 1024 + 1024;

/// The reading half of a [`QmpSession`], from [`QmpSession::split`], for a
/// mock that reads commands in one task and writes in another.
pub struct QmpReader {
    reader: JsonReader<OwnedReadHalf>,
    // Whether the client enabled OOB; `exec-oob` is refused until it has.
    oob: bool,
}

impl std::fmt::Debug for QmpReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QmpReader")
            .field("oob", &self.oob)
            .finish_non_exhaustive()
    }
}

/// The writing half of a [`QmpSession`], from [`QmpSession::split`].
#[derive(Debug)]
pub struct QmpWriter {
    writer: OwnedWriteHalf,
}

impl QmpReader {
    pub async fn recv_command_full(&mut self) -> Result<ParsedCommand> {
        let bytes = self.reader.next().await?;
        let mut value: Value =
            serde_json::from_slice(&bytes).map_err(Error::Malformed)?;
        let id = value.as_object_mut().and_then(|o| o.remove("id"));
        let oob = value.get("exec-oob").is_some();
        validate(&value, oob)?;
        if oob && !self.oob {
            return Err(Error::Protocol(ProtocolViolation::OobNotEnabled));
        }
        let verb = if oob { "exec-oob" } else { "execute" };
        Ok(ParsedCommand {
            name: value[verb].as_str().unwrap().to_owned(),
            arguments: value
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({})),
            oob,
            id,
        })
    }
}

impl QmpWriter {
    async fn send(&mut self, value: Value) -> Result<()> {
        framing::write_json(&mut self.writer, &value, LIMIT).await
    }
    pub async fn send_return_with_id(
        &mut self,
        value: Value,
        id: Option<Value>,
    ) -> Result<()> {
        let mut reply = json!({"return": value});
        if let Some(id) = id {
            reply["id"] = id;
        }
        self.send(reply).await
    }
    async fn send_error_with_id(
        &mut self,
        class: &str,
        desc: &str,
        id: Option<Value>,
    ) -> Result<()> {
        let mut reply = json!({"error": {"class": class, "desc": desc}});
        if let Some(id) = id {
            reply["id"] = id;
        }
        self.send(reply).await
    }
    pub async fn send_event(&mut self, name: &str, data: Value) -> Result<()> {
        self.send(
            json!({"event": name, "data": data, "timestamp": {"seconds": 0, "microseconds": 0}}),
        )
        .await
    }
}

/// A mock QMP server on one connection.
///
/// [`negotiate`](Self::negotiate) also answers the `query-commands` that a
/// client sends right after `qmp_capabilities`
/// ([`ClientOptions::probe_commands`](crate::ClientOptions::probe_commands)),
/// so later reads see only the commands of the test. By default it answers
/// with a `CommandNotFound` error, which leaves the client checking calls
/// against the release alone; [`offer_commands`](Self::offer_commands)
/// answers with a list instead.
#[derive(Debug)]
pub struct QmpSession {
    reader: QmpReader,
    writer: QmpWriter,
    last_id: Option<Value>,
    offered: Option<Vec<String>>,
    expect_probe: bool,
    probes: usize,
    // The first command after negotiation, when it was not the probe.
    pending: Option<ParsedCommand>,
}

impl QmpSession {
    pub async fn accept(listener: &UnixListener) -> Result<Self> {
        Ok(Self::from_stream(listener.accept().await?.0))
    }
    pub fn from_stream(stream: UnixStream) -> Self {
        let (read, write) = stream.into_split();
        Self {
            reader: QmpReader {
                reader: JsonReader::new(read, LIMIT),
                oob: false,
            },
            writer: QmpWriter { writer: write },
            last_id: None,
            offered: None,
            expect_probe: true,
            probes: 0,
            pending: None,
        }
    }
    /// Split the session into halves. The halves answer nothing on their
    /// own: split after [`negotiate`](Self::negotiate), which answers the
    /// client's `query-commands`.
    pub fn split(self) -> (QmpReader, QmpWriter) {
        (self.reader, self.writer)
    }
    /// Answer the client's `query-commands` with these command names.
    pub fn offer_commands<I, S>(&mut self, names: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.offered = Some(names.into_iter().map(Into::into).collect());
    }
    /// Whether [`negotiate`](Self::negotiate) waits for the client's
    /// `query-commands`, as by default. Turn it off for a client whose
    /// options turn the query off, when the test awaits the handshake
    /// before the client sends a command.
    pub fn expect_probe(&mut self, expect: bool) {
        self.expect_probe = expect;
    }
    /// How many `query-commands` probes the session answered.
    pub fn probes(&self) -> usize {
        self.probes
    }
    pub async fn send_greeting(
        &mut self,
        major: u32,
        minor: u32,
        micro: u32,
    ) -> Result<()> {
        self.writer.send(json!({"QMP": {"version": {"qemu": {"major": major, "minor": minor, "micro": micro}, "package": ""}, "capabilities": ["oob"]}})).await
    }
    /// Answer `qmp_capabilities`, then the client's `query-commands`. The
    /// greeting advertises OOB, which the client may enable; later
    /// `exec-oob` commands are refused unless it did. A first command that
    /// is not the probe, from a client that does not send it, is kept for
    /// the next read.
    pub async fn negotiate(&mut self) -> Result<()> {
        let command = self.recv_command_full().await?;
        expect(&command, "qmp_capabilities")?;
        match command.arguments.get("enable") {
            None => {}
            Some(enable) if enable == &json!(["oob"]) => self.reader.oob = true,
            Some(_) => {
                return Err(Error::Protocol(
                    ProtocolViolation::UnsupportedCapabilities,
                ));
            }
        }
        self.send_return(json!({})).await?;
        if !self.expect_probe {
            return Ok(());
        }
        let next = self.reader.recv_command_full().await?;
        // The client sends its probe with the handshake's ID 0; its later
        // requests, a query-commands of its own included, count from 1.
        if next.name != "query-commands"
            || next.oob
            || next.id != Some(json!(0))
        {
            self.pending = Some(next);
            return Ok(());
        }
        self.probes += 1;
        match &self.offered {
            Some(names) => {
                let names: Vec<Value> =
                    names.iter().map(|name| json!({"name": name})).collect();
                self.writer
                    .send_return_with_id(Value::Array(names), next.id)
                    .await
            }
            None => {
                self.writer
                    .send_error_with_id(
                        "CommandNotFound",
                        "The command query-commands has not been found",
                        next.id,
                    )
                    .await
            }
        }
    }
    pub async fn recv_command(&mut self) -> Result<(String, Value)> {
        let command = self.recv_command_full().await?;
        Ok((command.name, command.arguments))
    }
    pub async fn recv_command_full(&mut self) -> Result<ParsedCommand> {
        let command = match self.pending.take() {
            Some(command) => command,
            None => self.reader.recv_command_full().await?,
        };
        self.last_id = command.id.clone();
        Ok(command)
    }
    /// Replies to the most recently received command. For pipelined mocks,
    /// preserve each ParsedCommand.id and use send_return_with_id instead.
    pub async fn send_return(&mut self, value: Value) -> Result<()> {
        self.writer.send_return_with_id(value, self.last_id.clone()).await
    }
    pub async fn send_return_with_id(
        &mut self,
        value: Value,
        id: Option<Value>,
    ) -> Result<()> {
        self.writer.send_return_with_id(value, id).await
    }
    pub async fn send_error(&mut self, class: &str, desc: &str) -> Result<()> {
        self.writer.send_error_with_id(class, desc, self.last_id.clone()).await
    }
    pub async fn send_event(&mut self, name: &str, data: Value) -> Result<()> {
        self.writer.send_event(name, data).await
    }
}

/// A mock guest agent on one connection.
#[derive(Debug)]
pub struct QgaSession {
    reader: QmpReader,
    writer: QmpWriter,
}
impl QgaSession {
    pub async fn accept(listener: &UnixListener) -> Result<Self> {
        Ok(Self::from_stream(listener.accept().await?.0))
    }
    pub fn from_stream(stream: UnixStream) -> Self {
        let session = QmpSession::from_stream(stream);
        Self { reader: session.reader, writer: session.writer }
    }
    pub async fn handle_sync(&mut self) -> Result<i64> {
        self.reader.reader.reset_at_sentinel().await?;
        let command = self.reader.recv_command_full().await?;
        expect(&command, "guest-sync-delimited")?;
        let nonce = command.arguments["id"]
            .as_i64()
            .ok_or(Error::Protocol(ProtocolViolation::MissingSyncNonce))?;
        let mut bytes = vec![0xff];
        bytes.extend(engine::encode(&json!({"return": nonce}), LIMIT)?);
        self.writer.writer.write_all(&bytes).await?;
        Ok(nonce)
    }
    pub async fn recv_command(&mut self) -> Result<(String, Value)> {
        let command = self.reader.recv_command_full().await?;
        if command.oob {
            return Err(Error::Protocol(ProtocolViolation::GuestAgentOob));
        }
        Ok((command.name, command.arguments))
    }
    pub async fn send_return(&mut self, value: Value) -> Result<()> {
        self.writer.send_return_with_id(value, None).await
    }
    pub async fn send_error(&mut self, class: &str, desc: &str) -> Result<()> {
        self.writer.send_error_with_id(class, desc, None).await
    }
}
