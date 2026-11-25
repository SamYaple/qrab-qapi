use crate::{Error, Result};
use serde_json::Value;
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

/// Partial input lives in the reader, not in a disposable read future. This
/// makes selecting reads against outbound requests and shutdown cancellation safe.
///
/// Messages are returned as their bytes. Callers decode the envelope with
/// borrowed raw values and the payload straight into its type, so neither
/// the envelope nor a plain payload becomes a `serde_json::Value` tree
/// several times its size.
pub(crate) struct JsonReader<R> {
    reader: BufReader<R>,
    partial: Vec<u8>,
    limit: usize,
    depth: usize,
    in_string: bool,
    escaped: bool,
}

impl<R: AsyncRead + Unpin> JsonReader<R> {
    pub fn new(reader: R, limit: usize) -> Self {
        Self {
            reader: BufReader::new(reader),
            partial: Vec::new(),
            limit,
            depth: 0,
            in_string: false,
            escaped: false,
        }
    }

    pub async fn next(&mut self) -> Result<Vec<u8>> {
        self.next_within(self.limit).await
    }

    /// Read one message of at most `limit` JSON bytes. Call it with the same
    /// limit until it returns: a cancelled read keeps its partial input.
    pub async fn next_within(&mut self, limit: usize) -> Result<Vec<u8>> {
        loop {
            let available = self.reader.fill_buf().await?;
            if available.is_empty() {
                return if self.partial.is_empty() {
                    Err(Error::Closed)
                } else {
                    Err(Error::Protocol("truncated JSON message at EOF".into()))
                };
            }
            // QMP pretty output contains newlines inside the JSON object.
            // Locate the terminator without reparsing an ever-growing prefix.
            let mut end = None;
            for (index, byte) in available.iter().copied().enumerate() {
                if byte == b'\n' && self.depth == 0 && !self.in_string {
                    end = Some(index);
                    break;
                }
                if self.in_string {
                    if self.escaped {
                        self.escaped = false;
                    } else if byte == b'\\' {
                        self.escaped = true;
                    } else if byte == b'"' {
                        self.in_string = false;
                    }
                } else {
                    match byte {
                        b'"' => self.in_string = true,
                        b'{' | b'[' => self.depth += 1,
                        b'}' | b']' => self.depth = self.depth.saturating_sub(1),
                        _ => {}
                    }
                }
            }
            let take = end.unwrap_or(available.len());
            // A CR belongs to the terminator; leave one byte of allowance until LF.
            if self.partial.len().saturating_add(take) > limit.saturating_add(1) {
                return Err(Error::MessageTooLarge { limit });
            }
            self.partial.extend_from_slice(&available[..take]);
            self.reader.consume(take + usize::from(end.is_some()));
            if end.is_some() {
                if self.partial.last() == Some(&b'\r') {
                    self.partial.pop();
                }
                if self.partial.len() > limit {
                    return Err(Error::MessageTooLarge { limit });
                }
                self.escaped = false;
                let bytes = std::mem::take(&mut self.partial);
                if bytes.iter().all(u8::is_ascii_whitespace) {
                    continue;
                }
                return Ok(bytes);
            }
        }
    }

    #[cfg(feature = "mock")]
    pub async fn reset_at_sentinel(&mut self) -> Result<()> {
        self.partial.clear();
        self.depth = 0;
        self.in_string = false;
        self.escaped = false;
        while self.reader.read_u8().await? != 0xff {}
        Ok(())
    }

    /// Return the next line of the QGA handshake, resetting at *every*
    /// sentinel: a candidate message following a sentinel, or a whole line
    /// that no sentinel preceded, such as an agent's error reply to a sync
    /// command it lacks. The QGA handshake validates the candidate's envelope
    /// and nonce under its overall deadline and ignores stale or malformed
    /// data.
    pub async fn sync_message(&mut self) -> Result<SyncLine> {
        self.partial.clear();
        self.depth = 0;
        self.in_string = false;
        self.escaped = false;
        let mut started = false;
        // Set while dropping the rest of a line longer than the limit.
        let mut skipping = false;
        loop {
            let byte = self.reader.read_u8().await?;
            match byte {
                0xff => {
                    self.partial.clear();
                    started = true;
                    skipping = false;
                }
                b'\n' => {
                    let line = std::mem::take(&mut self.partial);
                    if started {
                        return Ok(SyncLine::Sentineled(line));
                    }
                    if !skipping && !line.is_empty() {
                        return Ok(SyncLine::Bare(line));
                    }
                    skipping = false;
                }
                _ if skipping => {}
                _ if self.partial.len() >= self.limit => {
                    // Stale data is not a protocol reply. Keep scanning for
                    // another sentinel without retaining an unbounded buffer.
                    self.partial.clear();
                    started = false;
                    skipping = true;
                }
                _ => self.partial.push(byte),
            }
        }
    }
}

/// A line of the QGA handshake, from [`JsonReader::sync_message`].
pub(crate) enum SyncLine {
    /// The line after a sentinel.
    Sentineled(Vec<u8>),
    /// A whole line that no sentinel preceded.
    Bare(Vec<u8>),
}

pub(crate) fn encode(value: &Value, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| Error::Encode(error.into()))?;
    if bytes.len() > limit {
        return Err(Error::InvalidRequest(format!(
            "JSON request exceeds {limit} bytes"
        )));
    }
    bytes.extend_from_slice(b"\r\n");
    Ok(bytes)
}

/// write_all can partially write or be cancelled. The owner must discard the
/// session on any failure; putting bytes in one buffer does not make I/O atomic.
pub(crate) async fn write_json<W: AsyncWrite + Unpin>(
    writer: &mut W,
    value: &Value,
    limit: usize,
) -> Result<()> {
    let bytes = encode(value, limit)?;
    writer.write_all(&bytes).await?;
    Ok(())
}

/// Write a whole frame, recording in `sent` whether any byte left. `write()`
/// is cancellation safe, so a cancelled call with `sent` false wrote nothing.
pub(crate) async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    bytes: &[u8],
    sent: &mut bool,
) -> Result<()> {
    let mut offset = 0;
    while offset < bytes.len() {
        let n = writer.write(&bytes[offset..]).await?;
        if n == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::WriteZero).into());
        }
        *sent = true;
        offset += n;
    }
    Ok(())
}
