# Snag

*[En español](README.es.md)*

A small, fast desktop front-end for [yt-dlp](https://github.com/yt-dlp/yt-dlp) on Windows. Paste a link, press Enter, get an MP3 or a video file.

Two builds of the same app ship here, sharing identical download logic and differing only in how the interface is drawn. The point of keeping both is that the tradeoff is measurable:

| | Processes | RAM idle | Binary |
| --- | --- | --- | --- |
| **Snag · egui** | 1 | ~70 MB | 5.0 MB |
| **Snag · Tauri** | 7 | ~369 MB | 5.9 MB |

Measured on Windows 11 with an empty download queue. The Tauri build embeds a Chromium engine through WebView2, which is where the difference comes from. **Use the egui build unless you have a reason not to.**

<!-- Add screenshots here: docs/egui.png and docs/tauri.png -->

## Install

Grab an installer from [Releases](../../releases) and run it. No admin rights needed.

Each installer carries the app inside it and, on a double-click, copies it to `%LOCALAPPDATA%\Programs\`, creates Desktop and Start Menu shortcuts, and registers the app under **Add or remove programs**. Nothing is written outside your user profile.

Both builds can be installed side by side. The title bar tells you which one you're looking at.

## Requirements

Snag drives two external programs and does no networking of its own:

```powershell
winget install yt-dlp.yt-dlp    # does the downloading
winget install Gyan.FFmpeg      # extracts MP3, merges video + audio
```

Neither is bundled, and neither needs a terminal: the **Tools** panel in the window installs what is missing through winget, shows the version of what is there, and updates either one (or both, with **Update all**) in a click. Snag looks for them on `PATH` first, then inside WinGet's package folders, so a stale `PATH` in an open terminal won't break it. Two status dots at the bottom of the window carry the same versions.

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

Versions live in one place: `version` in the root `Cargo.toml`. Every crate inherits it.

Bump it, commit, then tag and push:

```powershell
git tag v1.0.1
git push origin v1.0.1
```

The tag triggers a build that refuses to continue if the tag and the workspace version disagree, then creates the release and attaches both installers. Nothing is published by hand.

## How it works

| Folder | Contents |
| --- | --- |
| `core/` | Everything both builds need: tool discovery, settings, output parsing, and the runs of winget, yt-dlp and the release check. Where the tests live |
| `app-egui/` | The light build. Rust + [egui](https://github.com/emilk/egui), immediate-mode GUI drawn with OpenGL |
| `app-tauri/` | The web build. Rust + [Tauri](https://tauri.app), interface in plain HTML/CSS/JS, no bundler |
| `installer/` | Embeds an app with `include_bytes!` and installs it. One build per app |

Both apps spawn `yt-dlp` with `--newline`, read its stdout line by line, and parse the `[download] 42.3%` output into the progress bars. Each download runs on its own thread, so several links can go at once without blocking the window.

The egui build loads Segoe UI from `C:\Windows\Fonts` at runtime rather than bundling a font, which keeps the binary small and avoids redistributing a font that isn't ours to ship. Gradients, glows and the rounded progress bars are drawn as triangle meshes, since egui has no CSS.

## Windows only

Paths, the WinGet fallback, `explorer` integration and the font loading are all Windows-specific. Porting is possible but nothing here has been written with it in mind.

## Notes on use

Snag is a front-end: yt-dlp does the work, and what it supports is what Snag supports. Respect the terms of service of the sites you point it at and the copyright of what you download. Sites using DRM won't work, by design.

## License

MIT — see [LICENSE](LICENSE).

The tools Snag depends on carry their own terms: yt-dlp is released into the public domain under the Unlicense, and FFmpeg is LGPL/GPL. Neither is distributed with this project.
