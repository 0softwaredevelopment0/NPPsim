//! Control panel: rods, APR, boron, MCPs, pressurizer, BRU-A, feed pumps,
//! turbine, generator, scenarios, events, AZ keys. Port of `buildControls()`.

use crate::core::{EventKind, LogClass, Plant, RodSpeed, Scenario};
use crate::ui::theme;
use crate::ui::widgets::{section, toggle_switch};
use egui::{Color32, RichText, TextEdit};

/// State carried across frames for the rod-selection text input.
pub struct RodInput {
    pub pct: String,
}

/// Draw the controls; returns the mouse-driven rod direction (-1/0/+1)
/// while the ▲/▼ buttons are held down.
pub fn draw(ui: &mut egui::Ui, plant: &mut Plant, rod_sel: &mut [bool; 9], rod_input: &mut RodInput, rng: &mut crate::core::Rng) -> i32 {
    let mut mouse_dir = 0i32;
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        // ============== RODS ==============
        section(ui, "УПРАВЛЕНИЕ СТЕРЖНЯМИ ОР СУЗ");
        ui.horizontal(|ui| {
            if hold_button(ui, "▲ Извлечь") {
                mouse_dir = 1;
            }
            if ui.button(RichText::new("Стоп").size(11.0)).clicked() {
                plant.rod_dir = 0;
            }
            if hold_button(ui, "▼ Ввести") {
                mouse_dir = -1;
            }
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Скорость:").size(11.0));
            for (sp, name) in [
                (RodSpeed::Slow, "Медленно"),
                (RodSpeed::Norm, "Нормально"),
                (RodSpeed::Fast, "Быстро"),
            ] {
                let active = plant.rod_sp == sp;
                let btn = egui::Button::new(RichText::new(name).size(11.0)).fill(if active {
                    theme::SEL_BG
                } else {
                    theme::BTN
                });
                if ui.add(btn).clicked() {
                    plant.rod_sp = sp;
                }
            }
            let mean = plant.rod_extract_mean();
            let pos = RichText::new(format!("извлечение {:.1}%", mean))
                                .size(13.0)
                .color(if mean < 15.0 { theme::RED } else { theme::CYAN });
            ui.label(pos);
        });

        section(ui, "СТЕРЖНИ СУЗ: % ИЗВЛЕЧЕНИЯ");
        ui.horizontal(|ui| {
            ui.label(RichText::new("Извлечение, %:").size(11.0));
            TextEdit::singleline(&mut rod_input.pct)
                .desired_width(44.0)
                .font(egui::TextStyle::Monospace)
                .text_color(theme::CYAN)
                .show(ui);
            if ui.button(RichText::new("Применить").size(11.0)).clicked() {
                apply_rods(plant, rod_sel, rod_input);
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                apply_rods(plant, rod_sel, rod_input);
            }
        });
        ui.horizontal(|ui| {
            if ui.button(RichText::new("Выбрать все").size(11.0)).clicked() {
                *rod_sel = [true; 9];
            }
            if ui.button(RichText::new("Снять выбор").size(11.0)).clicked() {
                *rod_sel = [false; 9];
            }
            let names: Vec<String> = rod_sel
                .iter()
                .enumerate()
                .filter(|(_, s)| **s)
                .map(|(k, _)| (k + 1).to_string())
                .collect();
            ui.label(
                RichText::new(if names.is_empty() {
                    "выбрано: —".to_string()
                } else {
                    format!("выбрано: {}", names.join(","))
                })
                .size(10.0)
                .color(theme::AMBER),
            );
        });
        let mean = plant.rod_extract_mean();
        ui.label(
            RichText::new(format!("средн. извлечение: {}%", mean.round() as i64))
                .size(10.0)
                .color(if mean < 5.0 { theme::RED } else { theme::DIM }),
        );
        ui.label(
            RichText::new("Клик по стержню на круге — выбор · 0% = в зоне · 100% = извлечён")
                .size(9.0)
                .color(theme::DIM),
        );

        // ============== APR & BORON ==============
        section(ui, "АРК И БОР");
        ui.horizontal(|ui| {
            if ui.toggle_value(&mut plant.ark, RichText::new("АРК").size(11.0)).changed() {
                plant.add_log(
                    format!("АРК: {}", if plant.ark { "ВКЛ" } else { "ОТКЛ" }),
                    if plant.ark { LogClass::Ok } else { LogClass::Warn },
                );
            }
            ui.label(RichText::new("уставка:").size(11.0).color(theme::DIM));
            let mut sp = plant.p_set as i32;
            let slider = egui::Slider::new(&mut sp, 0..=110).show_value(false);
            ui.add_sized([110.0, 18.0], slider);
            plant.p_set = sp as f64;
            ui.label(RichText::new(format!("{}%", sp)).strong().size(12.0).color(theme::CYAN));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Бор, г/кг").size(11.0).color(theme::DIM));
            let mut bv = (plant.b * 10.0) as i32;
            ui.add(egui::Slider::new(&mut bv, 0..=100).show_value(false));
            plant.b = bv as f64 / 10.0;
            ui.label(RichText::new(theme::f1(plant.b)).strong().size(12.0).color(theme::GREEN));
        });

        // ============== PRIMARY ==============
        section(ui, "ПЕРВЫЙ КОНТУР — ГЦН И КД");
        ui.horizontal(|ui| {
            for k in 0..4 {
                let mut on = plant.gcn[k];
                if toggle_switch(ui, &format!("ГЦН-{}", k + 1), &mut on).changed() {
                    plant.gcn[k] = on;
                    plant.add_log(
                        format!("ГЦН-{}: {}", k + 1, if on { "ВКЛ" } else { "ОТКЛ" }),
                        if on { LogClass::Ok } else { LogClass::Warn },
                    );
                }
            }
        });
        ui.label(
            RichText::new(format!(
                "Расход ГЦН: {}% · насосы {}/4",
                (plant.pump_flow() * 100.0).round() as i64,
                plant.gcn_count()
            ))
            .size(10.0)
            .color(if plant.gcn_count() < 4 { theme::AMBER } else { theme::GREEN }),
        );
        ui.horizontal(|ui| {
            let mut on = plant.makeup;
            if toggle_switch(ui, "ПОДПИТКА КД", &mut on).changed() {
                plant.makeup = on;
                plant.add_log(
                    format!("Подпитка КД: {}", if on { "ВКЛ" } else { "ОТКЛ" }),
                    if on { LogClass::Ok } else { LogClass::Warn },
                );
            }
            ui.label(
                RichText::new("компенсатор давления: подпитка/слив")
                    .size(9.0)
                    .color(theme::DIM),
            );
        });

        // ============== BRU-A ==============
        section(ui, "БРУ-А (СБРОС ПАРА)");
        ui.horizontal(|ui| {
            ui.label(RichText::new("Режим:").size(10.0).color(theme::DIM));
            let idx = match plant.bru_mode {
                crate::core::types::BruMode::Auto => 0,
                crate::core::types::BruMode::Manual => 1,
                crate::core::types::BruMode::Off => 2,
            };
            egui::ComboBox::from_id_salt("bru_mode")
                .selected_text(["АВТ", "РУЧ", "ОТКЛ"][idx])
                .width(60.0)
                .show_ui(ui, |ui| {
                    for (i, name) in ["АВТ", "РУЧ", "ОТКЛ"].iter().enumerate() {
                        if ui.selectable_label(i == idx, *name).clicked() && i != idx {
                            plant.bru_mode = match i {
                                0 => crate::core::types::BruMode::Auto,
                                1 => crate::core::types::BruMode::Manual,
                                _ => crate::core::types::BruMode::Off,
                            };
                            plant.bru_manual = false;
                            plant.add_log(
                                format!(
                                    "БРУ-А: режим {}",
                                    match plant.bru_mode {
                                        crate::core::types::BruMode::Auto => "АВТ",
                                        crate::core::types::BruMode::Manual => "РУЧ",
                                        crate::core::types::BruMode::Off => "ОТКЛ",
                                    }
                                ),
                                LogClass::Info,
                            );
                        }
                    }
                });
            if ui
                .add_enabled(
                    plant.bru_mode == crate::core::types::BruMode::Manual,
                    egui::Button::new(RichText::new("Открыть").size(11.0)),
                )
                .clicked()
            {
                plant.bru_manual = !plant.bru_manual;
                plant.add_log(
                    format!(
                        "БРУ-А ручное: {}",
                        if plant.bru_manual { "ОТКРЫТ" } else { "ЗАКРЫТ" }
                    ),
                    if plant.bru_manual { LogClass::Warn } else { LogClass::Ok },
                );
            }
            let open = plant.bru_open_readonly();
            ui.label(
                RichText::new(if open { "ОТКРЫТ" } else { "ЗАКРЫТ" })
                                        .size(9.0)
                    .color(if open { theme::AMBER } else { theme::DIM }),
            );
        });

        // ============== SECONDARY ==============
        section(ui, "ВТОРОЙ КОНТУР — ПИТАНИЕ И ТУРБИНА");
        ui.horizontal(|ui| {
            for (k, name) in ["ПЭН-1", "ПЭН-2", "МПНА"].iter().enumerate() {
                let mut on = plant.fen[k];
                if toggle_switch(ui, name, &mut on).changed() {
                    plant.fen[k] = on;
                    plant.add_log(
                        format!("{name}: {}", if on { "ВКЛ" } else { "ОТКЛ" }),
                        if on { LogClass::Ok } else { LogClass::Warn },
                    );
                }
            }
        });
        ui.label(
            RichText::new(format!("Питательная вода: {}%", (plant.feed_flow() * 100.0).round() as i64))
                .size(10.0)
                .color(theme::GREEN),
        );
        ui.horizontal(|ui| {
            ui.label(RichText::new("Турбина: впуск пара").size(11.0).color(theme::DIM));
            let mut gv = (plant.gov * 100.0) as i32;
            ui.add(egui::Slider::new(&mut gv, 0..=100).show_value(false));
            plant.gov = gv as f64 / 100.0;
            ui.label(RichText::new(format!("{}%", gv)).strong().size(12.0).color(theme::GREEN));
        });
        ui.horizontal(|ui| {
            let mut on = plant.breaker;
            if toggle_switch(ui, "ГЕНЕРАТОР", &mut on).changed() {
                plant.breaker = on;
                plant.add_log(
                    format!(
                        "Выключатель генератора: {}",
                        if on { "ВКЛ" } else { "ОТКЛ" }
                    ),
                    if on { LogClass::Ok } else { LogClass::Warn },
                );
            }
            ui.label(
                RichText::new(if plant.breaker {
                    format!("{} Гц", theme::f2(plant.f))
                } else {
                    "СЕТЬ ОТКЛ".to_string()
                })
                                .size(12.0)
                .color(if plant.breaker { theme::GREEN } else { theme::RED }),
            );
        });

        // ============== SCENARIOS ==============
        section(ui, "СЦЕНАРИИ");
        ui.horizontal(|ui| {
            if ui.button(RichText::new("Работа 100%").size(11.0)).clicked() {
                plant.load_scenario(Scenario::Full);
            }
            if ui.button(RichText::new("Пуск блока").size(11.0)).clicked() {
                plant.load_scenario(Scenario::Start);
            }
            if ui.button(RichText::new("Останов").size(11.0)).clicked() {
                plant.load_scenario(Scenario::Stop);
            }
        });

        // ============== EVENTS ==============
        section(ui, "СОБЫТИЯ (АВАРИИ)");
        ui.horizontal(|ui| {
            if ui.button(RichText::new("Отказ ГЦН").size(11.0)).clicked() {
                plant.fire_event(EventKind::Pump, rng);
            }
            if ui.button(RichText::new("Течь 1 к.").size(11.0)).clicked() {
                plant.fire_event(EventKind::Loca, rng);
            }
            if ui.button(RichText::new("Разрыв пар.").size(11.0)).clicked() {
                plant.fire_event(EventKind::Steam, rng);
            }
        });
        ui.horizontal(|ui| {
            if ui.button(RichText::new("Ложное АЗ").size(11.0)).clicked() {
                plant.fire_event(EventKind::Scram, rng);
            }
            if ui.button(RichText::new("Отказ пит. насоса").size(11.0)).clicked() {
                plant.fire_event(EventKind::Feed, rng);
            }
            if ui.button(RichText::new("Случайное").size(11.0)).clicked() {
                plant.fire_event(EventKind::Random, rng);
            }
        });
        if ui
            .add(egui::Button::new(RichText::new("Устранить течь / изолировать паропровод").size(11.0)))
            .clicked()
        {
            plant.close_break();
        }

        // ============== AZ ==============
        section(ui, "АВАРИЙНАЯ ЗАЩИТА (АЗ)");
        use crate::core::types::AzState;
        ui.label(
            RichText::new(match plant.az_state {
                AzState::Armed => "АЗ: ВКЛ",
                AzState::Tripped => "АЗ: СРАБОТАЛА",
                AzState::Disabled => "АЗ: ОТКЛЮЧЕНА",
            })
                        .size(13.0)
            .color(match plant.az_state {
                AzState::Armed => theme::GREEN,
                AzState::Tripped => theme::RED,
                AzState::Disabled => theme::AMBER,
            }),
        );
        ui.horizontal(|ui| {
            let label = if plant.az_state == AzState::Disabled {
                "Включить АЗ"
            } else {
                "Отключить АЗ"
            };
            if ui
                .add_enabled(plant.az_state != AzState::Tripped, egui::Button::new(RichText::new(label).size(11.0)))
                .clicked()
            {
                plant.toggle_az();
            }
            if ui
                .add_enabled(
                    matches!(plant.az_state, AzState::Tripped | AzState::Disabled),
                    egui::Button::new(RichText::new("Взвести АЗ").size(11.0)),
                )
                .clicked()
            {
                plant.reset_scram();
            }
        });
        ui.horizontal(|ui| {
            let az_col = if plant.scram {
                Color32::from_rgb(0x8a, 0x15, 0x15)
            } else {
                Color32::from_rgb(0x5a, 0x10, 0x10)
            };
            for key in ["АЗ-1", "АЗ-2"] {
                let btn = egui::Button::new(RichText::new(key).strong().size(16.0).color(Color32::from_rgb(0xff, 0xd7, 0xd7)))
                    .fill(az_col)
                    .stroke(egui::Stroke::new(2.0, Color32::from_rgb(0xaa, 0x11, 0x11)))
                    .min_size(egui::vec2(52.0, 34.0));
                if ui.add(btn).clicked() {
                    plant.scram();
                }
            }
            ui.label(RichText::new("ключи аварийной защиты").size(9.0).color(theme::DIM));
        });
    });
    mouse_dir
}

fn hold_button(ui: &mut egui::Ui, label: &str) -> bool {
    let r = ui.button(RichText::new(label).size(11.0));
    r.is_pointer_button_down_on()
}

/// Port of the Java `applyRods()`: validate %, set selected rods, log.
fn apply_rods(plant: &mut Plant, rod_sel: &mut [bool; 9], rod_input: &RodInput) {
    let sel: Vec<usize> = (0..9).filter(|&k| rod_sel[k]).collect();
    if sel.is_empty() {
        plant.add_log(
            "Стержни: не выбран ни один стержень — кликните по стержню на круге",
            LogClass::Warn,
        );
        return;
    }
    let pct: i32 = match rod_input.pct.trim().parse() {
        Ok(v) => v,
        Err(_) => {
            plant.add_log("Стержни: некорректный % извлечения", LogClass::Warn);
            return;
        }
    };
    if !(0..=100).contains(&pct) {
        plant.add_log("Стержни: % извлечения вне диапазона 0–100", LogClass::Warn);
        return;
    }
    plant.set_rods(&sel, 100.0 - pct as f64);
    for &k in &sel {
        rod_sel[k] = false;
    }
    if plant.ark {
        plant.add_log("Внимание: АРК ВКЛ — регулятор перепозиционирует стержни", LogClass::Warn);
    }
    let list: Vec<String> = sel.iter().map(|k| (k + 1).to_string()).collect();
    plant.add_log(
        format!(
            "Стержни {}: извлечение {}% (ввод {}%)",
            list.join(" "),
            pct,
            100 - pct
        ),
        LogClass::Ok,
    );
}
