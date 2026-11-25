#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# This check owns its bounded builds; start with the complete API surface.
unset QEMU_MIN_VERSION QEMU_MAX_VERSION
# The generator tests build fixture crates with --offline from the registry
# cache, so a fresh checkout (CI) needs every locked package downloaded first.
cargo fetch --locked

# Strict Clippy on all four crates, the generated QMP and QGA sources included.
# The crates built on the macros, one per line:
qapi_crates=(
    -p qrab-qapi
    -p qrab-qapi-protocol
)
cargo clippy --locked -p qrab-qapi-codegen -p qrab-qapi-macros "${qapi_crates[@]}" --all-targets \
    -- -D warnings
cargo test --locked --release -p qrab-qapi-codegen -- --test-threads=2
cargo test --locked -p qrab-qapi-macros "${qapi_crates[@]}"

qapi_check_dir=$(mktemp -d)
trap 'rm -rf "$qapi_check_dir"' EXIT
cargo run --locked --release -p qrab-qapi-codegen -- vendor/qemu --output-dir "$qapi_check_dir/qmp"
cargo run --locked --release -p qrab-qapi-codegen -- vendor/qemu --qga --output-dir "$qapi_check_dir/qga"
diff -ru projects/qapi/src/generated "$qapi_check_dir/qmp"
diff -ru projects/qapi/src/qga "$qapi_check_dir/qga"

# Opt-in schema docs (GPL-2.0-or-later QEMU prose) may change only `///` lines:
# both modes write the same files, which differ only in those lines.
cargo run --locked --release -p qrab-qapi-codegen -- vendor/qemu --schema-docs --output-dir "$qapi_check_dir/qmp-docs"
cargo run --locked --release -p qrab-qapi-codegen -- vendor/qemu --qga --schema-docs --output-dir "$qapi_check_dir/qga-docs"
grep -q 'Allow client connections for VNC' "$qapi_check_dir/qmp-docs/commands.rs"
grep -q 'Get some information about the guest agent' "$qapi_check_dir/qga-docs/commands.rs"
qapi_code_lines() {
    grep -v '^ *///' "$1" || [[ $? == 1 ]]
}
for qapi_tree in qmp qga; do
    qapi_plain=$qapi_check_dir/$qapi_tree
    qapi_docs=$qapi_check_dir/$qapi_tree-docs
    find "$qapi_plain" -type f -printf '%P\n' | LC_ALL=C sort >"$qapi_plain.files"
    find "$qapi_docs" -type f -printf '%P\n' | LC_ALL=C sort >"$qapi_docs.files"
    [[ -s $qapi_plain.files ]]
    diff -u --label "$qapi_tree files" --label "$qapi_tree-docs files" \
        "$qapi_plain.files" "$qapi_docs.files"
    while IFS= read -r qapi_file; do
        qapi_code_lines "$qapi_plain/$qapi_file" >"$qapi_check_dir/code"
        qapi_code_lines "$qapi_docs/$qapi_file" >"$qapi_check_dir/code-docs"
        diff -u --label "$qapi_tree/$qapi_file" --label "$qapi_tree-docs/$qapi_file" \
            "$qapi_check_dir/code" "$qapi_check_dir/code-docs"
    done <"$qapi_plain.files"
done
# Its rustdoc carries the GPL prose, so it never goes to the ordinary target/doc.
qapi_schema_docs_target=${CARGO_TARGET_DIR:-target}/schema-docs
QRAB_QAPI_QEMU_SRC="$PWD/vendor/qemu" \
    cargo check --locked -p qrab-qapi --features schema-docs --target-dir "$qapi_schema_docs_target"
QRAB_QAPI_QEMU_SRC="$PWD/vendor/qemu" RUSTDOCFLAGS="${RUSTDOCFLAGS:+$RUSTDOCFLAGS }-D warnings" \
    cargo doc --locked --no-deps -p qrab-qapi --features schema-docs --target-dir "$qapi_schema_docs_target"

# Compile each maintained series using its latest pinned patch.
for qapi_series in 9.2 10.0 10.1 10.2 11.0 11.1; do
    qapi_max=$(awk -v prefix="v$qapi_series." \
        'index($1, prefix) == 1 { version = substr($1, 2) } END { if (!version) exit 1; print version }' \
        projects/qapi-codegen/qemu-versions.lock)
    QEMU_MIN_VERSION="$qapi_series.0" QEMU_MAX_VERSION="$qapi_max" \
        cargo check --locked -p qrab-qapi
done
# Check the combined maintained range as well as each individual series.
QEMU_MIN_VERSION=9.2.0 QEMU_MAX_VERSION="$qapi_max" cargo check --locked -p qrab-qapi

# Opt-in exploratory coverage; older releases are outside the maintained matrix.
if [[ ${QRAB_QAPI_HISTORICAL_BOUNDS:-0} == 1 ]]; then
    while IFS= read -r qapi_version; do
        printf 'Checking historical QAPI bounds for QEMU %s\n' "$qapi_version"
        QEMU_MIN_VERSION="$qapi_version" QEMU_MAX_VERSION="$qapi_version" \
            cargo check --locked -p qrab-qapi
    done < <(awk '
        !/^#/ && NF {
            split(substr($1, 2), parts, ".")
            version = parts[1] "." parts[2] "." (parts[3] ? parts[3] : 0)
            if (!seen[version]++) print version
        }
    ' projects/qapi-codegen/qemu-versions.lock)
fi
# Leave the ordinary development build with the complete API surface.
env -u QEMU_MIN_VERSION -u QEMU_MAX_VERSION cargo check --locked -p qrab-qapi

RUSTDOCFLAGS="${RUSTDOCFLAGS:+$RUSTDOCFLAGS }-D warnings" \
    cargo doc --locked --no-deps -p qrab-qapi-codegen -p qrab-qapi-macros "${qapi_crates[@]}"
