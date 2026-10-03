//! Custom MCR-style widgets: rocker toggle switch, section headers, big value cards.

use crate::ui::theme;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Vec2};

/// Rocker switch with a state lamp — port of the Java `ToggleSwitch` (78×46).
/// Layout top-to-bottom: name strip, track, ON/OFF caption — no overlap.
pub fn toggle_switch(ui: &mut egui::Ui, name: &str, on: &mut bool) -> Response {
    let size = Vec2::new(78.0, 50.0);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let on_c = *on;
    let painter = ui.painter_at(rect);
    let r = rect.min;
    // name
    painter.text(
        Pos2::new(r.x, r.y + 1.0),
        egui::Align2::LEFT_TOP,
        name,
        egui::FontId::proportional(10.0),
        if on_c { theme::TXT } else { theme::DIM },
    );
    // track
    let tw = 46.0;
    let th = 15.0;
    let ty = r.y + 19.0;
    let track = Rect::from_min_size(Pos2::new(r.x, ty), Vec2::new(tw, th));
    painter.rect_filled(track, 8.0, Color32::from_rgb(0x14, 0x1d, 0x26));
    painter.rect_stroke(track, 8.0, Stroke::new(1.0, theme::LINE), egui::StrokeKind::Inside);
    // handle
    let kx = if on_c { r.x + tw - 15.0 } else { r.x + 1.0 };
    let knob = Rect::from_min_size(Pos2::new(kx, ty - 3.0), Vec2::new(14.0, th + 6.0));
    painter.rect_filled(
        knob,
        7.0,
        if on_c {
            Color32::from_rgb(0x2f, 0x6f, 0xbf)
        } else {
            Color32::from_rgb(0x59, 0x61, 0x6b)
        },
    );
    painter.rect_stroke(
        knob,
        7.0,
        Stroke::new(1.0, Color32::from_rgb(0x9f, 0xb4, 0xc6)),
        egui::StrokeKind::Inside,
    );
    // ON/OFF caption
    painter.text(
        Pos2::new(r.x, ty + th + 8.0),
        egui::Align2::LEFT_CENTER,
        if on_c { "ВКЛ" } else { "ОТКЛ" },
        egui::FontId::proportional(8.0),
        if on_c { theme::GREEN } else { theme::DIM },
    );
    // state lamp
    let lc = Pos2::new(r.x + size.x - 8.5, ty + 4.5);
    painter.circle_filled(lc, 5.5, if on_c { theme::GREEN } else { Color32::from_rgb(0x4a, 0x2f, 0x2f) });
    painter.circle_stroke(
        lc,
        5.5,
        Stroke::new(1.0, if on_c { Color32::from_rgb(0x7d, 0xff, 0xb0) } else { Color32::from_rgb(0x8a, 0x4a, 0x4a) }),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, on_c, name)
    });
    response
}

/// Cyan uppercase section header with underline.
pub fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(6.0);
    let text = egui::RichText::new(title)
                .size(10.0)
        .color(theme::CYAN);
    ui.label(text);
    let rect = ui.available_rect_before_wrap();
    let sep = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 1.0));
    ui.painter().rect_filled(sep, 0.0, theme::LINE);
    ui.add_space(4.0);
}

/// Big value card (readout).
pub fn value_card(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
        egui::Frame::NONE
            .fill(theme::PANEL2)
            .stroke(Stroke::new(1.0, theme::LINE))
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.set_min_height(46.0);
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(label).size(9.0).color(theme::DIM));
                    ui.label(egui::RichText::new(value).strong().size(20.0).color(color).monospace());
                });
            });
    });
}

/// Status pill in the header.
pub fn status_pill(ui: &mut egui::Ui, color: Color32, text: &str) {
    egui::Frame::NONE
        .fill(theme::PANEL2)
        .stroke(Stroke::new(1.0, theme::LINE))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("●").size(14.0).color(color));
                ui.label(egui::RichText::new(text).strong().size(13.0).color(color));
            });
        });
}

/// Dashed line used for animated flows on the scheme.
pub fn dashed_line(
    painter: &egui::Painter,
    from: Pos2,
    to: Pos2,
    color: Color32,
    width: f32,
    dash: f32,
    gap: f32,
    phase: f32,
) {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let len = dx.hypot(dy);
    if len <= 0.001 {
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let period = dash + gap;
    let mut d = -phase.rem_euclid(period);
    while d < len {
        let a = d.max(0.0);
        let b = (d + dash).min(len);
        if b > a {
            painter.line_segment(
                [
                    Pos2::new(from.x + ux * a, from.y + uy * a),
                    Pos2::new(from.x + ux * b, from.y + uy * b),
                ],
                Stroke::new(width, color),
            );
        }
        d += period;
    }
}
