#!/usr/bin/env bash
# cargo-semver-checks of the library crates against a git baseline, as the
# semver job of .github/workflows/ci.yml runs it (see CONTRIBUTING.md):
#
#   contrib/check-semver.sh <base revision>
#
# The baseline is the newest release tag (v*) in the history of <base>: a
# change to the public API since the last release needs the version bump
# that the change requires, made by this change or an earlier one. Before
# the first release there is no such tag, nothing is published to break,
# and the check compares with <base> itself and only reports what it finds.
set -euo pipefail
cd "$(dirname "$0")/.."

base=${1:?usage: contrib/check-semver.sh <base revision>}
if ! git rev-parse -q --verify "$base^{commit}" >/dev/null; then
    echo "check-semver: $base is not a commit of this repository" >&2
    exit 2
fi
if release=$(git describe --tags --abbrev=0 --match 'v[0-9]*' "$base" 2>/dev/null); then
    baseline=$release enforced=1
    echo "check-semver: comparing with the release $release"
else
    baseline=$(git rev-parse "$base") enforced=0
    echo "check-semver: no release yet; comparing with $baseline, for information only"
fi

# cargo-semver-checks exits with 100 when a lint fails, and with 101 when
# it cannot run the check.
status=0
check() {
    local code=0
    cargo semver-checks --baseline-rev "$baseline" "$@" || code=$?
    if ((code > status)); then
        status=$code
    fi
}
# qrab-qapi-macros is a proc-macro crate, which has no API surface that
# cargo-semver-checks can read; its expansions reach the API of qrab-qapi.
# The other crates but qrab-qapi with all their features (the `mock`
# sessions).
check --workspace --exclude qrab-qapi-macros --exclude qrab-qapi
# qrab-qapi without `schema-docs`, which changes only the documentation and
# needs the QEMU source.
check -p qrab-qapi --default-features

if ((status == 100 && !enforced)); then
    message="breaking API changes since $baseline; they need a version bump once a release is tagged"
    if [[ ${GITHUB_ACTIONS:-} == true ]]; then
        echo "::warning title=semver::$message"
    fi
    echo "check-semver: $message (not enforced before the first release)"
    exit 0
fi
exit "$status"
