use crate::{
    ClientOptions, Error, Greeting, InvalidRequestKind, PeerCommands, QmpEvent, Required, Result,
    Socket, TimeoutPhase,
    engine::{
        Inbound,
        connection::{self, Output, QmpConnection, Request},
        qmp::{Established, QmpHandshake},
    },
    wire,
};
use qapi_core::QapiCommand;
use serde::de::DeserializeOwned;
use serde_json::{Value, json, value::RawValue};
use std::{
    future::{Future, poll_fn},
    io,
    os::fd::OwnedFd,
    path::Path,
    pin::{Pin, pin},
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::{
    net::UnixStream,
    sync::{mpsc, oneshot, watch},
    time::timeout,
};

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

/// The handle's side of a request: the sender of its outcome.
type Waiter = oneshot::Sender<Result<Box<RawValue>>>;

/// What a handle asks of the driver.
enum Command {
    Request(Request, Waiter),
    Shutdown,
}

/// The counters that the driver keeps for the handle.
#[derive(Default)]
struct Shared {
    depth: AtomicUsize,
    dropped: AtomicU64,
}

/// The driver's side of [`QmpEvents`].
struct Events {
    tx: mpsc::Sender<QmpEvent>,
    shared: Arc<Shared>,
}

impl Events {
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
            self.shared.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Records the connection as closed when the driver ends, however it ends:
/// with the connection's reason, or [`Error::Closed`] for a driver dropped
/// before the connection closed.
struct Closing {
    closed: watch::Sender<Option<Arc<Error>>>,
    shared: Arc<Shared>,
    reason: Option<Arc<Error>>,
}

impl Drop for Closing {
    fn drop(&mut self) {
        self.shared.depth.store(0, Ordering::Relaxed);
        let reason = self
            .reason
            .take()
            .unwrap_or_else(|| Arc::new(Error::Closed));
        self.closed.send_replace(Some(reason));
    }
}

/// A QMP connection, shared through `&self`, as through `Arc<Qmp>`.
///
/// Its [`QmpDriver`] owns the socket and every request: it writes one frame
/// at a time, routes the replies and the events and enforces the deadlines,
/// so no shared mutable routing state is needed.
/// [`from_stream`](Self::from_stream) and the connecting constructors spawn
/// it on tokio; [`handshake`](Self::handshake) returns it.
///
/// At most [`QMP_QUEUE_CAPACITY`](crate::QMP_QUEUE_CAPACITY) in-band and as
/// many out-of-band requests are dispatched at a time; the others wait in the
/// driver's queue, the out-of-band ones first. Cancellation before dispatch
/// skips the request. After dispatch it cannot undo remote execution:
/// admission stays reserved until the reply. A dispatched command reaching
/// its deadline closes the session, since its outcome and the remote queue
/// depth are then unknown. Commands are never retried automatically.
pub struct Qmp {
    greeting: Greeting,
    oob: bool,
    commands: Option<PeerCommands>,
    options: ClientOptions,
    requests: mpsc::UnboundedSender<Command>,
    closed: watch::Receiver<Option<Arc<Error>>>,
    shared: Arc<Shared>,
    // The clock of the socket's runtime, which the driver's deadlines use.
    clock: fn() -> Instant,
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

    /// Handshake on an already connected Unix socket, and spawn the
    /// connection's driver on the current tokio runtime.
    pub async fn from_stream_with_options(
        stream: UnixStream,
        options: ClientOptions,
    ) -> Result<(Self, QmpEvents)> {
        let (qmp, events, driver) = Self::handshake(stream, options).await?;
        tokio::spawn(driver);
        Ok((qmp, events))
    }

    /// Handshake on a connected socket, and return the client, its events
    /// and the connection's [`QmpDriver`], for code that runs the driver
    /// itself, such as on a `LocalSet` or next to its own work.
    /// [`from_stream_with_options`](Self::from_stream_with_options) spawns it
    /// instead. Until the driver runs, no request is sent and no deadline
    /// expires.
    ///
    /// ```no_run
    /// # async fn run(stream: tokio::net::UnixStream) -> qapi_protocol::Result<()> {
    /// use qapi_protocol::{ClientOptions, Qmp};
    /// let (qmp, _events, driver) = Qmp::handshake(stream, ClientOptions::default()).await?;
    /// tokio::spawn(driver);
    /// println!("QEMU {:?}", qmp.version());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn handshake<S: Socket>(
        mut socket: S,
        options: ClientOptions,
    ) -> Result<(Self, QmpEvents, QmpDriver)> {
        options.validate()?;
        let limit = options.max_message_bytes;
        let deadline =
            S::now()
                .checked_add(options.handshake_timeout)
                .ok_or(Error::InvalidRequest(InvalidRequestKind::Options {
                    option: "handshake_timeout",
                }))?;
        let mut inbound = Inbound::new();
        let steps = async {
            let mut handshake = QmpHandshake::new(&options);
            loop {
                if let Some(frame) = handshake.poll_transmit() {
                    write_all(&mut socket, &frame).await?;
                }
                if handshake.is_done() {
                    break;
                }
                handshake.handle_frame(&read_frame(&mut socket, &mut inbound, limit).await?)?;
            }
            Ok::<_, Error>(handshake.finish())
        };
        let Established {
            greeting,
            oob,
            commands,
            events: early,
        } = within::<S, _>(deadline, steps)
            .await
            .ok_or(Error::Timeout {
                phase: TimeoutPhase::Handshake,
            })??;
        let shared = Arc::new(Shared::default());
        let (event_tx, event_rx) = mpsc::channel(options.event_capacity);
        let events = Events {
            tx: event_tx,
            shared: shared.clone(),
        };
        // The events of the handshake are queued before the client is returned.
        for event in early {
            events.deliver(event);
        }
        let (requests, request_rx) = mpsc::unbounded_channel();
        let (closed_tx, closed) = watch::channel(None);
        let closing = Closing {
            closed: closed_tx,
            shared: shared.clone(),
            reason: None,
        };
        let connection = QmpConnection::new(&options, oob, Vec::new());
        let driver = drive(
            socket, inbound, connection, request_rx, events, closing, limit,
        );
        let qmp = Self {
            greeting,
            oob,
            commands,
            options,
            requests,
            closed,
            shared,
            clock: S::now,
        };
        Ok((qmp, QmpEvents { rx: event_rx }, QmpDriver(Box::pin(driver))))
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
        self.shared.depth.load(Ordering::Relaxed)
    }
    pub fn dropped_events(&self) -> u64 {
        self.shared.dropped.load(Ordering::Relaxed)
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
    /// as through `Arc<Qmp>`, while other tasks await replies, and the
    /// driver that owns the socket must stop, which is awaited here.
    pub async fn shutdown(&self) {
        let _ = self.requests.send(Command::Shutdown);
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

    /// The driver has stopped; wait for the reason it records as it ends.
    async fn lost(&self) -> Error {
        Error::Disconnected(self.closed().await)
    }

    /// Queue a request with the driver and wait for its outcome. The driver
    /// decides every deadline, queued or dispatched, so this future needs no
    /// timer of its own.
    async fn dispatch(
        &self,
        request: Value,
        fds: Vec<OwnedFd>,
        oob: bool,
        duration: Duration,
    ) -> Result<Box<RawValue>> {
        connection::check(
            &request,
            fds.len(),
            oob,
            self.oob,
            self.options.max_message_bytes,
        )?;
        if let Some(reason) = self.close_reason() {
            return Err(Error::Disconnected(reason));
        }
        let deadline = (self.clock)()
            .checked_add(duration)
            .ok_or(Error::InvalidRequest(InvalidRequestKind::TimeoutTooLarge {
                timeout: duration,
            }))?;
        let (waiter, reply) = oneshot::channel();
        let request = Request {
            json: request,
            fds,
            oob,
            deadline,
        };
        if self
            .requests
            .send(Command::Request(request, waiter))
            .is_err()
        {
            return Err(self.lost().await);
        }
        match reply.await {
            Ok(result) => result,
            Err(_) => Err(self.lost().await),
        }
    }
}

pub(crate) fn no_success_response<C: QapiCommand>() -> Error {
    Error::InvalidRequest(InvalidRequestKind::NoSuccessResponse {
        command: C::NAME.into(),
    })
}

/// The I/O and the deadlines of one QMP connection, as a future to spawn
/// or to poll next to other work. Until it runs, no request is sent and no
/// deadline expires. It ends once the connection closes, also when the
/// [`Qmp`] is dropped. [`Qmp::from_stream`] spawns it on tokio, and
/// [`Qmp::handshake`] returns it.
#[must_use = "the driver does the connection's I/O and deadlines: spawn it or poll it"]
pub struct QmpDriver(Pin<Box<dyn Future<Output = ()> + Send + 'static>>);

impl Future for QmpDriver {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.0.as_mut().poll(cx)
    }
}

impl std::fmt::Debug for QmpDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QmpDriver").finish_non_exhaustive()
    }
}

/// `future`, unless the socket's timer reaches `deadline` first.
async fn within<S: Socket, F: Future>(deadline: Instant, future: F) -> Option<F::Output> {
    let mut future = pin!(future);
    let mut sleep = pin!(S::sleep_until(deadline));
    poll_fn(|cx| {
        if let Poll::Ready(output) = future.as_mut().poll(cx) {
            return Poll::Ready(Some(output));
        }
        if sleep.as_mut().poll(cx).is_ready() {
            return Poll::Ready(None);
        }
        Poll::Pending
    })
    .await
}

async fn write_all<S: Socket>(socket: &mut S, mut bytes: &[u8]) -> Result<()> {
    while !bytes.is_empty() {
        let n = poll_fn(|cx| socket.poll_write(cx, bytes, &[])).await?;
        if n == 0 {
            return Err(io::Error::from(io::ErrorKind::WriteZero).into());
        }
        bytes = &bytes[n..];
    }
    Ok(())
}

async fn read_frame<S: Socket>(
    socket: &mut S,
    inbound: &mut Inbound,
    limit: usize,
) -> Result<Vec<u8>> {
    loop {
        if let Some(frame) = inbound.frame(limit)? {
            return Ok(frame);
        }
        let n = poll_fn(|cx| socket.poll_read(cx, inbound.space())).await?;
        if n == 0 {
            return Err(inbound.eof());
        }
        inbound.filled(n);
    }
}

/// The iterations of one poll of the driver, so that a busy connection
/// cannot starve the other tasks of its executor.
const BUDGET: usize = 64;

/// The driver: one poll loop over the connection's engine, the handle's
/// requests, the socket and one timer.
async fn drive<S: Socket>(
    mut socket: S,
    mut inbound: Inbound,
    mut connection: QmpConnection<Waiter>,
    mut requests: mpsc::UnboundedReceiver<Command>,
    events: Events,
    mut closing: Closing,
    limit: usize,
) {
    let mut sleep: Option<(Instant, Pin<Box<S::Sleep>>)> = None;
    poll_fn(|cx| {
        for _ in 0..BUDGET {
            let mut progress = false;
            // A deadline that passed expires first, so that its call reports
            // its own timeout, not the closure that the expiry causes.
            let now = S::now();
            if connection
                .poll_timeout()
                .is_some_and(|deadline| deadline <= now)
            {
                connection.handle_timeout(now);
                progress = true;
            }
            while connection.close_reason().is_none() {
                match requests.poll_recv(cx) {
                    Poll::Ready(Some(Command::Request(request, waiter))) => {
                        progress = true;
                        if let Err((waiter, error)) = connection.submit(request, waiter) {
                            let _ = waiter.send(Err(error));
                        }
                    }
                    // A shutdown, or every handle dropped.
                    Poll::Ready(Some(Command::Shutdown) | None) => {
                        progress = true;
                        connection.close(Error::Closed);
                    }
                    Poll::Pending => break,
                }
            }
            while connection.close_reason().is_none() {
                match inbound.frame(limit) {
                    Ok(Some(frame)) => {
                        progress = true;
                        connection.handle_frame(&frame);
                    }
                    Ok(None) => break,
                    Err(error) => {
                        progress = true;
                        connection.close(error);
                    }
                }
            }
            if let Some(transmit) = connection.poll_transmit(S::now(), |waiter| !waiter.is_closed())
            {
                match socket.poll_write(cx, transmit.bytes, transmit.fds) {
                    Poll::Ready(Ok(0)) => {
                        progress = true;
                        connection.close(io::Error::from(io::ErrorKind::WriteZero).into());
                    }
                    Poll::Ready(Ok(n)) => {
                        progress = true;
                        connection.advance(n);
                    }
                    Poll::Ready(Err(error)) => {
                        progress = true;
                        connection.close(error.into());
                    }
                    Poll::Pending => {}
                }
            }
            if connection.close_reason().is_none() {
                match socket.poll_read(cx, inbound.space()) {
                    Poll::Ready(Ok(0)) => {
                        progress = true;
                        connection.close(inbound.eof());
                    }
                    Poll::Ready(Ok(n)) => {
                        progress = true;
                        inbound.filled(n);
                    }
                    Poll::Ready(Err(error)) => {
                        progress = true;
                        connection.close(error.into());
                    }
                    Poll::Pending => {}
                }
            }
            // The depth is current before a caller learns its outcome.
            closing
                .shared
                .depth
                .store(connection.queue_depth(), Ordering::Relaxed);
            while let Some(output) = connection.poll_output() {
                progress = true;
                match output {
                    Output::Reply(waiter, result) => {
                        let _ = waiter.send(result);
                    }
                    Output::Event(event) => events.deliver(Some(event)),
                    Output::EventDropped => events.deliver(None),
                }
            }
            if let Some(reason) = connection.close_reason() {
                closing.reason = Some(reason.clone());
                return Poll::Ready(());
            }
            match connection.poll_timeout() {
                Some(deadline) => {
                    if sleep.as_ref().is_none_or(|(at, _)| *at != deadline) {
                        sleep = Some((deadline, Box::pin(S::sleep_until(deadline))));
                    }
                    let (_, timer) = sleep.as_mut().expect("the timer is armed");
                    if timer.as_mut().poll(cx).is_ready() {
                        sleep = None;
                        connection.handle_timeout(S::now().max(deadline));
                        progress = true;
                    }
                }
                None => sleep = None,
            }
            if !progress {
                return Poll::Pending;
            }
        }
        // The budget is spent: let the executor run its other tasks first.
        cx.waker().wake_by_ref();
        Poll::Pending
    })
    .await;
    // The socket closes before the connection is reported closed.
    drop(socket);
    drop(events);
    drop(closing);
}
