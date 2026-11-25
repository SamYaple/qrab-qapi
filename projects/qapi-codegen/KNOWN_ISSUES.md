# Known issues

The pinned history parses and generates both protocols. A few old schemas
have quirks that the generator tolerates with a warning, which
`qapi-codegen` prints and `unified::generate` returns as the `Warning`s of
its output. They are quirks of old source, not a minimum supported QEMU
release.

## Mislabeled legacy documentation

Some pre-2.9 comments name another symbol than the definition below them:

- `InputButton` above `InputAxis` (2.0 to 2.4), a copy of the comment of
  `InputButton`, which the file defines above it;
- `Rocker` above `RockerSwitch` (2.4 to 2.7), a symbol that the file does
  not define;
- `VncPriAuth` above `VncPrimaryAuth` (2.3 to 2.7), likewise.

The generator warns and keeps the text attached to the definition below it.
When the named definition comes later in the same file, legacy documentation
follows it instead, without overwriting its own comment. Formal documentation
from 2.9 requires the exact definition name. Case-only differences do not
trigger the legacy warning.

## String discriminators

QEMU 1.7 used `str` for the `BlockdevOptions.driver` discriminator.
The generator retains that string field and the declared payload branches,
and warns that the discriminator is not an enum. Its Rust type cannot restrict
the driver to the declared branch tags as newer enum discriminators do.
