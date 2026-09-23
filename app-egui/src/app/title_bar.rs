use super::App;
use crate::theme::sb;
use crate::widgets::icon_button;
use egui::{
    pos2, vec2, Align, Align2, Color32, CornerRadius, Id, Layout, Rect, Sense, Stroke, StrokeKind,
    Ui, UiBuilder, ViewportCommand,
};

impl App {
    pub(super) fn title_bar(&mut self, ui: &mut Ui) {
        let height = 40.0;
        let full = Rect::from_min_size(ui.max_rect().min, vec2(ui.available_width(), height));

        let drag = ui.interact(full, Id::new("titlebar"), Sense::click_and_drag());
        if drag.drag_started() {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }

        let painter = ui.painter();
        let ink = Color32::from_rgb(222, 222, 228);
        let mark = pos2(full.left() + 21.0, full.center().y - 0.5);
        let line = Stroke::new(1.9, ink);
        painter.line_segment([mark + vec2(0.0, -6.2), mark + vec2(0.0, 1.4)], line);
        painter.line_segment([mark + vec2(-3.7, -2.1), mark + vec2(0.0, 1.8)], line);
        painter.line_segment([mark + vec2(3.7, -2.1), mark + vec2(0.0, 1.8)], line);
        painter.line_segment([mark + vec2(-5.2, 5.4), mark + vec2(5.2, 5.4)], line);
        painter.text(
            pos2(full.left() + 35.0, full.center().y),
            Align2::LEFT_CENTER,
            concat!("Snag · egui v", env!("CARGO_PKG_VERSION")),
            sb(12.5),
            Color32::from_rgb(198, 203, 219),
        );

        let buttons =
            Rect::from_min_size(pos2(full.right() - 132.0, full.top()), vec2(132.0, height));
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(buttons)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                let close = icon_button(ui, "win-close", true, |p, r, c| {
                    let m = r.center();
                    let s = 4.5;
                    p.line_segment([m - vec2(s, s), m + vec2(s, s)], Stroke::new(1.3, c));
                    p.line_segment([m + vec2(s, -s), m - vec2(s, -s)], Stroke::new(1.3, c));
                });
                if close.clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }

                let max = icon_button(ui, "win-max", false, |p, r, c| {
                    let m = Rect::from_center_size(r.center(), vec2(9.0, 9.0));
                    p.rect_stroke(
                        m,
                        CornerRadius::same(2),
                        Stroke::new(1.3, c),
                        StrokeKind::Inside,
                    );
                });
                if max.clicked() {
                    let is_max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                    ui.ctx()
                        .send_viewport_cmd(ViewportCommand::Maximized(!is_max));
                }

                let min = icon_button(ui, "win-min", false, |p, r, c| {
                    let m = r.center();
                    p.line_segment(
                        [m - vec2(5.0, 0.0), m + vec2(5.0, 0.0)],
                        Stroke::new(1.3, c),
                    );
                });
                if min.clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
                }
            },
        );

        ui.advance_cursor_after_rect(full);
    }
}
