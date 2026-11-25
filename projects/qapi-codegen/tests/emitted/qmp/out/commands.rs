use qapi_macros::qapi;
use std::collections::HashMap;
use super::*;
/// QMP command `ant`; see the QEMU QMP Reference Manual.
#[qapi(name = "ant")]
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(added = "7.0.0")]
pub struct Ant {
    pub v: i64,
}

/// QMP command `backup`; see the QEMU QMP Reference Manual.
#[qapi(name = "backup")]
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(added = "6.1.0")]
#[qapi(deprecated(added = "6.2.0"))]
#[qapi(unstable(added = "6.2.0", removed = "7.0.0"))]
pub struct Backup {
    pub target: String,
}

/// QMP command `bag`; see the QEMU QMP Reference Manual.
#[qapi(name = "bag")]
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(added = "6.1.0")]
pub struct Bag {
    pub q: i64,
}

/// QMP command `flip`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "flip")]
#[qapi(returns = "()")]
#[qapi(added = "6.1.0")]
#[qapi(allow_oob(removed = "6.2.0"))]
#[qapi(allow_oob(added = "7.0.0"))]
#[qapi(allow_preconfig(added = "6.2.0"))]
pub struct Flip {}

/// QMP command `gone`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "gone")]
#[qapi(returns = "()")]
#[qapi(added = "6.1.0")]
#[qapi(deprecated(added = "6.2.0"))]
#[qapi(removed = "7.0.0")]
#[qapi(allow_oob)]
pub struct Gone {}

/// QMP command `inner`; see the QEMU QMP Reference Manual.
#[qapi(name = "inner")]
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(added = "6.1.0")]
pub struct Inner {
    pub a: i64,
}

/// QMP command `loose`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "loose")]
#[qapi(returns = "()")]
#[qapi(added = "6.1.0")]
pub struct Loose {
    /// Additional command arguments not modeled by this schema.
    #[qapi(flatten)]
    #[qapi(added = "7.0.0")]
    pub arguments: Option<HashMap<String, serde_json::Value>>,
    /// Additional command arguments not modeled by this schema.
    #[qapi(flatten)]
    #[qapi(removed = "6.2.0")]
    pub arguments_legacy: Option<HashMap<String, serde_json::Value>>,
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub x: Option<i64>,
}

/// QMP command `mid`; see the QEMU QMP Reference Manual.
#[qapi(name = "mid")]
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(added = "7.0.0")]
pub struct Mid {
    pub a: Ant,
}

/// QMP command `outer`; see the QEMU QMP Reference Manual.
#[qapi(name = "outer")]
#[qapi(command)]
#[qapi(returns = "Pod")]
#[qapi(added = "6.1.0")]
#[qapi(returns(added = "6.1.0", value = "()"))]
#[qapi(returns(added = "6.2.0", value = "Bag"))]
#[qapi(returns(added = "7.0.0", value = "Pod"))]
pub struct Outer {
    pub i: Inner,
}

/// QMP command `pod`; see the QEMU QMP Reference Manual.
#[qapi(name = "pod")]
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(added = "7.0.0")]
pub struct Pod {
    pub p: i64,
}

/// QMP command `query-kind`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-kind")]
#[qapi(returns = "Vec<KindInfo>")]
#[qapi(added = "6.1.0")]
#[qapi(returns(added = "6.1.0", value = "KindInfo"))]
#[qapi(returns(added = "7.0.0", value = "Vec<KindInfo>"))]
pub struct QueryKind {}

/// QMP command `quiet`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "quiet")]
#[qapi(returns = "()")]
#[qapi(added = "6.1.0")]
#[qapi(no_success_response(added = "6.2.0", removed = "7.0.0"))]
pub struct Quiet {}

/// QMP command `top-query`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "top-query")]
#[qapi(returns = "Top")]
#[qapi(added = "6.1.0")]
pub struct TopQuery {}
