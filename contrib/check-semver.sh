#!/usr/bin/env bash
# cargo-semver-checks of the library crates against their last releases, as
# the semver job of .github/workflows/ci.yml runs it (see CONTRIBUTING.md):
#
#   contrib/check-semver.sh <base revision>
#
# Each crate's baseline is its last release in the history of <base>: its
# newest tag <package>-v<version>, or before it has one, the newest tag
# v<version>, which released the first crates together. A change to a
# crate's public API since that release needs the version bump that the
# change requires, made by this change or an earlier one.
set -euo pipefail
cd "$(dirname "$0")/.."

base=${1:?usage: contrib/check-semver.sh <base revision>}
if ! git rev-parse -q --verify "$base^{commit}" >/dev/null; then
    echo "check-semver: $base is not a commit of this repository" >&2
    exit 2
fi

# cargo-semver-checks exits with 100 when a lint fails, and with 101 when
# it cannot run the check.
status=0
check() {
    local code=0
    cargo semver-checks "$@" || code=$?
    if ((code > status)); then
        status=$code
    fi
}
for manifest in projects/*/Cargo.toml; do
    name=$(sed -n 's/^name = "\(.*\)"$/\1/p' "$manifest" | head -n 1)
    # A proc-macro crate has no API surface that cargo-semver-checks can
    # read; the expansions of qrab-qapi-macros reach the API of qrab-qapi.
    if [[ $name == qrab-qapi-macros ]]; then
        continue
    fi
    # git describe takes the matching tag closest to <base>, so once a crate
    # has a tag of its own, that tag wins over the older v<version>.
    if ! release=$(git describe --tags --abbrev=0 --match "$name-v[0-9]*" \
        --match 'v[0-9]*' "$base" 2>/dev/null); then
        echo "check-semver: no release of $name in the history of $base; fetch the release tags" >&2
        exit 2
    fi
    # A crate that its last release does not have yet has no API to compare.
    if ! git cat-file -e "$release:$manifest" 2>/dev/null; then
        echo "check-semver: $name is new since $release, so there is nothing to compare"
        continue
    fi
    echo "check-semver: comparing $name with the release $release"
    if [[ $name == qrab-qapi ]]; then
        # Without `schema-docs`, which changes only the documentation and
        # needs the QEMU source.
        check --baseline-rev "$release" -p "$name" --default-features
    else
        # With all features, such as the `mock` sessions.
        check --baseline-rev "$release" -p "$name"
    fi
done
exit "$status"
