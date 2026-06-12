#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
profile="${1:-debug}"
binary="$root/target/$profile/f1-stalker"
app="$root/target/F1 Stalker.app"

if [[ ! -f "$binary" ]]; then
  echo "missing binary: $binary (run: cargo build --profile $profile)" >&2
  exit 1
fi

mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

cp "$binary" "$app/Contents/MacOS/F1 Stalker"

cat >"$app/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>F1 Stalker</string>
  <key>CFBundleIdentifier</key>
  <string>com.f1-stalker.app</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>F1 Stalker</string>
  <key>CFBundleDisplayName</key>
  <string>F1 Stalker</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF

iconset="$root/target/app-icon.iconset"
icns="$app/Contents/Resources/AppIcon.icns"
icons="$root/AppIcons/Assets.xcassets/AppIcon.appiconset"

rm -rf "$iconset"
mkdir -p "$iconset"
cp "$icons/16.png" "$iconset/icon_16x16.png"
cp "$icons/32.png" "$iconset/icon_16x16@2x.png"
cp "$icons/32.png" "$iconset/icon_32x32.png"
cp "$icons/64.png" "$iconset/icon_32x32@2x.png"
cp "$icons/128.png" "$iconset/icon_128x128.png"
cp "$icons/256.png" "$iconset/icon_128x128@2x.png"
cp "$icons/256.png" "$iconset/icon_256x256.png"
cp "$icons/512.png" "$iconset/icon_256x256@2x.png"
cp "$icons/512.png" "$iconset/icon_512x512.png"
cp "$icons/1024.png" "$iconset/icon_512x512@2x.png"
iconutil -c icns "$iconset" -o "$icns"

/usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string AppIcon" "$app/Contents/Info.plist" 2>/dev/null \
  || /usr/libexec/PlistBuddy -c "Set :CFBundleIconFile AppIcon" "$app/Contents/Info.plist"

"$root/scripts/sign-macos-app.sh" "$app"

echo "built $app"
echo "open \"$app\""
