#!/usr/bin/env bash
# Packs the two release-built Bevy examples into one archive for a platform.
# On macOS each becomes an app bundle.
#
#   scripts/package-bevy.sh <platform name> <output folder>
set -euo pipefail

platform="$1"
out="$2"
root="$(cd "$(dirname "$0")/.." && pwd)"
stage="$(mktemp -d)/cranpose-engines-bevy-$platform"
mkdir -p "$stage" "$out"

bundle() {
  local binary="$1" title="$2" identifier="$3"
  local app="$stage/$title.app"
  mkdir -p "$app/Contents/MacOS"
  cp "$root/target/release/$binary" "$app/Contents/MacOS/$binary"
  cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>$binary</string>
  <key>CFBundleIdentifier</key><string>$identifier</string>
  <key>CFBundleName</key><string>$title</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${GITHUB_REF_NAME:-dev}</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST
}

case "$platform" in
  macos-*)
    bundle bevy-in-cranpose "Bevy inside Cranpose" dev.cranpose.engines.bevy-in-cranpose
    bundle cranpose-hud-bevy "Cranpose HUD in Bevy" dev.cranpose.engines.cranpose-hud-bevy
    ditto -c -k --norsrc --noextattr --keepParent "$stage" "$out/cranpose-engines-bevy-$platform.zip"
    ;;
  windows-*)
    cp "$root/target/release/bevy-in-cranpose.exe" "$root/target/release/cranpose-hud-bevy.exe" "$stage/"
    (cd "$(dirname "$stage")" && 7z a -tzip "$root/$out/cranpose-engines-bevy-$platform.zip" "$(basename "$stage")" >/dev/null)
    ;;
  *)
    cp "$root/target/release/bevy-in-cranpose" "$root/target/release/cranpose-hud-bevy" "$stage/"
    tar -C "$(dirname "$stage")" -czf "$out/cranpose-engines-bevy-$platform.tar.gz" "$(basename "$stage")"
    ;;
esac
