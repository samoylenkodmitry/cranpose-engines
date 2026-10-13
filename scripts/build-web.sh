#!/usr/bin/env bash
# Builds the web examples into one folder for a static host:
#   <out>/index.html, <out>/<example>/index.html, <out>/<example>/pkg/ for
#   browsers with WebGPU and <out>/<example>/pkg-webgl2/ for the others.
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
  local build pkg
  for build in webgpu webgl2; do
    pkg="$out/$crate/pkg"
    [ "$build" = webgpu ] || pkg="$pkg-$build"
    wasm-pack build "$root/examples/$crate" --target web --profile web \
      --out-dir "$pkg" --no-default-features --features "$build"
    rm -f "$pkg/.gitignore" "$pkg/package.json" "$pkg/README.md"
  done
  sed -e "s/{{TITLE}}/$title/g" -e "s/{{PACKAGE}}/$package/g" \
    "$root/web/example.html" > "$out/$crate/index.html"
}

build bevy-in-cranpose "Bevy inside Cranpose"
build cranpose-hud-bevy "Cranpose HUD in Bevy"
