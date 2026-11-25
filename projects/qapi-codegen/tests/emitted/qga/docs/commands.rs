use qapi_macros::qapi;
use super::*;
/// QEMU Guest Agent command `guest-info`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(returns = "GuestInfo")]
#[qapi(name = "guest-info")]
#[qapi(added = "7.0.0")]
pub struct GuestInfoCommand {}

/// Shut the guest down; there is no reply.
#[qapi(command)]
#[qapi(name = "guest-shutdown")]
#[qapi(returns = "()")]
#[qapi(added = "6.2.0")]
#[qapi(no_success_response)]
pub struct GuestShutdown {
    /// how.
    pub mode: Option<String>,
}

/// QEMU Guest Agent command `guest-sync`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-sync")]
#[qapi(returns = "i64")]
#[qapi(added = "6.2.0")]
#[qapi(removed = "7.0.0")]
pub struct GuestSync {
    pub id: i64,
}
