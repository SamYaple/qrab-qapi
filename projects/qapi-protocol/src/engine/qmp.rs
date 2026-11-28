//! QMP: the greeting, the handshake, and the events.
use crate::{
    ClientOptions, Error, PeerCommands, ProtocolViolation, QmpEvent, Result,
    engine::encode,
    wire::{self, Envelope},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// The QMP greeting, as [`Qmp::greeting`](crate::Qmp::greeting) returns it.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Greeting {
    pub version: VersionInfo,
    /// The capabilities the peer offers, such as `oob`.
    pub capabilities: Vec<String>,
}

/// The peer's release and its package string.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VersionInfo {
    pub qemu: QemuVersion,
    pub package: String,
}

/// A QEMU release number; [`Qmp::version`](crate::Qmp::version) returns it
/// as a tuple.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[non_exhaustive]
pub struct QemuVersion {
    pub major: u32,
    pub minor: u32,
    pub micro: u32,
}

/// An event's message checked: the event, or `None` when it is larger than
/// `limit` and is dropped unparsed, so that a large event never becomes a
/// tree. The client counts the drop.
pub(crate) fn event(
    bytes: &[u8],
    envelope: &Envelope,
    limit: usize,
) -> Result<Option<QmpEvent>> {
    if envelope.ret.is_some()
        || envelope.error.is_some()
        || envelope.id.is_some()
    {
        return Err(Error::Protocol(
            ProtocolViolation::EventWithResponseFields,
        ));
    }
    if envelope.data.is_some_and(|data| !data.get().starts_with('{')) {
        return Err(Error::Protocol(ProtocolViolation::EventDataNotObject));
    }
    if bytes.len() > limit {
        return Ok(None);
    }
    serde_json::from_slice(bytes).map(Some).map_err(|error| {
        Error::Protocol(ProtocolViolation::InvalidEvent(error))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Greeting,
    Capabilities,
    Commands,
    Done,
}

/// What a finished QMP handshake learned.
pub(crate) struct Established {
    pub greeting: Greeting,
    /// Whether the peer offered OOB, which the handshake then enabled.
    pub oob: bool,
    /// The reply to the probe, `None` when it was off or refused.
    pub commands: Option<PeerCommands>,
    /// The events that arrived during the handshake, in order, with `None`
    /// for one dropped as too large.
    pub events: Vec<Option<QmpEvent>>,
}

/// The QMP handshake: the greeting, `qmp_capabilities`, and with
/// [`ClientOptions::probe_commands`] one `query-commands`, each request with
/// ID 0, under one deadline that the client enforces.
///
/// The client writes each frame of [`poll_transmit`](Self::poll_transmit)
/// before it reads on, and passes every message to
/// [`handle_frame`](Self::handle_frame) until [`is_done`](Self::is_done).
/// Any error ends the connection.
pub(crate) struct QmpHandshake {
    step: Step,
    probe: bool,
    limit: usize,
    event_limit: usize,
    greeting: Option<Greeting>,
    oob: bool,
    commands: Option<PeerCommands>,
    transmit: Option<Vec<u8>>,
    events: Vec<Option<QmpEvent>>,
}

impl QmpHandshake {
    pub fn new(options: &ClientOptions) -> Self {
        Self {
            step: Step::Greeting,
            probe: options.probe_commands,
            limit: options.max_message_bytes,
            event_limit: options.max_event_bytes,
            greeting: None,
            oob: false,
            commands: None,
            transmit: None,
            events: Vec::new(),
        }
    }

    /// The next frame to write, whole, before reading on.
    pub fn poll_transmit(&mut self) -> Option<Vec<u8>> {
        self.transmit.take()
    }

    pub fn is_done(&self) -> bool {
        self.step == Step::Done && self.transmit.is_none()
    }

    pub fn handle_frame(&mut self, bytes: &[u8]) -> Result<()> {
        debug_assert!(self.step != Step::Done, "the handshake is done");
        debug_assert!(
            self.transmit.is_none(),
            "write the frame before reading on"
        );
        let envelope = wire::envelope(bytes)?;
        if self.step == Step::Greeting {
            let greeting = envelope
                .greeting
                .ok_or(Error::Protocol(ProtocolViolation::MissingGreeting))?;
            let greeting: Greeting = serde_json::from_str(greeting.get())
                .map_err(|error| {
                    Error::Protocol(ProtocolViolation::InvalidGreeting(error))
                })?;
            self.oob = greeting.capabilities.iter().any(|cap| cap == "oob");
            let mut request = json!({"execute": "qmp_capabilities", "id": 0});
            if self.oob {
                request["arguments"] = json!({"enable": ["oob"]});
            }
            self.transmit = Some(encode(&request, self.limit)?);
            self.greeting = Some(greeting);
            self.step = Step::Capabilities;
            return Ok(());
        }
        // Events may precede a reply; they are kept for the session.
        if envelope.event.is_some() {
            self.events.push(event(bytes, &envelope, self.event_limit)?);
            return Ok(());
        }
        match self.step {
            Step::Capabilities => {
                if envelope.numeric_id() != Some(0) {
                    return Err(Error::Protocol(
                        ProtocolViolation::CapabilitiesReplyId,
                    ));
                }
                let value: Value = wire::decode(wire::response(&envelope)?)?;
                if !value.as_object().is_some_and(|o| o.is_empty()) {
                    return Err(Error::Protocol(
                        ProtocolViolation::CapabilitiesReply,
                    ));
                }
                if self.probe {
                    let request = json!({"execute": "query-commands", "id": 0});
                    self.transmit = Some(encode(&request, self.limit)?);
                    self.step = Step::Commands;
                } else {
                    self.step = Step::Done;
                }
            }
            Step::Commands => {
                if envelope.numeric_id() != Some(0) {
                    return Err(Error::Protocol(
                        ProtocolViolation::CommandsReplyId,
                    ));
                }
                self.commands = commands(&envelope)?;
                self.step = Step::Done;
            }
            Step::Greeting | Step::Done => {}
        }
        Ok(())
    }

    pub fn finish(self) -> Established {
        debug_assert!(self.is_done(), "the handshake is not done");
        Established {
            greeting: self
                .greeting
                .expect("a finished handshake has its greeting"),
            oob: self.oob,
            commands: self.commands,
            events: self.events,
        }
    }
}

/// The commands of the probe's reply: a list of names that the session
/// keeps. An error reply leaves the list unknown, so that only the checks
/// against the peer's release apply.
fn commands(envelope: &Envelope) -> Result<Option<PeerCommands>> {
    #[derive(Deserialize)]
    struct Command {
        name: String,
    }
    match wire::response(envelope) {
        Ok(raw) => {
            let commands: Vec<Command> = wire::decode(raw)?;
            Ok(Some(PeerCommands::new(
                commands.into_iter().map(|command| command.name),
                [],
            )))
        }
        Err(Error::Remote(_)) => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GREETING: &[u8] = br#"{"QMP": {"version": {"qemu": {"major": 11, "minor": 1, "micro": 3}, "package": ""}, "capabilities": ["oob"]}}"#;
    const PLAIN_GREETING: &[u8] = br#"{"QMP": {"version": {"qemu": {"major": 11, "minor": 1, "micro": 3}, "package": ""}, "capabilities": []}}"#;

    /// A frame's JSON, checking its terminator.
    fn json_of(frame: &[u8]) -> Value {
        let json = frame.strip_suffix(b"\r\n").expect("a frame ends with CRLF");
        serde_json::from_slice(json).unwrap()
    }

    fn handshake(probe: bool) -> QmpHandshake {
        QmpHandshake::new(&ClientOptions::default().with_probe_commands(probe))
    }

    #[test]
    fn the_handshake_enables_oob_when_offered_and_then_probes_the_commands() {
        let mut handshake = handshake(true);
        assert!(
            handshake.poll_transmit().is_none(),
            "it waits for the greeting"
        );
        handshake.handle_frame(GREETING).unwrap();
        assert_eq!(
            json_of(&handshake.poll_transmit().unwrap()),
            json!({"execute": "qmp_capabilities", "id": 0, "arguments": {"enable": ["oob"]}})
        );
        handshake.handle_frame(br#"{"return": {}, "id": 0}"#).unwrap();
        assert_eq!(
            json_of(&handshake.poll_transmit().unwrap()),
            json!({"execute": "query-commands", "id": 0})
        );
        assert!(!handshake.is_done());
        handshake
            .handle_frame(br#"{"return": [{"name": "query-status"}, {"name": "stop"}], "id": 0}"#)
            .unwrap();
        assert!(handshake.is_done());
        let established = handshake.finish();
        assert!(established.oob);
        assert_eq!(established.greeting.version.qemu.minor, 1);
        let commands = established.commands.unwrap();
        assert_eq!(
            commands.names().collect::<Vec<_>>(),
            ["query-status", "stop"]
        );
    }

    #[test]
    fn a_peer_without_oob_gets_capabilities_without_arguments() {
        let mut handshake = handshake(false);
        handshake.handle_frame(PLAIN_GREETING).unwrap();
        assert_eq!(
            json_of(&handshake.poll_transmit().unwrap()),
            json!({"execute": "qmp_capabilities", "id": 0})
        );
        handshake.handle_frame(br#"{"return": {}, "id": 0}"#).unwrap();
        assert!(handshake.poll_transmit().is_none(), "no probe when it is off");
        assert!(handshake.is_done());
        let established = handshake.finish();
        assert!(!established.oob);
        assert!(established.commands.is_none());
    }

    /// Handle `frame` as a client does: then write what the handshake
    /// answers with, if anything.
    fn answer(handshake: &mut QmpHandshake, frame: &[u8]) {
        handshake.handle_frame(frame).unwrap();
        handshake.poll_transmit();
    }

    #[test]
    fn a_refused_probe_leaves_the_commands_unknown() {
        let mut handshake = handshake(true);
        answer(&mut handshake, GREETING);
        answer(&mut handshake, br#"{"return": {}, "id": 0}"#);
        answer(
            &mut handshake,
            br#"{"error": {"class": "CommandNotFound", "desc": "no"}, "id": 0}"#,
        );
        assert!(handshake.finish().commands.is_none());
    }

    #[test]
    fn events_before_a_reply_are_kept_in_order_and_large_ones_dropped() {
        let options = ClientOptions::default().with_max_event_bytes(80);
        let mut handshake = QmpHandshake::new(&options);
        let large = format!(
            r#"{{"event": "BLOB", "data": {{"x": "{}"}}, "timestamp": {{"seconds": 1, "microseconds": 2}}}}"#,
            "y".repeat(80)
        );
        for frame in [
            GREETING,
            br#"{"event": "STOP", "data": {}, "timestamp": {"seconds": 1, "microseconds": 2}}"#,
            large.as_bytes(),
            br#"{"return": {}, "id": 0}"#,
            br#"{"event": "RESUME", "timestamp": {"seconds": 3, "microseconds": 4}}"#,
            br#"{"return": [], "id": 0}"#,
        ] {
            handshake.handle_frame(frame).unwrap();
            handshake.poll_transmit();
        }
        let events: Vec<Option<String>> = handshake
            .finish()
            .events
            .into_iter()
            .map(|event| event.map(|event| event.event))
            .collect();
        assert_eq!(events, [Some("STOP".into()), None, Some("RESUME".into())]);
    }

    #[test]
    fn the_first_message_must_be_the_greeting() {
        let mut handshake = handshake(true);
        let stop = br#"{"event": "STOP", "timestamp": {"seconds": 1, "microseconds": 2}}"#;
        assert!(matches!(
            handshake.handle_frame(stop),
            Err(Error::Protocol(ProtocolViolation::MissingGreeting))
        ));
        assert!(matches!(
            QmpHandshake::new(&ClientOptions::default())
                .handle_frame(br#"{"QMP": {"capabilities": []}}"#),
            Err(Error::Protocol(ProtocolViolation::InvalidGreeting(_)))
        ));
    }

    #[test]
    fn a_reply_without_id_zero_breaks_the_step_it_answers() {
        for reply in [&br#"{"return": {}, "id": 1}"#[..], br#"{"return": {}}"#]
        {
            let mut handshake = handshake(true);
            answer(&mut handshake, GREETING);
            assert!(matches!(
                handshake.handle_frame(reply),
                Err(Error::Protocol(ProtocolViolation::CapabilitiesReplyId))
            ));
        }
        let mut handshake = handshake(true);
        answer(&mut handshake, GREETING);
        answer(&mut handshake, br#"{"return": {}, "id": 0}"#);
        assert!(matches!(
            handshake.handle_frame(br#"{"return": [], "id": "0"}"#),
            Err(Error::Protocol(ProtocolViolation::CommandsReplyId))
        ));
    }

    #[test]
    fn the_capabilities_reply_is_an_empty_object() {
        let mut handshake = handshake(true);
        answer(&mut handshake, GREETING);
        assert!(matches!(
            handshake.handle_frame(br#"{"return": {"x": 1}, "id": 0}"#),
            Err(Error::Protocol(ProtocolViolation::CapabilitiesReply))
        ));
        let mut handshake = QmpHandshake::new(&ClientOptions::default());
        answer(&mut handshake, GREETING);
        assert!(matches!(
            handshake.handle_frame(br#"{"error": {"class": "GenericError", "desc": "no"}, "id": 0}"#),
            Err(Error::Remote(error)) if error.class == "GenericError"
        ));
    }

    #[test]
    fn a_probe_reply_that_is_not_a_command_list_fails() {
        let mut handshake = handshake(true);
        answer(&mut handshake, GREETING);
        answer(&mut handshake, br#"{"return": {}, "id": 0}"#);
        assert!(matches!(
            handshake.handle_frame(br#"{"return": {"name": "stop"}, "id": 0}"#),
            Err(Error::Decode(_))
        ));
    }

    #[test]
    fn messages_that_are_no_json_objects_fail() {
        assert!(matches!(
            handshake(true).handle_frame(b"[1]"),
            Err(Error::Protocol(ProtocolViolation::NotAnObject(_)))
        ));
        assert!(matches!(
            handshake(true).handle_frame(b"{\"QMP\":"),
            Err(Error::Malformed(_))
        ));
    }

    #[test]
    fn an_event_is_checked_before_it_is_parsed() {
        let check = |bytes: &[u8], limit| {
            event(bytes, &wire::envelope(bytes).unwrap(), limit)
        };
        let stop =
            br#"{"event": "STOP", "data": {}, "timestamp": {"seconds": 1, "microseconds": 2}}"#;
        assert_eq!(check(stop, 1024).unwrap().unwrap().event, "STOP");
        assert!(check(stop, stop.len() - 1).unwrap().is_none());
        assert!(check(stop, stop.len()).unwrap().is_some());
        assert!(matches!(
            check(
                br#"{"event": "STOP", "id": 1, "timestamp": {"seconds": 1, "microseconds": 2}}"#,
                1024
            ),
            Err(Error::Protocol(ProtocolViolation::EventWithResponseFields))
        ));
        assert!(matches!(
            check(
                br#"{"event": "STOP", "data": [], "timestamp": {"seconds": 1, "microseconds": 2}}"#,
                1024
            ),
            Err(Error::Protocol(ProtocolViolation::EventDataNotObject))
        ));
        // The size check comes after the shape checks, but before parsing.
        assert!(matches!(
            check(br#"{"event": "STOP"}"#, 1024),
            Err(Error::Protocol(ProtocolViolation::InvalidEvent(_)))
        ));
        assert!(check(br#"{"event": "STOP"}"#, 4).unwrap().is_none());
    }
}
