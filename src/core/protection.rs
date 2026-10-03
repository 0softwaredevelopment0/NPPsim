//! Emergency protection (AZ) state machine, turbine trip, leak isolation.

use super::state::Plant;
use super::types::{AzState, LogClass};

impl Plant {
    /// Trip the reactor: rods in, turbine closed, AZ latched off.
    pub fn do_scram(&mut self, msg: impl Into<String>) {
        if self.scram {
            return;
        }
        self.scram = true;
        self.t_scram = self.time;
        self.gov = 0.0;
        self.az_state = AzState::Tripped;
        let msg = msg.into();
        self.add_log(format!("АВАРИЙНАЯ ЗАЩИТА: {msg}"), LogClass::Crit);
        self.add_log(
            "АЗ выключена после срабатывания — взведите её перед перезапуском",
            LogClass::Warn,
        );
        self.alarm("AZ", "Срабатывание АЗ — ввод стержней");
        self.beep();
    }

    /// Automatic protection: trips only when AZ is armed.
    pub(crate) fn protect(&mut self, cond: &str, msg: &str) {
        match self.az_state {
            AzState::Armed => self.do_scram(msg),
            AzState::Disabled => {
                if self.alarm_get("AZ-OFF").is_none() {
                    self.add_log(
                        format!("!! АЗ ОТКЛЮЧЕНА — {msg} НЕ сработала!"),
                        LogClass::Crit,
                    );
                    self.alarm_warn(
                        "AZ-OFF",
                        format!("АЗ отключена: защита не сработала ({cond})"),
                        false,
                    );
                }
            }
            AzState::Tripped => {}
        }
    }

    /// Operator: toggle AZ armed/disabled (disabling the protection is dangerous!).
    pub fn toggle_az(&mut self) {
        match self.az_state {
            AzState::Tripped => {
                self.add_log("АЗ сработала и защёлкнута — сначала взведите её", LogClass::Warn);
            }
            AzState::Armed => {
                self.az_state = AzState::Disabled;
                self.add_log(
                    "!! АЗ ОТКЛЮЧЕНА оператором — автозащита НЕ сработает",
                    LogClass::Crit,
                );
                self.alarm("AZ-OFF", "АЗ отключена оператором");
            }
            AzState::Disabled => {
                self.az_state = AzState::Armed;
                self.add_log("АЗ включена. Автозащита активна.", LogClass::Ok);
            }
        }
    }

    /// Turbine trip: closes the steam inlet.
    pub fn trip_turbine(&mut self, msg: &str) {
        if self.gov == 0.0 {
            return;
        }
        self.gov = 0.0;
        self.add_log(format!("ОСТАНОВ ТУРБИНЫ: {msg}"), LogClass::Warn);
        self.alarm("T-2", format!("Останов турбины: {msg}"));
        self.beep();
    }

    /// Operator manual AZ button (always works).
    pub fn scram(&mut self) {
        self.do_scram("Нажатие кнопки АЗ оператором");
    }

    /// Re-arm AZ after a trip, or re-arm a disabled AZ.
    pub fn reset_scram(&mut self) {
        if self.scram {
            self.scram = false;
            self.az_state = AzState::Armed;
            self.add_log(
                "АЗ взведена (сброс). Разрешено управление стержнями.",
                LogClass::Ok,
            );
            return;
        }
        if self.az_state == AzState::Disabled {
            self.az_state = AzState::Armed;
            self.add_log("АЗ включена. Автозащита активна.", LogClass::Ok);
        }
    }

    /// Repair the leak / isolate the broken steam line.
    pub fn close_break(&mut self) {
        if !self.loca && !self.steam_break {
            return;
        }
        self.loca = false;
        self.steam_break = false;
        self.add_log("Течь устранена / паропровод изолирован.", LogClass::Ok);
    }
}
