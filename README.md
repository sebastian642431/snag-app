# Snag

*[En español](README.es.md)*

A small, fast desktop front-end for [yt-dlp](https://github.com/yt-dlp/yt-dlp) on Windows. Paste a link, press Enter, get an MP3 or a video file.

Two builds of the same app ship here, sharing identical download logic and differing only in how the interface is drawn. The point of keeping both is that the tradeoff is measurable:

| | Processes | RAM idle | Binary |
| --- | --- | --- | --- |
| **Snag · egui** | 1 | ~70 MB | 6.9 MB |
| **Snag · Tauri** | 7 | ~369 MB | 6.6 MB |

Measured on Windows 11 with an empty download queue. The Tauri build embeds a Chromium engine through WebView2, which is where the difference comes from. **Use the egui build unless you have a reason not to.**

<!-- Add screenshots here: docs/egui.png and docs/tauri.png -->

## Install

Grab an installer from [Releases](../../releases) and run it. No admin rights needed.

Each installer carries the app inside it and, on a double-click, copies it to `%LOCALAPPDATA%\Programs\`, creates Desktop and Start Menu shortcuts, and registers the app under **Add or remove programs**. Nothing is written outside your user profile.

Both builds can be installed side by side. The title bar tells you which one you're looking at, and which version.

Windows will show "Windows protected your PC" the first time: the installer is not signed with a paid certificate, so SmartScreen has no reputation for it. Click **More info**, then **Run anyway**. The installer opens a console, does its work, opens the app and closes on its own.

## Requirements

Snag drives two external programs and does no networking of its own:

```powershell
winget install yt-dlp.yt-dlp    # does the downloading
winget install Gyan.FFmpeg      # extracts MP3, merges video + audio
```

Neither is bundled, and neither needs a terminal: the **Tools** panel in the window installs what is missing through winget, shows the version of what is there, and updates either one (or both, with **Update all**) in a click. Snag looks for them on `PATH` first, then inside WinGet's package folders, so a stale `PATH` in an open terminal won't break it. Two status dots at the bottom of the window carry the same versions.

The same panel lists Snag itself. On startup the app asks GitHub for the newest release; if there is one, a banner offers it, **Later** hides the banner until the next launch, and the Tools row keeps a **Get it** button, so an update can be put off but never lost.

Downloads land in `Downloads\yt-dlp`, changeable from the app.

## Without a window

`Snag-console.cmd` is the same thing as a prompt: paste a link, pick MP3 or video, done. It needs no install.

## Uninstall

Through **Add or remove programs**, or by running `Uninstall.exe` from the install folder. It removes the app, the shortcuts and the registry entry, and leaves your downloads alone.

## Build from source

Requires a [Rust](https://rustup.rs) toolchain. The Tauri build also needs WebView2, which ships with Windows 11.

This is a Cargo workspace, so everything builds from the root and shares one `target/`:

```powershell
cargo build --release -p snag-egui -p snag-tauri
```

The installer embeds one app, chosen at compile time through environment variables, so it is built once per app:

```powershell
$env:PAYLOAD_EXE  = "$PWD\target\release\snag-egui.exe"
$env:PRODUCT_NAME = 'Snag egui'
$env:EXE_NAME     = 'Snag-egui.exe'
$env:APP_VERSION  = '1.0.0'
cargo build --release -p snag-installer
```

That leaves `target/release/snag-setup.exe`, which is what ships as `Install-Snag-egui.exe`.

Before pushing, the same three checks CI runs:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Releasing

The version lives in `version` in the root `Cargo.toml`, which every crate inherits, and once more in `app-tauri/tauri.conf.json`, which Tauri reads for its bundle metadata and CI does not check. Change both, add the entry to `CHANGELOG.md`, commit, then tag and push:

```powershell
git tag -a v1.0.2 -m "Snag 1.0.2"
git push origin v1.0.2
```

The tag starts three jobs at once: the checks, and one build per app, each with its own cache and installer. Publishing waits for all three, refuses to continue if the tag and the workspace version disagree, then creates the release and attaches `Install-Snag-egui.exe` and `Install-Snag-tauri.exe`. Nothing is published by hand. A cold run takes about five minutes; with warm caches, less.

## How it works

| Folder | Contents |
| --- | --- |
| `core/` | Everything both builds need: tool discovery, settings, output parsing, and the runs of winget, yt-dlp and the release check. Where the tests live |
| `app-egui/` | The light build. Rust + [egui](https://github.com/emilk/egui), immediate-mode GUI drawn with OpenGL. Split by responsibility: theme, paint, widgets, install, download, and one file per view under `app/` |
| `app-tauri/` | The web build. Rust + [Tauri](https://tauri.app), interface in plain HTML/CSS/JS, no bundler |
| `installer/` | Embeds an app with `include_bytes!` and installs it. One build per app |

Both apps hand a link to `core`, which spawns `yt-dlp` with `--newline`, reads its stdout line by line, and turns the `[download] 42.3%` output into events; each app only decides how to draw them. Each download runs on its own thread, so several links can go at once without blocking the window. Running winget and checking for a release work the same way.

The release profile uses thin LTO: measured against fat LTO with one codegen unit, a cold build takes a third less and a small change 60% less, for binaries 9% larger.

The egui build loads Segoe UI from `C:\Windows\Fonts` at runtime rather than bundling a font, which keeps the binary small and avoids redistributing a font that isn't ours to ship. Gradients, glows and the rounded progress bars are drawn as triangle meshes, since egui has no CSS.

## Windows only

Paths, the WinGet fallback, `explorer` integration and the font loading are all Windows-specific. Porting is possible but nothing here has been written with it in mind.

## Notes on use

Snag is a front-end: yt-dlp does the work, and what it supports is what Snag supports. Respect the terms of service of the sites you point it at and the copyright of what you download. Sites using DRM won't work, by design.

## License

MIT — see [LICENSE](LICENSE).

The tools Snag depends on carry their own terms: yt-dlp is released into the public domain under the Unlicense, and FFmpeg is LGPL/GPL. Neither is distributed with this project.
