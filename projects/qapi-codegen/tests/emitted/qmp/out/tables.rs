/// Every type that a field, variant payload or command return refers to.
pub static TYPE_REFS: &[TypeRef] = &[
    TypeRef { kind: TypeKind::Primitive, name: "int", prim: Some(Prim::I64), inner: None },
    TypeRef { kind: TypeKind::Primitive, name: "str", prim: Some(Prim::Str), inner: None },
    TypeRef { kind: TypeKind::Primitive, name: "number", prim: Some(Prim::F64), inner: None },
    TypeRef { kind: TypeKind::Struct, name: "Circle", prim: None, inner: None },
    TypeRef { kind: TypeKind::Enum, name: "CpuArch", prim: None, inner: None },
    TypeRef { kind: TypeKind::Enum, name: "CpuTarget", prim: None, inner: None },
    TypeRef { kind: TypeKind::Struct, name: "CpuX86", prim: None, inner: None },
    TypeRef { kind: TypeKind::Struct, name: "Inner", prim: None, inner: None },
    TypeRef { kind: TypeKind::Primitive, name: "bool", prim: Some(Prim::Bool), inner: None },
    TypeRef { kind: TypeKind::Struct, name: "Ant", prim: None, inner: None },
    TypeRef { kind: TypeKind::Struct, name: "Node", prim: None, inner: None },
    TypeRef { kind: TypeKind::List, name: "List<Node>", prim: None, inner: Some(10) },
    TypeRef { kind: TypeKind::Struct, name: "Pod", prim: None, inner: None },
    TypeRef { kind: TypeKind::Enum, name: "KindInfo", prim: None, inner: None },
    TypeRef { kind: TypeKind::List, name: "List<KindInfo>", prim: None, inner: Some(13) },
    TypeRef { kind: TypeKind::Enum, name: "ShapeKind", prim: None, inner: None },
    TypeRef { kind: TypeKind::Struct, name: "CircleWrapper", prim: None, inner: None },
    TypeRef { kind: TypeKind::Struct, name: "SquareWrapper", prim: None, inner: None },
    TypeRef { kind: TypeKind::Struct, name: "Square", prim: None, inner: None },
    TypeRef { kind: TypeKind::Enum, name: "Color", prim: None, inner: None },
    TypeRef { kind: TypeKind::Struct, name: "Outer", prim: None, inner: None },
    TypeRef { kind: TypeKind::Struct, name: "Top", prim: None, inner: None },
    TypeRef { kind: TypeKind::Struct, name: "Mid", prim: None, inner: None },
];

static FIELDS_Ant: &[FieldMeta] = &[
    FieldMeta { name: "v", wire_name: "v", ty: 0, optional: false, discriminator: &[], boxed: false, since: (7u32, 0u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Ant: StructMeta = StructMeta {
    name: "Ant",
    wire_name: "ant",
    fields: FIELDS_Ant,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (7u32, 0u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Backup: &[FieldMeta] = &[
    FieldMeta { name: "target", wire_name: "target", ty: 1, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Backup: StructMeta = StructMeta {
    name: "Backup",
    wire_name: "backup",
    fields: FIELDS_Backup,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[Releases { since: (6u32, 2u32, 0u32), until: None }],
    unstable: &[Releases { since: (6u32, 2u32, 0u32), until: Some((7u32, 0u32, 0u32)) }],
};
static FIELDS_Bag: &[FieldMeta] = &[
    FieldMeta { name: "q", wire_name: "q", ty: 0, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Bag: StructMeta = StructMeta {
    name: "Bag",
    wire_name: "bag",
    fields: FIELDS_Bag,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Circle: &[FieldMeta] = &[
    FieldMeta { name: "radius", wire_name: "radius", ty: 2, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Circle: StructMeta = StructMeta {
    name: "Circle",
    wire_name: "Circle",
    fields: FIELDS_Circle,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_CircleWrapper: &[FieldMeta] = &[
    FieldMeta { name: "data", wire_name: "data", ty: 3, optional: false, discriminator: &[], boxed: false, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_CircleWrapper: StructMeta = StructMeta {
    name: "CircleWrapper",
    wire_name: "CircleWrapper",
    fields: FIELDS_CircleWrapper,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 2u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static VARIANTS_Color: &[EnumVariantMeta] = &[
    EnumVariantMeta { name: "Red", wire_name: "red", payload_struct: None, payload_type: None, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    EnumVariantMeta { name: "Blue", wire_name: "blue", payload_struct: None, payload_type: None, since: (6u32, 1u32, 0u32), until: None, deprecated: &[Releases { since: (6u32, 2u32, 0u32), until: None }], unstable: &[] },
    EnumVariantMeta { name: "Green", wire_name: "green", payload_struct: None, payload_type: None, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static ENUM_Color: EnumMeta = EnumMeta {
    name: "Color",
    wire_name: "Color",
    variants: VARIANTS_Color,
    kind: EnumKind::Unit,
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Cpu: &[FieldMeta] = &[
    FieldMeta { name: "arch", wire_name: "arch", ty: 4, optional: false, discriminator: &[Releases { since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)) }], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    FieldMeta { name: "target", wire_name: "target", ty: 5, optional: false, discriminator: &[Releases { since: (6u32, 2u32, 0u32), until: None }], boxed: false, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Cpu: StructMeta = StructMeta {
    name: "Cpu",
    wire_name: "Cpu",
    fields: FIELDS_Cpu,
    is_union: true,
    discriminator_field: Some("target"),
    branch_enum: Some("CpuBranch"),
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static VARIANTS_CpuArch: &[EnumVariantMeta] = &[
    EnumVariantMeta { name: "X86", wire_name: "x86", payload_struct: None, payload_type: None, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    EnumVariantMeta { name: "Other", wire_name: "other", payload_struct: None, payload_type: None, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static ENUM_CpuArch: EnumMeta = EnumMeta {
    name: "CpuArch",
    wire_name: "CpuArch",
    variants: VARIANTS_CpuArch,
    kind: EnumKind::Unit,
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static VARIANTS_CpuBranch: &[EnumVariantMeta] = &[
    EnumVariantMeta { name: "X86", wire_name: "x86", payload_struct: Some("CpuX86"), payload_type: Some(6), since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)), deprecated: &[], unstable: &[] },
    EnumVariantMeta { name: "Other", wire_name: "other", payload_struct: None, payload_type: None, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    EnumVariantMeta { name: "X8664", wire_name: "x86-64", payload_struct: Some("CpuX86"), payload_type: Some(6), since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static ENUM_CpuBranch: EnumMeta = EnumMeta {
    name: "CpuBranch",
    wire_name: "CpuBranch",
    variants: VARIANTS_CpuBranch,
    kind: EnumKind::UnionBranch,
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static VARIANTS_CpuTarget: &[EnumVariantMeta] = &[
    EnumVariantMeta { name: "X8664", wire_name: "x86-64", payload_struct: None, payload_type: None, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    EnumVariantMeta { name: "Other", wire_name: "other", payload_struct: None, payload_type: None, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static ENUM_CpuTarget: EnumMeta = EnumMeta {
    name: "CpuTarget",
    wire_name: "CpuTarget",
    variants: VARIANTS_CpuTarget,
    kind: EnumKind::Unit,
    since: (6u32, 2u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_CpuX86: &[FieldMeta] = &[
    FieldMeta { name: "pc", wire_name: "pc", ty: 0, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_CpuX86: StructMeta = StructMeta {
    name: "CpuX86",
    wire_name: "CpuX86",
    fields: FIELDS_CpuX86,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_EvtEvent: &[FieldMeta] = &[
    FieldMeta { name: "x", wire_name: "x", ty: 7, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_EvtEvent: StructMeta = StructMeta {
    name: "EvtEvent",
    wire_name: "EVT",
    fields: FIELDS_EvtEvent,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static VARIANTS_FlagOrCircle: &[EnumVariantMeta] = &[
    EnumVariantMeta { name: "Flag", wire_name: "flag", payload_struct: None, payload_type: Some(8), since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    EnumVariantMeta { name: "Circle", wire_name: "circle", payload_struct: Some("Circle"), payload_type: Some(3), since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static ENUM_FlagOrCircle: EnumMeta = EnumMeta {
    name: "FlagOrCircle",
    wire_name: "FlagOrCircle",
    variants: VARIANTS_FlagOrCircle,
    kind: EnumKind::Alternate,
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Flip: &[FieldMeta] = &[
];
static STRUCT_Flip: StructMeta = StructMeta {
    name: "Flip",
    wire_name: "flip",
    fields: FIELDS_Flip,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_FooInfo: &[FieldMeta] = &[
    FieldMeta { name: "name", wire_name: "name", ty: 1, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_FooInfo: StructMeta = StructMeta {
    name: "FooInfo",
    wire_name: "FooInfo",
    fields: FIELDS_FooInfo,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Gone: &[FieldMeta] = &[
];
static STRUCT_Gone: StructMeta = StructMeta {
    name: "Gone",
    wire_name: "gone",
    fields: FIELDS_Gone,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: Some((7u32, 0u32, 0u32)),
    deprecated: &[Releases { since: (6u32, 2u32, 0u32), until: Some((7u32, 0u32, 0u32)) }],
    unstable: &[],
};
static FIELDS_Inner: &[FieldMeta] = &[
    FieldMeta { name: "a", wire_name: "a", ty: 0, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Inner: StructMeta = StructMeta {
    name: "Inner",
    wire_name: "inner",
    fields: FIELDS_Inner,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static VARIANTS_KindInfo: &[EnumVariantMeta] = &[
    EnumVariantMeta { name: "One", wire_name: "one", payload_struct: None, payload_type: None, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static ENUM_KindInfo: EnumMeta = EnumMeta {
    name: "KindInfo",
    wire_name: "KindInfo",
    variants: VARIANTS_KindInfo,
    kind: EnumKind::Unit,
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Lab: &[FieldMeta] = &[
    FieldMeta { name: "z", wire_name: "z", ty: 0, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)), deprecated: &[], unstable: &[] },
];
static STRUCT_Lab: StructMeta = StructMeta {
    name: "Lab",
    wire_name: "Lab",
    fields: FIELDS_Lab,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: Some((6u32, 2u32, 0u32)),
    deprecated: &[],
    unstable: &[Releases { since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)) }],
};
static FIELDS_Loose: &[FieldMeta] = &[
    FieldMeta { name: "x", wire_name: "x", ty: 0, optional: false, discriminator: &[], boxed: false, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Loose: StructMeta = StructMeta {
    name: "Loose",
    wire_name: "loose",
    fields: FIELDS_Loose,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[Releases { since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)) }, Releases { since: (7u32, 0u32, 0u32), until: None }],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Mid: &[FieldMeta] = &[
    FieldMeta { name: "a", wire_name: "a", ty: 9, optional: false, discriminator: &[], boxed: false, since: (7u32, 0u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Mid: StructMeta = StructMeta {
    name: "Mid",
    wire_name: "mid",
    fields: FIELDS_Mid,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (7u32, 0u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static VARIANTS_Mode: &[EnumVariantMeta] = &[
    EnumVariantMeta { name: "Fast", wire_name: "fast", payload_struct: None, payload_type: None, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static ENUM_Mode: EnumMeta = EnumMeta {
    name: "Mode",
    wire_name: "Mode",
    variants: VARIANTS_Mode,
    kind: EnumKind::Unit,
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[Releases { since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)) }],
};
static FIELDS_Node: &[FieldMeta] = &[
    FieldMeta { name: "next", wire_name: "next", ty: 10, optional: true, discriminator: &[], boxed: true, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    FieldMeta { name: "children", wire_name: "children", ty: 11, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    FieldMeta { name: "label", wire_name: "label", ty: 1, optional: true, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    FieldMeta { name: "tag", wire_name: "tag", ty: 1, optional: false, discriminator: &[], boxed: false, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    FieldMeta { name: "mark", wire_name: "mark", ty: 0, optional: true, discriminator: &[], boxed: false, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[Releases { since: (6u32, 2u32, 0u32), until: None }] },
];
static STRUCT_Node: StructMeta = StructMeta {
    name: "Node",
    wire_name: "Node",
    fields: FIELDS_Node,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Old: &[FieldMeta] = &[
    FieldMeta { name: "v", wire_name: "v", ty: 8, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Old: StructMeta = StructMeta {
    name: "Old",
    wire_name: "Old",
    fields: FIELDS_Old,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_OldEvtEvent: &[FieldMeta] = &[
];
static STRUCT_OldEvtEvent: StructMeta = StructMeta {
    name: "OldEvtEvent",
    wire_name: "OLD_EVT",
    fields: FIELDS_OldEvtEvent,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: Some((7u32, 0u32, 0u32)),
    deprecated: &[Releases { since: (6u32, 2u32, 0u32), until: Some((7u32, 0u32, 0u32)) }],
    unstable: &[],
};
static FIELDS_Outer: &[FieldMeta] = &[
    FieldMeta { name: "i", wire_name: "i", ty: 7, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Outer: StructMeta = StructMeta {
    name: "Outer",
    wire_name: "outer",
    fields: FIELDS_Outer,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Pod: &[FieldMeta] = &[
    FieldMeta { name: "p", wire_name: "p", ty: 0, optional: false, discriminator: &[], boxed: false, since: (7u32, 0u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Pod: StructMeta = StructMeta {
    name: "Pod",
    wire_name: "pod",
    fields: FIELDS_Pod,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (7u32, 0u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static VARIANTS_Probe: &[EnumVariantMeta] = &[
    EnumVariantMeta { name: "On", wire_name: "on", payload_struct: None, payload_type: None, since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)), deprecated: &[], unstable: &[] },
];
static ENUM_Probe: EnumMeta = EnumMeta {
    name: "Probe",
    wire_name: "Probe",
    variants: VARIANTS_Probe,
    kind: EnumKind::Unit,
    since: (6u32, 1u32, 0u32),
    until: Some((6u32, 2u32, 0u32)),
    deprecated: &[],
    unstable: &[Releases { since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)) }],
};
static FIELDS_QueryKind: &[FieldMeta] = &[
];
static STRUCT_QueryKind: StructMeta = StructMeta {
    name: "QueryKind",
    wire_name: "query-kind",
    fields: FIELDS_QueryKind,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Quiet: &[FieldMeta] = &[
];
static STRUCT_Quiet: StructMeta = StructMeta {
    name: "Quiet",
    wire_name: "quiet",
    fields: FIELDS_Quiet,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Shape: &[FieldMeta] = &[
    FieldMeta { name: "r_type", wire_name: "type", ty: 15, optional: false, discriminator: &[Releases { since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)) }], boxed: false, since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)), deprecated: &[], unstable: &[] },
    FieldMeta { name: "r#type", wire_name: "type", ty: 15, optional: false, discriminator: &[Releases { since: (6u32, 2u32, 0u32), until: None }], boxed: false, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Shape: StructMeta = StructMeta {
    name: "Shape",
    wire_name: "Shape",
    fields: FIELDS_Shape,
    is_union: true,
    discriminator_field: Some("r#type"),
    branch_enum: Some("ShapeBranch"),
    simple_union: &[Releases { since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)) }],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static VARIANTS_ShapeBranch: &[EnumVariantMeta] = &[
    EnumVariantMeta { name: "Circle", wire_name: "circle", payload_struct: Some("CircleWrapper"), payload_type: Some(16), since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    EnumVariantMeta { name: "Square", wire_name: "square", payload_struct: Some("SquareWrapper"), payload_type: Some(17), since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    EnumVariantMeta { name: "CircleLegacy", wire_name: "circle", payload_struct: Some("Circle"), payload_type: Some(3), since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)), deprecated: &[], unstable: &[] },
    EnumVariantMeta { name: "SquareLegacy", wire_name: "square", payload_struct: Some("Square"), payload_type: Some(18), since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)), deprecated: &[], unstable: &[] },
];
static ENUM_ShapeBranch: EnumMeta = EnumMeta {
    name: "ShapeBranch",
    wire_name: "ShapeBranch",
    variants: VARIANTS_ShapeBranch,
    kind: EnumKind::UnionBranch,
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static VARIANTS_ShapeKind: &[EnumVariantMeta] = &[
    EnumVariantMeta { name: "Circle", wire_name: "circle", payload_struct: None, payload_type: None, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    EnumVariantMeta { name: "Square", wire_name: "square", payload_struct: None, payload_type: None, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static ENUM_ShapeKind: EnumMeta = EnumMeta {
    name: "ShapeKind",
    wire_name: "ShapeKind",
    variants: VARIANTS_ShapeKind,
    kind: EnumKind::Unit,
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Square: &[FieldMeta] = &[
    FieldMeta { name: "side", wire_name: "side", ty: 2, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Square: StructMeta = StructMeta {
    name: "Square",
    wire_name: "Square",
    fields: FIELDS_Square,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_SquareWrapper: &[FieldMeta] = &[
    FieldMeta { name: "data", wire_name: "data", ty: 18, optional: false, discriminator: &[], boxed: false, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_SquareWrapper: StructMeta = StructMeta {
    name: "SquareWrapper",
    wire_name: "SquareWrapper",
    fields: FIELDS_SquareWrapper,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 2u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Thing: &[FieldMeta] = &[
    FieldMeta { name: "a", wire_name: "a", ty: 0, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    FieldMeta { name: "b", wire_name: "b", ty: 1, optional: true, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: Some((7u32, 0u32, 0u32)), deprecated: &[Releases { since: (6u32, 1u32, 0u32), until: Some((7u32, 0u32, 0u32)) }], unstable: &[] },
    FieldMeta { name: "c", wire_name: "c", ty: 0, optional: true, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[Releases { since: (6u32, 1u32, 0u32), until: None }] },
    FieldMeta { name: "color", wire_name: "color", ty: 19, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    FieldMeta { name: "e", wire_name: "e", ty: 0, optional: true, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[Releases { since: (6u32, 1u32, 0u32), until: Some((7u32, 0u32, 0u32)) }], unstable: &[] },
    FieldMeta { name: "d", wire_name: "d", ty: 0, optional: false, discriminator: &[], boxed: false, since: (7u32, 0u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Thing: StructMeta = StructMeta {
    name: "Thing",
    wire_name: "Thing",
    fields: FIELDS_Thing,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[Releases { since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)) }],
};
static FIELDS_Top: &[FieldMeta] = &[
    FieldMeta { name: "o", wire_name: "o", ty: 20, optional: false, discriminator: &[], boxed: false, since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Top: StructMeta = StructMeta {
    name: "Top",
    wire_name: "Top",
    fields: FIELDS_Top,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_TopQuery: &[FieldMeta] = &[
];
static STRUCT_TopQuery: StructMeta = StructMeta {
    name: "TopQuery",
    wire_name: "top-query",
    fields: FIELDS_TopQuery,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 1u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};
static FIELDS_Zed: &[FieldMeta] = &[
    FieldMeta { name: "m", wire_name: "m", ty: 22, optional: false, discriminator: &[], boxed: false, since: (7u32, 0u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_Zed: StructMeta = StructMeta {
    name: "Zed",
    wire_name: "Zed",
    fields: FIELDS_Zed,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (7u32, 0u32, 0u32),
    until: None,
    deprecated: &[],
    unstable: &[],
};

/// The commands, sorted by wire name.
pub static COMMANDS: &[CommandMeta] = &[
    CommandMeta { name: "Ant", wire_name: "ant", args: &STRUCT_Ant, returns: None, allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (7u32, 0u32, 0u32), until: None }], since: (7u32, 0u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "Backup", wire_name: "backup", args: &STRUCT_Backup, returns: None, allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: None }], since: (6u32, 1u32, 0u32), until: None, deprecated: &[Releases { since: (6u32, 2u32, 0u32), until: None }], unstable: &[Releases { since: (6u32, 2u32, 0u32), until: Some((7u32, 0u32, 0u32)) }] },
    CommandMeta { name: "Bag", wire_name: "bag", args: &STRUCT_Bag, returns: None, allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: None }], since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "Flip", wire_name: "flip", args: &STRUCT_Flip, returns: None, allow_oob: &[Releases { since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)) }, Releases { since: (7u32, 0u32, 0u32), until: None }], allow_preconfig: &[Releases { since: (6u32, 2u32, 0u32), until: None }], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: None }], since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "Gone", wire_name: "gone", args: &STRUCT_Gone, returns: None, allow_oob: &[Releases { since: (6u32, 1u32, 0u32), until: Some((7u32, 0u32, 0u32)) }], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: Some((7u32, 0u32, 0u32)) }], since: (6u32, 1u32, 0u32), until: Some((7u32, 0u32, 0u32)), deprecated: &[Releases { since: (6u32, 2u32, 0u32), until: Some((7u32, 0u32, 0u32)) }], unstable: &[] },
    CommandMeta { name: "Inner", wire_name: "inner", args: &STRUCT_Inner, returns: None, allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: None }], since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "Loose", wire_name: "loose", args: &STRUCT_Loose, returns: None, allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: None }], since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "Mid", wire_name: "mid", args: &STRUCT_Mid, returns: None, allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (7u32, 0u32, 0u32), until: None }], since: (7u32, 0u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "Outer", wire_name: "outer", args: &STRUCT_Outer, returns: Some(12), allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: None }], since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "Pod", wire_name: "pod", args: &STRUCT_Pod, returns: None, allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (7u32, 0u32, 0u32), until: None }], since: (7u32, 0u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "QueryKind", wire_name: "query-kind", args: &STRUCT_QueryKind, returns: Some(14), allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: None }], since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "Quiet", wire_name: "quiet", args: &STRUCT_Quiet, returns: None, allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: Some((6u32, 2u32, 0u32)) }, Releases { since: (7u32, 0u32, 0u32), until: None }], since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "TopQuery", wire_name: "top-query", args: &STRUCT_TopQuery, returns: Some(21), allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 1u32, 0u32), until: None }], since: (6u32, 1u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];

/// The structs, unions, and command and event structs, by Rust name.
pub static STRUCTS: Table<StructMeta> = Table {
    entries: &[
        ("Ant", &STRUCT_Ant),
        ("Backup", &STRUCT_Backup),
        ("Bag", &STRUCT_Bag),
        ("Circle", &STRUCT_Circle),
        ("CircleWrapper", &STRUCT_CircleWrapper),
        ("Cpu", &STRUCT_Cpu),
        ("CpuX86", &STRUCT_CpuX86),
        ("EvtEvent", &STRUCT_EvtEvent),
        ("Flip", &STRUCT_Flip),
        ("FooInfo", &STRUCT_FooInfo),
        ("Gone", &STRUCT_Gone),
        ("Inner", &STRUCT_Inner),
        ("Lab", &STRUCT_Lab),
        ("Loose", &STRUCT_Loose),
        ("Mid", &STRUCT_Mid),
        ("Node", &STRUCT_Node),
        ("Old", &STRUCT_Old),
        ("OldEvtEvent", &STRUCT_OldEvtEvent),
        ("Outer", &STRUCT_Outer),
        ("Pod", &STRUCT_Pod),
        ("QueryKind", &STRUCT_QueryKind),
        ("Quiet", &STRUCT_Quiet),
        ("Shape", &STRUCT_Shape),
        ("Square", &STRUCT_Square),
        ("SquareWrapper", &STRUCT_SquareWrapper),
        ("Thing", &STRUCT_Thing),
        ("Top", &STRUCT_Top),
        ("TopQuery", &STRUCT_TopQuery),
        ("Zed", &STRUCT_Zed),
    ],
};

/// The enums, alternates and union branch enums, by Rust name.
pub static ENUMS: Table<EnumMeta> = Table {
    entries: &[
        ("Color", &ENUM_Color),
        ("CpuArch", &ENUM_CpuArch),
        ("CpuBranch", &ENUM_CpuBranch),
        ("CpuTarget", &ENUM_CpuTarget),
        ("FlagOrCircle", &ENUM_FlagOrCircle),
        ("KindInfo", &ENUM_KindInfo),
        ("Mode", &ENUM_Mode),
        ("Probe", &ENUM_Probe),
        ("ShapeBranch", &ENUM_ShapeBranch),
        ("ShapeKind", &ENUM_ShapeKind),
    ],
};
