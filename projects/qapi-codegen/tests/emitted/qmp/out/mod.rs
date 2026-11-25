use qapi_macros::qapi;
use self::commands::*;
/// QMP struct `Circle`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct Circle {
    pub radius: f64,
}

/// QMP struct `CircleWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct CircleWrapper {
    pub data: Circle,
}

/// QMP enum `Color`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub enum Color {
    #[qapi(name = "blue")]
    #[qapi(deprecated(added = "6.2.0"))]
    Blue,
    #[qapi(name = "green")]
    #[qapi(added = "6.2.0")]
    Green,
    #[qapi(name = "red")]
    Red,
}

/// QMP union `Cpu`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct Cpu {
    #[qapi(discriminator(removed = "6.2.0"))]
    pub arch: CpuArch,
    #[qapi(discriminator)]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub target: Option<CpuTarget>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<CpuBranch>,
}

/// QMP enum `CpuArch`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub enum CpuArch {
    #[qapi(name = "other")]
    Other,
    #[qapi(name = "x86")]
    X86,
}

#[qapi(union_branch)]
#[qapi(added = "6.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum CpuBranch {
    #[qapi(name = "other")]
    Other,
    #[qapi(name = "x86")]
    #[qapi(removed = "6.2.0")]
    X86(CpuX86),
    #[qapi(name = "x86-64")]
    #[qapi(added = "6.2.0")]
    X8664(CpuX86),
}

/// QMP enum `CpuTarget`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub enum CpuTarget {
    #[qapi(name = "other")]
    Other,
    #[qapi(name = "x86-64")]
    X8664,
}

/// QMP struct `CpuX86`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct CpuX86 {
    pub pc: i64,
}

/// QMP alternate `FlagOrCircle`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub enum FlagOrCircle {
    #[qapi(name = "circle")]
    Circle(Circle),
    #[qapi(name = "flag")]
    Flag(bool),
}

/// QMP struct `FooInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct FooInfo {
    pub name: String,
}

/// QMP enum `KindInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub enum KindInfo {
    #[qapi(name = "one")]
    One,
}

/// QMP struct `Lab`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
#[qapi(unstable(added = "6.1.0"))]
#[qapi(removed = "6.2.0")]
pub struct Lab {
    pub z: i64,
}

/// QMP enum `Mode`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
#[qapi(unstable(added = "6.1.0", removed = "6.2.0"))]
pub enum Mode {
    #[qapi(name = "fast")]
    Fast,
}

/// QMP struct `Node`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct Node {
    pub children: Vec<Node>,
    #[qapi(condition = "CONFIG_LABEL")]
    pub label: Option<String>,
    #[qapi(feature = "unstable")]
    #[qapi(added = "6.2.0")]
    pub mark: Option<i64>,
    pub next: Option<Box<Node>>,
    #[qapi(condition = "CONFIG_TAG")]
    #[qapi(added = "6.2.0")]
    pub tag: Option<String>,
}

/// QMP struct `Old`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct Old {
    pub v: bool,
}

/// QMP enum `Probe`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
#[qapi(unstable(added = "6.1.0"))]
#[qapi(removed = "6.2.0")]
pub enum Probe {
    #[qapi(name = "on")]
    On,
}

/// QMP union `Shape`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
#[qapi(simple_union(removed = "6.2.0"))]
pub struct Shape {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "6.2.0")]
    #[qapi(required)]
    pub r_type: Option<ShapeKind>,
    #[qapi(discriminator)]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub r#type: Option<ShapeKind>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<ShapeBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "6.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum ShapeBranch {
    #[qapi(name = "circle")]
    #[qapi(added = "6.2.0")]
    Circle(CircleWrapper),
    #[qapi(name = "circle")]
    #[qapi(removed = "6.2.0")]
    CircleLegacy(Circle),
    #[qapi(name = "square")]
    #[qapi(added = "6.2.0")]
    Square(SquareWrapper),
    #[qapi(name = "square")]
    #[qapi(removed = "6.2.0")]
    SquareLegacy(Square),
}

/// QMP enum `ShapeKind`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub enum ShapeKind {
    #[qapi(name = "circle")]
    Circle,
    #[qapi(name = "square")]
    Square,
}

/// QMP struct `Square`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct Square {
    pub side: f64,
}

/// QMP struct `SquareWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct SquareWrapper {
    pub data: Square,
}

/// QMP struct `Thing`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
#[qapi(unstable(added = "6.1.0", removed = "6.2.0"))]
pub struct Thing {
    pub a: i64,
    #[qapi(deprecated(added = "6.1.0"))]
    #[qapi(removed = "7.0.0")]
    pub b: Option<String>,
    #[qapi(feature = "unstable")]
    pub c: Option<i64>,
    pub color: Color,
    #[qapi(added = "7.0.0")]
    #[qapi(required)]
    pub d: Option<i64>,
    #[qapi(deprecated(added = "6.1.0", removed = "7.0.0"))]
    pub e: Option<i64>,
}

/// QMP struct `Top`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct Top {
    pub o: Outer,
}

/// QMP struct `Zed`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.0.0")]
pub struct Zed {
    pub m: Mid,
}

pub mod commands;
pub mod events;
pub mod metadata;
