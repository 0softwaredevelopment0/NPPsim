//! Application layer: eframe/egui app that owns the Plant, runs the tick
//! loop, routes keyboard input and lays out all UI panels.

mod controls;
mod core_view;
mod header;
mod panels;
mod readouts;
mod scheme;
mod theme;
mod trends;
mod widgets;

use crate::core::constants::ROD_N;
use crate::core::{Plant, Rng, Scenario};
use controls::RodInput;
use egui::{Key, Panel, RichText, Vec2};

/// eframe app: the "view" layer over the pure `core` simulation.
pub struct NppApp {
    plant: Plant,
    rng: Rng,
    rod_sel: [bool; ROD_N],
    rod_input: RodInput,
    tab: core_view::ViewTab,
    /// Dead-unit overlay state: seen = already shown after this accident,
    /// open = window currently visible.
    overlay_open: bool,
    overlay_seen: bool,
    rotor_angle: f32,
    mouse_rod_dir: i32,
    sound_on: bool,
}

impl NppApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let mut plant = Plant::new();
        plant.load_scenario(Scenario::Full);
        plant.add_log(
            "Р”РёСЃРїРµС‚С‡РµСЂСЃРєР°СЏ: СЃРёРјСѓР»СЏС‚РѕСЂ Р·Р°РїСѓС‰РµРЅ. РљР»Р°РІРёС€Рё: в†‘/в†“ вЂ” СЃС‚РµСЂР¶РЅРё, A вЂ” РђР—, РїСЂРѕР±РµР» вЂ” СЃС‚РѕРї",
            crate::core::LogClass::Ok,
        );
        NppApp {
            plant,
            rng: Rng::new(),
            rod_input: RodInput {
                pct: "50".to_string(),
            },
            rod_sel: [false; ROD_N],
            tab: core_view::ViewTab::Core,
            overlay_open: false,
            overlay_seen: false,
            rotor_angle: 0.0,
            mouse_rod_dir: 0,
            sound_on: true,
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        let (up, down, space_pressed, a_pressed) = ctx.input(|i| {
            (
                i.key_down(Key::ArrowUp) || i.key_down(Key::W),
                i.key_down(Key::ArrowDown) || i.key_down(Key::S),
                i.key_pressed(Key::Space),
                i.key_pressed(Key::A),
            )
        });
        if a_pressed {
            self.plant.scram();
        }
        if space_pressed {
            self.plant.rod_dir = 0;
        } else if up {
            self.plant.rod_dir = 1;
        } else if down {
            self.plant.rod_dir = -1;
        } else {
            self.plant.rod_dir = self.mouse_rod_dir;
        }
    }

    fn tick(&mut self, ctx: &egui::Context) {
        // ---- simulation tick (real frame time Г— acceleration) ----
        let dt_real = (ctx.input(|i| i.unstable_dt) as f64).clamp(0.0001, 0.5);
        let rpm = if self.plant.breaker {
            3000.0 * (if self.plant.gov > 0.0 {
                1.0f64.min(self.plant.p2 / 6.4)
            } else {
                0.4
            })
        } else {
            0.0
        };
        self.rotor_angle = (self.rotor_angle + (rpm / 60.0 * 360.0 * dt_real) as f32) % 360.0;
        self.handle_keys(ctx);
        let dt_sim = dt_real * self.plant.rate;
        self.plant.advance(dt_sim);
    }

    fn draw_right(&mut self, ui: &mut egui::Ui) {
        Panel::right("right_panel")
            .default_size(372.0)
            .resizable(true)
            .frame(
                egui::Frame::default()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::same(6)),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        self.mouse_rod_dir = controls::draw(
                            ui,
                            &mut self.plant,
                            &mut self.rod_sel,
                            &mut self.rod_input,
                            &mut self.rng,
                        );
                        ui.add_space(6.0);
                        panels::tasks(ui, &self.plant);
                        panels::alarms(ui, &mut self.plant);
                        panels::journal(ui, &self.plant);
                    });
            });
    }

    fn draw_header(&mut self, ui: &mut egui::Ui) {
        Panel::top("header_panel")
            .frame(
                egui::Frame::default()
                    .fill(theme::PANEL)
                    .stroke(egui::Stroke::new(1.0, theme::LINE))
                    .inner_margin(egui::Margin::symmetric(10, 6)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("NPP-SIM").strong().size(15.0).color(theme::CYAN));
                    ui.label(
                        RichText::new("РЎРРњРЈР›РЇРўРћР  РЈРџР РђР’Р›Р•РќРРЇ РђР­РЎ вЂў Р’Р’Р­Р -1000")
                            .size(11.0)
                            .color(theme::DIM),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (sc, st) = header::plant_status(&self.plant);
                        widgets::status_pill(ui, sc, st);

                        if ui
                            .toggle_value(&mut self.sound_on, RichText::new("рџ”Љ Р—РІСѓРє").size(11.0))
                            .changed()
                        {
                            self.plant.sound = self.sound_on;
                        }
                        for r in header::RATES {
                            let txt = format!("{}Г—", r as i64);
                            let btn = egui::Button::new(RichText::new(txt).size(11.0)).fill(
                                if (self.plant.rate - r).abs() < f64::EPSILON {
                                    theme::SEL_BG
                                } else {
                                    theme::BTN
                                },
                            );
                            if ui.add(btn).clicked() {
                                self.plant.rate = r;
                            }
                        }
                        ui.vertical(|ui| {
                            ui.set_max_width(70.0);
                            ui.label(RichText::new("РњР’С‚(СЌ)").size(9.0).color(theme::DIM));
                            ui.label(
                                RichText::new(format!("{}", self.plant.mwe.round() as i64))
                                    .strong()
                                    .size(15.0)
                                    .color(theme::GREEN),
                            );
                        });
                        ui.vertical(|ui| {
                            ui.set_max_width(80.0);
                            ui.label(RichText::new("Р’СЂРµРјСЏ").size(9.0).color(theme::DIM));
                            ui.label(
                                RichText::new(theme::fmt_time(self.plant.time))
                                    .strong()
                                    .size(14.0)
                                    .color(theme::CYAN)
                                    .monospace(),
                            );
                        });
                    });
                });
            });
    }

    fn draw_central(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(theme::BG).inner_margin(egui::Margin::same(6)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (t, name) in [
                        (core_view::ViewTab::Core, "РћР±Р·РѕСЂ СЂРµР°РєС‚РѕСЂР°"),
                        (core_view::ViewTab::Scheme, "РЎС…РµРјР° Р±Р»РѕРєР°"),
                    ] {
                        let active = self.tab == t;
                        let btn = egui::Button::new(RichText::new(name).size(11.0)).fill(if active {
                            theme::SEL_BG
                        } else {
                            theme::BTN
                        });
                        if ui.add(btn).clicked() {
                            self.tab = t;
                        }
                    }
                });
                ui.add_space(2.0);

                let bottom_h = 70.0 + 160.0;
                let view =
                    Vec2::new(ui.available_width(), (ui.available_height() - bottom_h).max(120.0));
                ui.allocate_ui(view, |ui| match self.tab {
                    core_view::ViewTab::Core => {
                        core_view::draw(ui, &self.plant, &mut self.rod_sel)
                    }
                    core_view::ViewTab::Scheme => scheme::draw(ui, &self.plant, self.rotor_angle),
                });
                ui.add_space(4.0);
                readouts::draw(ui, &self.plant);
                trends::draw(ui, &self.plant);
            });
    }

    fn maybe_dead_overlay(&mut self, ctx: &egui::Context) {
        if !self.plant.dead {
            self.overlay_open = false;
            self.overlay_seen = false;
            return;
        }
        // show once per accident until the operator loads a scenario or closes
        if !self.overlay_seen {
            self.overlay_seen = true;
            self.overlay_open = true;
        }
        if !self.overlay_open {
            return;
        }
        let mut open = self.overlay_open;
        egui::Window::new(
            RichText::new("вљ  РђР’РђР РР™РќРђРЇ РћРЎРўРђРќРћР’РљРђ Р‘Р›РћРљРђ")
                .strong()
                .size(14.0)
                .color(theme::RED),
        )
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(
                "РџСЂРѕРёР·РѕС€Р»Рѕ РїРѕРІСЂРµР¶РґРµРЅРёРµ Р°РєС‚РёРІРЅРѕР№ Р·РѕРЅС‹. Р‘Р»РѕРє РѕСЃС‚Р°РЅРѕРІР»РµРЅ.\nР—Р°РіСЂСѓР·РёС‚Рµ СЃС†РµРЅР°СЂРёР№ РґР»СЏ РїСЂРѕРґРѕР»Р¶РµРЅРёСЏ С‚СЂРµРЅРёСЂРѕРІРєРё.",
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .button(RichText::new("РќРѕРІР°СЏ СЃРјРµРЅР° вЂ” Р Р°Р±РѕС‚Р° 100%").size(12.0))
                    .clicked()
                {
                    self.plant.load_scenario(Scenario::Full);
                }
            });
        });
        self.overlay_open = open;
    }
}

impl eframe::App for NppApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.tick(&ctx);
        self.draw_right(ui);
        self.draw_header(ui);
        self.draw_central(ui);
        self.maybe_dead_overlay(&ctx);
    }
}
