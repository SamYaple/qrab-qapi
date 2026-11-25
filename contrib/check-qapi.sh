#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# The crates built on the macros, one per line:
qapi_crates=(
    -p qrab-qapi
)
cargo clippy --locked -p qrab-qapi-codegen -p qrab-qapi-macros "${qapi_crates[@]}" --all-targets \
    -- -D warnings
cargo test --locked --release -p qrab-qapi-codegen -- --test-threads=2
cargo test --locked -p qrab-qapi-macros "${qapi_crates[@]}"
