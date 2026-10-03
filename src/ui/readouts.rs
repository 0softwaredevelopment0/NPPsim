//! Row of six big readout cards under the reactor/scheme view.

use crate::core::Plant;
use crate::ui::theme;
use crate::ui::widgets::value_card;

pub fn draw(ui: &mut egui::Ui, plant: &Plant) {
    let data = readouts(plant);
    ui.columns(6, |cols| {
        for (i, (label, value, color)) in data.into_iter().enumerate() {
            value_card(&mut cols[i], label, &value, color);
        }
    });
}

fn readouts(p: &Plant) -> [(&'static str, String, egui::Color32); 6] {
    [
        (
            "Реакторная мощность, %",
            theme::f1(p.p),
            theme::value_color(p.p, 90.0, 110.0),
        ),
        (
            "Темп. активной зоны, °C",
            theme::f1(p.t_core),
            theme::value_color(p.t_core, 305.0, 345.0),
        ),
        (
            "Давл. 1-го контура, МПа",
            theme::f2(p.p1),
            theme::value_color(p.p1, 15.4, 18.2),
        ),
        (
            "Давл. 2-го контура, МПа",
            theme::f2(p.p2),
            theme::value_color(p.p2, 6.2, 7.4),
        ),
        (
            "Эл. мощность, МВт",
            format!("{}", p.mwe.round() as i64),
            if p.gov > 0.0 { theme::GREEN } else { theme::DIM },
        ),
        (
            "Частота сети, Гц",
            if p.breaker { theme::f2(p.f) } else { "—".to_string() },
            theme::GREEN,
        ),
    ]
}
