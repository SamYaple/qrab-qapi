# QAPI generation

`qapi-codegen` uses nom **8.0.0** and parses QEMU's QAPI language into borrowed
schema tokens, lowers these to Rust IR, then merges releases to annotate
introductions, removals, flags and fields. QMP and QGA use the same pipeline.

## Validation

Each release's schema is checked before lowering as its own QEMU generator
checks it, in the phases of that generator: before 2.4 only union bases and
discriminators (from 2.0) and the members of event data (from 2.1); from 2.4
to 4.1 a first pass over every definition's keys, flag values, name and
uniqueness, then each expression, then from 2.5 the building of the schema
and from 2.6 its entities; from 4.2 each expression, then the definitions,
which builds the schema and finds the duplicates, then the entities
(references, members and their clashes, branches, arguments), and from 10.0
the limit of 64 features. The entities are the definitions and the types
that QEMU makes for them, each checked before its definition: the types of
arguments and union bases written out in place, array types and simple
unions' tag enums. Documentation comments are checked per definition from
2.9 to 8.2 and after all of them from 9.0. Definitions are taken in QEMU's
order, each included file in place of its include. The first error that
QEMU reports is the one returned, as a typed `SchemaErrorKind` in the file
and at the line, and for syntax errors the column, that QEMU names. A table
over every negative test schema of the pinned QEMU checks the kind, file,
line and column of each.

QEMU parses a whole file before it checks the shapes of its values, so from
2.4 the parser keeps every key, with the keys of members, branches, enum
values and features written as dictionaries, in its order in the source,
and every value of any JSON shape that the release's tokenizer reads. A
value of a shape that its place does not take, such as an enum's `data`
that is no array, a member's type that is an array of two names, a name
that is no string, a flag that is no boolean or, from 6.2, a condition that
is no configuration name, `all`, `any` or `not`, is refused where QEMU
checks it, as `ValueShape` (with the `Shape` its place takes),
`NameNotString`, `FlagValue`, `InvalidCondition`, `ConditionKeys` and the
other kinds that QEMU's checks name. Unknown and missing keys are reported
in QEMU's order: key by key in the source's order until 3.1, from 4.0 the
missing keys before the unknown ones. QEMU reads `null` until 4.1, and
takes it for an absent key where it looks the key up with `get`: the
arguments and return type of a command, the data of an event, the base of a
struct or union, the discriminator, the prefix and the conditions.

Before 2.4 the generators skip an expression without a metatype they read
(`enum`, `type`, `union`, `command`, and from 2.1 `event` and `include`),
such as a later `struct`, `alternate` or `pragma`, and ignore the keys they
do not read, whatever they hold: a union's `base` and `discriminator` before
1.6, a struct's `base` before 1.7, and the keys of later releases, such as
`if`, `features` or `boxed`. So does the parser. Before 2.4 an empty
`discriminator` names none, so the union is simple, and an alternate written
as a union has its base checked from 2.0, as any union's base, and otherwise
ignored.

Before 1.6 QEMU's parser skips only the lines that start with `#` and the
empty ones, continues an expression on each line that starts with a space,
and reads the rest as tokens of punctuation and strings, taking every token
for a value. Before 1.2 its tokenizer never finishes at another character
outside strings, a letter, a tab or the `#` of a comment that does not
start its line among them; from 1.2 it drops them, so `true` and `false`
are no values there. The parser emulates it: it refuses the schemas on
which it fails with `ParseError::crash` (`ParserCrash` names how), and from
1.2 to 1.5 it reads an expression whose dropped characters matter as that
tokenizer reads it.

Some schemas that their release's QEMU generator accepts are beyond the IR.
They fail with `SchemaErrorKind::Unsupported`, which names the construct and
where it is, never as invalid schemas; no mirrored release has one:

- an anonymous struct outside a struct's members and event data: before 2.4
  as a branch, an argument or a return type, and in 2.4 as a return type;
- a reference to an undefined type before 2.4, when QEMU resolved only union
  bases (from 2.0);
- struct bases that are not defined as structs, and base cycles, before 2.4;
- two definitions of one type name, two commands or two events of one
  name, or a type named like a builtin type, before 2.4;
- two members of one object whose names differ only in the optional
  marker, `'a'` and `'*a'`, before 2.5;
- a union discriminator that its base does not declare, or that is optional
  or an array, and a union base that is no struct, before 2.0;
- a flat union discriminator of a simple union's tag enum (`{Union}Kind`),
  from 4.2 to 6.1;
- an alternate as boxed command or event arguments, from 2.7 to 4.1;
- before 2.4, an expression with values that the generators read loosely,
  beyond the grammar of the release's schemas, such as an enum value
  written as an object;
- from 2.4, a value of a shape that the release takes without checking it:
  a member's or branch's condition in 4.0 and 4.1, a struct's features that
  are an empty object or string in 4.1, and in 2.4 `null` as a struct's
  members or as a member's or branch's type (`Construct::Value`).

Where QEMU's generator fails on a schema with an exception instead of an
error, the releases that fail refuse it here, with a kind that names the
condition: 9.0 on an `Errors` section outside a command without a `Returns`
section (`ErrorsWithoutReturns`), 10.0 to 11.0 on a member that
`documentation-exceptions` exempts when only member descriptions follow the
comment's introduction (`ExemptMemberAfterMembers`), and 2.5 to 2.8 on a flat
union whose base struct has no members (`EmptyBase`). The others are
`SchemaErrorKind::Crash`, whose `Crash` names them: from 2.0 to 2.3 a missing
`data` of an enum, a union or a union's base struct; from 2.1 to 2.3 event
data that names no struct; a cycle of struct bases in 2.4 and 2.5; in 2.5
`'a'` with `'*a'`, an alternate branch named `type` and one of type `any`;
from 2.0 to 4.1 a discriminator of a simple union's tag enum; in 2.10 a
string branch of an alternate after a number or boolean branch; from 2.6
to 4.1 the tag enum of an alternate as a type; from 2.4 to 4.1 a
discriminator that QEMU looks up as a key but is an array or object, or
whose type is one; from 4.2 to 6.1 a simple union's branch of an undefined
type; and the values that a release uses without checking them
(`UncheckedValue`), such as the `data` of a union or alternate that is no
object until 6.0, the empty name of an enum value from 2.6, and from 2.5 to
4.1 `null` as a struct's members or a member's type.

### Known divergences

- From 2.6 to 2.11.0 QEMU's generator, run by Python 2, checked the entities
  of a schema in the order of a dictionary's hashes, not of their
  definitions; 2.11.2 checks them in the order of their names, and 2.12 in
  the order of the definitions. The validation checks 2.6 to 2.11.0 in the
  order of the definitions, so of several errors in one schema it can report
  another first than those releases did.
- From 2.4 QEMU's generator follows a chain of struct bases by recursion,
  and Python's recursion limit makes it fail on a chain of about a thousand
  bases, a depth that depends on the Python that runs it. The validation
  follows a chain of any length, and checks and lowers it in time quadratic
  in its length, as QEMU does.
- Before 1.6 QEMU's tokenizer also reads malformed punctuation, such as a
  list that starts with a comma or a member without its colon, which fails
  to parse here.

