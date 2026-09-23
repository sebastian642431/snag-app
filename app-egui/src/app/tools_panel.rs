use super::App;
use crate::install::{self, Dep, Install, Progress, Tool};
use crate::theme::{rg, sb, BAD, CARD, CARD_HI, DIM, LINE, MUTED, OK, TEXT, WARN};
use crate::widgets::{install_bar, panel_button};
use egui::{Align, Color32, CornerRadius, Layout, Stroke, Ui};
use snag_core::tools::{ffmpeg_version, ytdlp_version};
use snag_core::update;
use snag_core::winget::Stage;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

impl App {
    fn tool(&self, dep: Dep) -> &Tool {
        match dep {
            Dep::Yt => &self.yt,
            Dep::Ff => &self.ff,
        }
    }

    fn installed(&self, dep: Dep) -> bool {
        match dep {
            Dep::Yt => self.ytdlp.is_some(),
            Dep::Ff => self.ffmpeg.is_some(),
        }
    }

    pub(super) fn version_of(&self, dep: Dep) -> String {
        self.tool(dep)
            .version
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_default()
    }

    fn act(&self, dep: Dep, action: &'static str, ctx: &egui::Context) {
        let tool = self.tool(dep);
        install::start(
            Arc::clone(&tool.job),
            Arc::clone(&tool.version),
            dep.id(),
            action,
            ctx.clone(),
        );
    }

    fn update_all(&self, ctx: &egui::Context) {
        let yt = (Arc::clone(&self.yt.job), Arc::clone(&self.yt.version));
        let ff = (Arc::clone(&self.ff.job), Arc::clone(&self.ff.version));
        let waiting = Progress {
            stage: Stage::Starting,
            pct: 0.0,
        };
        *yt.0.lock().unwrap() = Install::Running(waiting.clone());
        *ff.0.lock().unwrap() = Install::Running(waiting);
        let ctx = ctx.clone();
        thread::spawn(move || {
            install::run(&yt.0, &yt.1, Dep::Yt.id(), "upgrade", &ctx);
            install::run(&ff.0, &ff.1, Dep::Ff.id(), "upgrade", &ctx);
        });
    }

    pub(super) fn sync_version(&mut self, dep: Dep, ctx: &egui::Context) {
        let path = match dep {
            Dep::Yt => self.ytdlp.clone(),
            Dep::Ff => self.ffmpeg.clone(),
        };
        let tool = match dep {
            Dep::Yt => &mut self.yt,
            Dep::Ff => &mut self.ff,
        };

        if tool.seen != path {
            tool.seen = path.clone();
            *tool.version.lock().unwrap() = None;
        }

        let Some(path) = path else { return };
        if tool.version.lock().unwrap().is_some() || tool.reading.swap(true, Ordering::SeqCst) {
            return;
        }

        let slot = Arc::clone(&tool.version);
        let flag = Arc::clone(&tool.reading);
        let ctx = ctx.clone();
        thread::spawn(move || {
            let found = match dep {
                Dep::Yt => ytdlp_version(&path),
                Dep::Ff => ffmpeg_version(&path),
            };
            *slot.lock().unwrap() = Some(found.unwrap_or_default());
            flag.store(false, Ordering::SeqCst);
            ctx.request_repaint();
        });
    }

    fn tool_row(&self, ui: &mut Ui, dep: Dep) {
        let status = self.tool(dep).job.lock().unwrap().clone();
        let installed = self.installed(dep);
        let version = self.version_of(dep);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(dep.name()).font(sb(12.5)).color(TEXT));
                let (sub, color) = if !installed {
                    (dep.what().to_string(), MUTED)
                } else if version.is_empty() {
                    ("installed".to_string(), MUTED)
                } else {
                    (version, DIM)
                };
                ui.label(egui::RichText::new(sub).font(rg(11.0)).color(color));
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| match &status {
                Install::Running(progress) => {
                    let label = match progress.stage {
                        Stage::Starting => "Starting…".to_string(),
                        Stage::Downloading => {
                            format!("Downloading {}%", (progress.pct * 100.0).round())
                        }
                        Stage::Installing => "Installing…".to_string(),
                    };
                    ui.label(egui::RichText::new(label).font(rg(12.0)).color(WARN));
                }
                _ => {
                    let (label, action) = match (&status, installed) {
                        (Install::Failed(_), true) => ("Retry", "upgrade"),
                        (Install::Failed(_), false) => ("Retry", "install"),
                        (_, true) => ("Update", "upgrade"),
                        (_, false) => ("Install", "install"),
                    };
                    if panel_button(ui, label, !installed, self.winget).clicked() {
                        self.act(dep, action, ui.ctx());
                    }
                    ui.add_space(6.0);
                    match &status {
                        Install::Failed(message) => {
                            ui.label(egui::RichText::new(message).font(rg(10.5)).color(BAD));
                        }
                        Install::Done(note) if !note.is_empty() => {
                            ui.label(egui::RichText::new(note).font(rg(11.0)).color(OK));
                        }
                        _ => {}
                    }
                }
            });
        });

        if let Install::Running(progress) = &status {
            ui.add_space(7.0);
            let time = ui.input(|i| i.time);
            install_bar(ui, progress, time);
        }
    }

    /// The app itself, so a newer release stays reachable after "Later".
    fn snag_row(&self, ui: &mut Ui) {
        let release = self.update.lock().unwrap().clone();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Snag").font(sb(12.5)).color(TEXT));
                ui.label(
                    egui::RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                        .font(rg(11.0))
                        .color(DIM),
                );
            });
            if let Some(release) = release {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if panel_button(ui, "Get it", true, true).clicked() {
                        update::open(&release.page);
                    }
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(format!("{} available", release.tag))
                            .font(rg(11.0))
                            .color(OK),
                    );
                });
            }
        });
    }

    pub(super) fn tools_panel(&mut self, ui: &mut Ui) {
        let missing = self.ytdlp.is_none() || self.ffmpeg.is_none();
        let busy = self.yt.busy() || self.ff.busy();

        if !missing && !self.show_tools {
            return;
        }

        let (fill, line) = if missing {
            (CARD_HI, Color32::from_rgb(58, 58, 64))
        } else {
            (CARD, LINE)
        };

        ui.add_space(13.0);
        egui::Frame::default()
            .fill(fill)
            .stroke(Stroke::new(1.0, line))
            .corner_radius(CornerRadius::same(13))
            .inner_margin(egui::Margin::symmetric(15, 13))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.horizontal(|ui| {
                    let (title, color) = if missing {
                        ("Missing tools", WARN)
                    } else {
                        ("Tools", TEXT)
                    };
                    ui.label(egui::RichText::new(title).font(sb(13.5)).color(color));
                    if !missing {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if panel_button(ui, "Update all", false, self.winget && !busy).clicked()
                            {
                                self.update_all(ui.ctx());
                            }
                        });
                    }
                });

                if missing {
                    ui.add_space(1.0);
                    ui.label(
                        egui::RichText::new(
                            "Snag does not download on its own. It drives these two tools.",
                        )
                        .font(rg(11.5))
                        .color(MUTED),
                    );
                }

                ui.add_space(12.0);
                self.tool_row(ui, Dep::Yt);
                ui.add_space(10.0);
                self.tool_row(ui, Dep::Ff);
                ui.add_space(10.0);
                self.snag_row(ui);

                if missing || !self.winget {
                    ui.add_space(10.0);
                    let (note, color) = if self.winget {
                        (
                            "Installed through winget. No admin rights needed.",
                            Color32::from_rgb(110, 116, 133),
                        )
                    } else {
                        ("winget is not available here. Install them manually.", BAD)
                    };
                    ui.label(egui::RichText::new(note).font(rg(10.5)).color(color));
                }
            });
    }
}
