#!/usr/bin/env bash
# Builds the web examples into one folder for a static host:
#   <out>/index.html, <out>/<example>/index.html and <out>/<example>/pkg/.
#
#   scripts/build-web.sh [out]        # default out: dist/web
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$root/dist/web}"
mkdir -p "$out/screenshots"
# wasm-pack reads --out-dir from each crate's folder, so the path must be
# absolute.
out="$(cd "$out" && pwd)"
cp "$root/web/index.html" "$out/index.html"
cp "$root/screenshots/"*.jpg "$out/screenshots/"

build() {
  local crate="$1" title="$2"
  local package="${crate//-/_}"
  wasm-pack build "$root/examples/$crate" --target web --profile web \
    --out-dir "$out/$crate/pkg" --no-default-features --features web
  rm -f "$out/$crate/pkg/.gitignore" "$out/$crate/pkg/package.json" "$out/$crate/pkg/README.md"
  sed -e "s/{{TITLE}}/$title/g" -e "s/{{PACKAGE}}/$package/g" \
    "$root/web/example.html" > "$out/$crate/index.html"
}

build bevy-in-cranpose "Bevy inside Cranpose"
build cranpose-hud-bevy "Cranpose HUD in Bevy"
