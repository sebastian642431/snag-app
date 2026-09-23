use snag_core::winget::{self, Outcome, Stage};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, PartialEq)]
pub enum Install {
    Idle,
    Running(Progress),
    Done(String),
    Failed(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dep {
    Yt,
    Ff,
}

impl Dep {
    pub fn id(self) -> &'static str {
        match self {
            Dep::Yt => "yt-dlp.yt-dlp",
            Dep::Ff => "Gyan.FFmpeg",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Dep::Yt => "yt-dlp",
            Dep::Ff => "ffmpeg",
        }
    }

    pub fn what(self) -> &'static str {
        match self {
            Dep::Yt => "does the downloading",
            Dep::Ff => "extracts MP3, merges video",
        }
    }
}

pub struct Tool {
    pub job: Arc<Mutex<Install>>,
    pub version: Arc<Mutex<Option<String>>>,
    pub reading: Arc<AtomicBool>,
    pub seen: Option<PathBuf>,
}

impl Tool {
    pub fn new() -> Self {
        Self {
            job: Arc::new(Mutex::new(Install::Idle)),
            version: Arc::new(Mutex::new(None)),
            reading: Arc::new(AtomicBool::new(false)),
            seen: None,
        }
    }

    pub fn busy(&self) -> bool {
        matches!(*self.job.lock().unwrap(), Install::Running(_))
    }
}

#[derive(Clone, PartialEq)]
pub struct Progress {
    pub stage: Stage,
    pub pct: f32,
}

pub fn run(
    slot: &Arc<Mutex<Install>>,
    version: &Arc<Mutex<Option<String>>>,
    winget_id: &str,
    action: &str,
    ctx: &egui::Context,
) {
    let outcome = winget::run(winget_id, action, |stage, pct| {
        *slot.lock().unwrap() = Install::Running(Progress { stage, pct });
        ctx.request_repaint();
    });

    *slot.lock().unwrap() = match outcome {
        Outcome::Done | Outcome::AlreadyCurrent => {
            // The binary on disk changed, so the cached version is stale.
            *version.lock().unwrap() = None;
            Install::Done(match outcome {
                Outcome::AlreadyCurrent => "Already up to date".into(),
                _ if action == "upgrade" => "Updated".into(),
                _ => String::new(),
            })
        }
        Outcome::NotManaged => Install::Failed("not installed through winget".into()),
        Outcome::Failed(reason) => Install::Failed(reason),
    };
    ctx.request_repaint();
}

pub fn start(
    slot: Arc<Mutex<Install>>,
    version: Arc<Mutex<Option<String>>>,
    winget_id: &'static str,
    action: &'static str,
    ctx: egui::Context,
) {
    *slot.lock().unwrap() = Install::Running(Progress {
        stage: Stage::Starting,
        pct: 0.0,
    });
    ctx.request_repaint();
    thread::spawn(move || run(&slot, &version, winget_id, action, &ctx));
}
