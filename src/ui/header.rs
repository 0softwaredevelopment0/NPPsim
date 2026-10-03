//! Top bar constants and status derivation.

use crate::core::Plant;
use crate::ui::theme;

pub const RATES: [f64; 4] = [1.0, 10.0, 60.0, 600.0];

/// Status colour/text derived from the plant state (port of `refresh()` logic).
pub fn plant_status(p: &Plant) -> (egui::Color32, &'static str) {
    if p.dead {
        (theme::RED, "АВАРИЯ")
    } else if p.scram {
        (theme::RED, "АЗ СРАБОТАЛА")
    } else if p.az_state == crate::core::types::AzState::Disabled {
        (theme::AMBER, "АЗ ОТКЛЮЧЕНА")
    } else if !p.alarms.is_empty() {
        (theme::AMBER, "ПРЕДУПРЕЖДЕНИЕ")
    } else {
        (theme::GREEN, "РАБОТА")
    }
}
