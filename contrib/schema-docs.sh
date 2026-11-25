#!/usr/bin/env bash
# Build the qrab-qapi documentation with QEMU's GPL-2.0-or-later schema
# documentation, for local reading only. It goes to a target directory of its
# own, never to the ordinary target/doc; see NOTICE.
set -euo pipefail

usage() {
    echo "usage: $0 <qemu-tree> [--open]" >&2
    exit 2
}
[[ $# -ge 1 && $# -le 2 && $1 != -* ]] || usage
qemu_tree=$(cd "$1" && pwd)
shift
open=()
if [[ $# -eq 1 ]]; then
    [[ $1 == --open ]] || usage
    open=(--open)
fi

cd "$(dirname "$0")/.."
QRAB_QAPI_SCHEMA_DOCS=$qemu_tree \
    cargo doc --locked --no-deps -p qrab-qapi --features schema-docs \
    --target-dir "${CARGO_TARGET_DIR:-target}/schema-docs" "${open[@]}"
