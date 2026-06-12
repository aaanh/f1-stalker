#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
openf1_root="$root/../openf1-client/sdks/rust/openf1"

if ! command -v cargo-watch >/dev/null 2>&1; then
  echo "cargo-watch not found; installing..."
  cargo install cargo-watch
fi

watch_paths=(
  "$root/src"
  "$root/Cargo.toml"
  "$openf1_root/src"
  "$openf1_root/Cargo.toml"
)

args=(cargo watch --clear --delay 0.3)

for path in "${watch_paths[@]}"; do
  if [[ -d "$path" || -f "$path" ]]; then
    args+=(--watch "$path")
  fi
done

args+=(--exec "run --manifest-path $root/Cargo.toml")

echo "Hot reload: rebuild and restart on changes in f1-stalker and openf1-client"
exec "${args[@]}"
