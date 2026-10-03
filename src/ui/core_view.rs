//! Reactor overview: top view of the core "ball" with 9×9 fuel assemblies,
//! their fuel temperatures, melting state, and the 3×3 CPS rod positions.

use crate::core::constants::{clamp, CORE_N, MELT_T, ROD_N};
use crate::core::Plant;
use crate::ui::theme;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewTab {
    Core,
    Scheme,
}

struct Geo {
    cell: f32,
    cx: f32,
    cy: f32,
    rr: f32,
}

impl Geo {
    fn new(w: f32, h: f32) -> Self {
        let cell = (w * 0.58 / CORE_N as f32).min(h * 0.72 / CORE_N as f32).min(46.0);
        Geo {
            cell,
            cx: w * 0.38,
            cy: h * 0.53,
            rr: (cell * (CORE_N as f32 / 2.0 + 0.6) * 1.25).min(h * 0.45).min(w * 0.32),
        }
    }

    /// Screen centre of rod `k` (3×3 grid over the core).
    fn rod_pos(&self, k: usize) -> Pos2 {
        let gi = 1 + 3 * (k / 3);
        let gj = 1 + 3 * (k % 3);
        Pos2::new(
            self.cx + (gj as f32 - (CORE_N as f32 - 1.0) / 2.0) * self.cell,
            self.cy + (gi as f32 - (CORE_N as f32 - 1.0) / 2.0) * self.cell,
        )
    }

    fn rod_at(&self, mx: f32, my: f32) -> Option<usize> {
        for k in 0..ROD_N {
            let p = self.rod_pos(k);
            if (mx - p.x).hypot(my - p.y) < self.cell * 0.45 {
                return Some(k);
            }
        }
        None
    }
}

pub fn draw(ui: &mut egui::Ui, plant: &Plant, rod_sel: &mut [bool; ROD_N]) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
    let painter = ui.painter_at(rect);
    let w = rect.width();
    let h = rect.height();
    let g = Geo::new(w, h);

    // click → toggle rod selection
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            if let Some(k) = g.rod_at(pos.x - rect.min.x, pos.y - rect.min.y) {
                rod_sel[k] = !rod_sel[k];
            }
        }
    }

    // ---- reactor vessel (ball, top view) ----
    let center = Pos2::new(g.cx, g.cy);
    painter.circle_filled(center, g.rr, Color32::from_rgb(0x14, 0x1a, 0x22));
    theme::radial_glow(&painter, center, g.rr * 0.95, Color32::from_rgb(0x2a, 0x3d, 0x4d), 6);
    painter.circle_stroke(center, g.rr, Stroke::new(3.0, theme::LINE));
    painter.circle_stroke(center, g.rr * 1.08, Stroke::new(1.0, theme::DIM));

    // ---- fuel assemblies with individual fuel temperature ----
    let mut max_t = f64::MIN;
    let mut min_t = f64::MAX;
    let mut mxi = 0usize;
    let mut mxj = 0usize;
    for i in 0..CORE_N {
        for j in 0..CORE_N {
            let c = (CORE_N as f64 - 1.0) / 2.0;
            let d = ((i as f64 - c).hypot(j as f64 - c)) / c;
            if d > 1.02 {
                continue;
            }
            let t = plant.fuel_temp(i, j);
            let m = plant.melt[i][j];
            if t > max_t {
                max_t = t;
                mxi = i;
                mxj = j;
            }
            if t < min_t {
                min_t = t;
            }
            let p = pos(&g, i, j);
            let s = g.cell * 0.76;
            let r = Rect::from_center_size(p, Vec2::splat(s));
            painter.rect_filled(r, 6.0, theme::temp_color(t));
            if m > 0.01 {
                let a = (0.92f64).min(m) as f32;
                painter.rect_filled(
                    r,
                    6.0,
                    Color32::from_rgba_unmultiplied(0, 0, 0, (a * 255.0) as u8),
                );
            }
            if m >= 1.0 {
                painter.rect_stroke(r, 6.0, Stroke::new(2.0, theme::RED), egui::StrokeKind::Inside);
            }
        }
    }
    // hottest assembly frame
    let hot_p = pos(&g, mxi, mxj);
    let hot_r = Rect::from_center_size(hot_p, Vec2::splat(g.cell));
    painter.rect_stroke(hot_r, 6.0, Stroke::new(2.0, Color32::WHITE), egui::StrokeKind::Inside);

    // temperature numbers inside each assembly
    let num_font = FontId::monospace((g.cell * 0.30).max(8.0));
    for i in 0..CORE_N {
        for j in 0..CORE_N {
            let c = (CORE_N as f64 - 1.0) / 2.0;
            let d = ((i as f64 - c).hypot(j as f64 - c)) / c;
            if d > 1.02 {
                continue;
            }
            let t = plant.fuel_temp(i, j);
            painter.text(
                pos(&g, i, j),
                Align2::CENTER_CENTER,
                format!("{}", t.round() as i64),
                num_font.clone(),
                if t > 2200.0 { Color32::WHITE } else { theme::BG },
            );
        }
    }

    // ---- CPS rods: 3×3 positions, click to select ----
    for k in 0..ROD_N {
        let p = g.rod_pos(k);
        let rr = g.cell * 0.34;
        let ext = plant.rod_extract(k);
        painter.circle_filled(p, rr, theme::FIELD);
        let ring = if ext > 80.0 {
            Color32::from_rgb(0x2f, 0x6f, 0xbf)
        } else if ext > 20.0 {
            theme::DIM
        } else {
            Color32::from_rgb(0x8a, 0x2f, 0x2f)
        };
        painter.circle_stroke(p, rr, Stroke::new(2.0, ring));
        painter.text(
            p + Vec2::new(0.0, -2.0),
            Align2::CENTER_CENTER,
            format!("{}", k + 1),
            FontId::monospace((g.cell * 0.24).max(8.0)),
            theme::TXT,
        );
        painter.text(
            p + Vec2::new(0.0, rr + 9.0),
            Align2::CENTER_CENTER,
            format!("{}%", ext.round() as i64),
            FontId::monospace((g.cell * 0.19).max(7.0)),
            theme::DIM,
        );
        if rod_sel[k] {
            painter.circle_stroke(p, rr + 3.0, Stroke::new(3.0, theme::AMBER));
        }
    }

    // ---- header & stats ----
    let sel_count = rod_sel.iter().filter(|&&s| s).count();
    painter.text(
        Pos2::new(w * 0.03, h * 0.06),
        Align2::LEFT_CENTER,
        "ОБЗОР РЕАКТОРА — ТОПЛИВО АКТИВНОЙ ЗОНЫ (вид сверху)",
        FontId::proportional(11.0),
        theme::CYAN,
    );
    let m_tot = plant.melt_total();
    let stats = |painter: &egui::Painter, y: f32, s: String| {
        painter.text(
            Pos2::new(w * 0.03, y),
            Align2::LEFT_CENTER,
            s,
            FontId::monospace(11.0),
            theme::TXT,
        );
    };
    stats(
        &painter,
        h * 0.11,
        format!(
            "Макс. топливо: {}°C   Мин: {}°C   плавление ≥ {}°C",
            max_t.round() as i64,
            min_t.round() as i64,
            MELT_T.round() as i64
        ),
    );
    stats(
        &painter,
        h * 0.16,
        format!(
            "Мощность: {}%   Темп. зоны: {}°C   Расплавлено: {} ТК{}",
            theme::f1(plant.p),
            theme::f1(plant.t_core),
            plant.melted_count(),
            if m_tot > 0.01 {
                format!(
                    " ({}% зоны)",
                    (m_tot * 100.0 / (CORE_N * CORE_N) as f64).round() as i64
                )
            } else {
                String::new()
            }
        ),
    );
    stats(
        &painter,
        h * 0.21,
        format!(
            "Клик по стержню на круге — выбор · выбрано: {} из {ROD_N}",
            sel_count
        ),
    );

    // ---- radial temperature profile ----
    let px0 = w * 0.66;
    let px1 = w * 0.96;
    let py0 = h * 0.16;
    let py1 = h * 0.52;
    painter.rect_filled(
        Rect::from_min_size(Pos2::new(px0 - 8.0, py0 - 18.0), Vec2::new(px1 - px0 + 16.0, py1 - py0 + 40.0)),
        8.0,
        Color32::from_rgb(0x1a, 0x27, 0x33),
    );
    painter.text(
        Pos2::new(px0, py0 - 12.0),
        Align2::LEFT_CENTER,
        "Радиальный профиль температуры топлива",
        FontId::proportional(10.0),
        theme::DIM,
    );
    painter.line_segment([Pos2::new(px0, py0), Pos2::new(px0, py1)], Stroke::new(1.0, theme::LINE));
    painter.line_segment([Pos2::new(px0, py1), Pos2::new(px1, py1)], Stroke::new(1.0, theme::LINE));
    let t_lo = 300.0;
    let t_hi = 2800.0;
    let mut prev: Option<Pos2> = None;
    for k in 0..40 {
        let f = k as f64 / 39.0;
        let j = ((CORE_N - 1) as f64 / 2.0 * f).round() as usize;
        let t = plant.fuel_temp((CORE_N - 1) / 2, (CORE_N - 1) / 2 + j);
        let x = px0 + (f as f32) * (px1 - px0);
        let y = py1 - (py1 - py0) * clamp((t - t_lo) / (t_hi - t_lo), 0.0, 1.0) as f32;
        if let Some(p) = prev {
            painter.line_segment([p, Pos2::new(x, y)], Stroke::new(2.0, Color32::from_rgb(0xff, 0x9a, 0x3d)));
        }
        prev = Some(Pos2::new(x, y));
    }
    painter.text(Pos2::new(px0, py1 + 14.0), Align2::LEFT_CENTER, "центр", FontId::monospace(9.0), theme::DIM);
    painter.text(Pos2::new(px1 - 20.0, py1 + 14.0), Align2::LEFT_CENTER, "край", FontId::monospace(9.0), theme::DIM);
    painter.text(Pos2::new(px0 - 10.0, py1), Align2::LEFT_CENTER, "300°C", FontId::monospace(9.0), theme::DIM);

    // ---- colour scale legend ----
    let lx0 = w * 0.66;
    let lx1 = w * 0.96;
    let ly = h * 0.72;
    painter.text(
        Pos2::new(lx0, ly - 12.0),
        Align2::LEFT_CENTER,
        "Шкала температуры топлива ТК, °C",
        FontId::proportional(9.0),
        theme::DIM,
    );
    let steps = 60;
    for s in 0..steps {
        let f = s as f32 / (steps - 1) as f32;
        let x0 = lx0 + f * (lx1 - lx0);
        let seg = Rect::from_min_size(Pos2::new(x0, ly), Vec2::new((lx1 - lx0) / steps as f32 + 1.0, 14.0));
        painter.rect_filled(seg, 0.0, theme::temp_color(300.0 + f as f64 * 2500.0));
    }
    painter.text(Pos2::new(lx0 - 2.0, ly + 22.0), Align2::LEFT_CENTER, "300", FontId::proportional(9.0), theme::DIM);
    painter.text(Pos2::new(lx1 - 24.0, ly + 22.0), Align2::LEFT_CENTER, "2800", FontId::proportional(9.0), theme::DIM);
}

fn pos(g: &Geo, i: usize, j: usize) -> Pos2 {
    Pos2::new(
        g.cx + (j as f32 - (CORE_N as f32 - 1.0) / 2.0) * g.cell,
        g.cy + (i as f32 - (CORE_N as f32 - 1.0) / 2.0) * g.cell,
    )
}
