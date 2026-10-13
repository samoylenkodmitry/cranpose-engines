#!/usr/bin/env bash
# Packs the Godot example project with its release-built extension, which
# the project then loads from its own folder.
#
#   scripts/package-godot.sh <output folder>
set -euo pipefail

out="$1"
root="$(cd "$(dirname "$0")/.." && pwd)"
library="libcranpose_godot_demo.dylib"
stage="$(mktemp -d)/cranpose-engines-godot-macos-arm64"
mkdir -p "$out"
rsync -a --exclude .godot "$root/examples/godot/project/" "$stage/"
mkdir -p "$stage/.godot"
cp "$root/examples/godot/project/.godot/extension_list.cfg" "$stage/.godot/"
cp "$root/target/release/$library" "$stage/$library"
sed -i '' -E "s#^(macos\.(debug|release)) = .*#\1 = \"res://$library\"#" "$stage/cranpose.gdextension"
ditto -c -k --norsrc --noextattr --keepParent "$stage" "$out/cranpose-engines-godot-macos-arm64.zip"
