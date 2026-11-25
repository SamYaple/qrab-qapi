use crate::{Error, Result};
use serde_json::Value;
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
