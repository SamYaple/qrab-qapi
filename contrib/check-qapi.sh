#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

cargo test --locked --release -p qrab-qapi-codegen -- --test-threads=2
