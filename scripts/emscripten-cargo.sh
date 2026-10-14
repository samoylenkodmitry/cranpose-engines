#!/usr/bin/env bash
# cargo for the Godot extension's web build. Godot's web export loads it as
# an Emscripten side module. Its std comes from source on a nightly toolchain
# with rust-src ($GODOT_RUST_TOOLCHAIN), and it aborts on panic: Godot's web
# templates have no wasm exception support.
#
#   scripts/emscripten-cargo.sh build --profile web -p cranpose-godot-demo
#   scripts/emscripten-cargo.sh clippy -p cranpose-godot-demo -- -D warnings
set -euo pipefail

toolchain="${GODOT_RUST_TOOLCHAIN:-nightly-2026-10-03}"
command="$1"
shift
flags='["-C", "link-args=-sSIDE_MODULE=2", "-C", "llvm-args=-enable-emscripten-cxx-exceptions=0", "-C", "panic=abort", "-Z", "default-visibility=hidden", "-Z", "link-native-libraries=no"]'
exec cargo "+$toolchain" "$command" -Zbuild-std=std,panic_abort \
  --target wasm32-unknown-emscripten \
  --config "target.wasm32-unknown-emscripten.rustflags=$flags" "$@"
