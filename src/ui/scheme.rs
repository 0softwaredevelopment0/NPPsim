//! Unit process diagram: reactor, pressurizer, MCPs, SG, turbine, generator,
//! condenser, feed water — drawn in a virtual 1000×560 coordinate space.

use crate::core::Plant;
use crate::ui::theme;
use crate::ui::widgets::dashed_line;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

/// Virtual → screen transform (scale + centering).
struct Xf {
    s: f32,
    dx: f32,
    dy: f32,
}

impl Xf {
    fn new(rect: &Rect) -> Self {
        let s = (rect.width() / 1000.0).min(rect.height() / 560.0);
        Xf {
            s,
            dx: rect.min.x + (rect.width() - 1000.0 * s) / 2.0,
            dy: rect.min.y + (rect.height() - 560.0 * s) / 2.0,
        }
    }
    #[inline]
    fn p(&self, x: f64, y: f64) -> Pos2 {
        Pos2::new(self.dx + x as f32 * self.s, self.dy + y as f32 * self.s)
    }
    #[inline]
    fn v(&self, v: f64) -> f32 {
        v as f32 * self.s
    }
    fn rect(&self, x: f64, y: f64, w: f64, h: f64, r: f64) -> Rect {
        let _ = r;
        Rect::from_min_size(self.p(x, y), Vec2::new(self.v(w), self.v(h)))
    }
    fn fill(&self, p: &egui::Painter, x: f64, y: f64, w: f64, h: f64, r: f64, c: Color32) {
        p.rect_filled(self.rect(x, y, w, h, r), self.v(r), c);
    }
    fn stroke_rect(&self, p: &egui::Painter, x: f64, y: f64, w: f64, h: f64, r: f64, st: Stroke) {
        p.rect_stroke(self.rect(x, y, w, h, r), self.v(r), st, egui::StrokeKind::Inside);
    }
    fn pipe(&self, p: &egui::Painter, x1: f64, y1: f64, x2: f64, y2: f64, c: Color32) {
        p.line_segment([self.p(x1, y1), self.p(x2, y2)], Stroke::new(self.v(7.0), c));
    }
    fn flow(&self, p: &egui::Painter, x1: f64, y1: f64, x2: f64, y2: f64, rate: f64, phase: f32) {
        if rate <= 0.0 {
            return;
        }
        dashed_line(
            p,
            self.p(x1, y1),
            self.p(x2, y2),
            Color32::from_rgba_unmultiplied(111, 195, 255, 200),
            self.v(3.0),
            self.v(3.0),
            self.v(14.0),
            (phase * 40.0 * rate as f32) % ((self.v(3.0) + self.v(14.0)) * 100.0),
        );
    }
    fn label(&self, p: &egui::Painter, cx: f64, y: f64, s: &str, font: FontId, c: Color32) {
        p.text(self.p(cx, y), Align2::CENTER_CENTER, s, font, c);
    }
}

pub fn draw(ui: &mut egui::Ui, plant: &Plant, rotor_angle: f32) {
    let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    let painter = ui.painter_at(rect);
    let xf = Xf::new(&rect);

    let glow = clamp01(plant.p / 130.0) as f32;

    // --- reactor ---
    xf.fill(&painter, 140.0, 200.0, 150.0, 210.0, 16.0, Color32::from_rgb(0x15, 0x20, 0x2b));
    xf.stroke_rect(&painter, 140.0, 200.0, 150.0, 210.0, 16.0, Stroke::new(1.0, theme::LINE));
    if glow > 0.01 {
        theme::radial_glow(
            &painter,
            xf.p(215.0, 300.0),
            xf.v(95.0),
            Color32::from_rgb(255, 160, 40),
            6,
        );
    }
    xf.stroke_rect(
        &painter,
        180.0,
        255.0,
        70.0,
        90.0,
        6.0,
        Stroke::new(1.0, if plant.p > 110.0 { theme::RED } else { Color32::from_rgb(0x6b, 0x4a, 0x26) }),
    );
    xf.label(&painter, 215.0, 130.0, "РЕАКТОР", FontId::proportional(10.0), theme::DIM);
    xf.label(&painter, 215.0, 310.0, "АКТИВНАЯ ЗОНА", FontId::proportional(10.0), theme::DIM);

    // --- pressurizer ---
    xf.fill(&painter, 60.0, 130.0, 46.0, 120.0, 6.0, Color32::from_rgb(0x15, 0x20, 0x2b));
    xf.stroke_rect(&painter, 60.0, 130.0, 46.0, 120.0, 6.0, Stroke::new(1.0, theme::LINE));
    let kd_h = clamp01((plant.p1 - 14.0) / 4.0 * 40.0) as f32;
    painter.rect_filled(
        xf.rect(80.0, 178.0, 6.0, kd_h as f64, 0.0),
        0.0,
        Color32::from_rgb(0x2b, 0x5a, 0x8f),
    );
    xf.label(&painter, 83.0, 270.0, "КД", FontId::proportional(10.0), theme::DIM);
    xf.pipe(&painter, 83.0, 250.0, 83.0, 290.0, Color32::from_rgb(0x33, 0x48, 0x5c));
    xf.pipe(&painter, 83.0, 290.0, 140.0, 290.0, Color32::from_rgb(0x33, 0x48, 0x5c));

    // --- primary pipes ---
    let hot = if plant.t_core > 345.0 { theme::RED } else { Color32::from_rgb(0xc4, 0x6a, 0x2a) };
    let cold = Color32::from_rgb(0x3f, 0x7f, 0xbf);
    xf.pipe(&painter, 215.0, 215.0, 360.0, 215.0, hot);
    xf.pipe(&painter, 360.0, 215.0, 360.0, 235.0, hot);
    xf.pipe(&painter, 215.0, 395.0, 360.0, 395.0, cold);
    xf.pipe(&painter, 360.0, 395.0, 360.0, 375.0, cold);
    // MCP
    let mcp_c = xf.p(300.0, 395.0);
    painter.circle_filled(mcp_c, xf.v(20.0), Color32::from_rgb(0x15, 0x20, 0x2b));
    painter.circle_stroke(mcp_c, xf.v(20.0), Stroke::new(1.0, theme::LINE));
    let gcn_txt = format!("ГЦН {}/4", plant.gcn_count());
    xf.label(
        &painter,
        300.0,
        400.0,
        &gcn_txt,
        FontId::proportional(10.0),
        if plant.gcn_count() < 4 { theme::AMBER } else { theme::DIM },
    );
    if plant.pump > 0.05 {
        xf.flow(&painter, 215.0, 395.0, 345.0, 395.0, plant.pump, plant.time as f32);
        xf.flow(&painter, 215.0, 215.0, 345.0, 215.0, plant.pump, plant.time as f32);
    }

    // --- steam generator ---
    xf.fill(&painter, 360.0, 120.0, 200.0, 150.0, 20.0, Color32::from_rgb(0x15, 0x20, 0x2b));
    xf.stroke_rect(&painter, 360.0, 120.0, 200.0, 150.0, 20.0, Stroke::new(1.0, theme::LINE));
    let sg = clamp01(plant.p / 120.0) as f32;
    if sg > 0.01 {
        painter.rect_filled(
            xf.rect(368.0, 240.0, 184.0, 22.0, 8.0),
            xf.v(8.0),
            Color32::from_rgba_unmultiplied(0xff, 0x9a, 0x3d, (sg * 0.5 * 255.0) as u8),
        );
    }
    xf.label(&painter, 460.0, 150.0, "ПАРОГЕНЕРАТОР", FontId::proportional(10.0), theme::DIM);
    xf.label(&painter, 460.0, 292.0, &format!("ПГ {}°C", theme::f1(plant.t_sg)), FontId::proportional(10.0), theme::DIM);
    xf.label(&painter, 520.0, 315.0, &format!("Уровень ПГ: {}%", plant.level.round() as i64), FontId::proportional(10.0), theme::DIM);

    // --- steam ---
    xf.pipe(&painter, 520.0, 120.0, 520.0, 80.0, Color32::from_rgb(0x33, 0x48, 0x5c));
    xf.pipe(&painter, 520.0, 80.0, 620.0, 80.0, Color32::from_rgb(0x33, 0x48, 0x5c));
    xf.pipe(&painter, 400.0, 120.0, 400.0, 80.0, Color32::from_rgb(0x33, 0x48, 0x5c));
    xf.pipe(&painter, 400.0, 80.0, 470.0, 80.0, Color32::from_rgb(0x33, 0x48, 0x5c));
    if plant.p2 > 0.5 {
        xf.flow(&painter, 400.0, 84.0, 600.0, 84.0, plant.gov * plant.p2 / 6.4, plant.time as f32);
    }
    xf.label(&painter, 560.0, 70.0, "ПАР", FontId::proportional(10.0), theme::DIM);
    painter.rect_filled(xf.rect(516.0, 72.0, 8.0, 16.0, 0.0), 0.0, if plant.rel_s { theme::RED } else { Color32::from_rgb(0x7a, 0x2f, 0x2f) });
    xf.label(&painter, 540.0, 112.0, "ГПЗ", FontId::proportional(10.0), theme::DIM);

    // --- turbine (spinning rotor) ---
    xf.fill(&painter, 620.0, 130.0, 120.0, 70.0, 12.0, Color32::from_rgb(0x15, 0x20, 0x2b));
    xf.stroke_rect(&painter, 620.0, 130.0, 120.0, 70.0, 12.0, Stroke::new(1.0, theme::LINE));
    let tc = xf.p(680.0, 165.0);
    let rr = xf.v(34.0);
    for i in 0..6 {
        let a = rotor_angle.to_radians() + i as f32 * 60.0f32.to_radians();
        let (sin, cos) = a.sin_cos();
        painter.line_segment(
            [tc + Vec2::new(-rr * cos, -rr * sin), tc + Vec2::new(rr * cos, rr * sin)],
            Stroke::new(xf.v(4.0), Color32::from_rgb(0x4f, 0x6a, 0x80)),
        );
    }
    xf.label(&painter, 680.0, 120.0, "ТУРБИНА", FontId::proportional(10.0), theme::DIM);
    xf.label(&painter, 680.0, 115.0, "3000 об/мин", FontId::proportional(9.0), theme::DIM);

    // --- generator ---
    xf.fill(&painter, 760.0, 145.0, 90.0, 40.0, 8.0, Color32::from_rgb(0x15, 0x20, 0x2b));
    xf.stroke_rect(&painter, 760.0, 145.0, 90.0, 40.0, 8.0, Stroke::new(1.0, theme::LINE));
    xf.label(&painter, 805.0, 168.0, "ГЕНЕРАТОР", FontId::proportional(10.0), theme::DIM);
    xf.label(
        &painter,
        805.0,
        205.0,
        &format!("{} МВт", plant.mwe.round() as i64),
        FontId::proportional(10.0),
        if plant.mwe > 10.0 { theme::GREEN } else { theme::DIM },
    );
    xf.pipe(&painter, 760.0, 165.0, 712.0, 165.0, Color32::from_rgb(0x31, 0x45, 0x5a));
    xf.pipe(&painter, 850.0, 165.0, 895.0, 165.0, Color32::from_rgb(0x31, 0x45, 0x5a));
    xf.pipe(&painter, 895.0, 165.0, 895.0, 120.0, Color32::from_rgb(0x31, 0x45, 0x5a));
    xf.pipe(&painter, 895.0, 120.0, 930.0, 120.0, Color32::from_rgb(0x31, 0x45, 0x5a));
    let grid_txt = if plant.breaker { "СЕТЬ" } else { "ОТКЛЮЧЕНО" };
    painter.text(
        xf.p(905.0, 140.0),
        Align2::LEFT_CENTER,
        grid_txt,
        FontId::proportional(10.0),
        theme::DIM,
    );
    let brk_c = xf.p(850.0, 165.0);
    painter.circle_filled(
        brk_c,
        xf.v(7.0),
        if plant.breaker { Color32::from_rgb(0x1f, 0x4a, 0x2f) } else { Color32::from_rgb(0x5a, 0x1a, 0x1a) },
    );

    // --- condenser and cooling water ---
    xf.fill(&painter, 620.0, 330.0, 200.0, 70.0, 12.0, Color32::from_rgb(0x15, 0x20, 0x2b));
    xf.stroke_rect(&painter, 620.0, 330.0, 200.0, 70.0, 12.0, Stroke::new(1.0, theme::LINE));
    xf.label(&painter, 720.0, 372.0, "КОНДЕНСАТОР", FontId::proportional(10.0), theme::DIM);
    xf.pipe(&painter, 660.0, 200.0, 660.0, 330.0, Color32::from_rgb(0x33, 0x48, 0x5c));
    xf.pipe(&painter, 740.0, 200.0, 740.0, 330.0, Color32::from_rgb(0x33, 0x48, 0x5c));
    xf.pipe(&painter, 640.0, 420.0, 640.0, 460.0, Color32::from_rgb(0x2a, 0x3c, 0x4d));
    xf.pipe(&painter, 640.0, 460.0, 760.0, 460.0, Color32::from_rgb(0x2a, 0x3c, 0x4d));
    xf.pipe(&painter, 760.0, 460.0, 760.0, 400.0, Color32::from_rgb(0x2a, 0x3c, 0x4d));
    xf.flow(&painter, 645.0, 452.0, 755.0, 452.0, 1.0, plant.time as f32);
    xf.label(&painter, 700.0, 440.0, "ОХЛ. ВОДА (река)", FontId::proportional(10.0), theme::DIM);

    // --- feed water ---
    xf.pipe(&painter, 560.0, 380.0, 560.0, 320.0, cold);
    xf.pipe(&painter, 560.0, 320.0, 600.0, 320.0, cold);
    xf.pipe(&painter, 600.0, 320.0, 600.0, 150.0, cold);
    if plant.feed > 0.05 {
        xf.flow(&painter, 560.0, 375.0, 560.0, 325.0, plant.feed, plant.time as f32);
    }
    let fp_c = xf.p(590.0, 380.0);
    painter.circle_filled(fp_c, xf.v(17.0), Color32::from_rgb(0x15, 0x20, 0x2b));
    painter.circle_stroke(fp_c, xf.v(17.0), Stroke::new(1.0, theme::LINE));
    xf.label(&painter, 590.0, 385.0, "ПН", FontId::proportional(10.0), theme::DIM);

    painter.text(
        xf.p(60.0, 460.0),
        Align2::LEFT_CENTER,
        &format!("Т зоны {}°C", theme::f1(plant.t_core)),
        FontId::proportional(10.0),
        theme::TXT,
    );
}

#[inline]
fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}
