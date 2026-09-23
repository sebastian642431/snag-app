use super::App;
use crate::download::State;
use crate::theme::{rg, sb, ACCENT, BAD, CARD, DIM, LINE, MUTED, OK, TEXT};
use crate::widgets::{pill, progress_bar};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, Sense, Shape, Stroke, StrokeKind, Ui};
use snag_core::text::shorten;
use snag_core::ytdlp::Mode;

impl App {
    pub(super) fn job_list(&mut self, ui: &mut Ui, time: f64) {
        let jobs = self.jobs.lock().unwrap();

        if jobs.is_empty() {
            if self.ytdlp.is_none() || self.ffmpeg.is_none() {
                return;
            }
            ui.add_space(34.0);
            ui.vertical_centered(|ui| {
                let (badge, _) = ui.allocate_exact_size(vec2(54.0, 54.0), Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(badge, CornerRadius::same(17), CARD);
                painter.rect_stroke(
                    badge,
                    CornerRadius::same(17),
                    Stroke::new(1.0, LINE),
                    StrokeKind::Inside,
                );
                let ink = Color32::from_rgb(96, 96, 104);
                let origin = badge.center() + vec2(-1.5, 1.0);
                let at = |dx: f32, dy: f32| origin + vec2(dx, dy);

                let head = |cx: f32, cy: f32| {
                    Shape::Ellipse(egui::epaint::EllipseShape::filled(
                        at(cx, cy),
                        vec2(4.4, 3.2),
                        ink,
                    ))
                };
                painter.add(head(-5.2, 5.4));
                painter.add(head(5.4, 3.4));
                painter.line_segment([at(-1.2, 5.0), at(-1.2, -7.0)], Stroke::new(1.9, ink));
                painter.line_segment([at(9.4, 3.0), at(9.4, -9.0)], Stroke::new(1.9, ink));
                painter.line_segment([at(-1.2, -7.2), at(9.4, -9.2)], Stroke::new(3.2, ink));
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("Nothing downloaded yet")
                        .font(sb(13.5))
                        .color(DIM),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("Paste a link above and press Enter.")
                        .font(rg(12.5))
                        .color(MUTED),
                );
            });
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for job in jobs.iter().rev() {
                    egui::Frame::default()
                        .fill(CARD)
                        .stroke(Stroke::new(1.0, LINE))
                        .corner_radius(CornerRadius::same(14))
                        .inner_margin(egui::Margin::symmetric(15, 13))
                        .shadow(egui::epaint::Shadow {
                            offset: [0, 5],
                            blur: 18,
                            spread: 0,
                            color: Color32::from_black_alpha(70),
                        })
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            let (head, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 20.0),
                                Sense::hover(),
                            );
                            let painter = ui.painter();

                            let (label, color) = match job.state {
                                State::Running => {
                                    (format!("{}%", (job.pct * 100.0).round()), ACCENT)
                                }
                                State::Done => ("Done".to_string(), OK),
                                State::Failed => ("Failed".to_string(), BAD),
                            };
                            pill(painter, pos2(head.right(), head.center().y), &label, color);

                            let room = head.width() - label.len() as f32 * 6.6 - 32.0;
                            let chars = (room / 7.0).max(8.0) as usize;
                            painter.text(
                                pos2(head.left(), head.center().y),
                                Align2::LEFT_CENTER,
                                shorten(&job.title, chars),
                                sb(13.5),
                                TEXT,
                            );

                            ui.add_space(9.0);
                            progress_bar(ui, job, time);
                            ui.add_space(7.0);

                            let tag = match job.mode {
                                Mode::Mp3 => "MP3",
                                Mode::Video => "Video",
                            };
                            ui.label(
                                egui::RichText::new(format!("{tag} · {}", job.line))
                                    .font(rg(11.5))
                                    .color(MUTED),
                            );
                        });
                    ui.add_space(10.0);
                }
            });
    }
}
