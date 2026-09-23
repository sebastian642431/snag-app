# Changelog

Versions follow [semantic versioning](https://semver.org). The version lives in
the root `Cargo.toml`; a `vX.Y.Z` tag builds it and publishes the installers.

## 1.0.1 — 2026-09-23

### Changed
- The installer no longer asks "Open it now?". It installs, opens the app and
  closes on its own, and only waits for a key when something failed.

## 1.0.0 — 2026-09-23

First release. Two builds of the same downloader, sharing a `snag-core` crate
for everything that is not interface: tool discovery, settings, running
winget and yt-dlp, output parsing and the release check.

- **Snag · egui** — one process, around 50 MB of memory. Its source is split
  by responsibility: theme, widgets, install, download, one file per view.
- **Snag · Tauri** — the same app behind an HTML interface, around 95 MB
  across seven processes. Works from the keyboard and with a screen reader;
  a link that is not a link, or a download that could not start, says so
  under the field.

Both find yt-dlp and ffmpeg on their own, install them through winget with
live progress, show the running version in the title bar, and check GitHub
for a newer release: the banner can be put off until the next launch, and the
Tools panel always lists Snag itself with a "Get it" button when one exists.

The installers create shortcuts, register under Add/Remove Programs, and
uninstall without touching what you downloaded.

Tests cover version comparison, what a winget exit code means, the yt-dlp
command line per mode, yt-dlp and winget output parsing, the release check,
and text truncation that must not split a character. CI runs formatting,
`clippy -D warnings` and the tests on every push; a `vX.Y.Z` tag builds each
app on its own runner and publishes the installers only if those pass.
