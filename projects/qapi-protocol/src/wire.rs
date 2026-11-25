use crate::{Error, RemoteError, Result};
use qapi::QapiCommand;
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use serde_json::{Value, json, value::RawValue};
use std::any::TypeId;

pub(crate) fn request(name: &str, args: Value, oob: bool) -> Result<Value> {
    if name.is_empty() || !args.is_object() {
        return Err(Error::InvalidRequest(
            "command name must be nonempty and arguments must be an object".into(),
        ));
    }
    let verb = if oob { "exec-oob" } else { "execute" };
    let mut value = json!({verb: name});
    if !args.as_object().unwrap().is_empty() {
        value["arguments"] = args;
    }
    Ok(value)
}

/// Serialize a typed command for a peer release. A command outside the
/// peer's release, or a type, variant or field it uses that is, is
/// `Unavailable`.
pub(crate) fn command<C: QapiCommand>(
    cmd: &C,
    version: (u32, u32, u32),
    oob: bool,
) -> Result<Value> {
    if version < C::AVAILABLE_SINCE || C::REMOVED_IN.is_some_and(|removed| version >= removed) {
        return Err(Error::Unavailable {
            name: C::NAME.into(),
            version,
        });
    }
    let args = qapi::to_value(version, cmd).map_err(|error| match error {
        qapi::EncodeError::Unavailable { name, version } => Error::Unavailable { name, version },
        error => Error::Encode(error),
    })?;
    request(C::NAME, args, oob)
}

pub(crate) fn validate_request(value: &Value, oob: bool) -> Result<()> {
    let verb = if oob { "exec-oob" } else { "execute" };
    let object = value
        .as_object()
        .ok_or_else(|| Error::InvalidRequest("QMP request must be an object".into()))?;
    if !object
        .get(verb)
        .and_then(Value::as_str)
        .is_some_and(|s| !s.is_empty())
        || object.keys().any(|k| k != verb && k != "arguments")
        || object.get("arguments").is_some_and(|v| !v.is_object())
    {
        return Err(Error::InvalidRequest(format!(
            "expected {verb} and optional object arguments; IDs are owned by the connection"
        )));
    }
    Ok(())
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
    let envelope = serde_json::from_slice(bytes).map_err(|error| match error.classify() {
        serde_json::error::Category::Data => {
            Error::Protocol(format!("message must be a JSON object: {error}"))
        }
        _ => Error::Malformed(error),
    })?;
    // Serde also reads a struct from an array of its members in order, so
    // `[1]` would otherwise be the reply `{"return": 1}`.
    if bytes.trim_ascii_start().first() != Some(&b'{') {
        return Err(Error::Protocol("message must be a JSON object".into()));
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
        return Err(Error::Protocol(
            "response must contain exactly one of return or error".into(),
        ));
    }
    if let Some(error) = envelope.error {
        let error: RemoteError = serde_json::from_str(error.get())
            .map_err(|e| Error::Protocol(format!("invalid error response: {e}")))?;
        return Err(Error::Remote(error));
    }
    Ok(envelope.ret.unwrap())
}

/// Decode a reply straight from its bytes into the return type.
pub(crate) fn decode<T: DeserializeOwned + 'static>(raw: &RawValue) -> Result<T> {
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
    raw.get().starts_with('{') && serde_json::from_str::<Empty>(raw.get()).is_ok()
}
