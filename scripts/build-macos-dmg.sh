#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
profile="${1:-release}"
app="$root/target/F1 Stalker.app"
version="$(awk -F'"' '/^version = / { print $2; exit }' "$root/Cargo.toml")"
dmg="$root/target/F1-Stalker-${version}-macos-arm64.dmg"

if [[ ! -d "$app" ]]; then
  cargo build --profile "$profile" --manifest-path "$root/Cargo.toml"
  "$root/scripts/build-macos-app.sh" "$profile"
fi

staging="$root/target/dmg-staging"
rm -rf "$staging"
mkdir -p "$staging"
cp -R "$app" "$staging/"
ln -s /Applications "$staging/Applications"

rm -f "$dmg"
hdiutil create -volname "F1 Stalker" -srcfolder "$staging" -ov -format UDZO "$dmg"
rm -rf "$staging"

echo "built $dmg"
