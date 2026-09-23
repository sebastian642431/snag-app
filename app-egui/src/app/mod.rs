mod footer;
mod hero;
mod jobs;
mod title_bar;
mod tools_panel;
mod update_banner;

use crate::download::{self, Job, Request, State};
use crate::install::{Dep, Tool};
use crate::paint::glow;
use crate::theme::{install_fonts, ACCENT, BG, TEXT};
use egui::{pos2, vec2, Color32, CornerRadius, Id, Stroke, Ui};
use snag_core::settings::load_out;
use snag_core::text::shorten;
use snag_core::tools::{find_exe, find_ffmpeg, find_ytdlp};
use snag_core::update::{self, Release};
use snag_core::version;
use snag_core::ytdlp::Mode;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct App {
    url: String,
    mode: Mode,
    out: PathBuf,
    ytdlp: Option<PathBuf>,
    ffmpeg: Option<PathBuf>,
    jobs: Arc<Mutex<Vec<Job>>>,
    yt: Tool,
    ff: Tool,
    winget: bool,
    was_busy: bool,
    last_probe: f64,
    show_tools: bool,
    update: Arc<Mutex<Option<Release>>>,
    /// "Later" on the banner. Not persisted on purpose: a newer release is
    /// never hidden for good, only until the next launch.
    later: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
        cc.egui_ctx.all_styles_mut(|style| {
            style.spacing.item_spacing = vec2(9.0, 9.0);
            style.visuals.panel_fill = BG;
            style.visuals.window_fill = BG;
            style.visuals.override_text_color = Some(TEXT);
            style.visuals.selection.bg_fill = Color32::from_rgb(62, 62, 70);
            style.visuals.text_cursor.stroke = Stroke::new(1.6, ACCENT);
        });

        let found: Arc<Mutex<Option<Release>>> = Arc::new(Mutex::new(None));
        {
            let found = Arc::clone(&found);
            let ctx = cc.egui_ctx.clone();
            thread::spawn(move || {
                let Some(release) = update::latest() else {
                    return;
                };
                if !version::newer(&release.tag, env!("CARGO_PKG_VERSION")) {
                    return;
                }
                *found.lock().unwrap() = Some(release);
                ctx.request_repaint();
            });
        }

        Self {
            url: String::new(),
            mode: Mode::Mp3,
            out: load_out(),
            ytdlp: find_ytdlp(),
            ffmpeg: find_ffmpeg(),
            jobs: Arc::new(Mutex::new(Vec::new())),
            yt: Tool::new(),
            ff: Tool::new(),
            winget: find_exe("winget.exe").is_some(),
            was_busy: false,
            last_probe: 0.0,
            show_tools: false,
            update: found,
            later: false,
        }
    }

    fn start(&mut self, ctx: &egui::Context) {
        let url = self.url.trim().to_string();
        if !url.starts_with("http") {
            return;
        }
        let Some(exe) = self.ytdlp.clone() else {
            return;
        };

        let index = {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.push(Job {
                title: shorten(&url, 64),
                mode: self.mode,
                pct: 0.0,
                state: State::Running,
                line: "Starting".into(),
            });
            jobs.len() - 1
        };

        self.url.clear();

        let request = Request {
            exe,
            ffmpeg: self.ffmpeg.clone(),
            mode: self.mode,
            out: self.out.clone(),
            url,
        };
        download::run(Arc::clone(&self.jobs), index, request, ctx.clone());
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.039, 0.043, 0.063, 1.0]
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let time = ctx.input(|i| i.time);

        let busy = {
            let jobs = self.jobs.lock().unwrap();
            jobs.iter().any(|j| j.state == State::Running)
        };
        let installing = self.yt.busy() || self.ff.busy();
        let settled = self.was_busy && !installing;
        self.was_busy = installing;

        if !installing && (settled || time - self.last_probe > 2.5) {
            self.last_probe = time;
            self.ytdlp = find_ytdlp();
            self.ffmpeg = find_ffmpeg();
        }

        self.sync_version(Dep::Yt, &ctx);
        self.sync_version(Dep::Ff, &ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(2500));

        if busy || installing {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }

        let screen = ui.max_rect();
        let painter = ui.painter().clone();
        painter.rect_filled(screen, CornerRadius::same(0), BG);
        glow(
            &painter,
            pos2(screen.left() + 60.0, screen.top() - 40.0),
            420.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 14),
        );
        glow(
            &painter,
            pos2(screen.right() - 30.0, screen.top() + 40.0),
            360.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 9),
        );

        self.title_bar(ui);

        egui::Panel::bottom(Id::new("pie"))
            .show_separator_line(false)
            .show(ui, |ui| {
                self.footer(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(22, 0)))
            .show(ui, |ui| {
                self.hero(ui);
                self.update_banner(ui);
                self.tools_panel(ui);
                ui.add_space(18.0);
                self.job_list(ui, time);
            });
    }
}
