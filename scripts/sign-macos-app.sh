#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
app="${1:-$root/target/F1 Stalker.app}"
cert_name="${CODESIGN_CERT_NAME:-F1 Stalker Codesign}"
bundle_id="${CODESIGN_BUNDLE_ID:-com.f1-stalker.app}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  exit 0
fi

if [[ "${SKIP_CODESIGN:-0}" == "1" ]]; then
  echo "SKIP_CODESIGN=1, not signing $app"
  exit 0
fi

if [[ ! -d "$app" ]]; then
  echo "missing app bundle: $app" >&2
  exit 1
fi

identity="${CODESIGN_IDENTITY:-}"
if [[ -z "$identity" ]]; then
  identity="$(
    security find-identity -v -p codesigning | awk -v name="$cert_name" -F'"' '
      $2 == name {
        if (match($0, /[A-F0-9]{40}/)) {
          print substr($0, RSTART, RLENGTH)
        }
        exit
      }
    '
  )"
fi

if [[ -z "$identity" ]]; then
  if [[ "${CODESIGN_ADHOC_FALLBACK:-1}" == "1" ]]; then
    echo "no named codesign identity; using ad-hoc signature (-)" >&2
    identity="-"
  else
    echo "no codesign identity found; run: $root/scripts/setup-macos-codesign-cert.sh" >&2
    exit 1
  fi
fi

executable="$app/Contents/MacOS/F1 Stalker"
if [[ ! -f "$executable" ]]; then
  echo "missing executable: $executable" >&2
  exit 1
fi

echo "signing $app with: $identity"

codesign --force --sign "$identity" \
  --identifier "$bundle_id" \
  --timestamp=none \
  "$executable"

if [[ -f "$app/Contents/Resources/AppIcon.icns" ]]; then
  codesign --force --sign "$identity" \
    --identifier "$bundle_id" \
    --timestamp=none \
    "$app/Contents/Resources/AppIcon.icns"
fi

codesign --force --sign "$identity" \
  --identifier "$bundle_id" \
  --timestamp=none \
  "$app"

codesign --verify --verbose=2 "$app"
echo "signed $app"
