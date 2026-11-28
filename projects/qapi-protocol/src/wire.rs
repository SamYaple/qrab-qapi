use crate::{
    Error, InvalidRequestKind, PeerCommands, ProtocolViolation, RemoteError,
    Required, Result, Unavailability,
};
use qapi_core::QapiCommand;
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use serde_json::{Value, json, value::RawValue};
use std::any::TypeId;

pub(crate) fn request(name: &str, args: Value, oob: bool) -> Result<Value> {
    if name.is_empty() {
        return Err(Error::InvalidRequest(
            InvalidRequestKind::EmptyCommandName,
        ));
    }
    if !args.is_object() {
        return Err(Error::InvalidRequest(
            InvalidRequestKind::ArgumentsNotObject { command: name.into() },
        ));
    }
    let verb = if oob { "exec-oob" } else { "execute" };
    let mut value = json!({verb: name});
    if !args.as_object().unwrap().is_empty() {
        value["arguments"] = args;
    }
    Ok(value)
}

/// Serialize a typed command for a peer release and the command list the
/// peer gave, if any. A command outside the peer's release, a command the
/// peer did not list, or a type, variant or field it uses that is outside
/// the release, is `Unavailable`.
pub(crate) fn command<C: QapiCommand>(
    cmd: &C,
    version: (u32, u32, u32),
    peer: Option<&PeerCommands>,
    oob: bool,
) -> Result<Value> {
    if let Some(reason) = Required::of::<C>().refusal(version, peer) {
        return Err(Error::Unavailable {
            name: C::NAME.into(),
            version,
            reason,
        });
    }
    let args =
        qapi_core::to_value(version, cmd).map_err(|error| match error {
            qapi_core::EncodeError::Unavailable { name, version } => {
                Error::Unavailable {
                    name,
                    version,
                    reason: Unavailability::Release,
                }
            }
            error => Error::Encode(error),
        })?;
    request(C::NAME, args, oob)
}

/// Refuse a raw command `name` that the peer did not list. Without a list,
/// any name goes to the peer.
pub(crate) fn offered(
    name: &str,
    version: (u32, u32, u32),
    peer: Option<&PeerCommands>,
) -> Result<()> {
    match peer.and_then(|peer| peer.refusal(name)) {
        Some(reason) => {
            Err(Error::Unavailable { name: name.into(), version, reason })
        }
        None => Ok(()),
    }
}

/// The top-level members of a message, borrowed from its bytes without
/// parsing them. A present member is `Some`, even when it is `null`.
#[derive(Deserialize)]
pub(crate) struct Envelope<'a> {
    #[serde(default, borrow, rename = "return", deserialize_with = "present")]
    pub ret: Option<&'a RawValue>,
    #[serde(default, borrow, deserialize_with = "present")]
    pub error: Option<&'a RawValue>,
    #[serde(default, borrow, deserialize_with = "present")]
    pub id: Option<&'a RawValue>,
    #[serde(default, borrow, deserialize_with = "present")]
    pub event: Option<&'a RawValue>,
    #[serde(default, borrow, deserialize_with = "present")]
    pub data: Option<&'a RawValue>,
    #[serde(default, borrow, rename = "QMP", deserialize_with = "present")]
    pub greeting: Option<&'a RawValue>,
}

fn present<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<&'de RawValue>, D::Error> {
    <&RawValue>::deserialize(deserializer).map(Some)
}

impl Envelope<'_> {
    /// Our request IDs are numbers; any other ID is not ours.
    pub fn numeric_id(&self) -> Option<u64> {
        self.id.and_then(|id| serde_json::from_str(id.get()).ok())
    }
}

pub(crate) fn envelope(bytes: &[u8]) -> Result<Envelope<'_>> {
    let envelope =
        serde_json::from_slice(bytes).map_err(|error| {
            match error.classify() {
                serde_json::error::Category::Data => {
                    Error::Protocol(ProtocolViolation::NotAnObject(Some(error)))
                }
                _ => Error::Malformed(error),
            }
        })?;
    // Serde also reads a struct from an array of its members in order, so
    // `[1]` would otherwise be the reply `{"return": 1}`.
    if bytes.trim_ascii_start().first() != Some(&b'{') {
        return Err(Error::Protocol(ProtocolViolation::NotAnObject(None)));
    }
    Ok(envelope)
}

/// Missing return is never success, and null is a real return value. An error
/// envelope must preserve both string fields, including unknown class names.
pub(crate) fn response<'a>(envelope: &Envelope<'a>) -> Result<&'a RawValue> {
    if envelope.event.is_some()
        || envelope.greeting.is_some()
        || envelope.ret.is_some() == envelope.error.is_some()
    {
        return Err(Error::Protocol(ProtocolViolation::ResponseShape));
    }
    if let Some(error) = envelope.error {
        let error: RemoteError =
            serde_json::from_str(error.get()).map_err(|e| {
                Error::Protocol(ProtocolViolation::InvalidErrorResponse(e))
            })?;
        return Err(Error::Remote(error));
    }
    Ok(envelope.ret.unwrap())
}

/// Decode a reply straight from its bytes into the return type.
pub(crate) fn decode<T: DeserializeOwned + 'static>(
    raw: &RawValue,
) -> Result<T> {
    // QEMU returns {} for commands without a return value. Do not turn other
    // failed decodes into null: Option<T> would silently become None.
    if TypeId::of::<T>() == TypeId::of::<()>() && is_empty_object(raw) {
        return serde_json::from_str("null").map_err(Error::Decode);
    }
    serde_json::from_str(raw.get()).map_err(Error::Decode)
}

fn is_empty_object(raw: &RawValue) -> bool {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Empty {}
    raw.get().starts_with('{')
        && serde_json::from_str::<Empty>(raw.get()).is_ok()
}
