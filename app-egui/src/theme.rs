use egui::{Color32, FontFamily, FontId};
use std::sync::Arc;

pub const BG: Color32 = Color32::from_rgb(8, 8, 10);
pub const CARD: Color32 = Color32::from_rgb(18, 18, 21);
pub const CARD_HI: Color32 = Color32::from_rgb(26, 26, 30);
pub const LINE: Color32 = Color32::from_rgb(39, 39, 44);
pub const TEXT: Color32 = Color32::from_rgb(244, 244, 245);
pub const DIM: Color32 = Color32::from_rgb(161, 161, 170);
pub const MUTED: Color32 = Color32::from_rgb(113, 113, 122);
pub const ACCENT: Color32 = Color32::from_rgb(250, 250, 250);
pub const ACCENT_DARK: Color32 = Color32::from_rgb(203, 203, 208);
pub const ON_ACCENT: Color32 = Color32::from_rgb(10, 10, 12);
pub const OK: Color32 = Color32::from_rgb(74, 222, 128);
pub const BAD: Color32 = Color32::from_rgb(248, 113, 113);
pub const WARN: Color32 = Color32::from_rgb(234, 179, 8);

pub fn sb(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("sb".into()))
}

pub fn rg(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let root = std::env::var("WINDIR").unwrap_or_else(|_| "C:/Windows".into());

    if let Ok(data) = std::fs::read(format!("{root}/Fonts/segoeui.ttf")) {
        fonts
            .font_data
            .insert("ui".into(), Arc::new(egui::FontData::from_owned(data)));
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "ui".into());
    }

    let heavy = std::fs::read(format!("{root}/Fonts/seguisb.ttf"))
        .or_else(|_| std::fs::read(format!("{root}/Fonts/segoeuib.ttf")));

    if let Ok(data) = heavy {
        fonts
            .font_data
            .insert("ui-sb".into(), Arc::new(egui::FontData::from_owned(data)));
        fonts.families.insert(
            FontFamily::Name("sb".into()),
            vec!["ui-sb".into(), "ui".into()],
        );
    } else {
        let fallback = fonts
            .families
            .get(&FontFamily::Proportional)
            .cloned()
            .unwrap_or_default();
        fonts
            .families
            .insert(FontFamily::Name("sb".into()), fallback);
    }

    ctx.set_fonts(fonts);
}
