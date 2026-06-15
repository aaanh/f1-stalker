#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

github_repo="aaanh/f1-stalker"
github_url="https://github.com/${github_repo}.git"
gitlab_url="https://gitlab.com/aaanh/f1-stalker"

chmod +x .githooks/pre-push
git config core.hooksPath .githooks

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI is required: https://cli.github.com/"
  exit 1
fi

gh auth setup-git

if ! gh repo view "$github_repo" >/dev/null 2>&1; then
  gh repo create "$github_repo" \
    --public \
    --description "Read-only mirror of ${gitlab_url}" \
    --disable-wiki \
    --disable-issues
  echo "Created https://github.com/${github_repo}"
fi

if ! git remote get-url github >/dev/null 2>&1; then
  git remote add github "$github_url"
  echo "Added github remote: $github_url"
fi

echo "core.hooksPath=.githooks (pre-push mirrors origin pushes to github)"
