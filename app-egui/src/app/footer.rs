use super::App;
use crate::install::Dep;
use crate::theme::{rg, BAD, MUTED, OK};
use crate::widgets::toggle_button;
use egui::{pos2, vec2, Align, Align2, Color32, Layout, Rect, Sense, Stroke, Ui, UiBuilder};

impl App {
    pub(super) fn footer(&mut self, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::hover());
        let painter = ui.painter().clone();
        painter.line_segment(
            [
                pos2(rect.left(), rect.top()),
                pos2(rect.right(), rect.top()),
            ],
            Stroke::new(1.0, Color32::from_rgb(24, 24, 28)),
        );

        let mut x = rect.left() + 22.0;
        let mark = |ok: bool, name: &str, version: String| -> String {
            if !ok {
                format!("{name} missing")
            } else if version.is_empty() {
                format!("{name} ready")
            } else {
                format!("{name} {version}")
            }
        };
        let items = [
            (
                self.ytdlp.is_some(),
                mark(self.ytdlp.is_some(), "yt-dlp", self.version_of(Dep::Yt)),
            ),
            (
                self.ffmpeg.is_some(),
                mark(self.ffmpeg.is_some(), "ffmpeg", self.version_of(Dep::Ff)),
            ),
        ];

        for (ok, label) in items {
            let color = if ok { OK } else { BAD };
            painter.circle_filled(pos2(x, rect.center().y), 6.0, color.gamma_multiply(0.22));
            painter.circle_filled(pos2(x, rect.center().y), 3.5, color);
            let galley = painter.text(
                pos2(x + 11.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                rg(11.5),
                if ok {
                    MUTED
                } else {
                    Color32::from_rgb(201, 139, 139)
                },
            );
            x = galley.right() + 20.0;
        }

        if self.ytdlp.is_some() && self.ffmpeg.is_some() {
            let slot = Rect::from_min_max(pos2(rect.right() - 160.0, rect.top()), rect.max)
                .shrink2(vec2(16.0, 5.0));
            ui.scope_builder(
                UiBuilder::new()
                    .max_rect(slot)
                    .layout(Layout::right_to_left(Align::Center)),
                |ui| {
                    if toggle_button(ui, "Tools", self.show_tools, true).clicked() {
                        self.show_tools = !self.show_tools;
                    }
                },
            );
        }
    }
}
