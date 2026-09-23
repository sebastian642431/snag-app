use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
const NO_WINDOW: u32 = 0x0800_0000;

/// Runs a child process without flashing a console window.
pub fn quiet(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(NO_WINDOW);
    }
    cmd
}

pub mod text {
    /// Keeps the beginning of `text`, marking the cut with an ellipsis.
    pub fn shorten(text: &str, max: usize) -> String {
        if text.chars().count() <= max {
            return text.to_string();
        }
        let kept: String = text.chars().take(max.saturating_sub(1)).collect();
        format!("{kept}…")
    }

    /// Keeps the end of `text`, which is the useful half of a long path.
    pub fn tail(text: &str, max: usize) -> String {
        let count = text.chars().count();
        if count <= max {
            return text.to_string();
        }
        let kept: String = text.chars().skip(count - max + 1).collect();
        format!("…{kept}")
    }
}

pub mod version {
    /// Turns `v1.2.3`, `1.2` or `2026.08.19` into comparable numbers.
    pub fn number(text: &str) -> (u32, u32, u32) {
        let clean = text.trim().trim_start_matches(['v', 'V']);
        let mut parts = clean.split('.').map(|part| {
            part.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u32>()
                .unwrap_or(0)
        });
        (
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
        )
    }

    pub fn newer(candidate: &str, current: &str) -> bool {
        number(candidate) > number(current)
    }
}

pub mod parse {
    /// Reads the percentage out of a `[download]  42.3% of ...` line.
    pub fn download_pct(line: &str) -> Option<f32> {
        let rest = line.strip_prefix("[download]")?.trim_start();
        let end = rest.find('%')?;
        rest[..end].trim().parse::<f32>().ok()
    }

    /// Reads the file name out of a `Destination: <path>` line.
    pub fn destination(line: &str) -> Option<String> {
        let at = line.find("Destination: ")?;
        let path = std::path::PathBuf::from(line[at + "Destination: ".len()..].trim());
        path.file_stem().map(|s| s.to_string_lossy().into_owned())
    }

    /// Drops ANSI escapes and the block characters winget draws its bar with.
    pub fn strip_noise(raw: &str) -> String {
        let mut out = String::new();
        let mut chars = raw.chars();
        while let Some(c) = chars.next() {
            if c == '\u{1b}' {
                for skipped in chars.by_ref() {
                    if skipped.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
            if "█▒░▓■".contains(c) {
                continue;
            }
            out.push(c);
        }
        out.trim().to_string()
    }

    /// Reads progress out of a `1.50 MB / 12.30 MB` fragment.
    pub fn share(text: &str) -> Option<f32> {
        let (left, right) = text.split_once(" / ")?;
        let value = |part: &str| -> Option<f32> {
            let part = part.trim();
            let part = ["MB", "KB", "GB", "B"]
                .iter()
                .find_map(|unit| part.strip_suffix(unit))
                .unwrap_or(part);
            part.split_whitespace().last()?.parse::<f32>().ok()
        };
        let done = value(left)?;
        let total = value(right)?;
        if total <= 0.0 {
            return None;
        }
        Some((done / total).clamp(0.0, 1.0))
    }
}

pub mod tools {
    use super::*;

    pub fn find_exe(name: &str) -> Option<PathBuf> {
        let paths = std::env::var_os("PATH")?;
        std::env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    }

    fn winget_packages() -> Option<PathBuf> {
        let local = std::env::var_os("LOCALAPPDATA")?;
        Some(
            PathBuf::from(local)
                .join("Microsoft")
                .join("WinGet")
                .join("Packages"),
        )
    }

    /// Looks on PATH first, then inside WinGet's folders, so a stale PATH in an
    /// already open terminal does not hide a tool that is actually installed.
    pub fn find_ytdlp() -> Option<PathBuf> {
        if let Some(found) = find_exe("yt-dlp.exe") {
            return Some(found);
        }
        for entry in std::fs::read_dir(winget_packages()?).ok()?.flatten() {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with("yt-dlp.yt-dlp")
            {
                let exe = entry.path().join("yt-dlp.exe");
                if exe.is_file() {
                    return Some(exe);
                }
            }
        }
        None
    }

    /// Returns the folder holding ffmpeg.exe, which is what yt-dlp expects.
    pub fn find_ffmpeg() -> Option<PathBuf> {
        if let Some(found) = find_exe("ffmpeg.exe") {
            return found.parent().map(|p| p.to_path_buf());
        }
        for entry in std::fs::read_dir(winget_packages()?).ok()?.flatten() {
            if !entry.file_name().to_string_lossy().contains("FFmpeg") {
                continue;
            }
            for build in std::fs::read_dir(entry.path()).ok()?.flatten() {
                let bin = build.path().join("bin");
                if bin.join("ffmpeg.exe").is_file() {
                    return Some(bin);
                }
            }
        }
        None
    }

    fn first_line(exe: &Path, arg: &str) -> Option<String> {
        let mut cmd = Command::new(exe);
        cmd.arg(arg).stdout(Stdio::piped()).stderr(Stdio::null());
        let output = quiet(&mut cmd).output().ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().next()?.trim().to_string();
        (!line.is_empty()).then_some(line)
    }

    pub fn ytdlp_version(exe: &Path) -> Option<String> {
        first_line(exe, "--version")
    }

    pub fn ffmpeg_version(bin: &Path) -> Option<String> {
        let line = first_line(&bin.join("ffmpeg.exe"), "-version")?;
        Some(short_ffmpeg_version(&line))
    }

    /// `ffmpeg version 9.0.2-full_build-www.gyan.dev ...` -> `9.0.2`
    pub fn short_ffmpeg_version(line: &str) -> String {
        let rest = line.strip_prefix("ffmpeg version ").unwrap_or(line);
        let Some(word) = rest.split_whitespace().next() else {
            return String::new();
        };
        let short = word.split('-').next().unwrap_or(word);
        if short.starts_with(|c: char| c.is_ascii_digit()) {
            short.to_string()
        } else {
            text::shorten(word, 18)
        }
    }
}

pub mod settings {
    use super::*;

    fn dir() -> Option<PathBuf> {
        Some(PathBuf::from(std::env::var_os("APPDATA")?).join("Snag"))
    }

    pub fn folder() -> Option<PathBuf> {
        dir()
    }

    pub fn default_out() -> PathBuf {
        let home = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_default();
        home.join("Downloads").join("yt-dlp")
    }

    fn read(name: &str) -> Option<String> {
        let text = std::fs::read_to_string(dir()?.join(name)).ok()?;
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }

    fn write(name: &str, value: &str) {
        let Some(dir) = dir() else { return };
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join(name), value.as_bytes());
    }

    pub fn load_out() -> PathBuf {
        read("out.txt")
            .map(PathBuf::from)
            .unwrap_or_else(default_out)
    }

    pub fn save_out(path: &Path) {
        write("out.txt", &path.to_string_lossy());
    }
}

pub mod winget {
    use super::*;
    use std::io::{BufRead, BufReader, Read};
    use std::sync::{Arc, Mutex};

    const NO_UPGRADE: u32 = 0x8A15_002B;
    const NOT_INSTALLED: u32 = 0x8A15_0014;
    const ALREADY_INSTALLED: i32 = 43;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Stage {
        Starting,
        Downloading,
        Installing,
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum Outcome {
        /// Installed or upgraded.
        Done,
        /// Already at the newest version winget knows about.
        AlreadyCurrent,
        /// Present on the machine, but not through winget, so it cannot upgrade it.
        NotManaged,
        Failed(String),
    }

    pub fn available() -> bool {
        tools::find_exe("winget.exe").is_some()
    }

    /// Decides what a finished winget run actually meant.
    ///
    /// Exit codes alone are not enough: winget reports "nothing to upgrade" as a
    /// failure, and localised builds say it in the user's language, so the last
    /// line it printed has to be read as well.
    pub fn classify(code: Option<u32>, success: bool, last_message: &str) -> Outcome {
        let lower = last_message.to_lowercase();

        if code == Some(NO_UPGRADE)
            || lower.contains("no applicable upgrade")
            || lower.contains("no available upgrade")
            || lower.contains("no newer package")
        {
            return Outcome::AlreadyCurrent;
        }

        if success || code == Some(ALREADY_INSTALLED as u32) {
            return Outcome::Done;
        }

        if code == Some(NOT_INSTALLED) || lower.contains("no installed package") {
            return Outcome::NotManaged;
        }

        Outcome::Failed(if last_message.is_empty() {
            "winget could not install it".to_string()
        } else {
            text::shorten(last_message, 44)
        })
    }

    /// Runs winget, reporting progress as it reads the output, and returns what
    /// the run meant. Blocks until the process exits, so call it off the UI thread.
    pub fn run(id: &str, action: &str, mut on_progress: impl FnMut(Stage, f32)) -> Outcome {
        on_progress(Stage::Starting, 0.0);

        let mut cmd = Command::new("winget");
        cmd.args([
            action,
            "--id",
            id,
            "-e",
            "--accept-source-agreements",
            "--accept-package-agreements",
            "--disable-interactivity",
        ]);
        if action == "upgrade" {
            cmd.arg("--include-unknown");
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        quiet(&mut cmd);

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(err) => return Outcome::Failed(text::shorten(&err.to_string(), 44)),
        };

        // winget writes its own progress bar with carriage returns, so the last
        // useful line has to be tracked as it goes rather than read at the end.
        let tail = Arc::new(Mutex::new(String::new()));
        if let Some(stderr) = child.stderr.take() {
            let tail = Arc::clone(&tail);
            std::thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    if !line.trim().is_empty() {
                        *tail.lock().unwrap() = line;
                    }
                }
            });
        }

        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut chunk = Vec::new();
            let mut byte = [0u8; 1];
            let mut stage = Stage::Starting;
            let mut pct = 0.0f32;

            while matches!(reader.read(&mut byte), Ok(1)) {
                if byte[0] != b'\r' && byte[0] != b'\n' {
                    chunk.push(byte[0]);
                    continue;
                }

                let text = parse::strip_noise(&String::from_utf8_lossy(&chunk));
                chunk.clear();
                if text.is_empty() {
                    continue;
                }

                if let Some(share) = parse::share(&text) {
                    pct = share;
                    stage = Stage::Downloading;
                } else if text.chars().count() > 3 {
                    if stage != Stage::Starting {
                        stage = Stage::Installing;
                    }
                    *tail.lock().unwrap() = text;
                }

                on_progress(stage, pct);
            }
        }

        let status = child.wait();
        let code = status
            .as_ref()
            .ok()
            .and_then(|s| s.code())
            .map(|c| c as u32);
        let success = status.map(|s| s.success()).unwrap_or(false);
        let message = tail.lock().unwrap().clone();

        classify(code, success, &message)
    }
}

pub mod update {
    use super::*;

    pub const REPO: &str = "sebastian642431/snag-app";

    #[derive(Clone, PartialEq, Debug)]
    pub struct Release {
        pub tag: String,
        pub page: String,
    }

    /// Pulls one string value out of a JSON object by key. The release payload
    /// is the only JSON this project reads and it needs two fields of it, which
    /// does not justify a JSON dependency in a crate that otherwise has none.
    pub fn json_string(body: &str, key: &str) -> Option<String> {
        let needle = format!("\"{key}\"");
        let rest = &body[body.find(&needle)? + needle.len()..];
        let rest = rest[rest.find(':')? + 1..].trim_start();
        let rest = rest.strip_prefix('"')?;
        Some(rest[..rest.find('"')?].to_string())
    }

    pub fn parse_release(body: &str) -> Option<Release> {
        let tag = json_string(body, "tag_name").filter(|tag| !tag.is_empty())?;
        let page = json_string(body, "html_url")
            .unwrap_or_else(|| format!("https://github.com/{REPO}/releases/latest"));
        Some(Release { tag, page })
    }

    fn fetch(url: &str) -> Option<String> {
        let mut cmd = Command::new("curl");
        cmd.args([
            "-sL",
            "--max-time",
            "10",
            "-H",
            "User-Agent: Snag",
            "-H",
            "Accept: application/vnd.github+json",
            url,
        ]);
        cmd.stdout(Stdio::piped()).stderr(Stdio::null());
        let output = quiet(&mut cmd).output().ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Asks GitHub for the newest release. Blocks on the network, so call it
    /// off the UI thread.
    pub fn latest() -> Option<Release> {
        let body = fetch(&format!(
            "https://api.github.com/repos/{REPO}/releases/latest"
        ))?;
        parse_release(&body)
    }

    /// Opens a release page in the default browser. The address comes from
    /// GitHub's API, so anything outside github.com is refused rather than
    /// handed to the shell.
    pub fn open(page: &str) {
        if !page.starts_with("https://github.com/") {
            return;
        }
        let mut cmd = Command::new("cmd");
        cmd.args(["/c", "start", "", page]);
        let _ = quiet(&mut cmd).spawn();
    }
}

pub mod ytdlp {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Mode {
        Mp3,
        Video,
    }

    impl Mode {
        /// The names the HTML interface sends: `mp3` or `video`.
        pub fn from_name(name: &str) -> Option<Mode> {
            match name {
                "mp3" => Some(Mode::Mp3),
                "video" => Some(Mode::Video),
                _ => None,
            }
        }
    }

    /// What one line of yt-dlp output means for the interface.
    #[derive(Clone, PartialEq, Debug)]
    pub struct Event {
        /// Progress from 0 to 1, when the line carries any.
        pub pct: Option<f32>,
        /// The file name, once yt-dlp has decided it.
        pub title: Option<String>,
        /// What to show under the progress bar.
        pub line: String,
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum Outcome {
        Done,
        Failed(String),
    }

    /// Reads a line of yt-dlp output. Blank lines mean nothing.
    pub fn interpret(raw: &str) -> Option<Event> {
        let line = raw.trim();
        if line.is_empty() {
            return None;
        }
        let mut event = Event {
            pct: parse::download_pct(line).map(|pct| pct / 100.0),
            title: parse::destination(line),
            line: line.to_string(),
        };
        // yt-dlp reports nothing while ffmpeg works, so the bar would sit at
        // 100% looking finished when it is not.
        if line.starts_with("[ExtractAudio]") || line.contains("Merging formats") {
            event.pct = Some(0.99);
            event.line = "Processing with ffmpeg".to_string();
        }
        Some(event)
    }

    pub fn command(
        exe: &Path,
        ffmpeg: Option<&Path>,
        mode: Mode,
        out: &Path,
        url: &str,
    ) -> Command {
        let mut cmd = Command::new(exe);
        cmd.args(["--newline", "--no-playlist", "--restrict-filenames"]);
        if let Some(dir) = ffmpeg {
            cmd.arg("--ffmpeg-location").arg(dir);
        }
        match mode {
            Mode::Mp3 => {
                cmd.args(["-x", "--audio-format", "mp3", "--audio-quality", "0"]);
            }
            Mode::Video => {
                cmd.args(["-f", "bv*+ba/b", "--merge-output-format", "mp4"]);
            }
        }
        cmd.arg("-o").arg(out.join("%(title)s.%(ext)s")).arg(url);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        quiet(&mut cmd);
        cmd
    }

    /// Runs one download, reporting each line as it arrives, and returns how
    /// it ended. Blocks until yt-dlp exits, so call it off the UI thread.
    pub fn run(
        exe: &Path,
        ffmpeg: Option<&Path>,
        mode: Mode,
        out: &Path,
        url: &str,
        mut on_event: impl FnMut(Event),
    ) -> Outcome {
        let _ = std::fs::create_dir_all(out);

        let mut child = match command(exe, ffmpeg, mode, out, url).spawn() {
            Ok(child) => child,
            Err(err) => return Outcome::Failed(err.to_string()),
        };

        // The last thing yt-dlp wrote to stderr is the reason it failed.
        let errors = Arc::new(Mutex::new(String::new()));
        if let Some(stderr) = child.stderr.take() {
            let errors = Arc::clone(&errors);
            std::thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    if !line.trim().is_empty() {
                        *errors.lock().unwrap() = line;
                    }
                }
            });
        }

        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Some(event) = interpret(&line) {
                    on_event(event);
                }
            }
        }

        if child.wait().map(|s| s.success()).unwrap_or(false) {
            return Outcome::Done;
        }
        let message = errors.lock().unwrap().clone();
        Outcome::Failed(if message.is_empty() {
            "yt-dlp failed".to_string()
        } else {
            message
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winget::{classify, Outcome};

    #[test]
    fn a_clean_exit_means_it_installed() {
        assert_eq!(classify(Some(0), true, ""), Outcome::Done);
    }

    #[test]
    fn winget_code_43_also_means_installed() {
        assert_eq!(classify(Some(43), false, ""), Outcome::Done);
    }

    #[test]
    fn nothing_to_upgrade_is_not_a_failure() {
        assert_eq!(
            classify(Some(0x8A15_002B), false, ""),
            Outcome::AlreadyCurrent
        );
        assert_eq!(
            classify(Some(1), false, "No applicable upgrade found"),
            Outcome::AlreadyCurrent
        );
    }

    #[test]
    fn a_tool_installed_outside_winget_is_reported_as_such() {
        assert_eq!(classify(Some(0x8A15_0014), false, ""), Outcome::NotManaged);
    }

    #[test]
    fn a_real_failure_carries_the_last_line_winget_printed() {
        let out = classify(Some(1), false, "Installer hash does not match");
        assert_eq!(
            out,
            Outcome::Failed("Installer hash does not match".to_string())
        );
    }

    #[test]
    fn a_silent_failure_still_says_something() {
        assert_eq!(
            classify(Some(1), false, ""),
            Outcome::Failed("winget could not install it".to_string())
        );
    }

    #[test]
    fn versions_ignore_the_v_prefix() {
        assert_eq!(version::number("v1.2.3"), (1, 2, 3));
        assert_eq!(version::number("1.2.3"), (1, 2, 3));
    }

    #[test]
    fn missing_version_parts_count_as_zero() {
        assert_eq!(version::number("2"), (2, 0, 0));
        assert_eq!(version::number(""), (0, 0, 0));
    }

    #[test]
    fn date_style_tags_compare_in_order() {
        assert!(version::newer("2026.08.19", "2026.08.18"));
        assert!(!version::newer("2026.08.19", "2026.08.19"));
    }

    #[test]
    fn a_release_is_only_newer_when_it_really_is() {
        assert!(version::newer("1.0.1", "1.0.0"));
        assert!(!version::newer("1.0.0", "1.0.1"));
        assert!(!version::newer("1.0.0", "1.0.0"));
    }

    #[test]
    fn download_percentage_comes_out_of_the_line() {
        assert_eq!(
            parse::download_pct("[download]  42.3% of 10MiB"),
            Some(42.3)
        );
        assert_eq!(parse::download_pct("[download] 100% of 10MiB"), Some(100.0));
    }

    #[test]
    fn other_lines_have_no_percentage() {
        assert_eq!(parse::download_pct("[info] Writing thumbnail"), None);
        assert_eq!(parse::download_pct(""), None);
    }

    #[test]
    fn the_destination_keeps_spaces_and_drops_the_extension() {
        let line = r"[download] Destination: C:\Users\me\Downloads\My Song.mp3";
        assert_eq!(parse::destination(line).as_deref(), Some("My Song"));
    }

    #[test]
    fn share_turns_two_sizes_into_a_fraction() {
        assert_eq!(parse::share("5.00 MB / 10.00 MB"), Some(0.5));
        assert_eq!(parse::share("10.00 MB / 10.00 MB"), Some(1.0));
    }

    #[test]
    fn share_refuses_nonsense() {
        assert_eq!(parse::share("0 MB / 0 MB"), None);
        assert_eq!(parse::share("Successfully installed"), None);
    }

    #[test]
    fn noise_from_winget_is_removed() {
        assert_eq!(
            parse::strip_noise("  ██████  1.5 MB / 3 MB  "),
            "1.5 MB / 3 MB"
        );
        assert_eq!(parse::strip_noise("\u{1b}[32mdone\u{1b}[0m"), "done");
    }

    #[test]
    fn shortening_never_splits_a_character() {
        let text = "cancion con acentos áéíóú";
        let cut = text::shorten(text, 10);
        assert!(cut.chars().count() <= 10);
        assert!(cut.ends_with('…'));
    }

    #[test]
    fn tail_keeps_the_end_of_a_path() {
        let path = r"C:\Users\someone\Downloads\yt-dlp";
        let cut = text::tail(path, 16);
        assert!(cut.starts_with('…'));
        assert!(cut.ends_with("yt-dlp"));
    }

    #[test]
    fn short_text_is_left_alone() {
        assert_eq!(text::shorten("abc", 10), "abc");
        assert_eq!(text::tail("abc", 10), "abc");
    }

    #[test]
    fn the_ffmpeg_banner_reduces_to_a_number() {
        let line = "ffmpeg version 9.0.2-full_build-www.gyan.dev Copyright (c) 2000-2026";
        assert_eq!(tools::short_ffmpeg_version(line), "9.0.2");
    }

    #[test]
    fn a_git_built_ffmpeg_keeps_something_readable() {
        let line = "ffmpeg version N-125875-g5d4d3bdc61-win64-gpl";
        let short = tools::short_ffmpeg_version(line);
        assert!(!short.is_empty());
        assert!(short.chars().count() <= 18);
    }

    #[test]
    fn a_release_field_comes_out_of_the_json() {
        let body = r#"{"url": "x", "html_url": "https://github.com/o/r/releases/tag/v1.2.0", "id": 7, "tag_name":"v1.2.0"}"#;
        assert_eq!(
            update::json_string(body, "tag_name").as_deref(),
            Some("v1.2.0")
        );
        assert_eq!(
            update::json_string(body, "html_url").as_deref(),
            Some("https://github.com/o/r/releases/tag/v1.2.0")
        );
        assert_eq!(update::json_string(body, "missing"), None);
    }

    #[test]
    fn a_release_without_a_page_still_points_somewhere() {
        let release = update::parse_release(r#"{"tag_name": "v2.0.0"}"#).unwrap();
        assert_eq!(release.tag, "v2.0.0");
        assert!(release.page.starts_with("https://github.com/"));
    }

    #[test]
    fn an_empty_or_missing_tag_is_not_a_release() {
        assert_eq!(update::parse_release(r#"{"tag_name": ""}"#), None);
        assert_eq!(update::parse_release(r#"{"message": "Not Found"}"#), None);
    }

    #[test]
    fn a_download_line_reports_its_progress_as_a_fraction() {
        let event = ytdlp::interpret("[download]  42.5% of 10MiB").unwrap();
        assert_eq!(event.pct, Some(0.425));
        assert_eq!(event.title, None);
    }

    #[test]
    fn the_destination_line_names_the_job() {
        let event = ytdlp::interpret(r"[download] Destination: C:\out\My Song.webm").unwrap();
        assert_eq!(event.title.as_deref(), Some("My Song"));
        assert_eq!(event.pct, None);
    }

    #[test]
    fn ffmpeg_work_is_shown_as_almost_done() {
        for line in [
            r"[ExtractAudio] Destination: C:\out\a.mp3",
            r#"[Merger] Merging formats into "C:\out\a.mp4""#,
        ] {
            let event = ytdlp::interpret(line).unwrap();
            assert_eq!(event.pct, Some(0.99));
            assert_eq!(event.line, "Processing with ffmpeg");
        }
    }

    #[test]
    fn blank_output_is_ignored() {
        assert_eq!(ytdlp::interpret("   "), None);
    }

    #[test]
    fn the_mode_names_from_the_interface_are_recognised() {
        assert_eq!(ytdlp::Mode::from_name("mp3"), Some(ytdlp::Mode::Mp3));
        assert_eq!(ytdlp::Mode::from_name("video"), Some(ytdlp::Mode::Video));
        assert_eq!(ytdlp::Mode::from_name("flac"), None);
    }

    fn args_of(cmd: &std::process::Command) -> Vec<String> {
        cmd.get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn mp3_mode_extracts_audio_and_video_mode_merges_to_mp4() {
        let exe = Path::new("yt-dlp.exe");
        let out = Path::new(r"C:\out");
        let mp3 = args_of(&ytdlp::command(
            exe,
            None,
            ytdlp::Mode::Mp3,
            out,
            "https://x",
        ));
        assert!(mp3.windows(3).any(|w| w == ["-x", "--audio-format", "mp3"]));
        assert!(!mp3.contains(&"--merge-output-format".to_string()));

        let video = args_of(&ytdlp::command(
            exe,
            None,
            ytdlp::Mode::Video,
            out,
            "https://x",
        ));
        assert!(video
            .windows(2)
            .any(|w| w == ["--merge-output-format", "mp4"]));
        assert!(!video.contains(&"-x".to_string()));
    }

    #[test]
    fn ffmpeg_is_only_pointed_at_when_it_was_found() {
        let exe = Path::new("yt-dlp.exe");
        let out = Path::new(r"C:\out");
        let without = args_of(&ytdlp::command(
            exe,
            None,
            ytdlp::Mode::Mp3,
            out,
            "https://x",
        ));
        assert!(!without.contains(&"--ffmpeg-location".to_string()));

        let bin = Path::new(r"C:\ff\bin");
        let with = args_of(&ytdlp::command(
            exe,
            Some(bin),
            ytdlp::Mode::Mp3,
            out,
            "https://x",
        ));
        assert!(with
            .windows(2)
            .any(|w| w == ["--ffmpeg-location", r"C:\ff\bin"]));
    }

    #[test]
    fn the_url_is_the_last_argument_so_it_cannot_be_read_as_a_flag() {
        let exe = Path::new("yt-dlp.exe");
        let out = Path::new(r"C:\out");
        let args = args_of(&ytdlp::command(
            exe,
            None,
            ytdlp::Mode::Video,
            out,
            "https://x",
        ));
        assert_eq!(args.last().map(String::as_str), Some("https://x"));
        assert!(args.contains(&"--no-playlist".to_string()));
    }
}
