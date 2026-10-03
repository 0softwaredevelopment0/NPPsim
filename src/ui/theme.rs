//! "Dispatch room" dark palette and small formatting helpers (port of the Java GUI palette).

use egui::Color32;

pub const BG: Color32 = Color32::from_rgb(0x0a, 0x0e, 0x12);
pub const PANEL: Color32 = Color32::from_rgb(0x10, 0x16, 0x1d);
pub const PANEL2: Color32 = Color32::from_rgb(0x15, 0x1d, 0x26);
pub const LINE: Color32 = Color32::from_rgb(0x22, 0x30, 0x3d);
pub const TXT: Color32 = Color32::from_rgb(0xc8, 0xd6, 0xe0);
pub const DIM: Color32 = Color32::from_rgb(0x5f, 0x73, 0x85);
pub const GREEN: Color32 = Color32::from_rgb(0x3d, 0xdc, 0x84);
pub const AMBER: Color32 = Color32::from_rgb(0xff, 0xb4, 0x54);
pub const RED: Color32 = Color32::from_rgb(0xff, 0x5a, 0x5a);
pub const CYAN: Color32 = Color32::from_rgb(0x4c, 0xc9, 0xf0);
pub const YELLOW: Color32 = Color32::from_rgb(0xff, 0xe1, 0x4d);
pub const BTN: Color32 = Color32::from_rgb(0x18, 0x23, 0x2e);
pub const SEL_BG: Color32 = Color32::from_rgb(0x1a, 0x3a, 0x2a);
pub const FIELD: Color32 = Color32::from_rgb(0x0d, 0x14, 0x1b);

/// Apply the dark dispatch-room theme to the context.
pub fn apply(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        let v = &mut style.visuals;
        v.dark_mode = true;
        v.override_text_color = Some(TXT);
        v.panel_fill = BG;
        v.window_fill = PANEL;
        v.extreme_bg_color = FIELD;
        v.faint_bg_color = PANEL2;
        v.widgets.noninteractive.bg_fill = PANEL;
        v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TXT);
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, LINE);
        v.widgets.inactive.bg_fill = BTN;
        v.widgets.inactive.weak_bg_fill = BTN;
        v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TXT);
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, LINE);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x1f, 0x2d, 0x3a);
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, CYAN);
        v.widgets.active.weak_bg_fill = Color32::from_rgb(0x24, 0x35, 0x44);
        v.selection.bg_fill = Color32::from_rgb(0x1a, 0x3a, 0x2a);
        v.selection.stroke = egui::Stroke::new(1.0, GREEN);
    });

    let mut fonts = egui::FontDefinitions::default();
    // Prefer Windows system fonts; fall back to the built-ins if absent.
    for (family, name, path) in [
        (egui::FontFamily::Proportional, "segoeui", r"C:\Windows\Fonts\segoeui.ttf"),
        (egui::FontFamily::Monospace, "consola", r"C:\Windows\Fonts\consola.ttf"),
    ] {
        match std::fs::read(path) {
            Ok(data) => {
                fonts
                    .font_data
                    .insert(name.to_owned(), std::sync::Arc::new(egui::FontData::from_owned(data)));
                fonts.families.entry(family).or_default().insert(0, name.to_owned());
            }
            Err(_) => {
                eprintln!("NPP-SIM: system font not found, using default: {path}");
            }
        }
    }
    ctx.set_fonts(fonts);
}

/// "01:23:45" from seconds.
pub fn fmt_time(t: f64) -> String {
    let s = t as u64;
    format!("{:02}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
}

pub fn f1(v: f64) -> String {
    format!("{v:.1}")
}

pub fn f2(v: f64) -> String {
    format!("{v:.2}")
}

/// Traffic-light colour for a big readout value.
pub fn value_color(v: f64, ok_low: f64, warn_high: f64) -> Color32 {
    if v > warn_high {
        RED
    } else if v < ok_low {
        AMBER
    } else {
        GREEN
    }
}

/// Colour scale for assembly fuel temperature, °C (300..2800) — port of `tempColor`.
pub fn temp_color(t: f64) -> Color32 {
    const STOPS: [(f64, (u8, u8, u8)); 8] = [
        (300.0, (0x1e, 0x3f, 0x66)),
        (600.0, (0x2f, 0x6f, 0xbf)),
        (900.0, (0x3f, 0xb0, 0xe0)),
        (1200.0, (0x3d, 0xdc, 0x84)),
        (1600.0, (0xd8, 0xe0, 0x4d)),
        (2000.0, (0xff, 0x9a, 0x3d)),
        (2400.0, (0xff, 0x5a, 0x5a)),
        (2800.0, (0xff, 0xc8, 0xc8)),
    ];
    if t <= STOPS[0].0 {
        let (r, g, b) = STOPS[0].1;
        return Color32::from_rgb(r, g, b);
    }
    for i in 1..STOPS.len() {
        if t <= STOPS[i].0 {
            let f = (t - STOPS[i - 1].0) / (STOPS[i].0 - STOPS[i - 1].0);
            let (r1, g1, b1) = STOPS[i - 1].1;
            let (r2, g2, b2) = STOPS[i].1;
            return lerp_color(Color32::from_rgb(r1, g1, b1), Color32::from_rgb(r2, g2, b2), f);
        }
    }
    let (r, g, b) = STOPS[STOPS.len() - 1].1;
    Color32::from_rgb(r, g, b)
}

pub fn lerp_color(a: Color32, b: Color32, f: f64) -> Color32 {
    let l = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * f).round() as u8;
    Color32::from_rgb(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
    )
}

/// Radial glow substitute: concentric fading circles.
pub fn radial_glow(painter: &egui::Painter, center: egui::Pos2, radius: f32, color: Color32, steps: usize) {
    for k in 0..steps {
        let f = (k + 1) as f32 / steps as f32;
        let a = (1.0 - f) * 0.5;
        let c = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (a * 255.0) as u8);
        painter.circle_filled(center, radius * f, c);
    }
}
