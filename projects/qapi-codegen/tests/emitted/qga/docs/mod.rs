use crate as qapi;
use qapi_macros::qapi;
/// QEMU Guest Agent struct `GuestInfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "7.0.0")]
pub struct GuestInfo {
    pub version: String,
}

pub mod commands;
pub mod events;
pub mod metadata;
