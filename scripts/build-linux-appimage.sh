#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ ! -f target/release/f1-stalker ]]; then
  cargo build --release
fi

mkdir -p dist
cp target/release/f1-stalker dist/f1-stalker

echo "Built dist/f1-stalker"
