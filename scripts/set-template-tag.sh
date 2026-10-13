#!/usr/bin/env bash
# Points every template at a tag of this repository, before that tag is
# pushed: the release workflow checks that the templates name the tag.
#
#   scripts/set-template-tag.sh v0.2.0
set -euo pipefail

tag="$1"
root="$(cd "$(dirname "$0")/.." && pwd)"
find "$root/templates" -name Cargo.toml -not -path '*/target/*' -print0 \
  | xargs -0 sed -i.bak -E "s#(cranpose-engines\", tag = \")[^\"]+\"#\1$tag\"#"
find "$root/templates" -name 'Cargo.toml.bak' -delete
grep -rn 'cranpose-engines", tag = ' "$root/templates" --include=Cargo.toml
