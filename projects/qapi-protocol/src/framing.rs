use crate::{
    Result,
    engine::{Inbound, SyncLine, qga::QgaCall},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// The messages of a peer, read from `R` into the engine's [`Inbound`].
/// Partial input lives there, not in a disposable read future. This makes
/// selecting reads against outbound requests and shutdown cancellation safe.
///
/// Messages are returned as their bytes. Callers decode the envelope with
/// borrowed raw values and the payload straight into its type, so neither
/// the envelope nor a plain payload becomes a `serde_json::Value` tree
/// several times its size.
pub(crate) struct JsonReader<R> {
    reader: R,
    inbound: Inbound,
    limit: usize,
}

impl<R: AsyncRead + Unpin> JsonReader<R> {
    pub fn new(reader: R, limit: usize) -> Self {
        Self {
            reader,
            inbound: Inbound::new(),
            limit,
        }
    }

    pub async fn next(&mut self) -> Result<Vec<u8>> {
        self.next_within(self.limit).await
    }

    /// Read one message of at most `limit` JSON bytes. Call it with the same
    /// limit until it returns: a cancelled read keeps its partial input.
    pub async fn next_within(&mut self, limit: usize) -> Result<Vec<u8>> {
        loop {
            if let Some(message) = self.inbound.frame(limit)? {
                return Ok(message);
            }
            let n = self.reader.read(self.inbound.space()).await?;
            if n == 0 {
                return Err(self.inbound.eof());
            }
            self.inbound.filled(n);
        }
    }

    #[cfg(feature = "mock")]
    pub async fn reset_at_sentinel(&mut self) -> Result<()> {
        self.inbound.reset();
        while !self.inbound.skip_to_sentinel() {
            self.read_inside_line().await?;
        }
        Ok(())
    }

    /// Return the next line of the QGA handshake, as
    /// [`Inbound::sync_line`] scans it. The QGA handshake validates the
    /// candidate's envelope and nonce under its overall deadline and ignores
    /// stale or malformed data.
    pub async fn sync_message(&mut self) -> Result<SyncLine> {
        self.inbound.reset();
        loop {
            if let Some(line) = self.inbound.sync_line(self.limit) {
                return Ok(line);
            }
            self.read_inside_line().await?;
        }
    }

    /// Read more of a sentinel scan, where the end of input is unexpected.
    async fn read_inside_line(&mut self) -> Result<()> {
        let n = self.reader.read(self.inbound.space()).await?;
        if n == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "early eof").into());
        }
        self.inbound.filled(n);
        Ok(())
    }
}

/// write_all can partially write or be cancelled. The owner must discard the
/// session on any failure; putting bytes in one buffer does not make I/O atomic.
#[cfg(feature = "mock")]
pub(crate) async fn write_json<W: AsyncWrite + Unpin>(
    writer: &mut W,
    value: &serde_json::Value,
    limit: usize,
) -> Result<()> {
    let bytes = crate::engine::encode(value, limit)?;
    writer.write_all(&bytes).await?;
    Ok(())
}

/// Write a guest agent call's request, recording each write in the call.
/// `write()` is cancellation safe, so a cancelled write leaves the call's
/// count of written bytes right.
pub(crate) async fn write_call<W: AsyncWrite + Unpin>(
    writer: &mut W,
    call: &mut QgaCall,
) -> Result<()> {
    while !call.transmit().is_empty() {
        let n = writer.write(call.transmit()).await?;
        if n == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::WriteZero).into());
        }
        call.advance(n);
    }
    Ok(())
}
