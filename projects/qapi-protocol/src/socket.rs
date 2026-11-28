//! The sockets that the QMP client runs on.
use std::{
    future::Future,
    io,
    os::fd::{AsFd, OwnedFd},
    task::{Context, Poll},
    time::Instant,
};

mod sealed {
    pub trait Sealed {}
}

/// A connected Unix socket of an async runtime, with that runtime's clock
/// and timer, as [`Qmp::handshake`](crate::Qmp::handshake) takes it: with
/// the `tokio` feature, `tokio::net::UnixStream`.
///
/// The trait is sealed: this crate implements it for the sockets of the
/// runtimes it supports, and its methods are not public API.
pub trait Socket: sealed::Sealed + Send + Unpin + 'static {
    /// The runtime's timer.
    #[doc(hidden)]
    type Sleep: Future<Output = ()> + Send;

    /// Read into `buf`; zero at the end of the input.
    #[doc(hidden)]
    fn poll_read(
        &mut self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>>;

    /// Write a prefix of `buf`, with `fds` as SCM_RIGHTS in the same
    /// `sendmsg` when there are any.
    #[doc(hidden)]
    fn poll_write(
        &mut self,
        cx: &mut Context<'_>,
        buf: &[u8],
        fds: &[OwnedFd],
    ) -> Poll<io::Result<usize>>;

    /// The runtime's clock, such as tokio's, so that its paused test clock
    /// applies to the deadlines.
    #[doc(hidden)]
    fn now() -> Instant;

    #[doc(hidden)]
    fn sleep_until(deadline: Instant) -> Self::Sleep;
}

#[cfg(feature = "tokio")]
impl sealed::Sealed for tokio::net::UnixStream {}

#[cfg(feature = "tokio")]
impl Socket for tokio::net::UnixStream {
    type Sleep = tokio::time::Sleep;

    fn poll_read(
        &mut self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut buf = tokio::io::ReadBuf::new(buf);
        std::task::ready!(tokio::io::AsyncRead::poll_read(
            std::pin::Pin::new(self),
            cx,
            &mut buf
        ))?;
        Poll::Ready(Ok(buf.filled().len()))
    }

    fn poll_write(
        &mut self,
        cx: &mut Context<'_>,
        buf: &[u8],
        fds: &[OwnedFd],
    ) -> Poll<io::Result<usize>> {
        if fds.is_empty() {
            return tokio::io::AsyncWrite::poll_write(
                std::pin::Pin::new(self),
                cx,
                buf,
            );
        }
        loop {
            std::task::ready!(self.poll_write_ready(cx))?;
            match self.try_io(tokio::io::Interest::WRITABLE, || {
                send_with_fds(&*self, buf, fds)
            }) {
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                result => return Poll::Ready(result),
            }
        }
    }

    fn now() -> Instant {
        tokio::time::Instant::now().into_std()
    }

    fn sleep_until(deadline: Instant) -> Self::Sleep {
        tokio::time::sleep_until(tokio::time::Instant::from_std(deadline))
    }
}

/// One `sendmsg` carrying `fds` as SCM_RIGHTS with a prefix of `bytes`.
/// Like `write()`, it sends nothing unless it succeeds.
#[cfg(feature = "tokio")]
fn send_with_fds(
    socket: impl AsFd,
    bytes: &[u8],
    fds: &[OwnedFd],
) -> io::Result<usize> {
    use rustix::net::{
        SendAncillaryBuffer, SendAncillaryMessage, SendFlags, sendmsg,
    };
    let borrowed: Vec<_> = fds.iter().map(AsFd::as_fd).collect();
    let mut space = vec![
        std::mem::MaybeUninit::uninit();
        rustix::cmsg_space!(ScmRights(borrowed.len()))
    ];
    let mut control = SendAncillaryBuffer::new(&mut space);
    if !control.push(SendAncillaryMessage::ScmRights(&borrowed)) {
        return Err(io::Error::other(
            "the descriptors do not fit their control message",
        ));
    }
    let iov = [io::IoSlice::new(bytes)];
    Ok(sendmsg(socket, &iov, &mut control, SendFlags::NOSIGNAL)?)
}
