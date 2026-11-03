#!/usr/bin/env bash
# Package the existing native release binary. --build updates it first.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
case "${1:-}" in
  --build) cargo build --release --locked -p vectorcraft ;;
  "") ;;
  *) echo "Usage: bash packaging/macos/aetherys-app.sh [--build]" >&2; exit 2 ;;
esac
[[ "$(uname -s)" = Darwin ]] || { echo 'macOS required' >&2; exit 1; }
BIN="${CARGO_TARGET_DIR:-$ROOT/target}/release/vectorcraft"
[[ -x "$BIN" ]] || { echo 'Release binary missing; run with --build' >&2; exit 1; }
APP="$ROOT/dist/Aetherys World Editor.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
# Replace the executable atomically, preserving an already running app's inode.
/bin/cp -f "$BIN" "$APP/Contents/MacOS/.VectorCraft.new"
/bin/mv -f "$APP/Contents/MacOS/.VectorCraft.new" "$APP/Contents/MacOS/VectorCraft"
/bin/cp -f "$ROOT/assets/app-icon/vectorcraft.icns" "$APP/Contents/Resources/VectorCraft.icns"
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)
SHA=unknown
if [[ "${1:-}" = --build ]]; then SHA=$(git rev-parse HEAD); fi
sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/${VERSION%%-*}/g" -e "s/@BUILD_SHA@/$SHA/g" packaging/macos/Info.plist.in > "$APP/Contents/Info.plist"
# Reuse upstream document associations, minimum OS and resources; distinct local app identity.
/usr/libexec/PlistBuddy -c 'Set :CFBundleName Aetherys World Editor' "$APP/Contents/Info.plist"
/usr/libexec/PlistBuddy -c 'Set :CFBundleDisplayName Aetherys World Editor' "$APP/Contents/Info.plist"
/usr/libexec/PlistBuddy -c 'Set :CFBundleIdentifier local.aetherys.worldeditor' "$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"
codesign --force --sign - --timestamp=none --entitlements packaging/macos/entitlements.plist "$APP"
codesign --verify --strict "$APP"
echo "$APP"
