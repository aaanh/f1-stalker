#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
profile="${1:-release}"
version="$(awk -F'"' '/^version = / { print $2; exit }' "$root/Cargo.toml")"
repos_root="$(cd "$root/.." && pwd)"

echo "building F1 Stalker v${version} (${profile})"

if [[ "$(uname -s)" == "Darwin" ]]; then
  "$root/scripts/build-macos-dmg.sh" "$profile" arm64
  "$root/scripts/build-macos-dmg.sh" "$profile" universal

  rustup target add x86_64-pc-windows-gnu >/dev/null 2>&1 || true
  cargo build --profile "$profile" --target x86_64-pc-windows-gnu --manifest-path "$root/Cargo.toml"

  if [[ -x "$(command -v docker)" ]]; then
    docker run --platform linux/amd64 --rm \
      -v "$repos_root":/repos \
      -w "/repos/$(basename "$root")" \
      -e PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
      rust:1-bookworm \
      bash -c 'DEBIAN_FRONTEND=noninteractive apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq pkg-config libssl-dev libgtk-3-dev librsvg2-dev libxdo-dev >/dev/null && cargo build --profile '"$profile"''
  else
    echo "docker not found; skipping Linux build" >&2
    exit 1
  fi
else
  cargo build --profile "$profile" --manifest-path "$root/Cargo.toml"
  rustup target add x86_64-pc-windows-gnu x86_64-unknown-linux-gnu >/dev/null 2>&1 || true
  cargo build --profile "$profile" --target x86_64-pc-windows-gnu --manifest-path "$root/Cargo.toml"
  cargo build --profile "$profile" --target x86_64-unknown-linux-gnu --manifest-path "$root/Cargo.toml"
fi

"$root/scripts/publish-site-releases.sh"

echo "release artifacts ready for v${version}"
