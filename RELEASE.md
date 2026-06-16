# Release procedure

End-to-end steps for cutting a new F1 Stalker version. GitLab is canonical: https://gitlab.com/aaanh/f1-stalker

Official release artifacts (DMG, zip, tarball) are built locally on macOS with `scripts/build-release.sh`, committed into the repo site bundle, tagged, and published as a GitLab Release. GitHub is a read-only mirror.

## Prerequisites

### Tooling

- **Rust** >= 1.95 (see `rust-version` in `Cargo.toml`)
- **macOS** for the full cross-platform release pipeline (DMG signing, Windows cross-compile, Linux via Docker)
- **Docker** (on macOS) for the Linux AMD64 build inside `rust:1-bookworm`
- **`rustup` targets** (installed automatically by scripts when missing):
  - `x86_64-pc-windows-gnu`
  - `aarch64-apple-darwin`, `x86_64-apple-darwin` (universal DMG)
- **`glab`** authenticated to `gitlab.com` for creating GitLab Releases
- **Sibling repo**: `openf1-client` must exist at `../openf1-client` (path dependency in `Cargo.toml`). The Linux Docker build mounts the parent directory of this repo, so both repos must live under the same parent folder.

### Optional (macOS code signing)

```sh
./scripts/setup-macos-codesign-cert.sh
```

Creates a local self-signed identity (`F1 Stalker Codesign`). Without it, builds fall back to ad-hoc signing (`CODESIGN_ADHOC_FALLBACK=1`, default). Set `SKIP_CODESIGN=1` to skip signing entirely.

## Overview

```
1. Pre-flight (tests, changelog, version bump)
2. ./scripts/build-release.sh
3. Commit source + site artifacts
4. git tag + git push origin (+ tag)
5. glab release create (upload assets, set released_at)
```

## 1. Pre-flight

From the repo root:

```sh
cargo test
cargo build
```

Fix warnings and failing tests before releasing.

### Bump version

Update these files to the new semver (e.g. `0.1.3`):

| File | What to change |
|------|----------------|
| `Cargo.toml` | `version = "…"` |
| `CHANGELOG.md` | New `## [x.y.z] - YYYY-MM-DD` section; move `[Unreleased]` link |

Run `cargo build` once so `Cargo.lock` picks up the package version.

### Changelog

Follow [Keep a Changelog](https://keepachangelog.com/). Summarize user-visible **Added**, **Changed**, and **Fixed** items. The GitLab release description is usually copied from this section.

## 2. Build release artifacts

```sh
./scripts/build-release.sh
```

Reads the version from `Cargo.toml` and builds all platform artifacts, then runs `scripts/publish-site-releases.sh`.

### What `build-release.sh` does (macOS host)

1. **macOS ARM64 DMG** via `scripts/build-macos-dmg.sh release arm64`
2. **macOS Universal DMG** via `scripts/build-macos-dmg.sh release universal` (`lipo` of arm64 + x86_64)
3. **Windows AMD64** `cargo build --release --target x86_64-pc-windows-gnu`
4. **Linux AMD64** `cargo build --release` inside Docker (`rust:1-bookworm` + GTK dev packages)
5. **Publish to site** copies artifacts into `f1-stalker-site/public/downloads/v<version>/` and regenerates `f1-stalker-site/src/lib/releases.ts`

### Output artifacts

| Artifact | Path |
|----------|------|
| macOS ARM64 DMG | `target/F1-Stalker-<version>-macos-arm64.dmg` |
| macOS Universal DMG | `target/F1-Stalker-<version>-macos-universal.dmg` |
| Linux tarball | `f1-stalker-site/public/downloads/v<version>/F1-Stalker-<version>-linux-amd64.tar.gz` |
| Windows zip | `f1-stalker-site/public/downloads/v<version>/F1-Stalker-<version>-windows-amd64.zip` |

`target/` is gitignored; site copies under `f1-stalker-site/public/downloads/` are committed.

### Partial / manual builds

| Script | Purpose |
|--------|---------|
| `scripts/build-macos-dmg.sh [profile] [arm64\|universal]` | One macOS DMG variant |
| `scripts/build-macos-app.sh [profile] [binary]` | `.app` bundle only |
| `scripts/build-linux-appimage.sh` | Raw Linux binary to `dist/` (minimal; not used by main release flow) |

On non-macOS hosts, `build-release.sh` builds native + Windows + Linux targets directly (no DMG).

## 3. Commit

Stage and commit in logical chunks (specs, features, fixes, then release). A typical final release commit includes:

- `Cargo.toml`, `Cargo.lock`
- `CHANGELOG.md`
- `f1-stalker-site/src/lib/releases.ts`
- `f1-stalker-site/public/downloads/v<version>/` (all four artifacts)

Example:

```sh
git add Cargo.toml Cargo.lock CHANGELOG.md \
  f1-stalker-site/src/lib/releases.ts \
  f1-stalker-site/public/downloads/v0.1.3/

git commit -m "$(cat <<'EOF'
chore: release v0.1.3

Bump version, update changelog, and publish site download artifacts.
EOF
)"
```

## 4. Tag and push

```sh
git tag -a v0.1.3 -m "v0.1.3"
git push origin master
git push origin v0.1.3
```

If `scripts/setup-githooks.sh` was run, pushing `origin` also mirrors branches and tags to GitHub.

## 5. GitLab Release

Create the release and upload assets with `glab`. Use `GLAB_HOST=gitlab.com` if other GitLab instances are configured locally.

**Important:** GitLab sorts releases by `released_at`, not semver. Set `released_at` to a timestamp **after** the previous release, or the new version may appear older than the last one.

```sh
GLAB_HOST=gitlab.com glab release create v0.1.3 \
  --name "v0.1.3" \
  -F /path/to/release-notes.md \
  --released-at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  -R aaanh/f1-stalker \
  "f1-stalker-site/public/downloads/v0.1.3/F1-Stalker-0.1.3-macos-universal.dmg" \
  "f1-stalker-site/public/downloads/v0.1.3/F1-Stalker-0.1.3-macos-arm64.dmg" \
  "f1-stalker-site/public/downloads/v0.1.3/F1-Stalker-0.1.3-linux-amd64.tar.gz" \
  "f1-stalker-site/public/downloads/v0.1.3/F1-Stalker-0.1.3-windows-amd64.zip"
```

`release-notes.md` is usually the new `CHANGELOG.md` section (without the heading date line).

To update only the release date or notes on an existing release, run `glab release create` again with the same tag. If you change `released_at` without `-F`, release notes may be cleared; pass `-F` again to restore them.

Release page: https://gitlab.com/aaanh/f1-stalker/-/releases

## GitHub Actions note

`.github/workflows/release.yml` on the GitHub mirror builds raw per-target binaries on tag push. It does **not** produce DMGs, zip/tar bundles, or update the project site. The canonical distributables are the locally built artifacts published to GitLab Releases and `f1-stalker-site/public/downloads/`.

## Troubleshooting

| Problem | Likely cause | Fix |
|---------|----------------|-----|
| Linux build fails in Docker | Missing sibling `openf1-client` in parent dir | Clone `openf1-client` next to this repo |
| `missing release artifacts` | Partial build | Re-run `./scripts/build-release.sh`; check Docker is running |
| macOS "unidentified developer" | Ad-hoc / self-signed cert | User: Right-click → Open, or `xattr -dr com.apple.quarantine …` |
| New release listed below older version on GitLab | `released_at` earlier than previous release | Re-run `glab release create` with a later `--released-at` and `-F` for notes |
| `glab` OAuth errors | Expired token or wrong host | `glab auth login --hostname gitlab.com` or use `GLAB_HOST=gitlab.com` |

## Quick checklist

> Steps are for references, do not check the boxes

- [ ] `cargo test` passes
- [ ] `Cargo.toml` version bumped
- [ ] `CHANGELOG.md` updated
- [ ] `./scripts/build-release.sh` succeeded
- [ ] Site artifacts and `releases.ts` committed
- [ ] Tag created and pushed to `origin`
- [ ] GitLab Release created with assets and correct `released_at`
- [ ] Release notes visible on GitLab
- [ ] F1 Stalker site has the new release artifacts, on-page version updated