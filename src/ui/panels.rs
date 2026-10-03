//! Right column bottom panels: operator tasks, alarms, journal.

use crate::core::{LogClass, Plant, TaskState};
use crate::ui::theme;
use egui::{Color32, RichText, ScrollArea};

pub fn tasks(ui: &mut egui::Ui, plant: &Plant) {
    panel(ui, "ЗАДАНИЯ ОПЕРАТОРА", 96.0, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            if plant.tasks.is_empty() {
                ui.label(RichText::new("○ Нет активных заданий").size(11.0).color(theme::DIM));
            }
            for t in &plant.tasks {
                let (ic, c) = match t.state {
                    TaskState::Done => ("✔", theme::GREEN),
                    TaskState::Failed => ("✘", theme::RED),
                    TaskState::Active => ("○", theme::DIM),
                };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(ic).strong().size(12.0).color(c));
                    ui.label(
                        RichText::new(&t.text)
                            .size(11.0)
                            .color(match t.state {
                                TaskState::Done => theme::TXT,
                                TaskState::Failed => theme::RED,
                                TaskState::Active => Color32::from_rgb(0x9f, 0xb4, 0xc6),
                            }),
                    );
                });
            }
        });
    });
}

pub fn alarms(ui: &mut egui::Ui, plant: &mut Plant) {
    panel(ui, "Аварийная сигнализация", 96.0, |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            if plant.alarms.is_empty() {
                ui.label(RichText::new("— нет активных сигналов —").size(10.0).color(theme::DIM));
            }
            for (_, a) in &plant.alarms {
                let bg = if a.warn {
                    Color32::from_rgb(0x1a, 0x14, 0x0a)
                } else {
                    Color32::from_rgb(0x16, 0x0b, 0x0b)
                };
                let fg = if a.ack {
                    theme::DIM
                } else if a.warn {
                    theme::AMBER
                } else {
                    theme::RED
                };
                let txt = format!("{}  {}", theme::fmt_time(a.t), a.text);
                egui::Frame::NONE
                    .fill(bg)
                    .inner_margin(egui::Margin::symmetric(4, 1))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(txt).size(11.0).color(fg).monospace());
                    });
            }
        });
        if ui.add(egui::Button::new(RichText::new("Квитировать").size(11.0))).clicked() {
            plant.ack_all();
        }
    });
}

pub fn journal(ui: &mut egui::Ui, plant: &Plant) {
    panel(ui, "Журнал оператора", 140.0, |ui| {
        ScrollArea::vertical()
            .auto_shrink(false)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for e in &plant.log {
                    let (c, bold) = match e.cls {
                        LogClass::Crit => (theme::RED, true),
                        LogClass::Warn => (theme::AMBER, false),
                        LogClass::Ok => (theme::GREEN, false),
                        LogClass::Info => (Color32::from_rgb(0x9f, 0xb4, 0xc6), false),
                    };
                    let line = format!("{}  {}", theme::fmt_time(e.t), e.msg);
                    let mut rt = RichText::new(line).size(11.0).color(c).monospace();
                    if bold {
                        rt = rt.strong();
                    }
                    ui.label(rt);
                }
            });
    });
}

fn panel(ui: &mut egui::Ui, title: &str, height: f32, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::NONE
        .fill(theme::PANEL)
        .stroke(egui::Stroke::new(1.0, theme::LINE))
        .inner_margin(egui::Margin::same(4))
        .show(ui, |ui| {
            ui.set_min_height(height);
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).strong().size(10.0).color(theme::CYAN));
            add(ui);
        });
}
