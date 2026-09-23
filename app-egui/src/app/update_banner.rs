use super::App;
use crate::theme::{rg, sb, CARD_HI, MUTED, TEXT};
use crate::widgets::panel_button;
use egui::{Align, Color32, CornerRadius, Layout, Stroke, Ui};
use snag_core::update;

impl App {
    pub(super) fn update_banner(&mut self, ui: &mut Ui) {
        if self.later {
            return;
        }
        let Some(release) = self.update.lock().unwrap().clone() else {
            return;
        };

        ui.add_space(13.0);
        egui::Frame::default()
            .fill(CARD_HI)
            .stroke(Stroke::new(1.0, Color32::from_rgb(58, 58, 64)))
            .corner_radius(CornerRadius::same(13))
            .inner_margin(egui::Margin::symmetric(15, 12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Snag {} is available", release.tag))
                                .font(sb(12.5))
                                .color(TEXT),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "You are on {}",
                                env!("CARGO_PKG_VERSION")
                            ))
                            .font(rg(11.0))
                            .color(MUTED),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if panel_button(ui, "Get it", true, true).clicked() {
                            update::open(&release.page);
                        }
                        ui.add_space(6.0);
                        if panel_button(ui, "Later", false, true).clicked() {
                            self.later = true;
                        }
                    });
                });
            });
    }
}
