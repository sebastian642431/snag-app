use snag_core::text::shorten;
use snag_core::ytdlp::{self, Mode};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(PartialEq, Clone, Copy)]
pub enum State {
    Running,
    Done,
    Failed,
}

pub struct Job {
    pub title: String,
    pub mode: Mode,
    pub pct: f32,
    pub state: State,
    pub line: String,
}

pub struct Request {
    pub exe: PathBuf,
    pub ffmpeg: Option<PathBuf>,
    pub mode: Mode,
    pub out: PathBuf,
    pub url: String,
}

/// Runs one download on its own thread, keeping `jobs[index]` current as it goes.
pub fn run(jobs: Arc<Mutex<Vec<Job>>>, index: usize, request: Request, ctx: egui::Context) {
    thread::spawn(move || {
        let Request {
            exe,
            ffmpeg,
            mode,
            out,
            url,
        } = request;

        let outcome = ytdlp::run(&exe, ffmpeg.as_deref(), mode, &out, &url, |event| {
            let mut jobs = jobs.lock().unwrap();
            let job = &mut jobs[index];
            job.line = shorten(&event.line, 78);
            if let Some(pct) = event.pct {
                job.pct = pct;
            }
            if let Some(title) = event.title {
                job.title = shorten(&title, 64);
            }
            drop(jobs);
            ctx.request_repaint();
        });

        let mut jobs = jobs.lock().unwrap();
        let job = &mut jobs[index];
        match outcome {
            ytdlp::Outcome::Done => {
                job.state = State::Done;
                job.pct = 1.0;
                job.line = "Done".into();
            }
            ytdlp::Outcome::Failed(reason) => {
                job.state = State::Failed;
                job.line = shorten(&reason, 78);
            }
        }
        ctx.request_repaint();
    });
}
