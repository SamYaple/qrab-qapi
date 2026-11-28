//! The guest agent protocol: the sync handshake and the sequential calls.
use crate::{
    ClientOptions, Error, InvalidRequestKind, PeerCommands, RemoteError,
    Result, TimeoutPhase,
    engine::{SyncLine, encode},
    wire,
};
use serde_json::{Value, json, value::RawValue};

/// The error reply of an agent that lacks or blocks `guest-sync-delimited`:
/// QEMU reports both as `CommandNotFound`, naming the command.
fn sync_refusal(line: &[u8]) -> Option<RemoteError> {
    let envelope = wire::envelope(line).ok()?;
    match wire::response(&envelope) {
        Err(Error::Remote(error))
            if error.class == "CommandNotFound"
                && error.desc.contains("guest-sync-delimited") =>
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
    let (enabled, disabled): (Vec<_>, Vec<_>) =
        listed.iter().partition(|command| command.enabled);
    Some(PeerCommands::new(
        enabled.into_iter().map(|command| command.name.clone()),
        disabled.into_iter().map(|command| command.name.clone()),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Sync,
    Info,
    Done,
}

/// The guest agent handshake, under one deadline that the client enforces:
/// a sentinel and `guest-sync-delimited` with the client's random `nonce`,
/// the scan for the reply that echoes it, then `guest-info`.
///
/// The client writes each frame of [`poll_transmit`](Self::poll_transmit)
/// before it reads on. While [`is_syncing`](Self::is_syncing), it passes the
/// lines of the sentinel scan ([`Inbound::sync_line`](super::Inbound::sync_line))
/// to [`handle_sync_line`](Self::handle_sync_line), and then the messages
/// to [`handle_frame`](Self::handle_frame), until [`is_done`](Self::is_done).
/// Any error ends the connection.
pub(crate) struct QgaHandshake {
    step: Step,
    nonce: i64,
    probe: bool,
    limit: usize,
    transmit: Option<Vec<u8>>,
    version: (u32, u32, u32),
    commands: Option<PeerCommands>,
}

impl QgaHandshake {
    pub fn new(nonce: i64, options: &ClientOptions) -> Result<Self> {
        let command =
            wire::request("guest-sync-delimited", json!({"id": nonce}), false)?;
        let mut bytes = vec![0xff];
        bytes.extend(encode(&command, options.max_message_bytes)?);
        Ok(Self {
            step: Step::Sync,
            nonce,
            probe: options.probe_commands,
            limit: options.max_message_bytes,
            transmit: Some(bytes),
            version: (0, 0, 0),
            commands: None,
        })
    }

    /// The next frame to write, whole, before reading on.
    pub fn poll_transmit(&mut self) -> Option<Vec<u8>> {
        self.transmit.take()
    }

    /// Whether the next input is a line of the sentinel scan.
    pub fn is_syncing(&self) -> bool {
        self.step == Step::Sync
    }

    pub fn is_done(&self) -> bool {
        self.step == Step::Done
    }

    /// A line of the sentinel scan. Only the reply that echoes the nonce
    /// ends it; stale or malformed lines are ignored. Only the command
    /// running sends a sentinel, so an agent that cannot run it answers
    /// without one.
    pub fn handle_sync_line(&mut self, line: SyncLine) -> Result<()> {
        debug_assert!(self.is_syncing(), "the sync is done");
        debug_assert!(
            self.transmit.is_none(),
            "write the frame before reading on"
        );
        match line {
            SyncLine::Sentineled(candidate) => {
                let synchronized = wire::envelope(&candidate)
                    .and_then(|envelope| {
                        wire::response(&envelope).and_then(wire::decode::<i64>)
                    })
                    .is_ok_and(|reply| reply == self.nonce);
                if synchronized {
                    let info = wire::request("guest-info", json!({}), false)?;
                    self.transmit = Some(encode(&info, self.limit)?);
                    self.step = Step::Info;
                }
            }
            SyncLine::Bare(line) => {
                if let Some(error) = sync_refusal(&line) {
                    return Err(Error::SyncUnsupported(error));
                }
            }
        }
        Ok(())
    }

    /// The reply to `guest-info`: the agent's release and, with
    /// [`ClientOptions::probe_commands`], its commands.
    pub fn handle_frame(&mut self, bytes: &[u8]) -> Result<()> {
        debug_assert_eq!(self.step, Step::Info, "not waiting for guest-info");
        debug_assert!(
            self.transmit.is_none(),
            "write the frame before reading on"
        );
        let info: GuestInfo =
            wire::decode(wire::response(&wire::envelope(bytes)?)?)?;
        self.version = qapi_core::parse_qga_version(&info.version)
            .map_err(Error::AgentVersion)?;
        self.commands = self.probe.then(|| agent_commands(&info)).flatten();
        self.step = Step::Done;
        Ok(())
    }

    /// The agent's release and the commands it listed.
    pub fn finish(self) -> ((u32, u32, u32), Option<PeerCommands>) {
        debug_assert!(self.is_done(), "the handshake is not done");
        (self.version, self.commands)
    }
}

/// One request of a guest agent session, which runs one at a time: its
/// bytes and how many of them have left.
///
/// The client writes [`transmit`](Self::transmit), records each write with
/// [`advance`](Self::advance), and then reads the reply, if the command
/// sends one, up to the call's reply limit. A timeout before any byte left
/// keeps the session aligned ([`timeout`](Self::timeout)); one after it, or
/// any failure to write or read, ends it, since a late reply could be
/// mistaken for the next command's.
pub(crate) struct QgaCall {
    bytes: Vec<u8>,
    offset: usize,
}

impl QgaCall {
    /// A request of at most `limit` JSON bytes whose reply may have up to
    /// `reply_limit` bytes.
    pub fn new(
        request: &Value,
        limit: usize,
        reply_limit: usize,
    ) -> Result<Self> {
        if reply_limit == 0 {
            return Err(Error::InvalidRequest(
                InvalidRequestKind::ZeroReplyLimit,
            ));
        }
        Self::without_reply(request, limit)
    }

    /// A request of at most `limit` JSON bytes that sends no reply.
    pub fn without_reply(request: &Value, limit: usize) -> Result<Self> {
        Ok(Self { bytes: encode(request, limit)?, offset: 0 })
    }

    /// The bytes still to write.
    pub fn transmit(&self) -> &[u8] {
        &self.bytes[self.offset..]
    }

    /// Record that `written` bytes of [`transmit`](Self::transmit) left.
    pub fn advance(&mut self, written: usize) {
        self.offset += written;
        debug_assert!(self.offset <= self.bytes.len());
    }

    /// The call's deadline elapsed: the error, and whether the session
    /// stays aligned, which it does while no byte has left.
    pub fn timeout(&self) -> (Error, bool) {
        let aligned = self.offset == 0;
        let phase = if aligned {
            TimeoutPhase::BeforeDispatch
        } else {
            TimeoutPhase::AfterDispatch
        };
        (Error::Timeout { phase }, aligned)
    }
}

/// A reply's payload, and whether the session stays aligned. A complete,
/// well-formed reply leaves it aligned, whether it is an error reply or
/// later fails to decode into the return type; any other reply ends it.
pub(crate) fn reply(bytes: &[u8]) -> (Result<&RawValue>, bool) {
    match wire::envelope(bytes) {
        Ok(envelope) => {
            let raw = wire::response(&envelope);
            let aligned = matches!(raw, Ok(_) | Err(Error::Remote(_)));
            (raw, aligned)
        }
        Err(error) => (Err(error), false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProtocolViolation;

    const NONCE: i64 = -4_242_424_242;

    fn sentineled(bytes: &[u8]) -> SyncLine {
        SyncLine::Sentineled(bytes.to_vec())
    }

    fn synced(probe: bool) -> QgaHandshake {
        let options = ClientOptions::default().with_probe_commands(probe);
        let mut handshake = QgaHandshake::new(NONCE, &options).unwrap();
        handshake.poll_transmit().unwrap();
        handshake
            .handle_sync_line(sentineled(
                format!(r#"{{"return": {NONCE}}}"#).as_bytes(),
            ))
            .unwrap();
        assert!(!handshake.is_syncing());
        handshake.poll_transmit().expect("guest-info follows the sync");
        handshake
    }

    #[test]
    fn the_sync_frame_is_a_sentinel_and_the_sync_command_with_the_nonce() {
        let mut handshake =
            QgaHandshake::new(NONCE, &ClientOptions::default()).unwrap();
        let frame = handshake.poll_transmit().unwrap();
        assert_eq!(frame[0], 0xff);
        let json = frame[1..].strip_suffix(b"\r\n").expect("a CRLF ends it");
        assert_eq!(
            serde_json::from_slice::<Value>(json).unwrap(),
            json!({"execute": "guest-sync-delimited", "arguments": {"id": NONCE}})
        );
        assert!(handshake.is_syncing());
        assert!(handshake.poll_transmit().is_none());
    }

    #[test]
    fn only_the_reply_that_echoes_the_nonce_ends_the_sync() {
        let mut handshake =
            QgaHandshake::new(NONCE, &ClientOptions::default()).unwrap();
        handshake.poll_transmit();
        for stale in [
            &br#"{"return": 1}"#[..],
            br#"{"return": "x"}"#,
            br#"{"error": {"class": "GenericError", "desc": "x"}}"#,
            b"not json",
        ] {
            handshake.handle_sync_line(sentineled(stale)).unwrap();
            assert!(handshake.is_syncing());
        }
        // A bare line that is not a refusal of the sync is stale too.
        let other = br#"{"error": {"class": "CommandNotFound", "desc": "The command guest-ping has not been found"}}"#;
        handshake.handle_sync_line(SyncLine::Bare(other.to_vec())).unwrap();
        assert!(handshake.is_syncing());
        handshake
            .handle_sync_line(sentineled(
                format!(r#"{{"return": {NONCE}}}"#).as_bytes(),
            ))
            .unwrap();
        assert!(!handshake.is_syncing());
        let info = handshake.poll_transmit().unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(
                info.strip_suffix(b"\r\n").unwrap()
            )
            .unwrap(),
            json!({"execute": "guest-info"})
        );
    }

    #[test]
    fn an_agent_that_refuses_the_sync_is_unsupported() {
        let mut handshake =
            QgaHandshake::new(NONCE, &ClientOptions::default()).unwrap();
        handshake.poll_transmit();
        let refusal = br#"{"error": {"class": "CommandNotFound", "desc": "The command guest-sync-delimited has not been found"}}"#;
        assert!(matches!(
            handshake.handle_sync_line(SyncLine::Bare(refusal.to_vec())),
            Err(Error::SyncUnsupported(error)) if error.class == "CommandNotFound"
        ));
    }

    #[test]
    fn guest_info_gives_the_release_and_the_enabled_and_disabled_commands() {
        let mut handshake = synced(true);
        handshake
            .handle_frame(
                br#"{"return": {"version": "9.2.0", "supported_commands": [
                {"name": "guest-info", "enabled": true},
                {"name": "guest-exec", "enabled": false},
                {"name": "guest-ping", "enabled": true}]}}"#,
            )
            .unwrap();
        assert!(handshake.is_done());
        let (version, commands) = handshake.finish();
        assert_eq!(version, (9, 2, 0));
        let commands = commands.unwrap();
        assert_eq!(
            commands.names().collect::<Vec<_>>(),
            ["guest-info", "guest-ping"]
        );
        assert!(commands.is_disabled("guest-exec"));
    }

    #[test]
    fn without_the_probe_or_a_real_list_the_commands_are_unknown() {
        let info = br#"{"return": {"version": "9.2.0", "supported_commands": [{"name": "guest-info", "enabled": true}]}}"#;
        let mut handshake = synced(false);
        handshake.handle_frame(info).unwrap();
        assert!(handshake.finish().1.is_none());
        // A list that lacks guest-info, which every agent that answers has.
        let mut handshake = synced(true);
        handshake
            .handle_frame(br#"{"return": {"version": "9.2.0", "supported_commands": []}}"#)
            .unwrap();
        assert!(handshake.finish().1.is_none());
        // qemu-ga 0.15 sends only its version.
        let mut handshake = synced(true);
        handshake
            .handle_frame(br#"{"return": {"version": "0.15.0"}}"#)
            .unwrap();
        assert_eq!(handshake.finish(), ((0, 15, 0), None));
    }

    #[test]
    fn a_bad_guest_info_reply_fails_the_handshake() {
        assert!(matches!(
            synced(true).handle_frame(br#"{"return": {"version": "nine"}}"#),
            Err(Error::AgentVersion(_))
        ));
        assert!(matches!(
            synced(true).handle_frame(
                br#"{"error": {"class": "GenericError", "desc": "no"}}"#
            ),
            Err(Error::Remote(_))
        ));
        assert!(matches!(
            synced(true).handle_frame(br#"{"return": {"version": 9}}"#),
            Err(Error::Decode(_))
        ));
    }

    #[test]
    fn a_call_checks_its_limits_before_anything_is_sent() {
        let request = json!({"execute": "guest-ping"});
        assert!(matches!(
            QgaCall::new(&request, 1024, 0),
            Err(Error::InvalidRequest(InvalidRequestKind::ZeroReplyLimit))
        ));
        assert!(matches!(
            QgaCall::new(&request, 8, 1024),
            Err(Error::InvalidRequest(
                InvalidRequestKind::RequestTooLarge { .. }
            ))
        ));
        assert!(QgaCall::without_reply(&request, 1024).is_ok());
    }

    #[test]
    fn a_timeout_keeps_the_session_only_while_no_byte_has_left() {
        let mut call =
            QgaCall::new(&json!({"execute": "guest-ping"}), 1024, 1024)
                .unwrap();
        assert_eq!(call.transmit(), b"{\"execute\":\"guest-ping\"}\r\n");
        assert!(matches!(
            call.timeout(),
            (Error::Timeout { phase: TimeoutPhase::BeforeDispatch }, true)
        ));
        call.advance(1);
        assert_eq!(call.transmit(), b"\"execute\":\"guest-ping\"}\r\n");
        assert!(matches!(
            call.timeout(),
            (Error::Timeout { phase: TimeoutPhase::AfterDispatch }, false)
        ));
    }

    #[test]
    fn a_well_formed_reply_keeps_the_session_aligned_and_others_end_it() {
        let (raw, aligned) = reply(br#"{"return": {"a": 1}}"#);
        assert_eq!(raw.unwrap().get(), r#"{"a": 1}"#);
        assert!(aligned);
        let (raw, aligned) =
            reply(br#"{"error": {"class": "GenericError", "desc": "no"}}"#);
        assert!(matches!(raw, Err(Error::Remote(_))));
        assert!(aligned);
        let (raw, aligned) =
            reply(br#"{"return": 1, "error": {"class": "GenericError", "desc": "no"}}"#);
        assert!(matches!(
            raw,
            Err(Error::Protocol(ProtocolViolation::ResponseShape))
        ));
        assert!(!aligned);
        let (raw, aligned) = reply(b"[]");
        assert!(matches!(
            raw,
            Err(Error::Protocol(ProtocolViolation::NotAnObject(_)))
        ));
        assert!(!aligned);
    }
}
