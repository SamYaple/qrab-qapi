//! The protocol's engine: QMP and the guest agent protocol as state
//! machines that take the bytes a client read and give it the bytes to
//! write.
//!
//! Nothing here reads or writes a socket, reads a clock or awaits, so the
//! clients of every runtime share these rules.
mod inbound;
pub(crate) mod qga;
pub(crate) mod qmp;

pub(crate) use inbound::{Inbound, SyncLine};
pub use qmp::{Greeting, QemuVersion, VersionInfo};

use crate::{Error, InvalidRequestKind, Result};
use serde_json::Value;

/// A request's bytes on the wire: its JSON, of at most `limit` bytes, and
/// the line terminator.
pub(crate) fn encode(value: &Value, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| Error::Encode(error.into()))?;
    if bytes.len() > limit {
        return Err(Error::InvalidRequest(InvalidRequestKind::RequestTooLarge {
            size: bytes.len(),
            limit,
        }));
    }
    bytes.extend_from_slice(b"\r\n");
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_request_is_its_json_and_a_crlf() {
        let bytes = encode(&json!({"execute": "stop"}), 64).unwrap();
        assert_eq!(bytes, b"{\"execute\":\"stop\"}\r\n");
    }

    #[test]
    fn the_request_limit_counts_json_bytes_without_the_terminator() {
        let request = json!({"execute": "stop"});
        let size = serde_json::to_vec(&request).unwrap().len();
        assert!(encode(&request, size).is_ok());
        assert!(matches!(
            encode(&request, size - 1),
            Err(Error::InvalidRequest(InvalidRequestKind::RequestTooLarge { size: s, limit }))
                if s == size && limit == size - 1
        ));
    }
}
