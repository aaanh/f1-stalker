#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(awk -F'"' '/^version = / { print $2; exit }' "$root/Cargo.toml")"
site_dir="$root/f1-stalker-site/public/downloads/v$version"

macos_dmg="$root/target/F1-Stalker-${version}-macos-arm64.dmg"
linux_bin="$root/target/release/f1-stalker"
windows_exe="$root/target/x86_64-pc-windows-gnu/release/f1-stalker.exe"

missing=()
[[ -f "$macos_dmg" ]] || missing+=("$macos_dmg")
[[ -f "$linux_bin" ]] || missing+=("$linux_bin")
[[ -f "$windows_exe" ]] || missing+=("$windows_exe")

if ((${#missing[@]} > 0)); then
  echo "missing release artifacts:" >&2
  printf '  %s\n' "${missing[@]}" >&2
  exit 1
fi

mkdir -p "$site_dir"

cp "$macos_dmg" "$site_dir/"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$linux_bin" "$tmp/f1-stalker"
tar -czf "$site_dir/F1-Stalker-${version}-linux-amd64.tar.gz" -C "$tmp" f1-stalker

cp "$windows_exe" "$tmp/f1-stalker.exe"
(
  cd "$tmp"
  zip -q "$site_dir/F1-Stalker-${version}-windows-amd64.zip" f1-stalker.exe
)

cat >"$root/f1-stalker-site/src/lib/releases.ts" <<EOF
export const APP_VERSION = "${version}" as const

export type ReleaseDownload = {
  label: string
  href: string
  fileName: string
}

export const releaseDownloads: ReleaseDownload[] = [
  {
    label: "macOS ARM64",
    href: "/downloads/v${version}/F1-Stalker-${version}-macos-arm64.dmg",
    fileName: "F1-Stalker-${version}-macos-arm64.dmg",
  },
  {
    label: "Linux AMD64",
    href: "/downloads/v${version}/F1-Stalker-${version}-linux-amd64.tar.gz",
    fileName: "F1-Stalker-${version}-linux-amd64.tar.gz",
  },
  {
    label: "Windows 10/11 AMD64",
    href: "/downloads/v${version}/F1-Stalker-${version}-windows-amd64.zip",
    fileName: "F1-Stalker-${version}-windows-amd64.zip",
  },
]
EOF

echo "published v${version} to f1-stalker-site/public/downloads/v${version}/"
