#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use snag_core::settings::{load_out, save_out};
use snag_core::tools::{ffmpeg_version, find_exe, find_ffmpeg, find_ytdlp, ytdlp_version};
use snag_core::update;
use snag_core::version;
use snag_core::winget::{self, Outcome, Stage};
use snag_core::ytdlp::{self, Mode};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct AppState {
    out: Mutex<PathBuf>,
    versions: Mutex<Cache>,
}

#[derive(Default)]
struct Cache {
    yt_path: Option<PathBuf>,
    yt: Option<String>,
    ff_path: Option<PathBuf>,
    ff: Option<String>,
}

#[derive(Serialize)]
struct Probe {
    version: &'static str,
    ytdlp: Option<String>,
    ytdlp_version: Option<String>,
    ffmpeg: bool,
    ffmpeg_version: Option<String>,
    winget: bool,
    out: String,
}

#[derive(Clone, Serialize)]
struct Update {
    id: u64,
    title: String,
    pct: f32,
    state: String,
    line: String,
}

#[derive(Serialize)]
struct UpdateInfo {
    tag: String,
    page: String,
    current: String,
}

#[tauri::command]
fn check_update() -> Option<UpdateInfo> {
    let current = env!("CARGO_PKG_VERSION");
    let release = update::latest()?;
    if !version::newer(&release.tag, current) {
        return None;
    }
    Some(UpdateInfo {
        tag: release.tag,
        page: release.page,
        current: current.to_string(),
    })
}

#[tauri::command]
fn open_url(url: String) {
    update::open(&url);
}

#[derive(Clone, Serialize)]
struct InstallUpdate {
    dep: String,
    state: String,
    stage: String,
    pct: f32,
    message: String,
}

fn winget_id(dep: &str) -> Option<&'static str> {
    match dep {
        "ytdlp" => Some("yt-dlp.yt-dlp"),
        "ffmpeg" => Some("Gyan.FFmpeg"),
        _ => None,
    }
}

fn forget_versions(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        let mut cache = state.versions.lock().unwrap();
        cache.yt_path = None;
        cache.ff_path = None;
    }
}

fn say(app: &AppHandle, dep: &str, state: &str, stage: &str, pct: f32, message: &str) {
    let _ = app.emit(
        "install",
        InstallUpdate {
            dep: dep.to_string(),
            state: state.to_string(),
            stage: stage.to_string(),
            pct,
            message: message.to_string(),
        },
    );
}

/// The names the interface switches on; see the `install` listener in app.js.
fn stage_name(stage: Stage) -> &'static str {
    match stage {
        Stage::Starting => "starting",
        Stage::Downloading => "downloading",
        Stage::Installing => "installing",
    }
}

fn run_winget(app: &AppHandle, dep: &str, action: &str) {
    let Some(id) = winget_id(dep) else { return };

    let outcome = winget::run(id, action, |stage, pct| {
        say(app, dep, "running", stage_name(stage), pct, "");
    });

    match outcome {
        Outcome::Done | Outcome::AlreadyCurrent => {
            forget_versions(app);
            let note = match outcome {
                Outcome::AlreadyCurrent => "Already up to date",
                _ if action == "upgrade" => "Updated",
                _ => "",
            };
            say(app, dep, "done", "", 1.0, note);
        }
        Outcome::NotManaged => say(app, dep, "failed", "", 0.0, "not installed through winget"),
        Outcome::Failed(reason) => say(app, dep, "failed", "", 0.0, &reason),
    }
}

#[tauri::command]
fn install_dep(app: AppHandle, dep: String, action: String) {
    thread::spawn(move || run_winget(&app, &dep, &action));
}

#[tauri::command]
fn update_all(app: AppHandle) {
    say(&app, "ytdlp", "running", "starting", 0.0, "");
    say(&app, "ffmpeg", "running", "starting", 0.0, "");
    thread::spawn(move || {
        run_winget(&app, "ytdlp", "upgrade");
        run_winget(&app, "ffmpeg", "upgrade");
    });
}

#[tauri::command]
async fn probe(state: State<'_, AppState>) -> Result<Probe, ()> {
    let yt = find_ytdlp();
    let ff = find_ffmpeg();

    let (yt_version, ff_version) = {
        let mut cache = state.versions.lock().unwrap();
        if cache.yt_path != yt {
            cache.yt_path = yt.clone();
            cache.yt = yt.as_deref().and_then(ytdlp_version);
        }
        if cache.ff_path != ff {
            cache.ff_path = ff.clone();
            cache.ff = ff.as_deref().and_then(ffmpeg_version);
        }
        (cache.yt.clone(), cache.ff.clone())
    };

    Ok(Probe {
        version: env!("CARGO_PKG_VERSION"),
        ytdlp: yt.map(|p| p.to_string_lossy().into_owned()),
        ytdlp_version: yt_version,
        ffmpeg: ff.is_some(),
        ffmpeg_version: ff_version,
        winget: find_exe("winget.exe").is_some(),
        out: state.out.lock().unwrap().to_string_lossy().into_owned(),
    })
}

#[tauri::command]
async fn pick_folder(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let current = state.out.lock().unwrap().clone();
    let picked = rfd::AsyncFileDialog::new()
        .set_directory(&current)
        .pick_folder()
        .await;

    match picked {
        Some(handle) => {
            let path = handle.path().to_path_buf();
            save_out(&path);
            *state.out.lock().unwrap() = path.clone();
            Ok(Some(path.to_string_lossy().into_owned()))
        }
        None => Ok(None),
    }
}

#[tauri::command]
fn open_folder(state: State<'_, AppState>) {
    let dir = state.out.lock().unwrap().clone();
    let _ = std::fs::create_dir_all(&dir);
    let _ = Command::new("explorer").arg(&dir).spawn();
}

#[tauri::command]
fn start(app: AppHandle, state: State<'_, AppState>, url: String, mode: String) -> u64 {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let out = state.out.lock().unwrap().clone();
    let url = url.trim().to_string();

    let push = |app: &AppHandle, update: Update| {
        let _ = app.emit("job", update);
    };

    let refuse = |app: &AppHandle, line: &str| {
        push(
            app,
            Update {
                id,
                title: url.clone(),
                pct: 0.0,
                state: "failed".into(),
                line: line.into(),
            },
        );
        id
    };

    let Some(exe) = find_ytdlp() else {
        return refuse(&app, "yt-dlp not found");
    };
    let Some(mode) = Mode::from_name(&mode) else {
        return refuse(&app, "unknown download mode");
    };

    let ffmpeg = find_ffmpeg();

    push(
        &app,
        Update {
            id,
            title: url.clone(),
            pct: 0.0,
            state: "running".into(),
            line: "Starting".into(),
        },
    );

    thread::spawn(move || {
        let mut title = url.clone();
        let mut pct = 0.0f32;

        let outcome = ytdlp::run(&exe, ffmpeg.as_deref(), mode, &out, &url, |event| {
            if let Some(found) = event.pct {
                pct = found;
            }
            if let Some(found) = event.title {
                title = found;
            }
            let _ = app.emit(
                "job",
                Update {
                    id,
                    title: title.clone(),
                    pct,
                    state: "running".into(),
                    line: event.line,
                },
            );
        });

        let (state, pct, line) = match outcome {
            ytdlp::Outcome::Done => ("done", 1.0, "Done".to_string()),
            ytdlp::Outcome::Failed(reason) => ("failed", 0.0, reason),
        };
        let _ = app.emit(
            "job",
            Update {
                id,
                title,
                pct,
                state: state.into(),
                line,
            },
        );
    });

    id
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            out: Mutex::new(load_out()),
            versions: Mutex::new(Cache::default()),
        })
        .invoke_handler(tauri::generate_handler![
            probe,
            start,
            pick_folder,
            open_folder,
            install_dep,
            update_all,
            check_update,
            open_url
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Snag");
}
