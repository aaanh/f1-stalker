#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
profile="${1:-release}"
variant="${2:-arm64}"
version="$(awk -F'"' '/^version = / { print $2; exit }' "$root/Cargo.toml")"
app="$root/target/F1 Stalker.app"

case "$variant" in
  arm64)
    cargo build --profile "$profile" --manifest-path "$root/Cargo.toml"
    binary="$root/target/$profile/f1-stalker"
    dmg="$root/target/F1-Stalker-${version}-macos-arm64.dmg"
    ;;
  universal)
    rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null 2>&1 || true
    cargo build --profile "$profile" --target aarch64-apple-darwin --manifest-path "$root/Cargo.toml"
    cargo build --profile "$profile" --target x86_64-apple-darwin --manifest-path "$root/Cargo.toml"
    binary="$root/target/$profile/f1-stalker-universal"
    lipo -create \
      "$root/target/aarch64-apple-darwin/$profile/f1-stalker" \
      "$root/target/x86_64-apple-darwin/$profile/f1-stalker" \
      -output "$binary"
    dmg="$root/target/F1-Stalker-${version}-macos-universal.dmg"
    ;;
  *)
    echo "unknown macOS variant: $variant (expected arm64 or universal)" >&2
    exit 1
    ;;
esac

rm -rf "$app"
"$root/scripts/build-macos-app.sh" "$profile" "$binary"

staging="$root/target/dmg-staging-${variant}"
rm -rf "$staging"
mkdir -p "$staging"
cp -R "$app" "$staging/"
ln -s /Applications "$staging/Applications"

rm -f "$dmg"
hdiutil create -volname "F1 Stalker" -srcfolder "$staging" -ov -format UDZO "$dmg"
rm -rf "$staging"

echo "built $dmg"
