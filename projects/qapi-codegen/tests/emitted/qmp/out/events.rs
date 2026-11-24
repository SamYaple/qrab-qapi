use crate as qapi;
use qapi_macros::qapi;
use super::*;
/// QMP event `EVT`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "EVT")]
#[qapi(added = "6.1.0")]
pub struct EvtEvent {
    pub x: Inner,
}

/// QMP event `OLD_EVT`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "OLD_EVT")]
#[qapi(added = "6.1.0")]
#[qapi(deprecated(added = "6.2.0"))]
#[qapi(removed = "7.0.0")]
pub struct OldEvtEvent {}
