use crate::download::{Job, State};
use crate::install::Progress;
use crate::paint::grad;
use crate::theme::{
    mix, rg, sb, ACCENT, ACCENT_DARK, BAD, CARD, CARD_HI, DIM, LINE, MUTED, OK, ON_ACCENT, TEXT,
    WARN,
};
use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind,
    Ui, Vec2,
};
use snag_core::winget::Stage;
use snag_core::ytdlp::Mode;

pub fn icon_button(
    ui: &mut Ui,
    id: &str,
    danger: bool,
    draw: impl Fn(&egui::Painter, Rect, Color32),
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(44.0, 40.0), Sense::click());
    let t = ui
        .ctx()
        .animate_bool_with_time(Id::new(id), response.hovered(), 0.12);
    let painter = ui.painter();

    if t > 0.0 {
        let bg = if danger {
            Color32::from_rgb(229, 72, 77)
        } else {
            Color32::from_rgb(38, 42, 55)
        };
        painter.rect_filled(rect, CornerRadius::same(0), bg.gamma_multiply(t));
    }

    let color = if danger && t > 0.5 {
        Color32::WHITE
    } else {
        mix(MUTED, TEXT, t)
    };
    draw(painter, rect, color);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn solid_button(ui: &mut Ui, label: &str, size: Vec2, enabled: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        size,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hovered = enabled && response.hovered();
    let t = ui.ctx().animate_bool_with_time(response.id, hovered, 0.13);
    let painter = ui.painter();

    if enabled {
        let halo = rect.expand(3.0 + 4.0 * t);
        painter.rect_filled(
            halo,
            CornerRadius::same(14),
            Color32::from_rgba_unmultiplied(255, 255, 255, (12.0 + 16.0 * t) as u8),
        );
        painter.rect_filled(
            rect,
            CornerRadius::same(11),
            mix(ACCENT, Color32::WHITE, 0.12 * t),
        );
        painter.rect_stroke(
            rect,
            CornerRadius::same(11),
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 22)),
            StrokeKind::Inside,
        );
    } else {
        painter.rect_filled(rect, CornerRadius::same(11), CARD);
    }

    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        sb(13.5),
        if enabled { ON_ACCENT } else { MUTED },
    );

    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response.on_hover_cursor(egui::CursorIcon::NotAllowed)
    }
}

pub fn ghost_button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    let width = label.chars().count() as f32 * 7.0 + 22.0;
    ghost_sized(ui, label, vec2(width, 28.0), enabled)
}

pub fn toggle_button(ui: &mut Ui, label: &str, expanded: bool, enabled: bool) -> Response {
    let width = label.chars().count() as f32 * 7.0 + 40.0;
    let (rect, response) = ui.allocate_exact_size(
        vec2(width, 28.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let t = ui
        .ctx()
        .animate_bool_with_time(response.id, enabled && response.hovered(), 0.12);
    let painter = ui.painter();

    painter.rect_filled(rect, CornerRadius::same(8), mix(CARD, CARD_HI, t));
    painter.rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0, mix(LINE, ACCENT.gamma_multiply(0.5), t)),
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        rg(12.0),
        mix(DIM, TEXT, t),
    );

    let tip = pos2(rect.right() - 14.0, rect.center().y);
    let dir = if expanded { -1.0 } else { 1.0 };
    let chevron = Stroke::new(1.7, mix(MUTED, TEXT, t));
    painter.line_segment(
        [tip + vec2(-3.5, -1.7 * dir), tip + vec2(0.0, 1.9 * dir)],
        chevron,
    );
    painter.line_segment(
        [tip + vec2(3.5, -1.7 * dir), tip + vec2(0.0, 1.9 * dir)],
        chevron,
    );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn panel_button(ui: &mut Ui, label: &str, primary: bool, enabled: bool) -> Response {
    let size = vec2(104.0, 30.0);
    if primary {
        solid_button(ui, label, size, enabled)
    } else {
        ghost_sized(ui, label, size, enabled)
    }
}

pub fn ghost_sized(ui: &mut Ui, label: &str, size: Vec2, enabled: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        size,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hovered = enabled && response.hovered();
    let t = ui.ctx().animate_bool_with_time(response.id, hovered, 0.12);
    let painter = ui.painter();

    painter.rect_filled(rect, CornerRadius::same(8), mix(CARD, CARD_HI, t));
    painter.rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0, mix(LINE, ACCENT.gamma_multiply(0.5), t)),
        StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        rg(12.0),
        if enabled { mix(DIM, TEXT, t) } else { MUTED },
    );

    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response.on_hover_cursor(egui::CursorIcon::NotAllowed)
    }
}

pub fn segmented(ui: &mut Ui, mode: &mut Mode) {
    let (rect, _) = ui.allocate_exact_size(vec2(148.0, 40.0), Sense::hover());
    let painter = ui.painter().clone();
    painter.rect_filled(rect, CornerRadius::same(12), CARD);
    painter.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(1.0, LINE),
        StrokeKind::Inside,
    );

    let inner = rect.shrink(4.0);
    let half = inner.width() / 2.0;
    let target = if *mode == Mode::Mp3 { 0.0 } else { 1.0 };
    let slide = ui
        .ctx()
        .animate_value_with_time(Id::new("seg"), target, 0.16);
    let knob = Rect::from_min_size(
        pos2(inner.left() + half * slide, inner.top()),
        vec2(half, inner.height()),
    );

    painter.rect_filled(
        knob.expand(2.0),
        CornerRadius::same(11),
        Color32::from_rgba_unmultiplied(255, 255, 255, 18),
    );
    painter.rect_filled(knob, CornerRadius::same(9), ACCENT);

    for (index, (label, value)) in [("MP3", Mode::Mp3), ("Video", Mode::Video)]
        .iter()
        .enumerate()
    {
        let cell = Rect::from_min_size(
            pos2(inner.left() + half * index as f32, inner.top()),
            vec2(half, inner.height()),
        );
        let response = ui
            .interact(cell, Id::new(("seg", index)), Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.clicked() {
            *mode = *value;
        }
        let active = *mode == *value;
        painter.text(
            cell.center(),
            Align2::CENTER_CENTER,
            label,
            sb(13.0),
            if active { ON_ACCENT } else { MUTED },
        );
    }
}

pub fn progress_bar(ui: &mut Ui, job: &Job, time: f64) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 7.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(40, 40, 45));

    let (from, to) = match job.state {
        State::Running => (ACCENT_DARK, ACCENT),
        State::Done => (Color32::from_rgb(47, 168, 106), OK),
        State::Failed => (BAD, BAD),
    };

    let pct = if job.state == State::Failed {
        1.0
    } else {
        job.pct.clamp(0.0, 1.0)
    };
    let width = rect.width() * pct;
    if width < 1.0 {
        return;
    }

    let fill = Rect::from_min_size(rect.min, vec2(width.max(8.0), rect.height()));
    let clip = painter.with_clip_rect(fill);
    grad(&clip, fill, from, to, true);
    clip.circle_filled(pos2(fill.left() + 3.5, fill.center().y), 3.5, from);
    clip.circle_filled(pos2(fill.right() - 3.5, fill.center().y), 3.5, to);

    if job.state == State::Running {
        let span = fill.width() + 70.0;
        let x = fill.left() - 70.0 + (time * 0.55).fract() as f32 * span;
        let shine = Rect::from_min_size(pos2(x, fill.top()), vec2(46.0, fill.height()));
        grad(
            &clip,
            shine,
            Color32::TRANSPARENT,
            Color32::from_rgba_unmultiplied(255, 255, 255, 105),
            true,
        );
    }
}

pub fn pill(painter: &egui::Painter, anchor: Pos2, label: &str, color: Color32) {
    let width = label.len() as f32 * 6.6 + 18.0;
    let rect = Rect::from_min_size(pos2(anchor.x - width, anchor.y - 10.0), vec2(width, 20.0));
    painter.rect_filled(rect, CornerRadius::same(10), color.gamma_multiply(0.16));
    painter.text(rect.center(), Align2::CENTER_CENTER, label, sb(11.0), color);
}

pub fn install_bar(ui: &mut Ui, progress: &Progress, time: f64) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 5.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(3), Color32::from_rgb(45, 45, 50));

    if progress.stage == Stage::Downloading && progress.pct > 0.0 {
        let width = (rect.width() * progress.pct).max(5.0);
        painter.rect_filled(
            Rect::from_min_size(rect.min, vec2(width, rect.height())),
            CornerRadius::same(3),
            WARN,
        );
        return;
    }

    let clip = painter.with_clip_rect(rect);
    let span = rect.width() + 100.0;
    let x = rect.left() - 100.0 + (time * 0.55).fract() as f32 * span;
    clip.rect_filled(
        Rect::from_min_size(pos2(x, rect.top()), vec2(90.0, rect.height())),
        CornerRadius::same(3),
        WARN,
    );
}
