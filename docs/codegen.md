# QAPI generation

`qapi-codegen` uses nom **8.0.0** and parses QEMU's QAPI language into borrowed
schema tokens, lowers these to Rust IR, then merges releases to annotate
introductions, removals, flags and fields. QMP and QGA use the same pipeline.

## Inputs

`vendor/qemu` is a git submodule pinned at QEMU **v11.1.3**, commit
`005c61c0ef6f6ae615283440b4bb7be51b2c6ddd`. QEMU source and release tags are
fetched from the submodule's remote, the `vendor/qemu` URL in `.gitmodules`.

`projects/qapi-codegen/qemu-versions.lock` pins every selected stable release
from v0.15.0 through v11.1.3 by tag and peeled commit. QGA input starts at
0.15; QMP has no schema before 1.0. The generator checks all
pins before parsing. Extra local tags and release candidates cannot alter the
default output. Missing or moved pins fail generation. The library's
`unified::generate` takes the protocol (`Protocol::Qmp` or `Protocol::Qga`),
the documentation (`Docs::Reference` or `Docs::Schema`) and the releases:
`Releases::Pinned` with a lockfile, which it checks against the tree, or
`Releases::Tags` with an explicit list of tags, which it reads without a
lockfile; the command line always reads a lockfile. Stable release tags may
have two, three or four numeric components. Historical fourth-component
releases retain their full ordering in discovery and pins, while lifecycle
metadata and peer versions use the three components carried by QMP
greetings. The four subpatch
releases have schema-equivalence regression checks against their base tags.

`vendor/qemu` is the only submodule. Initialize it and fetch the release tags:

```sh
git submodule sync -- vendor/qemu
git submodule update --init vendor/qemu
git -C vendor/qemu fetch origin --tags
```

## Compatibility policy

The maintained QEMU matrix covers release series **9.2, 10.0, 10.1, 10.2,
11.0 and 11.1**. The latest pinned patch in each series is the representative
schema input. Schema coverage does not imply live runtime testing of every
release.

Older QEMU is **untested, not unsupported** below this matrix. Historical
schemas remain generation inputs and regression fixtures back to QEMU 1.0,
where the QMP schema first appears, and QGA 0.15. Keep version-specific grammar where the
language changed, and accept explicit older input sets through the library or
`--versions-file`; do not add a minimum-version rejection to the parser.

Lexing follows QEMU's Python parser, including Unicode/C0 whitespace and
LF, CRLF or CR line endings. Ordinary literals and documentation borrow the
original source; escaped literals own decoded text. Parse errors still report
the original line and column. A differential lexical corpus checks
acceptance against the parser in the pinned QEMU tree.
Documentation markers allow Python whitespace before `#`, including Unicode,
but formal opening and closing `##` markers must end the line exactly.
The generator parses and lowers each release under that release's grammar
on worker threads of its own, so no release reads another's grammar and the
calling thread's grammar is left as it was, also when generation fails.
Legacy comments naming a later definition in the same file follow that
definition across intervening helpers; existing attached documentation wins.
Formal symbol checks begin in 2.9; global member/feature links follow the 4.2
validator. Earlier comments stay permissive, including inline double-hash
comments. Returns sections belong to commands from 2.9. The 9.0 rules also
reject Returns on void commands and Errors on non-command declarations.

## Running the generator

From the repository root, with `vendor/qemu` initialized:

```sh
cargo run --release --locked -p qrab-qapi-codegen -- vendor/qemu --output-dir out/qmp
cargo run --release --locked -p qrab-qapi-codegen -- vendor/qemu --qga --output-dir out/qga
```

`qapi-codegen --help` lists the options. The QEMU tree defaults to
`vendor/qemu`, and relative paths are relative to the working directory. An
option's value is the next argument, whatever it starts with, or follows `=`
(`--output-dir=out`), and `--` ends the options.

Each invocation writes `mod.rs`, `commands.rs`, `events.rs` and `metadata.rs`
into the required `--output-dir`. It first checks the tree against every pin
of the embedded `qemu-versions.lock`, or of the lockfile that
`--versions-file` names, in the same `tag commit` format. Then it reads,
checks and lowers each pinned release and generates the four modules, which
must all parse as Rust. Nothing is written before that: each changed file then
goes to a temporary file in the output directory, and the temporary files
replace the old ones once all of them are written. A lockfile that pins no
release with the protocol's schema, QMP from 1.0 and QGA from 0.15, is
refused, since its modules would be empty.

The generator prints a `warning:` line for each tolerated quirk of an old
schema, which `projects/qapi-codegen/KNOWN_ISSUES.md` describes, and the time
taken. An error is printed as `error:` and its message, then each of its
causes on a line of its own:

```text
error: Failed to generate the QGA modules from qemu
  caused by: Failed to process v11.1.3
  caused by: Invalid QAPI command 'broken' in v11.1.3:qga/qapi-schema.json:1:1: 'returns': Unknown type 'Missing'
```

The errors name their input: the lockfile, with the line of an invalid pin,
the tree whose tags do not match the pins, and a schema file of a release as
git names it, `<tag>:<path>`. The exit status is 0 on success, 2 for invalid
arguments, which the usage follows, and 1 for any other failure.

Use release mode for generation and parser integration tests; unoptimized
parsing across the full release history is much slower.

## Advancing the QEMU baseline

To move to a newer stable QEMU release, from the repository root:

1. Fetch the release tags into `vendor/qemu` from its remote and check out
   the new release there, which moves the gitlink:

   ```sh
   git -C vendor/qemu fetch origin --tags
   git -C vendor/qemu checkout vX.Y.Z
   ```

2. Add every stable release up to the new one that the lockfile lacks to
   `projects/qapi-codegen/qemu-versions.lock`, in release order, each as its
   tag and peeled commit (`git -C vendor/qemu rev-parse 'vX.Y.Z^{commit}'`).
   Keep the older pins: the version lifetimes come from them. The tests
   require the pins to be exactly the tree's stable releases up to the
   newest pin, and `vendor/qemu` to be checked out at that pin.
3. Name the new baseline under [Inputs](#inputs): the release, its commit
   and the range of the pins. A release that starts a new series in the
   maintained matrix joins the [compatibility policy](#compatibility-policy)
   and the series of `recent_release_matrix_matches_each_releases_own_parser`
   in `projects/qapi-codegen/tests/current_schemas.rs`. In
   `projects/qapi-codegen/src/versions.rs`,
   `pinned_releases_are_ordered_and_end_at_current_baseline` names the
   newest pin, and `CURRENT` in `tests/upstream_corpus.rs` is the current
   grammar for that test's own inputs.
4. Run `bash contrib/check-qapi.sh`, which runs the generator's tests, and
   commit the gitlink and the lockfile together, with the changes above.

## Regeneration

`qrab-qapi` checks in the generator's output. Regenerate it from the
repository root:

```sh
cargo run --release --locked -p qrab-qapi-codegen -- vendor/qemu \
  --output-dir projects/qapi/src/generated
cargo run --release --locked -p qrab-qapi-codegen -- vendor/qemu --qga \
  --output-dir projects/qapi/src/qga
```

Normal builds consume these files directly and perform no network access or
code generation; only the opt-in `schema-docs` feature generates at build
time. `contrib/check-qapi.sh` regenerates both trees and requires them to
match byte for byte. It also generates both with `--schema-docs`, diffs both
modes with their `///` lines removed, and builds the API docs of the
`schema-docs` build with warnings treated as errors.

When advancing the QEMU baseline, regenerate both trees once the new release
is pinned, and commit them with the pins. A release that starts a new series
of the maintained matrix also joins the series that `contrib/check-qapi.sh`
compiles the generated code for.

## Reading the full QEMU docs locally

The `qapi` crate's opt-in `schema-docs` feature regenerates the QMP and QGA
modules during the build with the schema documentation included. Its output
contains QEMU's GPL-2.0-or-later text, which must not be published, so build
it into a target directory of its own rather than the `target/doc` that
ordinary documentation builds use, as `contrib/check-qapi.sh` does:

```sh
cargo doc -p qrab-qapi --features schema-docs --target-dir target/schema-docs --open
```

Its build script reads the QEMU tree named by `QRAB_QAPI_QEMU_SRC`, defaulting
to the workspace `vendor/qemu`, which must hold the pinned release tags. It
writes both trees to the build's `OUT_DIR` and compiles them in place of
`src/generated` and `src/qga` under the same module paths. Default builds
neither need a QEMU tree nor build `qapi-codegen`. The feature adds roughly
half a minute to a clean build, mostly generation across every pinned release.
For hover docs in an editor, enable it in rust-analyzer, with its build in
that target directory too:

```json
{
  "rust-analyzer.cargo.features": ["qrab-qapi/schema-docs"],
  "rust-analyzer.cargo.targetDir": "target/schema-docs"
}
```

The docs-enabled build is for local reading. The QEMU text in it is
GPL-2.0-or-later; do not publish or ship artifacts built with the feature.

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
which it fails with `ParseErrorKind::Crash` (its `ParserCrash` names how),
and from 1.2 to 1.5 it reads an expression whose dropped characters matter
as that tokenizer reads it.

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

## Correctness boundaries

QEMU's parser requires commas, and reports where an expression is
malformed, from 1.6, and it rejects duplicate dictionary keys from 2.0
(before, the last value wins). The permissive separator grammar remains
restricted to pre-1.6 schemas for legacy compatibility. Includes are canonicalized, aliases are read once, and include
cycles fail. Complete-file parsing rejects any unconsumed input. String decoding
follows three eras: permissive escaped characters before 2.4, JSON-style escapes
and nonzero ASCII Unicode escapes through 4.1, then printable ASCII with escaped
backslashes from 4.2. Include paths are decoded once; names and references use
the same decoded identity. Literal grammar versions participate in schema hashes.
The string-valued fields of the generator's internal AST are `Cow<str>`.
Conditions before 6.2 retain C expressions and lists of expressions; newer
conditions require configuration identifiers and nonempty boolean expressions.
Empty conditions, `''` or `[]` and from 4.2 blank strings, fail where QEMU
checks them: definitions' from 3.0, enum values' from 4.0, members',
branches' and features' from 4.2 (struct features' from 4.1).
Empty enums and structs are valid, but their `data` key is still required
(from 2.4 a `MissingKey`).
The predefined `QType` and `q_empty` definitions of QEMU 2.6 onward are
synthesized in every selected release of that era once any selected release
refers to them. The QType number variants follow the 2.10 `qint`/`qfloat` to `qnum`
transition, and each era's builtin types and reserved names follow QEMU's: the
untyped `visitor`, `dict` and `**` before 2.4, `**` in 2.4, `any` from 2.5 and
`null` from 2.10. Implicit-definition versions participate in schema hashes,
so identical source files across that boundary still retain both wire shapes.
From 4.2, when QEMU accepts them, implicit `ElementList` references normalize
to ordinary arrays. Scalar builtin lists are predefined; named lists require an
explicit array use somewhere in the schema graph. Nested arrays are rejected
before lowering, as are declarations with the `Kind` suffix (2.4 to 6.1) or the
`List` suffix (from 2.5) that QEMU reserves, for commands and events too until
6.0.
Alternates reject overlapping JSON-type branches, including broad `any` values.
Before 2.10 integer and floating-point branches remain distinct; newer schemas
also apply QEMU's string and enum scalar-conversion conflict rules.
Alternate decoding tries integer payloads before floating-point conversion,
preserving large values exactly. Within an explicit peer context, an integer
range error cannot fall through to a floating branch; integer promotion applies
only when no integer branch is active.
Boxed command and event data must name an object type. Missing, anonymous,
scalar and false-boxing forms fail before lowering; valid named objects keep
their fields flat on the wire.
Named union argument types require boxing for both commands and events;
scalars, enums and alternates cannot serve as argument objects.
Command flags enforce QEMU's true-only execution flags, false-only modern
generation/reply flags and incompatible OOB/coroutine modes. Legacy string
generation and reply flags retain their historical handling, including
`success-response: 'no'` before 2.4.
Flags that disappear and return retain separate ranges, including historical
simple-union layouts. Instability also tracks stabilization and later
reintroduction for types and members.
Member expansion retains all features, so ordinary features cannot hide
deprecation or instability. Duplicate feature names and C-name collisions fail
with source context on definitions, fields and enum values.
Members deprecated on introduction record that first release immediately,
including when later patch schemas are identical and skipped during merging.
From 8.1, conditional members in command/event argument objects require boxing,
including members inherited from a base. Earlier schemas retain the older rules.
Type references resolve against exact QAPI names across all source files before
Rust name mapping. Undefined member, branch and return types report their source
and definition; historical inline objects include the nested member path.
From 2.4, definition names share one namespace across files and declaration
kinds; duplicates report both sources before Rust symbol planning.
QName validation preserves the 2.4/2.5 underscore-and-dot grammar, the 2.5
reserved prefix, and the downstream-prefix grammar from 2.6. Numeric enum
members remain valid; casing and pragma style rules are separate.
Legacy names with no Rust spelling use `QapiUnnamed` or `qapi_unnamed`, with
normal collision allocation. Pre-2.6 empty-string enum values keep their wire tag.
Pre-2.4 Unicode names use stable ASCII escapes, preserving their wire spelling
and avoiding invalid Rust identifiers or Unicode normalization collisions.
Member clash checks include inherited fields and nested flat-union branches;
each branch keeps its own member namespace. Branch types resolve independently
of definition order. Names that collapse to the same QAPI C member identifier
fail with source context.
Historical anonymous structs lower recursively, including optional and empty
objects. Extracted type names avoid declared types and generated role names,
so anonymous objects cannot overwrite unrelated schema definitions.
QEMU's positive schema fixtures are checked through parsing and IR lowering.
Flat unions may omit every payload branch when their discriminator enum is
nonempty. Generated branch enums carry `union_branch` even when all variants
have no payload, preserving discriminator dispatch and empty-object wire shape.
Discriminators must be required, unconditional scalar enums: from 2.0 a
branch must be a value of the enum and the discriminator an enum, and from
2.6 a union needs a branch or an enum value. Before 2.0 a non-enum
discriminator is a tolerated quirk, reported as a warning.
Derived structs resolve bases independently of definition and include order;
named bases of structs and unions must refer to plain structs.
Missing bases, inheritance cycles and conflicting Rust type names fail with
schema context.

A missing QMP schema is allowed only before 1.0, and a missing QGA schema only
before 0.15. These exceptions reflect source availability, not minimum supported
peer versions. Present schemas that fail parsing or IR processing are errors,
never skipped releases. Early guest-agent `qapi-schema-guest.json` files are
extracted alongside the later `qga/qapi-schema.json` layout. Generated Rust
that does not parse also fails generation, naming the file, line and column.
The quirks of old schemas that generation tolerates with a warning,
mislabeled documentation before 2.9 and a string discriminator in 1.7, are
described in `projects/qapi-codegen/KNOWN_ISSUES.md`.

Every command struct is in `commands.rs` and every event struct in
`events.rs`, so the `commands` module holds every command and `events`
every event. Both import `mod.rs` by glob, and `mod.rs` imports one of them
by glob, as `use self::commands::*;`, when a type outside it names one of
its structs as a field, payload or return type: the QMP transaction actions
name the commands `BlockdevBackup`, `BlockdevSnapshotSync`,
`BlockDirtyBitmapAdd` and `DriveBackup`. A glob, unlike a list of names,
still resolves when the version bounds leave some of the structs out. The
import is private, so a glob re-export of `mod.rs`, such as qrab-qapi's at
its crate root, holds only the data types.

A type that a release renames without changing its Rust name, such as
`SGXInfo`, which QEMU 10.1 spells `SgxInfo`, keeps one lifetime and takes the
newest release's spelling.

Commands sharing a Rust name with their referenced argument struct retain the
existing merged representation, including the exact command wire name. A
command colliding with a distinct response type gets a `Command` suffix: the
QGA command `guest-file-read` is `GuestFileReadCommand` in `commands.rs`,
which returns `GuestFileRead` of `mod.rs`. The same distinction applies to
guest-file-write, guest-file-seek, guest-exec, guest-exec-status and
guest-fsfreeze-status, whose response is an enum.

A command whose return type changed across pins keeps each one in the
registry as `UnifiedStruct::returns_history`, and the emitter writes them as
`returns(added = "...", value = "...")` beside the newest, `returns`. The
changes are pinned by a test: `device-list-properties` (`DevicePropertyInfo`
until 2.12, then `ObjectPropertyInfo`), `query-target` (`TargetInfo` until
10.1, then `QemuTargetInfo`) and `guest-fstrim` (unit until 2.4, then
`GuestFilesystemTrimResponse`).

The test suite compares definition sets with QEMU's own Python parser for both
current QMP and QGA schemas.

Definition documentation is parsed as complete comment blocks. A closing `##`
is never treated as prose, and malformed modern blocks cannot fall back to
ordinary comments. Member and feature text is compared with QEMU's own parser
for the production schemas and its documentation fixture. Legacy headers,
optional closing markers and the historical `###` terminator remain accepted for
older inputs.

Generated code carries no QEMU schema documentation text. The QAPI schema and
its comments are GPL-2.0-or-later, while the generated crate is Apache-2.0, so
default output uses the comments only for validation and version facts such
as `Since:`. Each schema-declared command, event, struct, enum, union and
alternate receives one generated line naming its wire name and pointing to the
upstream QEMU QMP Reference Manual or QEMU Guest Agent Protocol Reference.
Fields, variants and synthesized helper types carry no schema text, and the
metadata tables carry none in any mode.

`qapi-codegen --schema-docs` instead renders the schema prose, member
descriptions and examples as rustdoc; an item without prose keeps its
reference line. The reStructuredText becomes Markdown that rustdoc shows as
written: brackets, angle brackets and backticks that open no code span are
escaped, URLs become links, and literal blocks, indented where Markdown would
read code or after a paragraph ending in `::`, become `text` code blocks, so
the documentation has no doctests. That output differs from the default only
in `///` lines, and it must never be checked in or published.

Historical simple unions retain their nested `{"type": ..., "data": ...}` wire
shape, including scalar, array and null payloads. The `simple_union` property
tracks transitions to flat unions per type rather than using a global cutoff.

When a branch payload type or wire spelling changes, the latest Rust variant
name remains stable and earlier representations are emitted as lifetime-gated
`Legacy` variants. Repeated changes receive distinct suffixes without colliding
with schema-defined names. Generated metadata carries each representation's
original wire tag and lifetime. Fields and enum members that disappear and
later return retain separate lifetimes, so the intervening releases do not
inherit the restored member.

Field payload changes, optionality changes and wire renames retain `_legacy`
representations with separate lifetimes. A required wire field remains
required in metadata even when version merging needs a Rust `Option`.

When a named type changes between a string enum, alternate and object, each
run of releases with one kind is a lifetime-gated type of its own. The first
run keeps the ordinary Rust name and later runs receive `Generated` names, so
adding a newer release never renames an existing type. References follow each
release's definition; a return to an earlier kind starts a separate lifetime.
Structs and tagged unions retain their shared object field history.
Named data types that disappear and return also receive separate lifetime-gated
representations, even when the wire kind is unchanged. Their union helpers
follow the same lifetime.
Commands and events also retain separate historical lifetimes, named the same
way. A persistent named argument type stays usable while an absent command is
represented by separate request DTOs.
Sharing also stops when a named argument type predates the command, so request
availability never inherits an earlier data-only lifetime.

QAPI build conditions describe the remote QEMU or guest-agent build. Those of
definitions, members and enum values are emitted as
`#[qapi(condition = "...")]` attributes, those of features not at all, and
none becomes a Rust host OS gate.
Conditional fields use `Option` because a peer can omit them even when their
schema declaration is required when enabled. Metadata retains that declaration's
wire optionality; version checks cannot determine the peer's build configuration.
Condition presence changes retain field history so later unconditional fields
become required. Generated conditions describe the newest declaration rather
than an obsolete condition inherited from an earlier release.

Recursive fields are boxed, as Rust needs to size their types. Recursive-type
analysis ignores `Vec` and existing `Box` indirections, which already make
recursive values sized.

The historical `union` form with an empty discriminator dictionary is an
untagged alternate, not a simple tagged union. It lowers to the same bare-value
Rust enum as modern `alternate` syntax, preserving the original introduction
version and feature metadata.

Rust identifiers are escaped by role. Reserved type names use a `Qapi` prefix:
`Self`, `HashMap`, prelude names such as `String` and `Option`, and the
types and traits at the root of `qrab-qapi`, such as `EncodeError` and
`QapiCommand`, which would shadow a generated type in its glob re-export of
the QMP types. Enum variants keep ordinary names such as `String` and
`None`, with `Self` escaped. Fields `self`, `crate` and `super` receive a
trailing underscore and retain their wire-name attributes. Type
declarations use type-name mapping, not builtin-value mapping: a declared
`Visitor` remains a real struct, while the historical lowercase `visitor`
opaque reference remains a JSON value.
A simple union's tag enum is `{Union}Kind`, the name QEMU gives it, in the
namespace QEMU reserved for it; the flat unions that replaced simple unions in
6.2 declare that enum by the same name, and both are one Rust type. Generated
union branches, event types and command fallbacks claim their names like
declarations do: whichever symbol a release has first keeps the plain name,
and a later one gets a `Generated` or numbered `Generated2` suffix.
References and wire names remain attached to the correct definitions.
Wire-distinct commands and events that normalize to the same Rust name also
receive distinct identities across the full history, even if they never coexist.
For example, `request-1` and `request1` keep separate command names and lifetimes.
Exact QAPI type references resolve before Rust conversion. Coexisting declarations
such as `Option`/`QapiOption` or `FooBAR`/`FooBar` receive distinct Rust symbols;
mutually exclusive historical spellings can retain a shared identity.
Coexisting field names that normalize alike receive stable `_alt` names across
the selected history. Allocation reserves future declared fields and distinguishes
virtual flatten receivers from wire fields, preserving keys such as `pc` and `PC`.
Enum, alternate and branch variants use the same history-aware reservation,
with `Alt`/`AltN` names for collisions. Original tags remain in wire attributes,
including keyword escapes such as `self` and numeric names such as `a-1`.
Name planning covers every selected release before IR lowering, so a newer
declaration cannot overwrite an older generated branch or anonymous object.
Extraction, name collection and lowering use bounded workers; schema hashes
still deduplicate identical releases during merging.

Metadata resolves field and variant references and command returns as QAPI
names, spelled as the newest release with the field, variant or return spells
them: a command that returns `int` refers to the builtin `int`, as a field of
that type does, a declared `String` stays distinct from the builtin `str`, and
a field typed `visitor`, `dict` or `**` before QEMU 2.5 and `any` later refers
to `any`. One entry of `TYPE_REFS` names one type for every release. A
field's `type_ref()`, a variant's `payload_type()` and a command's `returns()`
are references to entries of the `TYPE_REFS` of their own module, for scalar,
null, enum, struct and list payloads; `payload_struct()` is retained
for direct struct payloads.
`StructMeta::name()` and `EnumMeta::name()` are the Rust lookup identities;
`wire_name()` retains the QAPI spelling through alias and legacy-type
allocation, the wire name of a command or event. Synthesized types fall back
to their generated name. A `TypeRef` has a `prim()` only as a primitive and an
`element()` only as a list.
The generator writes only the tables: `qapi::metadata` and
`qapi::qga::metadata` re-export one set of their types, which `qrab-qapi`
defines by hand in its `metadata_types` module. The types are
`#[non_exhaustive]` and read through methods, and only the generated statics
construct them.
Union metadata selects the newest observed discriminator and branch fields.
Historical discriminator representations remain ordinary versioned fields;
every generated branch enum is a `UnionBranch`, and `StructMeta::simple_union()`
lists the releases in which its union has the simple layout, as `free_form()`
lists those in which a `gen: false` command takes undeclared arguments.
The flags of commands (`allow_oob()`, `allow_preconfig()`, `success_response()`)
and the `deprecated` and `unstable` features of structs, enums, commands,
events, fields and variants are such lists of releases as well, so the tables
describe every release; `CommandMeta::allows_oob(version)`,
`StructMeta::is_deprecated(version)` and the other accessors ask about one.
The `#[qapi]` attributes carry the same runs: a `deprecated(...)` or
`unstable(...)` attribute per run, with `removed` where a later release that
still has the item drops the feature. A run that lasts until the item goes,
such as the deprecation of the `MEM_UNPLUG_ERROR` event from 6.2, ends with
the item's own `removed` (9.1).

## The generated code at run time

The `#[qapi]` attribute of `qrab-qapi-macros` expands the generated items, and
`qrab-qapi` provides the version context they check against. The expansion
names `qrab-qapi` by the absolute path `::qapi`: its public runtime items, and
Serde and the helpers that only the expansion uses through the hidden
`qapi::__private` module, which is not part of the public API. A crate that
uses the attribute depends on `qrab-qapi` under its library name, and
`qrab-qapi` declares `extern crate self as qapi` for its own generated
modules, which therefore import no alias.

QAPI's untyped values, `any` and the `visitor`, `dict` and `**` of the
oldest schemas, are `serde_json::Value`. A `gen: false` command, whose
schema declares only some of its arguments, takes the others in a
flattened `arguments` field of type
`Option<serde_json::Map<String, serde_json::Value>>`: the map that a JSON
object holds as a `serde_json::Value::Object`, and that `QapiFlatUnion`
decodes union branches from. Its key order is that of `serde_json::Map`:
sorted by key, or insertion order where serde_json's `preserve_order`
feature is enabled. The QAPI crates do not enable it, but Cargo unifies
features, so any crate of a build that enables it changes the order for
all of them.

Generated command constants describe the newest schema flags. The
`allows_oob`, `allows_preconfig` and `sends_success_response` trait methods
query their added/removed ranges for a particular peer release, and repeated
macro ranges combine independently of order. Command `is_unstable(version)`
queries the peer release; reflection and the `UNSTABLE` constant describe the
newest declaration. The macro sets every constant; a command implemented by
hand sets only `Returns`, `NAME` and `AVAILABLE_SINCE`, and the others default
to a stable, still available command that replies and runs neither out of band
nor in preconfig.

A command whose return type changed across the releases that a build
supports has an enum as its `Returns`, with a variant per return type named
by the release that introduced it:
`qapi::commands::QueryTargetReturns` is `V1_2_0(TargetInfo)` or
`V10_1_0(QemuTargetInfo)`. A reply decoded in a peer's version scope, as the
protocol clients decode them, takes the variant of the peer's release, and
unscoped decoding the newest. A variant whose type is unit reads QEMU's `{}`.

Generated enum serializers enforce both type and variant lifetimes. The same
checks apply to alternate values and union branch payloads, including scalar
payloads whose Rust types carry no QEMU metadata. Invalid outgoing values name
the unavailable type, variant or field and peer version, and `qapi::to_value`
returns them as the typed `EncodeError::Unavailable`. Explicitly scoped decoding
also checks type and variant lifetimes, including union dispatch. Unscoped
decoding remains permissive so recorded and newer replies can still be inspected.
Union serialization also checks that a present typed payload matches its active
discriminator, for both flat and historical nested layouts. The branch trait's
optional `wire_tag` hook keeps handwritten implementations compatible.

Use `qapi::with_version` when decoding a recorded historical simple-union
layout; unscoped decoding selects the newest layout. Discriminator dispatch
selects a branch representation for the scoped peer version; unscoped
inspection uses the newest shape and does not fall back to an obsolete
permissive shape after a decoding error. String enum decoding selects the
active representation of a shared wire tag.

A field that is set is serialized only within its lifetime: outside it,
serialization fails with `EncodeError::Unavailable` instead of discarding the
value, as it does for the inactive representation of a shared wire name, which
decoding never populates. Explicit peer contexts enforce active required fields and check type
lifetimes; unscoped record inspection remains lenient. Present nullable values
retain `Some(null)` separately from absent fields.

`QEMU_MIN_VERSION`/`QEMU_MAX_VERSION` builds prune representations outside the
requested range: a command's return enum keeps the variants of the range,
and a range with one return type has it as the `Returns` type. The newest
release of the range takes the place of the newest pin in the command
constants, the documentation of unstable types and unscoped decoding.
`contrib/check-qapi.sh` compiles all maintained release series and restores an
unbounded development build afterwards.

Serde macro expansions allocate helper names against user fields, referenced
types and other generated helpers. Fields such as `map` and internal-looking
identifiers cannot capture JSON storage, visitors or serde generic parameters.
These cases have compile-and-round-trip regressions.

## Compiled checks

The generator tests also compile what they generate against `qrab-qapi`.
QEMU's positive schema fixtures, its schema stress test included, compile as
complete generated Rust modules. A QEMU-validated naming fixture is generated,
compiled offline and round-tripped to check declarations, references, commands
and metadata together.
