//! The input side of a connection: the bytes read so far, split into
//! messages.
use crate::{Error, ProtocolViolation, Result};

/// The read space, as large as a default `BufReader`'s.
const READ_SPACE: usize = 8 * 1024;

/// A line of the guest agent handshake, from [`Inbound::sync_line`].
pub(crate) enum SyncLine {
    /// The line after a sentinel.
    Sentineled(Vec<u8>),
    /// A whole line that no sentinel preceded.
    Bare(Vec<u8>),
}

/// The bytes read from a peer, and the messages among them.
///
/// A client reads into [`space`](Self::space), records the count with
/// [`filled`](Self::filled), and takes messages with [`frame`](Self::frame)
/// until it returns `None`, having consumed every byte read. Partial input
/// stays here, so the client may drop a read at any point.
///
/// QMP sends one JSON object per line, but its pretty output also breaks
/// lines inside the object: a message ends at a newline outside every
/// string, object and array. The guest agent handshake scans for its
/// sentinel instead ([`sync_line`](Self::sync_line)).
pub(crate) struct Inbound {
    buf: Box<[u8]>,
    start: usize,
    end: usize,
    partial: Vec<u8>,
    depth: usize,
    in_string: bool,
    escaped: bool,
    // The guest agent handshake's scan: whether a sentinel began the
    // current line, and whether the line is being dropped as too long.
    sentinel: bool,
    skipping: bool,
}

impl Inbound {
    pub fn new() -> Self {
        Self {
            buf: vec![0; READ_SPACE].into_boxed_slice(),
            start: 0,
            end: 0,
            partial: Vec::new(),
            depth: 0,
            in_string: false,
            escaped: false,
            sentinel: false,
            skipping: false,
        }
    }

    /// The space to read into, once [`frame`](Self::frame),
    /// [`sync_line`](Self::sync_line) or `skip_to_sentinel` has consumed
    /// every byte read.
    pub fn space(&mut self) -> &mut [u8] {
        debug_assert_eq!(self.start, self.end, "unconsumed input");
        self.start = 0;
        self.end = 0;
        &mut self.buf
    }

    /// Record that `n` bytes were read into [`space`](Self::space).
    pub fn filled(&mut self, n: usize) {
        debug_assert!(n <= self.buf.len());
        self.end = n;
    }

    /// Why the input ended here: [`Error::Closed`] between messages, or a
    /// truncated message inside one.
    pub fn eof(&self) -> Error {
        if self.partial.is_empty() {
            Error::Closed
        } else {
            Error::Protocol(ProtocolViolation::Truncated)
        }
    }

    /// Drop any partial message and scan, as the guest agent handshake does
    /// before each line it reads.
    pub fn reset(&mut self) {
        self.partial.clear();
        self.depth = 0;
        self.in_string = false;
        self.escaped = false;
        self.sentinel = false;
        self.skipping = false;
    }

    /// The next message of at most `limit` JSON bytes, without its line
    /// terminator, or `None` once every byte read is consumed. Blank lines
    /// are skipped. Keep the limit until a message is returned.
    pub fn frame(&mut self, limit: usize) -> Result<Option<Vec<u8>>> {
        while self.start < self.end {
            let input = &self.buf[self.start..self.end];
            // Locate the terminator without reparsing an ever-growing prefix.
            let mut end = None;
            for (index, byte) in input.iter().copied().enumerate() {
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
            let take = end.unwrap_or(input.len());
            // A CR belongs to the terminator; leave one byte of allowance until LF.
            if self.partial.len().saturating_add(take) > limit.saturating_add(1) {
                return Err(Error::MessageTooLarge { limit });
            }
            self.partial.extend_from_slice(&input[..take]);
            self.start += take + usize::from(end.is_some());
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
                return Ok(Some(bytes));
            }
        }
        Ok(None)
    }

    /// The next line of the guest agent handshake, resetting at *every*
    /// sentinel (0xFF): the candidate that follows a sentinel, or a whole
    /// line that no sentinel preceded, such as an agent's error reply to a
    /// sync command it lacks. A line longer than `limit` is stale data, not
    /// a reply: it is dropped while the scan goes on, without retaining an
    /// unbounded buffer. `None` once every byte read is consumed.
    pub fn sync_line(&mut self, limit: usize) -> Option<SyncLine> {
        while self.start < self.end {
            let byte = self.buf[self.start];
            self.start += 1;
            match byte {
                0xff => {
                    self.partial.clear();
                    self.sentinel = true;
                    self.skipping = false;
                }
                b'\n' => {
                    let line = std::mem::take(&mut self.partial);
                    let skipped = std::mem::take(&mut self.skipping);
                    if std::mem::take(&mut self.sentinel) {
                        return Some(SyncLine::Sentineled(line));
                    }
                    if !skipped && !line.is_empty() {
                        return Some(SyncLine::Bare(line));
                    }
                }
                _ if self.skipping => {}
                _ if self.partial.len() >= limit => {
                    self.partial.clear();
                    self.sentinel = false;
                    self.skipping = true;
                }
                _ => self.partial.push(byte),
            }
        }
        None
    }

    /// Drop the input up to and including the next sentinel, as a guest
    /// agent does before a sync command. `false` once every byte read is
    /// consumed without one.
    #[cfg(feature = "mock")]
    pub fn skip_to_sentinel(&mut self) -> bool {
        match self.buf[self.start..self.end]
            .iter()
            .position(|&byte| byte == 0xff)
        {
            Some(index) => {
                self.start += index + 1;
                true
            }
            None => {
                self.start = self.end;
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(inbound: &mut Inbound, bytes: &[u8]) {
        inbound.space()[..bytes.len()].copy_from_slice(bytes);
        inbound.filled(bytes.len());
    }

    /// Every message of `chunks`, each chunk one read.
    fn frames(chunks: &[&[u8]], limit: usize) -> Result<Vec<Vec<u8>>> {
        let mut inbound = Inbound::new();
        let mut messages = Vec::new();
        for chunk in chunks {
            feed(&mut inbound, chunk);
            while let Some(message) = inbound.frame(limit)? {
                messages.push(message);
            }
        }
        Ok(messages)
    }

    fn sync_lines(inbound: &mut Inbound, limit: usize) -> Vec<(bool, Vec<u8>)> {
        let mut lines = Vec::new();
        while let Some(line) = inbound.sync_line(limit) {
            lines.push(match line {
                SyncLine::Sentineled(line) => (true, line),
                SyncLine::Bare(line) => (false, line),
            });
        }
        lines
    }

    #[test]
    fn a_message_ends_at_a_newline_outside_its_strings_objects_and_arrays() {
        let pretty = b"{\n  \"return\": [\n    1\n  ],\n  \"id\": 1\n}\r\n";
        let tricky = b"{\"s\": \"}]\\\"{\", \"t\": \"\\\\\"}\n";
        assert_eq!(
            frames(&[pretty, tricky], 1024).unwrap(),
            [&pretty[..pretty.len() - 2], &tricky[..tricky.len() - 1]]
        );
    }

    #[test]
    fn a_message_split_across_reads_is_kept_until_its_end() {
        let messages = frames(&[b"{\"ret", b"urn\": {}, \"id\"", b": 1}\n"], 1024).unwrap();
        assert_eq!(messages, [b"{\"return\": {}, \"id\": 1}".to_vec()]);
    }

    #[test]
    fn several_messages_of_one_read_come_one_at_a_time() {
        let mut inbound = Inbound::new();
        feed(&mut inbound, b"{\"a\":1}\n{\"b\":2}\r\n{\"c\"");
        assert_eq!(inbound.frame(64).unwrap().unwrap(), b"{\"a\":1}");
        assert_eq!(inbound.frame(64).unwrap().unwrap(), b"{\"b\":2}");
        assert!(inbound.frame(64).unwrap().is_none());
        feed(&mut inbound, b":3}\n");
        assert_eq!(inbound.frame(64).unwrap().unwrap(), b"{\"c\":3}");
    }

    #[test]
    fn blank_lines_are_skipped() {
        let messages = frames(&[b"\r\n  \n", b"\n{\"a\":1}\n"], 64).unwrap();
        assert_eq!(messages, [b"{\"a\":1}".to_vec()]);
    }

    #[test]
    fn the_limit_counts_json_bytes_without_the_terminator() {
        // Seven JSON bytes fit a limit of seven, with either terminator.
        assert_eq!(frames(&[b"{\"a\":1}\r\n"], 7).unwrap().len(), 1);
        assert_eq!(frames(&[b"{\"a\":1}\n"], 7).unwrap().len(), 1);
        assert!(matches!(
            frames(&[b"{\"a\":12}\n"], 7),
            Err(Error::MessageTooLarge { limit: 7 })
        ));
        // A message without its end yet fails as soon as it cannot fit.
        assert!(matches!(
            frames(&[b"{\"a\":", b"1234"], 7),
            Err(Error::MessageTooLarge { limit: 7 })
        ));
    }

    #[test]
    fn eof_between_messages_is_closed_and_inside_one_is_truncated() {
        let mut inbound = Inbound::new();
        assert!(matches!(inbound.eof(), Error::Closed));
        feed(&mut inbound, b"{\"a\":1}\n{\"b\"");
        assert!(inbound.frame(64).unwrap().is_some());
        assert!(inbound.frame(64).unwrap().is_none());
        assert!(matches!(
            inbound.eof(),
            Error::Protocol(ProtocolViolation::Truncated)
        ));
    }

    #[test]
    fn the_sync_candidate_is_the_line_after_a_sentinel() {
        let mut inbound = Inbound::new();
        inbound.reset();
        feed(
            &mut inbound,
            b"stale}\ngarbage\xff{\"return\": 5}\n{\"return\": {}}\n",
        );
        let lines = sync_lines(&mut inbound, 64);
        assert_eq!(
            lines,
            [
                (false, b"stale}".to_vec()),
                (true, b"{\"return\": 5}".to_vec()),
                (false, b"{\"return\": {}}".to_vec()),
            ]
        );
    }

    #[test]
    fn the_bytes_behind_the_sync_line_are_kept_for_the_next_message() {
        let mut inbound = Inbound::new();
        inbound.reset();
        feed(&mut inbound, b"\xff{\"return\": 5}\n{\"return\": {}}\n");
        assert!(matches!(
            inbound.sync_line(64),
            Some(SyncLine::Sentineled(line)) if line == b"{\"return\": 5}"
        ));
        assert_eq!(inbound.frame(64).unwrap().unwrap(), b"{\"return\": {}}");
    }

    #[test]
    fn a_sync_line_split_across_reads_is_kept_until_its_end() {
        let mut inbound = Inbound::new();
        inbound.reset();
        feed(&mut inbound, b"\xff{\"ret");
        assert!(inbound.sync_line(64).is_none());
        feed(&mut inbound, b"urn\": 5}\n");
        assert!(matches!(
            inbound.sync_line(64),
            Some(SyncLine::Sentineled(line)) if line == b"{\"return\": 5}"
        ));
    }

    #[test]
    fn an_overlong_sync_line_is_dropped_up_to_its_end() {
        let mut inbound = Inbound::new();
        inbound.reset();
        // Too long without a sentinel, and too long after one: neither is
        // returned, and a later sentinel starts a candidate again.
        feed(&mut inbound, b"abcdefgh\n\xffabcdefgh\nxy\n\xffok\n");
        assert_eq!(
            sync_lines(&mut inbound, 4),
            [(false, b"xy".to_vec()), (true, b"ok".to_vec())]
        );
    }

    #[test]
    fn a_sentinel_inside_an_overlong_line_starts_a_candidate() {
        let mut inbound = Inbound::new();
        inbound.reset();
        feed(&mut inbound, b"abcdefgh\xffok\n");
        assert_eq!(sync_lines(&mut inbound, 4), [(true, b"ok".to_vec())]);
    }

    #[test]
    fn reset_drops_a_partial_message() {
        let mut inbound = Inbound::new();
        feed(&mut inbound, b"{\"a\": [");
        assert!(inbound.frame(64).unwrap().is_none());
        inbound.reset();
        assert!(matches!(inbound.eof(), Error::Closed));
        feed(&mut inbound, b"\xff{\"b\":1}\n");
        assert!(matches!(
            inbound.sync_line(64),
            Some(SyncLine::Sentineled(line)) if line == b"{\"b\":1}"
        ));
    }

    #[cfg(feature = "mock")]
    #[test]
    fn skipping_to_the_sentinel_keeps_what_follows() {
        let mut inbound = Inbound::new();
        feed(&mut inbound, b"noise");
        assert!(!inbound.skip_to_sentinel());
        feed(
            &mut inbound,
            b"more\xff{\"execute\":\"guest-sync-delimited\"}\n",
        );
        assert!(inbound.skip_to_sentinel());
        assert_eq!(
            inbound.frame(64).unwrap().unwrap(),
            b"{\"execute\":\"guest-sync-delimited\"}"
        );
    }
}
