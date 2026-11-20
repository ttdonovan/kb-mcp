#!/usr/bin/env bash
# Fails if any crates.io package in a lockfile (default: Cargo.lock) was published less than MIN_AGE_DAYS (default 7) ago.
# Reads "pubtime" from the sparse index; versions without it pass, as Cargo's own min-publish-age does.
# Replace with registry.global-min-publish-age in .cargo/config.toml once the toolchain is >= 1.100.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
min_days="${MIN_AGE_DAYS:-7}"
lock="${1:-$ROOT/Cargo.lock}"
now="$(date +%s)"
cutoff=$((now - min_days * 86400))
# "name version" pairs allowed younger than min_days, each with its reason; remove once old enough.
exceptions=()
# ALLOW_YOUNG_PREFIXES: space-separated crate families (a name, or the name plus "-...") allowed at any
# age. Set only by a caller that accepts it, such as a one-off local check.
read -r -a young_prefixes <<<"${ALLOW_YOUNG_PREFIXES:-}"

# allowed_family <name>: whether name is in one of young_prefixes.
allowed_family() {
    local p
    for p in ${young_prefixes[@]+"${young_prefixes[@]}"}; do
        [[ "$1" == "$p" || "$1" == "$p"-* ]] && return 0
    done
    return 1
}

# index_path <name>: the sparse index path for a crate name.
index_path() {
    local n
    n="$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')"
    case "${#n}" in
        1) printf '1/%s' "$n" ;;
        2) printf '2/%s' "$n" ;;
        3) printf '3/%s/%s' "${n:0:1}" "$n" ;;
        *) printf '%s/%s/%s' "${n:0:2}" "${n:2:2}" "$n" ;;
    esac
}

packages="$(awk '
    /^\[\[package\]\]/ { name = ""; version = ""; source = "" }
    /^name = /    { gsub(/"/, "", $3); name = $3 }
    /^version = / { gsub(/"/, "", $3); version = $3 }
    /^source = "registry\+https:\/\/github.com\/rust-lang\/crates.io-index"/ { print name, version }
' "$lock")"
[ -n "$packages" ] || {
    printf 'error: no crates.io packages found in %s\n' "$lock" >&2
    exit 1
}

young=0
total=0
while read -r name version; do
    total=$((total + 1))
    pubtime="$(curl -fsS --retry 3 --max-time 30 "https://index.crates.io/$(index_path "$name")" |
        jq -r --arg v "$version" 'select(.vers == $v) | .pubtime // empty')"
    [ -n "$pubtime" ] || continue
    published="$(jq -rn --arg t "$pubtime" '$t | fromdateiso8601')"
    if [ "$published" -gt "$cutoff" ]; then
        if [[ " ${exceptions[*]/%/,} " == *" $name $version, "* ]]; then
            printf 'allowed: %s %s published %s (an exception)\n' "$name" "$version" "$pubtime"
            continue
        fi
        if allowed_family "$name"; then
            printf 'allowed: %s %s published %s (ALLOW_YOUNG_PREFIXES)\n' "$name" "$version" "$pubtime"
            continue
        fi
        age=$(((now - published) / 86400))
        printf 'too new: %s %s published %s (%s days ago)\n' "$name" "$version" "$pubtime" "$age"
        young=$((young + 1))
    fi
done <<<"$packages"

if [ "$young" -gt 0 ]; then
    printf 'error: %s of %s crates are younger than %s days\n' "$young" "$total" "$min_days" >&2
    exit 1
fi
printf 'ok: %s crates are at least %s days old\n' "$total" "$min_days"
