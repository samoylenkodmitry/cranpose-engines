#!/usr/bin/env bash
# Builds the web examples into one folder for a static host:
#   <out>/index.html, <out>/<example>/index.html, <out>/<example>/pkg/ for
#   browsers with WebGPU and <out>/<example>/pkg-webgl2/ for the others.
#
#   scripts/build-web.sh [out]                          # every package
#   scripts/build-web.sh out bevy-in-cranpose webgl2    # one package
#   scripts/build-web.sh out godot                      # the Godot export
#
# The Godot export, <out>/godot/, needs Emscripten 4.0.20 (`emcc` on PATH),
# the Rust toolchain of scripts/emscripten-cargo.sh with rust-src, Godot 4.7.2
# as $GODOT (default `godot`) and its web templates
# (scripts/godot-web-templates.py 4.7.2).
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

# The Godot demo extension as an Emscripten side module, where the
# project's .gdextension expects it, then Godot's web export of the project.
godot() {
  local target="${CARGO_TARGET_DIR:-$root/target}"
  local library="wasm32-unknown-emscripten/web/cranpose_godot_demo.wasm"
  (cd "$root" && scripts/emscripten-cargo.sh build --profile web -p cranpose-godot-demo)
  if [ "$target" != "$root/target" ]; then
    mkdir -p "$(dirname "$root/target/$library")"
    cp "$target/$library" "$root/target/$library"
  fi
  mkdir -p "$out/godot"
  local project="$root/examples/godot/project"
  "${GODOT:-godot}" --headless --path "$project" --import
  "${GODOT:-godot}" --headless --path "$project" --export-release Web "$out/godot/index.html"
}

if [ $# -ge 3 ]; then
  build "$2" "$3"
elif [ "${2:-}" = godot ]; then
  godot
else
  for crate in bevy-in-cranpose cranpose-hud-bevy; do
    for build in webgpu webgl2; do
      build "$crate" "$build"
    done
  done
  godot
fi
