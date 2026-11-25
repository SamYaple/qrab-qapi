# Testing

The default workspace members (`qrab-qapi-macros`, `qrab-qapi` and
`qrab-qapi-protocol`) build from the checked-in generated sources. Their tests
need no QEMU checkout, Python, KVM, root privileges or network:

```sh
cargo test --locked
cargo clippy --locked -p qrab-qapi-macros -p qrab-qapi -p qrab-qapi-protocol --all-targets \
  --no-deps -- -D warnings
```

The timeout fault tests use Tokio's test clock or short explicit deadlines.

## Generator and regeneration check

Parser and generator changes have a separate check:

```sh
bash contrib/check-qapi.sh
```

It requires Git, Python 3.14 or newer (the oracle tests import old releases'
Python 2 lexers, which parse only since PEP 758) and the initialized
`vendor/qemu` submodule with the release tags pinned in
`projects/qapi-codegen/qemu-versions.lock`. It runs strict Clippy
on all four crates, the generated sources included, compares
definition coverage with QEMU's own parser, tests the generator, and
regenerates QMP and QGA to verify the checked-in outputs byte for byte. It also
builds the API documentation with warnings treated as errors, in both the
default and the `schema-docs` mode.

`contrib/check-qapi.sh` also compiles `qrab-qapi` with `QEMU_MIN_VERSION` and
`QEMU_MAX_VERSION` bounds for every maintained release series, selecting its
latest pinned patch. The final check restores the full development API surface.

For an exploratory compile at every distinct pinned historical version, use
`QRAB_QAPI_HISTORICAL_BOUNDS=1 contrib/check-qapi.sh`. Fourth-component tags
share their three-component bounds. This longer sweep does not expand the
maintained release matrix or replace live peer tests.

For an explicit scan of every schema-era stable release tag present in
`vendor/qemu` up to the pinned baseline, independent of the normal locked input
selection:

```sh
cargo test --locked --release -p qrab-qapi-codegen --test mirrored_releases \
  -- --ignored --nocapture
```

It lowers QMP inputs from 1.0 and QGA inputs from 0.15, including historical
four-component release tags.

## Live QEMU smoke tests

The default suite leaves the real-QEMU smoke tests ignored. Run them
explicitly against an installed binary, or a locally built current release:

```sh
QRAB_TEST_QEMU=/path/to/qemu-system-x86_64 \
  cargo test --locked -p qrab-qapi-protocol --test live_qemu -- --ignored --nocapture
```

It uses `-machine none -accel tcg` and a temporary QMP socket: no KVM, guest
image, network download or root privileges. It checks the greeting, typed
query-version/query-status, query-commands against generated metadata, typed
query-qmp-schema decoding, concurrent and OOB commands, error recovery, and
clean quit. Both compact and pretty monitor output are tested through `-mon`.
If the binary advertises QEMU 11.1's `monitor-qmp` object, the same checks
also run through that frontend.

A minimal current-QEMU build for this test is:

```sh
mkdir -p target/qemu-11.1.3
cd target/qemu-11.1.3
../../vendor/qemu/configure --target-list=x86_64-softmmu \
  --without-default-features --without-default-devices --enable-tcg \
  --disable-docs --disable-werror
ninja -j 8 qemu-system-x86_64
```

The real guest-agent test uses a private Unix listener, temporary files and an
explicit RPC allowlist. It checks sync, guest-info, ping, a typed 2 MiB file
read and typed guest-exec/status:

```sh
# Include --enable-guest-agent in the QEMU configure command above.
ninja -C target/qemu-11.1.3 -j 8 qga/qemu-ga
QRAB_TEST_QGA="$PWD/target/qemu-11.1.3/qga/qemu-ga" \
  cargo test --locked -p qrab-qapi-protocol --test live_qga -- --ignored --nocapture
```

It runs as the current user and reads no host agent configuration.
