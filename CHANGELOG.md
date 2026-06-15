# Changelog

All notable changes to F1 Stalker are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.1] - 2026-06-15

Patch release. Dashboard polish, standings table, accessibility, and pin/chart behaviour updates.

### Added

- **Standings table** dashboard section with full-grid drivers and constructors (`src/domain/standings.rs`, `src/ui/standings_table.rs`).
- Championship / **Latest race** mode toggle and Drivers / Constructors tabs for standings, persisted as `standings_mode` and `standings_tab` in settings.
- **Font scale** (85%&ndash;135%): Settings row with Smaller/Larger controls, persisted `font_scale`, global shortcuts Cmd/Ctrl +, −, and 0, macOS View menu items.
- **Compare rivals** button in the pinned-driver chart header; activates rival-only chart focus when both rivals are picked.
- Wide **team logo display URLs** (`team_logo_display_url`) for rival fighter panels.
- Constructor chart empty states when no pins are set and rival compare is inactive.

### Changed

- **Unlimited driver pins**: removed the six-pin cap; only duplicate pins are blocked. UI copy updated (no "Pin limit reached" / "Full" states).
- Chart section renamed from "Championship charts" to **Pinned drivers/constructors**.
- **Constructor charts** now show teams linked to pinned drivers only, instead of the global top ten.
- **Compare rivals** moved from the Driver rivalry section header to the chart section; rivalry section hints updated accordingly.
- Championship narrative banner uses larger, scaled typography; hidden while rival chart compare is active.
- **LayoutConfig** takes `font_scale` and scales text sizes across dashboard, charts, pinned drivers, driver picker, rivalry mode, settings, and components.
- Rival mode panels: team logos, portrait sizing, scaled typography, and refined gap banner styling.
- Pinned driver cards: scaled name/code/team sizes; improved team logo fit URLs.
- Driver picker: scaled search, sort chips, and group headers.
- Title bar window drag uses press/move/release events instead of a delayed drag task.
- `ChampionshipTab` and `ChartMode` gain `from_key` / `key` helpers for settings persistence.

### Fixed

- Title bar drag no longer relies on a fixed 220 ms sleep before initiating window drag.

## [0.1.0] - 2026

Initial v1 release (committed on `master`).

### Added

- v1 dashboard: calendar triplet, countdown, pinned drivers, championship charts, quali/sprint grids, weather panels.
- **Rival mode**: pick two drivers, head-to-head stats and gap narrative.
- **Theme presets**: dark, light, and ten constructor-inspired palettes with hot reload.
- **Sprint grid** for pinned drivers after Sprint Qualifying.
- **Pre-season testing** calendar toggle.
- Desktop **notifications** for pinned-driver standings changes and optional session reminders.
- **System tray** and background-on-close setting.
- Simplified **first-run** welcome overlay.
- **Championship narrative** banner (leader/gap or world champion).
- macOS native menu bar, custom title bar, and window controls.
- GitHub Actions release workflow and build scripts (macOS DMG, Windows zip, Linux tarball).
- Project site with downloadable release artifacts.

[Unreleased]: https://gitlab.com/aaanh/f1-stalker/compare/v0.1.1...master
[0.1.1]: https://gitlab.com/aaanh/f1-stalker/compare/v0.1.0...v0.1.1
[0.1.0]: https://gitlab.com/aaanh/f1-stalker/-/tags/v0.1.0
