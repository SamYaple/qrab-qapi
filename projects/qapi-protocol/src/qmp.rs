use crate::{
    ClientOptions, Error, Greeting, InvalidRequestKind, PeerCommands, ProtocolViolation, QmpEvent,
    Required, Result, TimeoutPhase,
    engine::{
        self,
        qmp::{Established, QmpHandshake},
    },
    framing::JsonReader,
    wire::{self, Envelope},
};
use qapi_core::QapiCommand;
use serde::de::DeserializeOwned;
use serde_json::{Value, json, value::RawValue};
use std::{
    collections::HashMap,
    os::fd::{AsFd, OwnedFd},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    net::UnixStream,
    sync::{OwnedSemaphorePermit, Semaphore, mpsc, oneshot, watch},
    task::AbortHandle,
    time::{Instant, timeout, timeout_at},
};

/// QEMU recommends at most eight unanswered in-band commands so it continues
/// reading OOB requests. See the vendored docs/interop/qmp-spec.rst.
pub const QMP_QUEUE_CAPACITY: usize = 8;
const OOB_CAPACITY: usize = 8;
/// Linux's SCM_MAX_FD: `sendmsg` refuses more descriptors in one message.
const MAX_FDS: usize = 253;

// A request's dispatch state. The driver claims a queued request before
// writing it, and a caller whose deadline elapses abandons it if it can,
// so exactly one of them decides whether it is sent.
const QUEUED: u8 = 0;
const DISPATCHED: u8 = 1;
const ABANDONED: u8 = 2;

/// The events of one QMP connection, in the order the peer sent them.
///
/// The queue holds at most [`ClientOptions::event_capacity`] events; an
/// event that does not fit is dropped and counted by
/// [`Qmp::dropped_events`]. Dropping this receiver discards later events
/// without closing the connection.
#[derive(Debug)]
pub struct QmpEvents {
    rx: mpsc::Receiver<QmpEvent>,
}

impl QmpEvents {
    /// Wait for the next event. `None` once the connection is closed and
    /// every queued event was received. Cancellation safe: an event is
    /// never lost by dropping the future, as in `tokio::select!`.
    pub async fn recv(&mut self) -> Option<QmpEvent> {
        self.rx.recv().await
    }

    /// The next queued event, without waiting. `None` when no event is
    /// queued, whether or not the connection is closed.
    pub fn try_recv(&mut self) -> Option<QmpEvent> {
        self.rx.try_recv().ok()
    }
}

struct Request {
    json: Value,
    // Sent as SCM_RIGHTS with the frame's first bytes, then closed.
    fds: Vec<OwnedFd>,
    deadline: Instant,
    state: Arc<AtomicU8>,
    reply: oneshot::Sender<Result<Box<RawValue>>>,
    // The request owns admission, even after its caller stops waiting.
    _permit: OwnedSemaphorePermit,
}

struct Events {
    limit: usize,
    tx: mpsc::Sender<QmpEvent>,
    dropped: Arc<AtomicU64>,
}

impl Events {
    fn publish(&self, bytes: &[u8], envelope: &Envelope) -> Result<()> {
        self.deliver(engine::qmp::event(bytes, envelope, self.limit)?);
        Ok(())
    }

    /// Queue an event, or count it as dropped: one too large to parse
    /// (`None`), or one that finds the queue full. Dropping the event
    /// receiver does not close a healthy QMP connection.
    fn deliver(&self, event: Option<QmpEvent>) {
        let dropped = match event {
            Some(event) => matches!(
                self.tx.try_send(event),
                Err(mpsc::error::TrySendError::Full(_))
            ),
            None => true,
        };
        if dropped {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// One task owns the socket and pending map. No shared mutable routing state,
/// reaper task or secondary event worker is needed.
///
/// Cancellation before dispatch skips the request. After dispatch it cannot
/// undo remote execution: admission stays reserved until the reply. A dispatched
/// command reaching its deadline closes the session, since its outcome and the
/// remote queue depth are then unknown. Commands are never retried automatically.
pub struct Qmp {
    greeting: Greeting,
    oob: bool,
    commands: Option<PeerCommands>,
    options: ClientOptions,
    requests: mpsc::Sender<Request>,
    normal: Arc<Semaphore>,
    out_of_band: Arc<Semaphore>,
    depth: watch::Receiver<usize>,
    closed: watch::Receiver<Option<Arc<Error>>>,
    shutdown: watch::Sender<()>,
    abort: AbortHandle,
    dropped_events: Arc<AtomicU64>,
}

impl std::fmt::Debug for Qmp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Qmp")
            .field("version", &self.version())
            .field("closed", &self.is_closed())
            .finish()
    }
}

impl Qmp {
    pub async fn new(path: &Path) -> Result<(Self, QmpEvents)> {
        Self::connect_with_options(path, ClientOptions::default()).await
    }

    pub async fn connect_with_options(
        path: &Path,
        options: ClientOptions,
    ) -> Result<(Self, QmpEvents)> {
        options.validate()?;
        let stream = timeout(options.connect_timeout, UnixStream::connect(path))
            .await
            .map_err(|_| Error::Timeout {
                phase: TimeoutPhase::Connect,
            })??;
        Self::from_stream_with_options(stream, options).await
    }

    /// Handshake on an already connected Unix socket (including inherited FDs).
    pub async fn from_stream(stream: UnixStream) -> Result<(Self, QmpEvents)> {
        Self::from_stream_with_options(stream, ClientOptions::default()).await
    }

    pub async fn from_stream_with_options(
        stream: UnixStream,
        options: ClientOptions,
    ) -> Result<(Self, QmpEvents)> {
        options.validate()?;
        let (read, mut write) = stream.into_split();
        let mut reader = JsonReader::new(read, options.max_message_bytes);
        let (event_tx, event_rx) = mpsc::channel(options.event_capacity);
        let dropped_events = Arc::new(AtomicU64::new(0));
        let events = Events {
            limit: options.max_event_bytes,
            tx: event_tx,
            dropped: dropped_events.clone(),
        };
        let established = timeout(options.handshake_timeout, async {
            let mut handshake = QmpHandshake::new(&options);
            loop {
                if let Some(frame) = handshake.poll_transmit() {
                    write.write_all(&frame).await?;
                }
                if handshake.is_done() {
                    break;
                }
                handshake.handle_frame(&reader.next().await?)?;
            }
            Ok::<_, Error>(handshake.finish())
        })
        .await
        .map_err(|_| Error::Timeout {
            phase: TimeoutPhase::Handshake,
        })??;
        let Established {
            greeting,
            oob,
            commands,
            events: early,
        } = established;
        for event in early {
            events.deliver(event);
        }

        let normal = Arc::new(Semaphore::new(QMP_QUEUE_CAPACITY));
        let out_of_band = Arc::new(Semaphore::new(OOB_CAPACITY));
        let (requests, request_rx) = mpsc::channel(QMP_QUEUE_CAPACITY + OOB_CAPACITY);
        let (depth_tx, depth) = watch::channel(0);
        let (closed_tx, closed) = watch::channel(None);
        let (shutdown, shutdown_rx) = watch::channel(());
        let lifecycle = Lifecycle {
            normal: normal.clone(),
            oob: out_of_band.clone(),
            depth: depth_tx,
            closed: closed_tx,
            reason: None,
        };
        let limit = options.max_message_bytes;
        let task = tokio::spawn(async move {
            let mut lifecycle = lifecycle;
            let result = drive(
                &mut reader,
                &mut write,
                request_rx,
                shutdown_rx,
                events,
                &lifecycle.depth,
                limit,
            )
            .await;
            lifecycle.reason = Some(Arc::new(result.err().unwrap_or(Error::Closed)));
        });
        let abort = task.abort_handle();
        // Lifecycle always runs on normal exit, panic or abort; dropping this
        // JoinHandle detaches ownership, not cleanup responsibility.
        drop(task);
        Ok((
            Self {
                greeting,
                oob,
                commands,
                options,
                requests,
                normal,
                out_of_band,
                depth,
                closed,
                shutdown,
                abort,
                dropped_events,
            },
            QmpEvents { rx: event_rx },
        ))
    }

    pub fn greeting(&self) -> &Greeting {
        &self.greeting
    }
    pub fn version(&self) -> (u32, u32, u32) {
        let v = self.greeting.version.qemu;
        (v.major, v.minor, v.micro)
    }

    /// The commands this QEMU listed in reply to the `query-commands` the
    /// client sent while connecting. `None` when
    /// [`ClientOptions::probe_commands`] was off or the peer answered with
    /// an error; the calls are then checked against the release alone.
    pub fn commands(&self) -> Option<&PeerCommands> {
        self.commands.as_ref()
    }

    /// Whether the peer can run `C`: its release has the command and, when
    /// the peer listed its commands, the list names it. The arguments of a
    /// call can still use an item the release lacks.
    pub fn supports<C: QapiCommand>(&self) -> bool {
        Required::of::<C>()
            .refusal(self.version(), self.commands())
            .is_none()
    }

    /// Check that the peer can run every command in `required`, as
    /// [`supports`](Self::supports) does, without sending anything. The
    /// error lists all those it cannot run:
    ///
    /// ```no_run
    /// # async fn run(qmp: &qapi_protocol::Qmp) -> qapi_protocol::Result<()> {
    /// use qapi::commands::{BlockdevAdd, QueryStatus};
    /// use qapi_protocol::Required;
    /// qmp.require(&[Required::of::<QueryStatus>(), Required::of::<BlockdevAdd>()])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn require(&self, required: &[Required]) -> Result<()> {
        crate::require(required, self.version(), self.commands())
    }
    /// Number of dispatched requests awaiting a reply, including cancelled callers.
    pub fn queue_depth(&self) -> usize {
        *self.depth.borrow()
    }
    pub fn dropped_events(&self) -> u64 {
        self.dropped_events.load(Ordering::Relaxed)
    }
    pub fn is_closed(&self) -> bool {
        self.closed.borrow().is_some()
    }
    pub fn close_reason(&self) -> Option<Arc<Error>> {
        self.closed.borrow().clone()
    }

    /// Wait for a peer disconnect, protocol failure, timeout or local shutdown.
    pub async fn closed(&self) -> Arc<Error> {
        let mut closed = self.closed.clone();
        loop {
            if let Some(reason) = closed.borrow().clone() {
                return reason;
            }
            if closed.changed().await.is_err() {
                return Arc::new(Error::Closed);
            }
        }
    }

    /// Close the connection and wait until the driver has stopped. Pending
    /// callers fail with [`Error::Disconnected`].
    ///
    /// It takes `&self`, unlike the guest agent's synchronous
    /// [`Qga::shutdown`](crate::qga::Qga::shutdown): a QMP client is shared,
    /// as through `Arc<Qmp>`, while other tasks await replies, and one task
    /// that owns the socket must stop, which is awaited here.
    pub async fn shutdown(&self) {
        self.shutdown.send_replace(());
        let _ = self.closed().await;
    }

    pub async fn execute<C: QapiCommand>(&self, cmd: &C) -> Result<C::Returns> {
        self.execute_with_timeout(cmd, self.options.command_timeout)
            .await
    }

    pub async fn execute_with_timeout<C: QapiCommand>(
        &self,
        cmd: &C,
        duration: Duration,
    ) -> Result<C::Returns> {
        if !C::sends_success_response(self.version()) {
            return Err(no_success_response::<C>());
        }
        let request = wire::command(cmd, self.version(), self.commands(), false)?;
        let reply = self.dispatch(request, Vec::new(), false, duration).await?;
        qapi_core::with_version(self.version(), || wire::decode(&reply))
    }

    pub async fn execute_oob<C: QapiCommand>(&self, cmd: &C) -> Result<C::Returns> {
        self.execute_oob_with_timeout(cmd, self.options.command_timeout)
            .await
    }

    pub async fn execute_oob_with_timeout<C: QapiCommand>(
        &self,
        cmd: &C,
        duration: Duration,
    ) -> Result<C::Returns> {
        if !C::allows_oob(self.version()) {
            return Err(Error::InvalidRequest(InvalidRequestKind::OobNotAllowed {
                command: C::NAME.into(),
            }));
        }
        if !C::sends_success_response(self.version()) {
            return Err(no_success_response::<C>());
        }
        let request = wire::command(cmd, self.version(), self.commands(), true)?;
        let reply = self.dispatch(request, Vec::new(), true, duration).await?;
        qapi_core::with_version(self.version(), || wire::decode(&reply))
    }

    /// Run the command `name` with `args`, a JSON object (`{}` for none),
    /// and decode its reply into `T`, such as a `serde_json::Value`, in the
    /// peer's version scope, as [`Qga::execute_raw`](crate::qga::Qga::execute_raw)
    /// does. The connection owns the request ID. A name that the peer's
    /// [`commands`](Self::commands) do not list fails before anything is sent.
    pub async fn execute_raw<T: DeserializeOwned + 'static>(
        &self,
        name: &str,
        args: Value,
    ) -> Result<T> {
        self.execute_raw_with_timeout(name, args, self.options.command_timeout)
            .await
    }
    pub async fn execute_raw_with_timeout<T: DeserializeOwned + 'static>(
        &self,
        name: &str,
        args: Value,
        duration: Duration,
    ) -> Result<T> {
        self.raw(name, args, Vec::new(), false, duration).await
    }
    /// [`execute_raw`](Self::execute_raw) out of band, as `exec-oob`.
    pub async fn execute_raw_oob<T: DeserializeOwned + 'static>(
        &self,
        name: &str,
        args: Value,
    ) -> Result<T> {
        self.execute_raw_oob_with_timeout(name, args, self.options.command_timeout)
            .await
    }
    pub async fn execute_raw_oob_with_timeout<T: DeserializeOwned + 'static>(
        &self,
        name: &str,
        args: Value,
        duration: Duration,
    ) -> Result<T> {
        self.raw(name, args, Vec::new(), true, duration).await
    }

    /// Run an in-band raw command with `fds` attached to it as SCM_RIGHTS,
    /// as QMP's `getfd` and `add-fd` expect. At most 253 descriptors fit in
    /// one message (Linux's `SCM_MAX_FD`). The descriptors are moved in and
    /// closed here once sent, or when the request is refused or expires.
    pub async fn execute_raw_with_fds<T: DeserializeOwned + 'static>(
        &self,
        name: &str,
        args: Value,
        fds: Vec<OwnedFd>,
    ) -> Result<T> {
        self.execute_raw_with_fds_and_timeout(name, args, fds, self.options.command_timeout)
            .await
    }
    pub async fn execute_raw_with_fds_and_timeout<T: DeserializeOwned + 'static>(
        &self,
        name: &str,
        args: Value,
        fds: Vec<OwnedFd>,
        duration: Duration,
    ) -> Result<T> {
        self.raw(name, args, fds, false, duration).await
    }

    /// `getfd`: hand QEMU `fd` under `name` for a later `netdev_add`
    /// (`fds=name:...`) or `device_add`.
    pub async fn getfd(&self, name: &str, fd: OwnedFd) -> Result<()> {
        self.execute_raw_with_fds::<serde::de::IgnoredAny>(
            "getfd",
            json!({"fdname": name}),
            vec![fd],
        )
        .await
        .map(drop)
    }

    async fn raw<T: DeserializeOwned + 'static>(
        &self,
        name: &str,
        args: Value,
        fds: Vec<OwnedFd>,
        oob: bool,
        duration: Duration,
    ) -> Result<T> {
        let request = wire::request(name, args, oob)?;
        wire::offered(name, self.version(), self.commands())?;
        let reply = self.dispatch(request, fds, oob, duration).await?;
        qapi_core::with_version(self.version(), || wire::decode(&reply))
    }

    fn disconnected(&self) -> Error {
        Error::Disconnected(
            self.close_reason()
                .unwrap_or_else(|| Arc::new(Error::Closed)),
        )
    }

    /// The driver has stopped; wait for the reason it records as it ends.
    async fn lost(&self) -> Error {
        Error::Disconnected(self.closed().await)
    }

    async fn dispatch(
        &self,
        request: Value,
        fds: Vec<OwnedFd>,
        oob: bool,
        duration: Duration,
    ) -> Result<Box<RawValue>> {
        engine::encode(&request, self.options.max_message_bytes)?;
        if oob && !self.oob {
            return Err(Error::InvalidRequest(InvalidRequestKind::OobNotNegotiated));
        }
        if fds.len() > MAX_FDS {
            return Err(Error::InvalidRequest(InvalidRequestKind::TooManyFds {
                count: fds.len(),
                max: MAX_FDS,
            }));
        }
        if self.is_closed() {
            return Err(self.disconnected());
        }
        let deadline = Instant::now()
            .checked_add(duration)
            .ok_or(Error::InvalidRequest(InvalidRequestKind::TimeoutTooLarge {
                timeout: duration,
            }))?;
        let state = Arc::new(AtomicU8::new(QUEUED));
        let timed_out = || {
            let unsent = state
                .compare_exchange(QUEUED, ABANDONED, Ordering::AcqRel, Ordering::Acquire)
                .is_ok();
            Error::Timeout {
                phase: if unsent {
                    TimeoutPhase::BeforeDispatch
                } else {
                    TimeoutPhase::AfterDispatch
                },
            }
        };
        let operation = async {
            let permits = if oob { &self.out_of_band } else { &self.normal };
            let Ok(permit) = permits.clone().acquire_owned().await else {
                return Err(self.lost().await);
            };
            let (reply, receive) = oneshot::channel();
            let request = Request {
                json: request,
                fds,
                deadline,
                state: state.clone(),
                reply,
                _permit: permit,
            };
            if self.requests.send(request).await.is_err() {
                return Err(self.lost().await);
            }
            match receive.await {
                Ok(result) => result,
                // The driver drops its requests as it stops, also when this
                // call's own deadline stopped it. A call whose deadline has
                // passed reports its timeout, whichever of the two the
                // runtime noticed first.
                Err(_) if Instant::now() >= deadline => Err(timed_out()),
                Err(_) => Err(self.lost().await),
            }
        };
        timeout_at(deadline, operation)
            .await
            .unwrap_or_else(|_| Err(timed_out()))
    }
}

pub(crate) fn no_success_response<C: QapiCommand>() -> Error {
    Error::InvalidRequest(InvalidRequestKind::NoSuccessResponse {
        command: C::NAME.into(),
    })
}

impl Drop for Qmp {
    fn drop(&mut self) {
        self.abort.abort();
    }
}

struct Lifecycle {
    normal: Arc<Semaphore>,
    oob: Arc<Semaphore>,
    depth: watch::Sender<usize>,
    closed: watch::Sender<Option<Arc<Error>>>,
    reason: Option<Arc<Error>>,
}
impl Drop for Lifecycle {
    fn drop(&mut self) {
        self.closed.send_replace(Some(
            self.reason
                .clone()
                .unwrap_or_else(|| Arc::new(Error::Closed)),
        ));
        self.normal.close();
        self.oob.close();
        self.depth.send_replace(0);
    }
}

struct Pending {
    deadline: Instant,
    reply: oneshot::Sender<Result<Box<RawValue>>>,
    _permit: OwnedSemaphorePermit,
}

struct Writing {
    id: u64,
    bytes: Vec<u8>,
    offset: usize,
    fds: Vec<OwnedFd>,
}

/// One `sendmsg` carrying `fds` with (a prefix of) `bytes`. Like `write()`
/// it is cancellation safe: nothing is sent until it returns.
async fn send_with_fds(
    stream: &UnixStream,
    bytes: &[u8],
    fds: &[OwnedFd],
) -> std::io::Result<usize> {
    use rustix::net::{SendAncillaryBuffer, SendAncillaryMessage, SendFlags, sendmsg};
    let borrowed: Vec<_> = fds.iter().map(AsFd::as_fd).collect();
    let mut space =
        vec![std::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(borrowed.len()))];
    let mut control = SendAncillaryBuffer::new(&mut space);
    if !control.push(SendAncillaryMessage::ScmRights(&borrowed)) {
        return Err(std::io::Error::other(
            "the descriptors do not fit their control message",
        ));
    }
    let iov = [std::io::IoSlice::new(bytes)];
    stream
        .async_io(tokio::io::Interest::WRITABLE, || {
            sendmsg(stream, &iov, &mut control, SendFlags::NOSIGNAL).map_err(std::io::Error::from)
        })
        .await
}

async fn drive(
    reader: &mut JsonReader<tokio::net::unix::OwnedReadHalf>,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    mut requests: mpsc::Receiver<Request>,
    mut shutdown: watch::Receiver<()>,
    events: Events,
    depth: &watch::Sender<usize>,
    limit: usize,
) -> Result<()> {
    let mut pending: HashMap<u64, Pending> = HashMap::new();
    let mut writing: Option<Writing> = None;
    let mut id = 0_u64;
    loop {
        let deadline = pending.values().map(|request| request.deadline).min();
        let expire = async {
            match deadline {
                Some(deadline) => tokio::time::sleep_until(deadline).await,
                None => std::future::pending::<()>().await,
            }
        };
        tokio::select! {
            _ = shutdown.changed() => return Ok(()),
            _ = expire => return Err(Error::Timeout { phase: TimeoutPhase::AfterDispatch }),
            message = reader.next() => {
                let bytes = message?;
                let envelope = wire::envelope(&bytes)?;
                if envelope.event.is_some() { events.publish(&bytes, &envelope)?; continue; }
                if envelope.id.is_none() {
                    // QEMU may report a parser error before reading a request
                    // ID. It cannot be correlated, so close and retain its cause.
                    return match wire::response(&envelope) {
                        Err(error @ Error::Remote(_)) => Err(error),
                        _ => Err(Error::Protocol(ProtocolViolation::MissingResponseId)),
                    };
                }
                // QMP IDs can be arbitrary JSON. Anything that is not one of
                // our numeric IDs is unknown, not a malformed protocol value.
                let Some(response_id) = envelope.numeric_id() else { continue; };
                if !pending.contains_key(&response_id) { continue; }
                if writing.as_ref().is_some_and(|w| w.id == response_id) {
                    return Err(Error::Protocol(ProtocolViolation::EarlyResponse));
                }
                // Only the payload is kept, as raw JSON for the caller to decode.
                let response = wire::response(&envelope).map(RawValue::to_owned);
                if !matches!(response, Ok(_) | Err(Error::Remote(_))) { return response.map(|_| ()); }
                let request = pending.remove(&response_id).unwrap();
                depth.send_replace(pending.len());
                let _ = request.reply.send(response);
            }
            // write(), unlike write_all(), is cancellation safe. Retaining its
            // offset lets reads, deadlines and shutdown progress under socket
            // backpressure without losing or repeating bytes in the frame.
            written = async {
                let frame = writing.as_ref().unwrap();
                if frame.fds.is_empty() {
                    writer.write(&frame.bytes[frame.offset..]).await
                } else {
                    send_with_fds(writer.as_ref(), &frame.bytes[frame.offset..], &frame.fds).await
                }
            }, if writing.is_some() => {
                let n = written?;
                if n == 0 { return Err(std::io::Error::from(std::io::ErrorKind::WriteZero).into()); }
                let frame = writing.as_mut().unwrap();
                // The descriptors went with these bytes; QEMU holds its own copies.
                frame.fds.clear();
                frame.offset += n;
                if frame.offset == frame.bytes.len() { writing = None; }
            }
            request = requests.recv(), if writing.is_none() => {
                let Some(Request { mut json, fds, deadline: request_deadline, state, reply, _permit }) = request else { return Ok(()); };
                if reply.is_closed() { continue; }
                if deadline.is_some_and(|d| d <= Instant::now()) {
                    return Err(Error::Timeout { phase: TimeoutPhase::AfterDispatch });
                }
                if request_deadline <= Instant::now() {
                    let _ = reply.send(Err(Error::Timeout { phase: TimeoutPhase::BeforeDispatch }));
                    continue;
                }
                // A caller that timed out first has abandoned the request.
                if state.compare_exchange(QUEUED, DISPATCHED, Ordering::AcqRel, Ordering::Acquire).is_err() { continue; }
                id = id.checked_add(1).ok_or(Error::Protocol(ProtocolViolation::IdsExhausted))?;
                json["id"] = json!(id);
                let bytes = match engine::encode(&json, limit) {
                    Ok(bytes) => bytes,
                    Err(error) => { let _ = reply.send(Err(error)); continue; }
                };
                // Keep only the encoded frame while writing, not a second copy
                // of potentially large arguments throughout remote execution.
                drop(json);
                pending.insert(id, Pending { deadline: request_deadline, reply, _permit });
                writing = Some(Writing { id, bytes, offset: 0, fds });
                depth.send_replace(pending.len());
            }
        }
    }
}
