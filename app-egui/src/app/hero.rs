use super::App;
use crate::theme::{mix, rg, sb, ACCENT, CARD, CARD_HI, DIM, LINE, MUTED, TEXT};
use crate::widgets::{ghost_button, segmented, solid_button};
use egui::{
    pos2, vec2, Align, Align2, Color32, CornerRadius, Id, Layout, Rect, Sense, Stroke, StrokeKind,
    Ui, UiBuilder,
};
use snag_core::settings::save_out;
use snag_core::text::tail;
use std::process::Command;

impl App {
    pub(super) fn hero(&mut self, ui: &mut Ui) {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("Paste a link")
                .font(sb(25.0))
                .color(TEXT),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("Grab the audio as MP3, or the whole video.")
                .font(rg(13.0))
                .color(MUTED),
        );
        ui.add_space(14.0);

        let (field, _) = ui.allocate_exact_size(vec2(ui.available_width(), 50.0), Sense::hover());
        let focus = ui.memory(|m| m.has_focus(Id::new("url-input")));
        let t = ui
            .ctx()
            .animate_bool_with_time(Id::new("field-focus"), focus, 0.15);

        let painter = ui.painter().clone();
        if t > 0.0 {
            painter.rect_filled(
                field.expand(3.0 * t),
                CornerRadius::same(16),
                Color32::from_rgba_unmultiplied(255, 255, 255, (22.0 * t) as u8),
            );
        }
        painter.rect_filled(field, CornerRadius::same(14), mix(CARD, CARD_HI, t));
        painter.rect_stroke(
            field,
            CornerRadius::same(14),
            Stroke::new(1.0 + 0.3 * t, mix(LINE, ACCENT, t)),
            StrokeKind::Inside,
        );

        let dot = pos2(field.left() + 22.0, field.center().y);
        let dot_color = if self.ytdlp.is_some() {
            mix(MUTED, ACCENT, t)
        } else {
            Color32::from_rgb(74, 81, 99)
        };
        painter.circle_filled(dot, 4.0, dot_color);

        let inner = Rect::from_min_max(
            pos2(field.left() + 38.0, field.top()),
            pos2(field.right() - 14.0, field.bottom()),
        );

        let usable = self.ytdlp.is_some();
        let hint = if usable {
            "https://…"
        } else {
            "Install yt-dlp first"
        };

        if !usable {
            ui.interact(field, Id::new("field-off"), Sense::hover())
                .on_hover_cursor(egui::CursorIcon::NotAllowed);
        }

        let mut submitted = false;
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(inner)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                let edit = egui::TextEdit::singleline(&mut self.url)
                    .id(Id::new("url-input"))
                    .frame(egui::Frame::default())
                    .font(rg(14.0))
                    .hint_text(egui::RichText::new(hint).color(Color32::from_rgb(91, 98, 118)))
                    .desired_width(ui.available_width())
                    .vertical_align(Align::Center);
                let response = ui.add_enabled(usable, edit);
                if !usable {
                    response
                        .clone()
                        .on_hover_cursor(egui::CursorIcon::NotAllowed);
                }
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    submitted = true;
                }
            },
        );

        ui.add_space(9.0);

        let ready = self.ytdlp.is_some() && self.url.trim().starts_with("http");
        ui.horizontal(|ui| {
            segmented(ui, &mut self.mode);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if solid_button(ui, "Download", vec2(112.0, 40.0), ready).clicked() {
                    submitted = true;
                }
            });
        });

        if submitted {
            let ctx = ui.ctx().clone();
            self.start(&ctx);
            ui.memory_mut(|m| m.request_focus(Id::new("url-input")));
        }

        ui.add_space(12.0);

        let (folder, _) = ui.allocate_exact_size(vec2(ui.available_width(), 42.0), Sense::hover());
        let painter = ui.painter().clone();
        painter.rect_filled(folder, CornerRadius::same(12), CARD);
        painter.rect_stroke(
            folder,
            CornerRadius::same(12),
            Stroke::new(1.0, LINE),
            StrokeKind::Inside,
        );

        let glyph = Color32::from_rgb(126, 134, 156);
        let body = Rect::from_center_size(
            pos2(folder.left() + 22.0, folder.center().y + 1.0),
            vec2(14.0, 10.0),
        );
        painter.rect_filled(body, CornerRadius::same(2), glyph);
        painter.rect_filled(
            Rect::from_min_size(pos2(body.left(), body.top() - 3.0), vec2(6.5, 3.5)),
            CornerRadius::same(1),
            glyph,
        );

        let path = self.out.to_string_lossy().to_string();
        painter.text(
            pos2(folder.left() + 38.0, folder.center().y),
            Align2::LEFT_CENTER,
            tail(&path, 32),
            rg(12.0),
            DIM,
        );

        let slot = Rect::from_min_max(pos2(folder.right() - 148.0, folder.top()), folder.max)
            .shrink2(vec2(6.0, 5.0));
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(slot)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                if ghost_button(ui, "Change", true).clicked() {
                    if let Some(picked) = rfd::FileDialog::new()
                        .set_directory(&self.out)
                        .pick_folder()
                    {
                        self.out = picked;
                        save_out(&self.out);
                    }
                }
                if ghost_button(ui, "Open", true).clicked() {
                    let _ = std::fs::create_dir_all(&self.out);
                    let _ = Command::new("explorer").arg(&self.out).spawn();
                }
            },
        );
    }
}
