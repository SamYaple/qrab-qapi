#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

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
# The schema-docs build needs both the feature and QRAB_QAPI_SCHEMA_DOCS. Its
# rustdoc carries the GPL prose, so it never goes to the ordinary target/doc.
qapi_schema_docs_target=${CARGO_TARGET_DIR:-target}/schema-docs
qapi_schema_docs_probes=(
    'Allow client connections for VNC'
    'Get some information about the guest agent'
)
qapi_has_schema_docs() {  # target dir
    local qapi_probe
    for qapi_probe in "${qapi_schema_docs_probes[@]}"; do
        grep -rqF "$qapi_probe" "$1/doc/qapi" ||
            { echo "no schema documentation in $1/doc/qapi: \"$qapi_probe\"" >&2; return 1; }
    done
}
qapi_no_schema_docs() {  # target dir
    local qapi_probe
    [[ -f $1/doc/qapi/index.html ]] || { echo "no documentation in $1/doc/qapi" >&2; return 1; }
    for qapi_probe in "${qapi_schema_docs_probes[@]}"; do
        ! grep -rqF "$qapi_probe" "$1/doc/qapi" ||
            { echo "schema documentation in $1/doc/qapi: \"$qapi_probe\"" >&2; return 1; }
    done
}
# The feature alone builds the checked-in modules.
env -u QRAB_QAPI_SCHEMA_DOCS \
    cargo doc --locked --no-deps -p qrab-qapi --features schema-docs --target-dir "$qapi_schema_docs_target"
qapi_no_schema_docs "$qapi_schema_docs_target"
QRAB_QAPI_SCHEMA_DOCS="$PWD/vendor/qemu" \
    cargo check --locked -p qrab-qapi --features schema-docs --target-dir "$qapi_schema_docs_target"
QRAB_QAPI_SCHEMA_DOCS="$PWD/vendor/qemu" RUSTDOCFLAGS="${RUSTDOCFLAGS:+$RUSTDOCFLAGS }-D warnings" \
    cargo doc --locked --no-deps -p qrab-qapi --features schema-docs --target-dir "$qapi_schema_docs_target"
qapi_has_schema_docs "$qapi_schema_docs_target"

# The variable alone changes nothing either: the ordinary documentation, built
# with it set, carries no schema prose.
QRAB_QAPI_SCHEMA_DOCS="$PWD/vendor/qemu" RUSTDOCFLAGS="${RUSTDOCFLAGS:+$RUSTDOCFLAGS }-D warnings" \
    cargo doc --locked --no-deps -p qrab-qapi-codegen -p qrab-qapi-macros "${qapi_crates[@]}"
qapi_no_schema_docs "${CARGO_TARGET_DIR:-target}"
