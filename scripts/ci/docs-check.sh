#!/usr/bin/env bash
# `just docs check` / `just book check`: build a book and fail on an error or warning, an empty chapter, or a broken link.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 1

book="${1:?usage: docs-check.sh <docs|book>}"
fail=0
log=$(mdbook build "$book" 2>&1) || { echo "$log"; exit 1; }

# A link in an included doc resolves from that doc, not from its chapter.
while IFS=$'\t' read -r chapter link; do
    wrapper="$book/src/$chapter"
    [ -f "$wrapper" ] || wrapper="${wrapper%index.md}README.md"
    inc=$(sed -n 's/.*{{#include \([^}:]*\).*/\1/p' "$wrapper" | head -1)
    from=$(dirname "$wrapper")
    [ -n "$inc" ] && from=$(dirname "$from/$inc")
    [ -e "$from/${link%%#*}" ] && continue
    echo "broken link in $wrapper: $link"
    fail=1
done < <(sed -n 's/^warning: mdbook-llms: broken link in \(.*\): \(.*\)$/\1\t\2/p' <<< "$log")

if grep -v 'mdbook-llms: broken link' <<< "$log" | grep -E 'ERROR|WARN|warning'; then fail=1; fi

# A missing include comes out empty, and mdBook says nothing.
while IFS= read -r empty; do
    echo "empty chapter: $empty"
    fail=1
done < <(find "target/$book/html" -name '*.md' -size -2c)

[ "$fail" = 0 ] && echo "The $book book builds, and every link leads somewhere."
exit "$fail"
