//! Trend charts: power, core temperature, primary pressure, electric power.

use crate::core::Plant;
use crate::ui::theme;
use egui::{Color32, FontId, Pos2, Sense, Stroke, Vec2};

const SERIES: [(f64, Color32); 4] = [
    (250.0, theme::GREEN),
    (420.0, Color32::from_rgb(0xff, 0x7a, 0x1a)),
    (25.0, theme::CYAN),
    (1000.0, theme::YELLOW),
];
const LEGEND: [&str; 4] = ["Мощность %", "Темп. °C", "Давл.1 МПа", "МВт(э)"];

pub fn draw(ui: &mut egui::Ui, plant: &Plant) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 150.0), Sense::hover());
    let painter = ui.painter_at(rect);
    let w = rect.width();
    let h = rect.height();

    // legend
    let mut x = 10.0;
    for (i, name) in LEGEND.iter().enumerate() {
        painter.text(Pos2::new(x, rect.min.y + 12.0), egui::Align2::LEFT_CENTER, name, FontId::monospace(9.0), theme::DIM);
        x += name.len() as f32 * 5.6 + 4.0;
        painter.line_segment(
            [Pos2::new(x, rect.min.y + 12.0), Pos2::new(x + 24.0, rect.min.y + 12.0)],
            Stroke::new(2.0, SERIES[i].1),
        );
        x += 34.0;
    }

    // grid
    for i in 0..=4 {
        let y = rect.max.y - 14.0 - (h - 30.0) * i as f32 / 4.0;
        painter.line_segment(
            [Pos2::new(rect.min.x + 4.0, y), Pos2::new(rect.max.x - 4.0, y)],
            Stroke::new(1.0, Color32::from_rgb(0x1a, 0x27, 0x33)),
        );
    }

    let tr = &plant.trends;
    if tr.len() < 2 {
        return;
    }
    let n = tr.len();
    for (si, (scale, color)) in SERIES.iter().enumerate() {
        let mut pts = Vec::with_capacity(n);
        for (i, t) in tr.iter().enumerate() {
            let x = rect.min.x + 4.0 + (w - 8.0) * i as f32 / (n - 1).max(1) as f32;
            let y = rect.max.y - 14.0 - (h - 30.0) * (t[si] / scale).clamp(0.0, 1.0) as f32;
            pts.push(Pos2::new(x, y));
        }
        painter.add(egui::Shape::line(pts, Stroke::new(1.6, *color)));
    }
}
