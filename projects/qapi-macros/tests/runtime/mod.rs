//! A stand-in for the run-time support of `qrab-qapi`, which the expansion
//! calls by the path `::qapi`: the same items with the same contract, so
//! that these tests compile and run expansions of the attribute on their
//! own. Each test crate names itself `qapi` and imports these items into
//! its root. It records each name that a serializer reports unavailable.
#![allow(dead_code, reason = "each test binary uses a part of the runtime")]

use std::cell::{Cell, RefCell};

/// What the expansion uses besides the public API.
pub mod __private {
    pub use serde;
    pub use serde_json;

    pub fn unavailable_error<E: serde::ser::Error>(name: &str) -> E {
        super::UNAVAILABLE.with_borrow_mut(|names| names.push(name.to_owned()));
        E::custom(unavailable_type_message(name))
    }

    pub fn unavailable_type_message(name: &str) -> String {
        let (major, minor, patch) = super::qemu_version();
        format!("{name} is not available in QEMU {major}.{minor}.{patch}")
    }
}

thread_local! {
    static VERSION: Cell<Option<(u32, u32, u32)>> = const { Cell::new(None) };
    static DEFAULT: Cell<Option<(u32, u32, u32)>> = const { Cell::new(None) };
    static UNAVAILABLE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Scope serialization and decoding to a peer's version.
pub fn with_version<T>(version: (u32, u32, u32), f: impl FnOnce() -> T) -> T {
    let outer = VERSION.replace(Some(version));
    let result = f();
    VERSION.set(outer);
    result
}

/// Run `f` with a default version for serialization outside any scope, as
/// the real runtime's process-wide default (here per thread).
pub fn with_default<T>(version: (u32, u32, u32), f: impl FnOnce() -> T) -> T {
    let outer = DEFAULT.replace(Some(version));
    let result = f();
    DEFAULT.set(outer);
    result
}

pub fn scoped_qemu_version() -> Option<(u32, u32, u32)> {
    VERSION.get()
}

pub fn qemu_version() -> (u32, u32, u32) {
    VERSION.get().or(DEFAULT.get()).unwrap_or((0, 0, 0))
}

/// The names reported unavailable on this thread since the last call.
pub fn take_unavailable() -> Vec<String> {
    UNAVAILABLE.take()
}

pub trait QapiFlatUnion: Sized {
    fn wire_tag(&self) -> Option<&'static str> {
        None
    }

    fn deserialize_variant(
        discriminator: &str,
        map: serde_json::Map<String, serde_json::Value>,
    ) -> Result<Self, serde_json::Error>;

    fn deserialize_variant_value(
        discriminator: &str,
        value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        use serde::de::Error as _;
        match value {
            serde_json::Value::Object(map) => {
                Self::deserialize_variant(discriminator, map)
            }
            _ => Err(serde_json::Error::custom(
                "expected an object for a flat union payload",
            )),
        }
    }
}

pub trait QapiCommand: serde::Serialize {
    type Returns: serde::de::DeserializeOwned + 'static;
    const NAME: &'static str;
    const ALLOW_OOB: bool = false;
    const ALLOW_PRECONFIG: bool = false;
    const SUCCESS_RESPONSE: bool = true;
    const UNSTABLE: bool = false;
    const AVAILABLE_SINCE: (u32, u32, u32);
    const REMOVED_IN: Option<(u32, u32, u32)> = None;
    const PRECONFIG_SINCE: Option<(u32, u32, u32)> = None;

    fn allows_oob(_version: (u32, u32, u32)) -> bool {
        Self::ALLOW_OOB
    }

    fn allows_preconfig(_version: (u32, u32, u32)) -> bool {
        Self::ALLOW_PRECONFIG
    }

    fn sends_success_response(_version: (u32, u32, u32)) -> bool {
        Self::SUCCESS_RESPONSE
    }

    fn is_unstable(_version: (u32, u32, u32)) -> bool {
        Self::UNSTABLE
    }
}

pub trait PreconfigSafe: QapiCommand {}

pub trait QapiEvent: serde::de::DeserializeOwned {
    const NAME: &'static str;
}
