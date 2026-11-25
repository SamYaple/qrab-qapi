//! A stand-in for the run-time support of `qrab-qapi`, which the expansion
//! calls by the path `qapi`: the same items with the same contract, so that
//! these tests compile and run expansions of the attribute on their own.
//! It records each name that a serializer reports unavailable.
#![allow(dead_code, reason = "each test binary uses a part of the runtime")]

use std::cell::{Cell, RefCell};

pub use serde;
pub use serde_json;

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

pub fn unavailable_error<E: serde::ser::Error>(name: &str) -> E {
    UNAVAILABLE.with_borrow_mut(|names| names.push(name.to_owned()));
    E::custom(unavailable_type_message(name))
}

pub fn unavailable_type_message(name: &str) -> String {
    let (major, minor, patch) = qemu_version();
    format!("{name} is not available in QEMU {major}.{minor}.{patch}")
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
            serde_json::Value::Object(map) => Self::deserialize_variant(discriminator, map),
            _ => Err(serde_json::Error::custom(
                "expected an object for a flat union payload",
            )),
        }
    }
}
