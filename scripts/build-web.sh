#!/usr/bin/env bash
# Builds the web examples into one folder for a static host:
#   <out>/index.html, <out>/<example>/index.html, <out>/<example>/pkg/ for
#   browsers with WebGPU and <out>/<example>/pkg-webgl2/ for the others.
#
#   scripts/build-web.sh [out]                          # every package
#   scripts/build-web.sh out bevy-in-cranpose webgl2    # one package
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$root/dist/web}"
mkdir -p "$out/screenshots"
# wasm-pack reads --out-dir from each crate's folder, so the path must be
# absolute.
out="$(cd "$out" && pwd)"
cp "$root/web/index.html" "$out/index.html"
cp "$root/screenshots/"*.jpg "$out/screenshots/"

title() {
  case "$1" in
    bevy-in-cranpose) echo "Bevy inside Cranpose" ;;
    cranpose-hud-bevy) echo "Cranpose HUD in Bevy" ;;
    *) echo "unknown example $1" >&2; exit 1 ;;
  esac
}

build() {
  local crate="$1" build="$2"
  local package="${crate//-/_}" pkg="$out/$crate/pkg"
  [ "$build" = webgpu ] || pkg="$pkg-$build"
  wasm-pack build "$root/examples/$crate" --target web --profile web \
    --out-dir "$pkg" --no-default-features --features "$build"
  rm -f "$pkg/.gitignore" "$pkg/package.json" "$pkg/README.md"
  sed -e "s/{{TITLE}}/$(title "$crate")/g" -e "s/{{PACKAGE}}/$package/g" \
    "$root/web/example.html" > "$out/$crate/index.html"
}

if [ $# -ge 3 ]; then
  build "$2" "$3"
else
  for crate in bevy-in-cranpose cranpose-hud-bevy; do
    for build in webgpu webgl2; do
      build "$crate" "$build"
    done
  done
fi
