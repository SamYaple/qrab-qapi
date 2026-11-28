//! The QMP connection after the handshake: admission, request IDs, one
//! frame written at a time, reply routing, events, deadlines and closure.
use crate::{
    ClientOptions, Error, InvalidRequestKind, ProtocolViolation, QmpEvent, Result, TimeoutPhase,
    engine::{encode, qmp::event},
    wire,
};
use serde_json::{Value, json, value::RawValue};
use std::{
    collections::{HashMap, VecDeque},
    os::fd::OwnedFd,
    sync::Arc,
    time::Instant,
};

/// QEMU recommends at most eight unanswered in-band commands so it continues
/// reading OOB requests. See the vendored docs/interop/qmp-spec.rst.
pub const QMP_QUEUE_CAPACITY: usize = 8;
pub(crate) const OOB_CAPACITY: usize = 8;
/// Linux's SCM_MAX_FD: `sendmsg` refuses more descriptors in one message.
pub(crate) const MAX_FDS: usize = 253;

/// A request for the connection: the command object without its ID, the
/// descriptors to send with its first bytes, whether it is out of band,
/// and its deadline.
pub(crate) struct Request {
    pub json: Value,
    pub fds: Vec<OwnedFd>,
    pub oob: bool,
    pub deadline: Instant,
}

/// What the client routes after an input. `W` is the client's waiter, such
/// as the sender of a reply channel.
pub(crate) enum Output<W> {
    /// The outcome of the request that `W` waits for.
    Reply(W, Result<Box<RawValue>>),
    Event(QmpEvent),
    /// An event dropped unparsed, as larger than the event limit.
    EventDropped,
}

/// The bytes to write next. The descriptors go with the first of them, in
/// one `sendmsg`, and are empty once a byte of the frame has left.
pub(crate) struct Transmit<'a> {
    pub bytes: &'a [u8],
    pub fds: &'a [OwnedFd],
}

struct Queued<W> {
    json: Value,
    fds: Vec<OwnedFd>,
    oob: bool,
    deadline: Instant,
    waiter: W,
}

struct Pending<W> {
    deadline: Instant,
    oob: bool,
    waiter: W,
}

struct Writing {
    id: u64,
    bytes: Vec<u8>,
    offset: usize,
    fds: Vec<OwnedFd>,
}

/// The QMP connection that a finished handshake leaves.
///
/// The client [`submit`](Self::submit)s requests, writes what
/// [`poll_transmit`](Self::poll_transmit) gives it and records each write
/// with [`advance`](Self::advance), passes every message it reads to
/// [`handle_frame`](Self::handle_frame), wakes at
/// [`poll_timeout`](Self::poll_timeout) to call
/// [`handle_timeout`](Self::handle_timeout), and routes
/// [`poll_output`](Self::poll_output). Time comes from the client, so a
/// runtime's own clock drives the deadlines. Run the expiry first when it
/// is due, so that a call whose deadline passed reports its timeout, not
/// the closure it caused.
///
/// The rules:
///
/// - At most [`QMP_QUEUE_CAPACITY`] in-band and as many OOB requests are
///   dispatched at a time. A dispatched request keeps its slot until its
///   reply or its deadline, also when its caller stopped waiting, since the
///   peer may still run it.
/// - OOB requests go before queued in-band ones, since OOB exists to
///   overtake a busy in-band queue; each class goes in submit order, each
///   request as soon as its class has a slot. A queued request whose waiter
///   is gone is dropped.
/// - IDs count from 1; the handshake used 0. Other IDs, and IDs that are no
///   number, are not this client's and are ignored.
/// - A queued request whose deadline passes fails with
///   [`TimeoutPhase::BeforeDispatch`] and the connection stays. A dispatched
///   one fails with [`TimeoutPhase::AfterDispatch`] and closes the
///   connection, since its outcome and the peer's queue are then unknown;
///   every other waiter gets [`Error::Disconnected`] holding that timeout.
/// - A reply that arrives before its request is fully written, an error
///   reply without an ID (a parser error), a response without an ID, an
///   invalid event or a malformed reply close the connection.
pub(crate) struct QmpConnection<W> {
    limit: usize,
    event_limit: usize,
    oob: bool,
    last_id: u64,
    queued: VecDeque<Queued<W>>,
    pending: HashMap<u64, Pending<W>>,
    // Dispatched requests per class, in-band then OOB.
    in_flight: [usize; 2],
    writing: Option<Writing>,
    outputs: VecDeque<Output<W>>,
    closed: Option<Arc<Error>>,
}

impl<W> QmpConnection<W> {
    /// The connection of a handshake that negotiated `oob`, with the events
    /// that arrived during the handshake (`None` for one dropped) as its
    /// first outputs.
    pub fn new(options: &ClientOptions, oob: bool, events: Vec<Option<QmpEvent>>) -> Self {
        Self {
            limit: options.max_message_bytes,
            event_limit: options.max_event_bytes,
            oob,
            last_id: 0,
            queued: VecDeque::new(),
            pending: HashMap::new(),
            in_flight: [0, 0],
            writing: None,
            outputs: events
                .into_iter()
                .map(|event| event.map_or(Output::EventDropped, Output::Event))
                .collect(),
            closed: None,
        }
    }

    /// Queue a request. It is refused, with its waiter back, when it cannot
    /// be sent: too large, OOB without the peer's consent, with too many
    /// descriptors, or on a closed connection.
    pub fn submit(&mut self, request: Request, waiter: W) -> std::result::Result<(), (W, Error)> {
        if let Err(error) = encode(&request.json, self.limit) {
            return Err((waiter, error));
        }
        if request.oob && !self.oob {
            return Err((
                waiter,
                Error::InvalidRequest(InvalidRequestKind::OobNotNegotiated),
            ));
        }
        if request.fds.len() > MAX_FDS {
            return Err((
                waiter,
                Error::InvalidRequest(InvalidRequestKind::TooManyFds {
                    count: request.fds.len(),
                    max: MAX_FDS,
                }),
            ));
        }
        if let Some(reason) = &self.closed {
            return Err((waiter, Error::Disconnected(reason.clone())));
        }
        self.queued.push_back(Queued {
            json: request.json,
            fds: request.fds,
            oob: request.oob,
            deadline: request.deadline,
            waiter,
        });
        Ok(())
    }

    /// The bytes to write next, starting the next request that can go when
    /// no frame is being written. `live` says whether a waiter still waits;
    /// queued requests of those that do not are dropped.
    pub fn poll_transmit(
        &mut self,
        now: Instant,
        mut live: impl FnMut(&W) -> bool,
    ) -> Option<Transmit<'_>> {
        self.handle_timeout(now);
        if self.closed.is_some() {
            return None;
        }
        if self.writing.is_none() {
            self.start_next(&mut live);
        }
        self.writing.as_ref().map(|frame| Transmit {
            bytes: &frame.bytes[frame.offset..],
            fds: &frame.fds,
        })
    }

    fn start_next(&mut self, live: &mut impl FnMut(&W) -> bool) {
        self.queued.retain(|request| live(&request.waiter));
        while let Some(index) = self.next_queued() {
            let Queued {
                mut json,
                fds,
                oob,
                deadline,
                waiter,
            } = self
                .queued
                .remove(index)
                .expect("the position is in the queue");
            let Some(id) = self.last_id.checked_add(1) else {
                let reason = Arc::new(Error::Protocol(ProtocolViolation::IdsExhausted));
                self.outputs.push_back(Output::Reply(
                    waiter,
                    Err(Error::Disconnected(reason.clone())),
                ));
                self.fail(reason);
                return;
            };
            self.last_id = id;
            json["id"] = json!(id);
            let bytes = match encode(&json, self.limit) {
                Ok(bytes) => bytes,
                Err(error) => {
                    self.outputs.push_back(Output::Reply(waiter, Err(error)));
                    continue;
                }
            };
            self.pending.insert(
                id,
                Pending {
                    deadline,
                    oob,
                    waiter,
                },
            );
            self.in_flight[usize::from(oob)] += 1;
            self.writing = Some(Writing {
                id,
                bytes,
                offset: 0,
                fds,
            });
            return;
        }
    }

    /// The queued request to dispatch next: the first OOB one while OOB has
    /// a slot, else the first in-band one while in-band has a slot.
    fn next_queued(&self) -> Option<usize> {
        [true, false].into_iter().find_map(|oob| {
            if self.in_flight[usize::from(oob)] >= capacity(oob) {
                return None;
            }
            self.queued.iter().position(|request| request.oob == oob)
        })
    }

    /// Record that `written` bytes of the last transmit left. The
    /// descriptors went with the first of them and are closed here; the
    /// peer holds its own copies.
    pub fn advance(&mut self, written: usize) {
        let Some(frame) = self.writing.as_mut() else {
            return;
        };
        if written > 0 {
            frame.fds.clear();
        }
        frame.offset += written;
        debug_assert!(frame.offset <= frame.bytes.len());
        if frame.offset >= frame.bytes.len() {
            self.writing = None;
        }
    }

    /// Handle one message from the peer. A message that breaks the
    /// protocol closes the connection with that error.
    pub fn handle_frame(&mut self, bytes: &[u8]) {
        if self.closed.is_some() {
            return;
        }
        if let Err(error) = self.frame(bytes) {
            self.fail(Arc::new(error));
        }
    }

    fn frame(&mut self, bytes: &[u8]) -> Result<()> {
        let envelope = wire::envelope(bytes)?;
        if envelope.event.is_some() {
            let output = event(bytes, &envelope, self.event_limit)?
                .map_or(Output::EventDropped, Output::Event);
            self.outputs.push_back(output);
            return Ok(());
        }
        if envelope.id.is_none() {
            // QEMU may report a parser error before reading a request ID. It
            // cannot be correlated, so close and retain its cause.
            return match wire::response(&envelope) {
                Err(error @ Error::Remote(_)) => Err(error),
                _ => Err(Error::Protocol(ProtocolViolation::MissingResponseId)),
            };
        }
        let Some(id) = envelope.numeric_id() else {
            return Ok(());
        };
        if !self.pending.contains_key(&id) {
            return Ok(());
        }
        if self.writing.as_ref().is_some_and(|frame| frame.id == id) {
            return Err(Error::Protocol(ProtocolViolation::EarlyResponse));
        }
        // Only the payload is kept, as raw JSON for the caller to decode.
        let response = wire::response(&envelope).map(RawValue::to_owned);
        if !matches!(response, Ok(_) | Err(Error::Remote(_))) {
            return response.map(drop);
        }
        let request = self.pending.remove(&id).expect("the ID is pending");
        self.in_flight[usize::from(request.oob)] -= 1;
        self.outputs
            .push_back(Output::Reply(request.waiter, response));
        Ok(())
    }

    /// The earliest deadline of a queued or dispatched request.
    pub fn poll_timeout(&self) -> Option<Instant> {
        let queued = self.queued.iter().map(|request| request.deadline);
        let dispatched = self.pending.values().map(|request| request.deadline);
        queued.chain(dispatched).min()
    }

    /// Expire the deadlines that `now` reached: queued requests fail before
    /// dispatch and the connection stays; dispatched ones fail after it and
    /// close the connection.
    pub fn handle_timeout(&mut self, now: Instant) {
        if self.closed.is_some() {
            return;
        }
        let mut kept = VecDeque::with_capacity(self.queued.len());
        for request in self.queued.drain(..) {
            if request.deadline <= now {
                self.outputs.push_back(Output::Reply(
                    request.waiter,
                    Err(Error::Timeout {
                        phase: TimeoutPhase::BeforeDispatch,
                    }),
                ));
            } else {
                kept.push_back(request);
            }
        }
        self.queued = kept;
        let mut expired: Vec<u64> = self
            .pending
            .iter()
            .filter(|(_, request)| request.deadline <= now)
            .map(|(id, _)| *id)
            .collect();
        if expired.is_empty() {
            return;
        }
        expired.sort_unstable();
        for id in expired {
            let request = self.pending.remove(&id).expect("the ID is pending");
            self.outputs.push_back(Output::Reply(
                request.waiter,
                Err(Error::Timeout {
                    phase: TimeoutPhase::AfterDispatch,
                }),
            ));
        }
        self.fail(Arc::new(Error::Timeout {
            phase: TimeoutPhase::AfterDispatch,
        }));
    }

    /// Close the connection: an I/O failure, the end of the input or a local
    /// shutdown. Every waiter fails with [`Error::Disconnected`] holding
    /// `reason`.
    pub fn close(&mut self, reason: Error) {
        self.fail(Arc::new(reason));
    }

    fn fail(&mut self, reason: Arc<Error>) {
        if self.closed.is_some() {
            return;
        }
        let mut pending: Vec<(u64, Pending<W>)> = self.pending.drain().collect();
        pending.sort_unstable_by_key(|(id, _)| *id);
        for (_, request) in pending {
            self.outputs.push_back(Output::Reply(
                request.waiter,
                Err(Error::Disconnected(reason.clone())),
            ));
        }
        for request in self.queued.drain(..) {
            self.outputs.push_back(Output::Reply(
                request.waiter,
                Err(Error::Disconnected(reason.clone())),
            ));
        }
        self.writing = None;
        self.in_flight = [0, 0];
        self.closed = Some(reason);
    }

    pub fn poll_output(&mut self) -> Option<Output<W>> {
        self.outputs.pop_front()
    }

    pub fn close_reason(&self) -> Option<&Arc<Error>> {
        self.closed.as_ref()
    }

    /// Dispatched requests awaiting a reply, whose callers may have stopped
    /// waiting.
    pub fn queue_depth(&self) -> usize {
        self.pending.len()
    }
}

fn capacity(oob: bool) -> usize {
    if oob {
        OOB_CAPACITY
    } else {
        QMP_QUEUE_CAPACITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn connection(oob: bool) -> QmpConnection<u32> {
        QmpConnection::new(&ClientOptions::default(), oob, Vec::new())
    }

    fn request(name: &str, oob: bool, deadline: Instant) -> Request {
        let verb = if oob { "exec-oob" } else { "execute" };
        Request {
            json: json!({verb: name}),
            fds: Vec::new(),
            oob,
            deadline,
        }
    }

    /// Write every frame that can go now, whole, and return their JSON.
    fn write_all(connection: &mut QmpConnection<u32>, now: Instant) -> Vec<Value> {
        let mut frames = Vec::new();
        while let Some(transmit) = connection.poll_transmit(now, |_| true) {
            let bytes = transmit.bytes.to_vec();
            connection.advance(bytes.len());
            let json = bytes.strip_suffix(b"\r\n").expect("a CRLF ends a frame");
            frames.push(serde_json::from_slice(json).unwrap());
        }
        frames
    }

    fn outputs(connection: &mut QmpConnection<u32>) -> Vec<(u32, Result<String>)> {
        let mut replies = Vec::new();
        while let Some(output) = connection.poll_output() {
            if let Output::Reply(waiter, result) = output {
                replies.push((waiter, result.map(|raw| raw.get().to_owned())));
            }
        }
        replies
    }

    fn reply(id: u64) -> Vec<u8> {
        format!(r#"{{"return": {{}}, "id": {id}}}"#).into_bytes()
    }

    #[test]
    fn requests_get_ids_from_one_and_replies_route_by_id() {
        let mut connection = connection(false);
        let now = Instant::now();
        let deadline = now + Duration::from_secs(5);
        for waiter in 0..3 {
            connection
                .submit(request("query-status", false, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        let frames = write_all(&mut connection, now);
        let ids: Vec<&Value> = frames.iter().map(|frame| &frame["id"]).collect();
        assert_eq!(ids, [&json!(1), &json!(2), &json!(3)]);
        assert_eq!(frames[0]["execute"], "query-status");
        assert_eq!(connection.queue_depth(), 3);
        connection.handle_frame(br#"{"return": "third", "id": 3}"#);
        connection.handle_frame(br#"{"return": "not ours", "id": "3"}"#);
        connection.handle_frame(br#"{"return": "unknown", "id": 9}"#);
        connection.handle_frame(br#"{"return": "first", "id": 1}"#);
        let replies = outputs(&mut connection);
        assert_eq!(replies.len(), 2);
        assert!(matches!(&replies[0], (2, Ok(raw)) if raw == r#""third""#));
        assert!(matches!(&replies[1], (0, Ok(raw)) if raw == r#""first""#));
        assert_eq!(connection.queue_depth(), 1);
        assert!(connection.close_reason().is_none());
    }

    #[test]
    fn a_frame_is_written_in_parts_and_its_descriptors_go_with_the_first() {
        let mut connection = connection(false);
        let now = Instant::now();
        let (read, write) = rustix::pipe::pipe().unwrap();
        let mut getfd = request("getfd", false, now + Duration::from_secs(5));
        getfd.json["arguments"] = json!({"fdname": "tap0"});
        getfd.fds = vec![read, write];
        connection
            .submit(getfd, 7)
            .map_err(|(_, error)| error)
            .unwrap();
        let transmit = connection.poll_transmit(now, |_| true).unwrap();
        let whole = transmit.bytes.to_vec();
        assert_eq!(transmit.fds.len(), 2);
        connection.advance(3);
        let transmit = connection.poll_transmit(now, |_| true).unwrap();
        assert_eq!(transmit.bytes, &whole[3..]);
        assert!(
            transmit.fds.is_empty(),
            "the descriptors went with the first bytes"
        );
        let rest = transmit.bytes.len();
        connection.advance(rest);
        assert!(connection.poll_transmit(now, |_| true).is_none());
        assert_eq!(connection.queue_depth(), 1);
    }

    #[test]
    fn eight_in_band_and_eight_oob_requests_are_dispatched_at_a_time() {
        let mut connection = connection(true);
        let now = Instant::now();
        let deadline = now + Duration::from_secs(5);
        for waiter in 0..10 {
            let oob = waiter == 9;
            let name = if oob { "migrate-pause" } else { "query-status" };
            connection
                .submit(request(name, oob, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        // The OOB request goes first; the ninth in-band one waits for a slot.
        let frames = write_all(&mut connection, now);
        assert_eq!(frames.len(), 9);
        assert_eq!(frames[0]["exec-oob"], "migrate-pause");
        assert_eq!(frames[0]["id"], 1);
        assert_eq!(connection.queue_depth(), 9);
        connection.handle_frame(&reply(2));
        let frames = write_all(&mut connection, now);
        assert_eq!(frames.len(), 1, "a reply frees one in-band slot");
        assert_eq!(frames[0]["execute"], "query-status");
        assert_eq!(frames[0]["id"], 10);
    }

    #[test]
    fn out_of_band_requests_go_before_queued_in_band_ones() {
        let mut connection = connection(true);
        let now = Instant::now();
        let deadline = now + Duration::from_secs(5);
        for (waiter, name, oob) in [
            (0, "query-status", false),
            (1, "stop", false),
            (2, "migrate-pause", true),
            (3, "query-yank", true),
        ] {
            connection
                .submit(request(name, oob, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        let names: Vec<String> = write_all(&mut connection, now)
            .iter()
            .map(|frame| {
                let name = frame["exec-oob"].as_str().or(frame["execute"].as_str());
                name.unwrap().to_owned()
            })
            .collect();
        assert_eq!(
            names,
            ["migrate-pause", "query-yank", "query-status", "stop"]
        );
    }

    #[test]
    fn in_band_requests_go_while_the_oob_slots_are_full() {
        let mut connection = connection(true);
        let now = Instant::now();
        let deadline = now + Duration::from_secs(5);
        for waiter in 0..9 {
            connection
                .submit(request("migrate-pause", true, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        connection
            .submit(request("query-status", false, deadline), 9)
            .map_err(|(_, error)| error)
            .unwrap();
        let frames = write_all(&mut connection, now);
        assert_eq!(frames.len(), 9, "the ninth OOB request waits for a slot");
        assert_eq!(frames[8]["execute"], "query-status");
        connection.handle_frame(&reply(1));
        let frames = write_all(&mut connection, now);
        assert_eq!(frames[0]["exec-oob"], "migrate-pause");
        assert_eq!(frames[0]["id"], 10);
    }

    #[test]
    fn a_queued_request_whose_waiter_is_gone_is_dropped() {
        let mut connection = connection(false);
        let now = Instant::now();
        let deadline = now + Duration::from_secs(5);
        for waiter in 0..3 {
            connection
                .submit(request("query-status", false, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        let mut ids = Vec::new();
        while let Some(transmit) = connection.poll_transmit(now, |waiter| *waiter != 1) {
            let bytes = transmit.bytes.to_vec();
            connection.advance(bytes.len());
            ids.push(serde_json::from_slice::<Value>(&bytes).unwrap()["id"].clone());
        }
        assert_eq!(ids, [json!(1), json!(2)], "waiter 1 got no ID");
        assert!(outputs(&mut connection).is_empty(), "and no reply");
        assert_eq!(connection.poll_timeout(), Some(deadline));
    }

    #[test]
    fn a_dispatched_request_keeps_its_slot_after_its_waiter_is_gone() {
        let mut connection = connection(false);
        let now = Instant::now();
        let deadline = now + Duration::from_secs(5);
        for waiter in 0..9 {
            connection
                .submit(request("query-status", false, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        assert_eq!(write_all(&mut connection, now).len(), 8);
        // Every caller stops waiting: the peer may still run what it read.
        assert!(connection.poll_transmit(now, |_| false).is_none());
        assert_eq!(connection.queue_depth(), 8);
        connection.handle_frame(&reply(4));
        assert!(
            matches!(outputs(&mut connection)[..], [(3, Ok(_))]),
            "the reply goes to its waiter, which the client may then discard"
        );
    }

    #[test]
    fn a_queued_request_that_times_out_leaves_the_connection_up() {
        let mut connection = connection(false);
        let now = Instant::now();
        let soon = now + Duration::from_millis(10);
        let later = now + Duration::from_secs(10);
        for waiter in 0..9 {
            let deadline = if waiter == 8 { soon } else { later };
            connection
                .submit(request("query-status", false, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        assert_eq!(write_all(&mut connection, now).len(), 8);
        assert_eq!(connection.poll_timeout(), Some(soon));
        connection.handle_timeout(soon);
        assert!(matches!(
            outputs(&mut connection)[..],
            [(
                8,
                Err(Error::Timeout {
                    phase: TimeoutPhase::BeforeDispatch
                })
            )]
        ));
        assert!(connection.close_reason().is_none());
        assert_eq!(connection.poll_timeout(), Some(later));
    }

    #[test]
    fn a_dispatched_request_that_times_out_closes_the_connection() {
        let mut connection = connection(false);
        let now = Instant::now();
        let soon = now + Duration::from_millis(10);
        let later = now + Duration::from_secs(10);
        for (waiter, deadline) in [(0, later), (1, soon), (2, later)] {
            connection
                .submit(request("query-status", false, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        assert_eq!(write_all(&mut connection, now).len(), 3);
        connection
            .submit(request("query-status", false, later), 3)
            .map_err(|(_, error)| error)
            .unwrap();
        connection.handle_timeout(soon);
        let replies = outputs(&mut connection);
        assert!(matches!(
            replies[0],
            (
                1,
                Err(Error::Timeout {
                    phase: TimeoutPhase::AfterDispatch
                })
            )
        ));
        let others: Vec<u32> = replies[1..]
            .iter()
            .map(|(waiter, result)| {
                assert!(matches!(
                    result,
                    Err(Error::Disconnected(reason))
                        if matches!(**reason, Error::Timeout { phase: TimeoutPhase::AfterDispatch })
                ));
                *waiter
            })
            .collect();
        assert_eq!(others, [0, 2, 3], "the dispatched, then the queued");
        assert!(connection.close_reason().is_some());
        assert_eq!(connection.queue_depth(), 0);
    }

    #[test]
    fn nothing_is_dispatched_after_a_dispatched_deadline_passed() {
        let mut connection = connection(false);
        let now = Instant::now();
        let soon = now + Duration::from_millis(10);
        connection
            .submit(request("query-status", false, soon), 0)
            .map_err(|(_, error)| error)
            .unwrap();
        assert_eq!(write_all(&mut connection, now).len(), 1);
        connection
            .submit(
                request("query-status", false, soon + Duration::from_secs(10)),
                1,
            )
            .map_err(|(_, error)| error)
            .unwrap();
        // The client polls to write before its timer fired.
        assert!(connection.poll_transmit(soon, |_| true).is_none());
        assert!(matches!(
            outputs(&mut connection)[..],
            [
                (0, Err(Error::Timeout { .. })),
                (1, Err(Error::Disconnected(_)))
            ]
        ));
    }

    #[test]
    fn a_reply_before_its_request_is_written_closes_the_connection() {
        let mut connection = connection(false);
        let now = Instant::now();
        connection
            .submit(
                request("query-status", false, now + Duration::from_secs(5)),
                0,
            )
            .map_err(|(_, error)| error)
            .unwrap();
        connection.poll_transmit(now, |_| true).unwrap();
        connection.advance(1);
        connection.handle_frame(&reply(1));
        assert!(matches!(
            connection.close_reason().map(|reason| &**reason),
            Some(Error::Protocol(ProtocolViolation::EarlyResponse))
        ));
        assert!(matches!(
            outputs(&mut connection)[..],
            [(0, Err(Error::Disconnected(_)))]
        ));
    }

    #[test]
    fn a_response_without_an_id_closes_the_connection() {
        let mut connection = connection(false);
        connection
            .handle_frame(br#"{"error": {"class": "GenericError", "desc": "JSON parse error"}}"#);
        assert!(matches!(
            connection.close_reason().map(|reason| &**reason),
            Some(Error::Remote(error)) if error.desc == "JSON parse error"
        ));
        let mut connection = self::connection(false);
        connection.handle_frame(br#"{"return": {}}"#);
        assert!(matches!(
            connection.close_reason().map(|reason| &**reason),
            Some(Error::Protocol(ProtocolViolation::MissingResponseId))
        ));
    }

    #[test]
    fn an_error_reply_goes_to_its_waiter_and_a_malformed_one_closes() {
        let mut connection = connection(false);
        let now = Instant::now();
        let deadline = now + Duration::from_secs(5);
        for waiter in 0..2 {
            connection
                .submit(request("query-status", false, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        write_all(&mut connection, now);
        connection.handle_frame(br#"{"error": {"class": "GenericError", "desc": "no"}, "id": 1}"#);
        assert!(matches!(
            &outputs(&mut connection)[..],
            [(0, Err(Error::Remote(error)))] if error.class == "GenericError"
        ));
        assert!(connection.close_reason().is_none());
        connection.handle_frame(
            br#"{"return": 1, "error": {"class": "GenericError", "desc": "no"}, "id": 2}"#,
        );
        assert!(matches!(
            connection.close_reason().map(|reason| &**reason),
            Some(Error::Protocol(ProtocolViolation::ResponseShape))
        ));
        let mut connection = self::connection(false);
        connection.handle_frame(b"{\"return\":");
        assert!(matches!(
            connection.close_reason().map(|reason| &**reason),
            Some(Error::Malformed(_))
        ));
    }

    #[test]
    fn events_are_outputs_after_those_of_the_handshake() {
        let options = ClientOptions::default().with_max_event_bytes(80);
        let stop: QmpEvent = serde_json::from_str(
            r#"{"event": "STOP", "timestamp": {"seconds": 1, "microseconds": 2}}"#,
        )
        .unwrap();
        let mut connection: QmpConnection<u32> =
            QmpConnection::new(&options, false, vec![Some(stop), None]);
        connection.handle_frame(
            br#"{"event": "RESUME", "timestamp": {"seconds": 3, "microseconds": 4}}"#,
        );
        let large = format!(
            r#"{{"event": "BLOB", "data": {{"x": "{}"}}, "timestamp": {{"seconds": 1, "microseconds": 2}}}}"#,
            "y".repeat(80)
        );
        connection.handle_frame(large.as_bytes());
        let mut events = Vec::new();
        while let Some(output) = connection.poll_output() {
            events.push(match output {
                Output::Event(event) => Some(event.event),
                Output::EventDropped => None,
                Output::Reply(..) => panic!("no request was sent"),
            });
        }
        assert_eq!(
            events,
            [Some("STOP".into()), None, Some("RESUME".into()), None]
        );
        connection.handle_frame(
            br#"{"event": "STOP", "id": 1, "timestamp": {"seconds": 1, "microseconds": 2}}"#,
        );
        assert!(matches!(
            connection.close_reason().map(|reason| &**reason),
            Some(Error::Protocol(ProtocolViolation::EventWithResponseFields))
        ));
    }

    #[test]
    fn requests_that_cannot_be_sent_are_refused_with_their_waiter() {
        let deadline = Instant::now() + Duration::from_secs(5);
        let options = ClientOptions::default().with_max_message_bytes(64);
        let mut connection: QmpConnection<u32> = QmpConnection::new(&options, false, Vec::new());
        let mut large = request("query-status", false, deadline);
        large.json["arguments"] = json!({"padding": "x".repeat(64)});
        assert!(matches!(
            connection.submit(large, 1),
            Err((
                1,
                Error::InvalidRequest(InvalidRequestKind::RequestTooLarge { .. })
            ))
        ));
        assert!(matches!(
            connection.submit(request("migrate-pause", true, deadline), 2),
            Err((
                2,
                Error::InvalidRequest(InvalidRequestKind::OobNotNegotiated)
            ))
        ));
        let mut many = request("add-fd", false, deadline);
        many.fds = (0..=MAX_FDS)
            .map(|_| std::fs::File::open("/dev/null").unwrap().into())
            .collect();
        assert!(matches!(
            connection.submit(many, 3),
            Err((
                3,
                Error::InvalidRequest(InvalidRequestKind::TooManyFds {
                    count: 254,
                    max: 253
                })
            ))
        ));
        connection.close(Error::Closed);
        assert!(matches!(
            connection.submit(request("query-status", false, deadline), 4),
            Err((4, Error::Disconnected(reason))) if matches!(*reason, Error::Closed)
        ));
    }

    #[test]
    fn a_request_too_large_with_its_id_fails_alone() {
        let now = Instant::now();
        let deadline = now + Duration::from_secs(5);
        let mut tight = request("query-status", false, deadline);
        tight.json["arguments"] = json!({"padding": ""});
        let size = serde_json::to_vec(&tight.json).unwrap().len();
        let options = ClientOptions::default().with_max_message_bytes(size);
        let mut connection: QmpConnection<u32> = QmpConnection::new(&options, false, Vec::new());
        connection
            .submit(tight, 0)
            .map_err(|(_, error)| error)
            .unwrap();
        connection
            .submit(request("stop", false, deadline), 1)
            .map_err(|(_, error)| error)
            .unwrap();
        let frames = write_all(&mut connection, now);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0]["id"], 2, "the refused request used ID 1");
        assert!(matches!(
            outputs(&mut connection)[..],
            [(
                0,
                Err(Error::InvalidRequest(
                    InvalidRequestKind::RequestTooLarge { .. }
                ))
            )]
        ));
        assert!(connection.close_reason().is_none());
    }

    #[test]
    fn running_out_of_ids_closes_the_connection() {
        let mut connection = connection(false);
        let now = Instant::now();
        let deadline = now + Duration::from_secs(5);
        connection.last_id = u64::MAX - 1;
        for waiter in 0..2 {
            connection
                .submit(request("query-status", false, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        let frames = write_all(&mut connection, now);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0]["id"], u64::MAX);
        let replies = outputs(&mut connection);
        assert_eq!(replies.len(), 2);
        assert!(replies.iter().all(|(_, result)| matches!(
            result,
            Err(Error::Disconnected(reason))
                if matches!(**reason, Error::Protocol(ProtocolViolation::IdsExhausted))
        )));
        assert!(matches!(
            connection.close_reason().map(|reason| &**reason),
            Some(Error::Protocol(ProtocolViolation::IdsExhausted))
        ));
    }

    #[test]
    fn closing_fails_every_waiter_with_the_reason() {
        let mut connection = connection(false);
        let now = Instant::now();
        let deadline = now + Duration::from_secs(5);
        for waiter in 0..10 {
            connection
                .submit(request("query-status", false, deadline), waiter)
                .map_err(|(_, error)| error)
                .unwrap();
        }
        write_all(&mut connection, now);
        connection.close(Error::Closed);
        let replies = outputs(&mut connection);
        assert_eq!(replies.len(), 10);
        assert!(replies.iter().all(|(_, result)| matches!(
            result,
            Err(Error::Disconnected(reason)) if matches!(**reason, Error::Closed)
        )));
        assert_eq!(connection.queue_depth(), 0);
        assert!(connection.poll_transmit(now, |_| true).is_none());
        assert!(connection.poll_timeout().is_none());
        // Messages after the closure change nothing.
        connection.handle_frame(&reply(1));
        assert!(connection.poll_output().is_none());
    }

    #[test]
    fn the_timeout_is_the_earliest_deadline_queued_or_dispatched() {
        let mut connection = connection(false);
        let now = Instant::now();
        assert!(connection.poll_timeout().is_none());
        let dispatched = now + Duration::from_secs(3);
        connection
            .submit(request("query-status", false, dispatched), 0)
            .map_err(|(_, error)| error)
            .unwrap();
        write_all(&mut connection, now);
        assert_eq!(connection.poll_timeout(), Some(dispatched));
        let queued = now + Duration::from_secs(1);
        connection
            .submit(request("query-status", false, queued), 1)
            .map_err(|(_, error)| error)
            .unwrap();
        assert_eq!(connection.poll_timeout(), Some(queued));
    }
}
