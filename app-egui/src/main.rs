#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod download;
mod install;
mod paint;
mod theme;
mod widgets;

use app::App;

fn app_icon() -> egui::IconData {
    const SIDE: u32 = 256;
    const PIXELS: &[u8] = include_bytes!("icon.rgba");
    egui::IconData {
        rgba: PIXELS.to_vec(),
        width: SIDE,
        height: SIDE,
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 700.0])
            .with_min_inner_size([460.0, 540.0])
            .with_decorations(false)
            .with_resizable(true)
            .with_icon(app_icon())
            .with_title("Snag"),
        ..Default::default()
    };
    eframe::run_native("Snag", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
