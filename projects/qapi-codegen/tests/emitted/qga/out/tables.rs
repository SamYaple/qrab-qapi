/// Every type that a field, variant payload or command return refers to.
pub static TYPE_REFS: &[TypeRef] = &TYPES;
static TYPES: [TypeRef; 3] = [
    TypeRef { kind: TypeKind::Primitive, name: "str", prim: Some(Prim::Str), element: None },
    TypeRef { kind: TypeKind::Struct, name: "GuestInfo", prim: None, element: None },
    TypeRef { kind: TypeKind::Primitive, name: "int", prim: Some(Prim::I64), element: None },
];

static FIELDS_GuestInfo: &[FieldMeta] = &[
    FieldMeta { name: "version", wire_name: "version", ty: &TYPES[0], optional: false, discriminator: &[], boxed: false, since: (7u32, 0u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_GuestInfo: StructMeta = StructMeta {
    name: "GuestInfo",
    wire_name: "GuestInfo",
    fields: FIELDS_GuestInfo,
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
static FIELDS_GuestInfoCommand: &[FieldMeta] = &[
];
static STRUCT_GuestInfoCommand: StructMeta = StructMeta {
    name: "GuestInfoCommand",
    wire_name: "guest-info",
    fields: FIELDS_GuestInfoCommand,
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
static FIELDS_GuestShutdown: &[FieldMeta] = &[
    FieldMeta { name: "mode", wire_name: "mode", ty: &TYPES[0], optional: true, discriminator: &[], boxed: false, since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
];
static STRUCT_GuestShutdown: StructMeta = StructMeta {
    name: "GuestShutdown",
    wire_name: "guest-shutdown",
    fields: FIELDS_GuestShutdown,
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
static FIELDS_GuestSync: &[FieldMeta] = &[
    FieldMeta { name: "id", wire_name: "id", ty: &TYPES[2], optional: false, discriminator: &[], boxed: false, since: (6u32, 2u32, 0u32), until: Some((7u32, 0u32, 0u32)), deprecated: &[], unstable: &[] },
];
static STRUCT_GuestSync: StructMeta = StructMeta {
    name: "GuestSync",
    wire_name: "guest-sync",
    fields: FIELDS_GuestSync,
    is_union: false,
    discriminator_field: None,
    branch_enum: None,
    simple_union: &[],
    free_form: &[],
    since: (6u32, 2u32, 0u32),
    until: Some((7u32, 0u32, 0u32)),
    deprecated: &[],
    unstable: &[],
};

/// The commands, sorted by wire name.
pub static COMMANDS: &[CommandMeta] = &[
    CommandMeta { name: "GuestInfoCommand", wire_name: "guest-info", args: &STRUCT_GuestInfoCommand, returns: Some(&TYPES[1]), allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (7u32, 0u32, 0u32), until: None }], since: (7u32, 0u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "GuestShutdown", wire_name: "guest-shutdown", args: &STRUCT_GuestShutdown, returns: None, allow_oob: &[], allow_preconfig: &[], success_response: &[], since: (6u32, 2u32, 0u32), until: None, deprecated: &[], unstable: &[] },
    CommandMeta { name: "GuestSync", wire_name: "guest-sync", args: &STRUCT_GuestSync, returns: Some(&TYPES[2]), allow_oob: &[], allow_preconfig: &[], success_response: &[Releases { since: (6u32, 2u32, 0u32), until: Some((7u32, 0u32, 0u32)) }], since: (6u32, 2u32, 0u32), until: Some((7u32, 0u32, 0u32)), deprecated: &[], unstable: &[] },
];

/// The structs, unions, and command and event structs, by Rust name.
pub static STRUCTS: Table<StructMeta> = Table {
    entries: &[
        ("GuestInfo", &STRUCT_GuestInfo),
        ("GuestInfoCommand", &STRUCT_GuestInfoCommand),
        ("GuestShutdown", &STRUCT_GuestShutdown),
        ("GuestSync", &STRUCT_GuestSync),
    ],
};

/// The enums, alternates and union branch enums, by Rust name.
pub static ENUMS: Table<EnumMeta> = Table {
    entries: &[
    ],
};
