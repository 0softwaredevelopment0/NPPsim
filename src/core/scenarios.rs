//! Shift scenarios: reset + initial conditions + operator task lists.

use super::state::Plant;
use super::types::{LogClass, Task, TaskKind};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scenario {
    /// "Работа 100%" — hold rated power.
    Full,
    /// "Пуск блока" — startup.
    Start,
    /// "Останов" — shutdown.
    Stop,
}

impl Plant {
    pub fn load_scenario(&mut self, k: Scenario) {
        self.reset_state();
        match k {
            Scenario::Full => {
                self.p = 100.0;
                self.t_core = 310.0;
                self.t_sg = 280.0;
                self.p1 = 15.7;
                self.p2 = 6.4;
                self.set_all_rods(3.0);
                self.gov = 0.9;
                self.breaker = true;
                self.b = 5.4;
                self.gcn = [true; 4];
                self.fen = [false; 3];
                self.fen[0] = true;
                self.fen[1] = true;
                self.xe = 1.674;
                self.io = 10.45;
                self.level = 50.0;
                self.rate = 1.0;
                self.ark = true;
                self.p_set = 100.0;
                self.add_log("=== НОВАЯ СМЕНА / СЦЕНАРИЙ: РАБОТА 100% ===", LogClass::Ok);
                self.add_log(
                    "Задание: удерживайте параметры в норме, используя все средства управления.",
                    LogClass::Info,
                );
                self.add_task(Task::new(
                    "Удерживайте мощность 90–110% (2 мин)",
                    TaskKind::PowerAbove,
                    90.0,
                    120.0,
                ));
                self.add_task(Task::new(
                    "Давление 1-го контура 15–18 МПа (2 мин)",
                    TaskKind::P1InRange,
                    0.0,
                    120.0,
                ));
                self.add_task(Task::new(
                    "Не допускайте срабатывания АЗ",
                    TaskKind::NoScram,
                    0.0,
                    0.0,
                ));
                self.add_task(Task::new(
                    "Не допускайте повреждения активной зоны",
                    TaskKind::NoDamage,
                    0.0,
                    0.0,
                ));
            }
            Scenario::Start => {
                self.p = 0.05;
                self.t_core = 280.0;
                self.t_sg = 275.0;
                self.p1 = 15.7;
                self.p2 = 4.4;
                self.set_all_rods(4.0);
                self.gov = 0.0;
                self.breaker = true;
                self.b = 6.0;
                self.gcn = [true; 4];
                self.fen = [false; 3];
                self.fen[0] = true;
                self.xe = 0.0;
                self.io = 0.0;
                self.level = 50.0;
                self.rate = 60.0;
                self.ark = false;
                self.p_set = 50.0;
                self.add_log("=== НОВАЯ СМЕНА / СЦЕНАРИЙ: ПУСК БЛОКА ===", LogClass::Ok);
                self.add_log(
                    "Задание: управляйте стержнями, бором, ГЦН, турбиной — выведите блок на мощность.",
                    LogClass::Info,
                );
                self.add_task(Task::new(
                    "Поднимите мощность выше 50% (30 с)",
                    TaskKind::PowerAbove,
                    50.0,
                    30.0,
                ));
                self.add_task(Task::new(
                    "Включите турбину — впуск пара > 0",
                    TaskKind::TurbineOn,
                    0.0,
                    0.0,
                ));
                self.add_task(Task::new(
                    "Выйдите на эл. мощность ≥ 950 МВт (30 с)",
                    TaskKind::MWeAbove,
                    950.0,
                    30.0,
                ));
                self.add_task(Task::new(
                    "Не допускайте срабатывания АЗ",
                    TaskKind::NoScram,
                    0.0,
                    0.0,
                ));
                self.add_task(Task::new(
                    "Не допускайте повреждения активной зоны",
                    TaskKind::NoDamage,
                    0.0,
                    0.0,
                ));
            }
            Scenario::Stop => {
                self.p = 100.0;
                self.t_core = 310.0;
                self.t_sg = 280.0;
                self.p1 = 15.7;
                self.p2 = 6.4;
                self.set_all_rods(3.0);
                self.gov = 0.9;
                self.breaker = true;
                self.b = 5.4;
                self.gcn = [true; 4];
                self.fen = [false; 3];
                self.fen[0] = true;
                self.fen[1] = true;
                self.xe = 1.674;
                self.io = 10.45;
                self.level = 50.0;
                self.rate = 10.0;
                self.ark = true;
                self.p_set = 20.0;
                self.add_log("=== НОВАЯ СМЕНА / СЦЕНАРИЙ: ОСТАНОВ ===", LogClass::Ok);
                self.add_log(
                    "Задание: остановите блок, используя АРК, стержни и управление турбиной.",
                    LogClass::Info,
                );
                self.add_task(Task::new(
                    "Снизьте мощность ниже 10% (1 мин)",
                    TaskKind::PowerBelow,
                    10.0,
                    60.0,
                ));
                self.add_task(Task::new(
                    "Введите стержни ОР СУЗ в зону (≥ 95%)",
                    TaskKind::RodsInserted,
                    95.0,
                    0.0,
                ));
                self.add_task(Task::new(
                    "Не допускайте срабатывания АЗ",
                    TaskKind::NoScram,
                    0.0,
                    0.0,
                ));
                self.add_task(Task::new(
                    "Не допускайте повреждения активной зоны",
                    TaskKind::NoDamage,
                    0.0,
                    0.0,
                ));
            }
        }
    }

    /// Reset to the base (cold-ish, default) state.
    pub fn reset_state(&mut self) {
        self.p = 0.05;
        self.t_core = 280.0;
        self.t_sg = 275.0;
        self.p1 = 15.7;
        self.p2 = 4.4;
        self.ins = 4.0;
        self.rod_dir = 0;
        self.rod_sp = super::types::RodSpeed::Norm;
        self.pump = 1.0;
        self.feed = 0.5;
        self.gov = 0.0;
        self.breaker = true;
        self.b = 6.0;
        self.xe = 0.0;
        self.io = 0.0;
        self.gcn = [true; 4];
        self.fen = [false; 3];
        self.fen[0] = true;
        self.fen[1] = true;
        self.bru_mode = super::types::BruMode::Auto;
        self.bru_manual = false;
        self.bru_was_open = false;
        self.makeup = true;
        self.gpz_count = 0;
        self.rod_ins = [4.0; super::constants::ROD_N];
        self.melt = [[0.0; super::constants::CORE_N]; super::constants::CORE_N];
        self.scram = false;
        self.t_scram = -1.0;
        self.ark = true;
        self.p_set = 100.0;
        self.steam_break = false;
        self.loca = false;
        self.rel_p = false;
        self.rel_s = false;
        self.level = 50.0;
        self.f = 50.0;
        self.mwe = 0.0;
        self.dmg_t = 0.0;
        self.dmg_p = 0.0;
        self.dmg_f = 0.0;
        self.leak_p = 0.0;
        self.kd_slip = 0.0;
        self.dead = false;
        self.samp_t = 0.0;
        self.az_state = super::types::AzState::Armed;
        self.alarms.clear();
        self.log.clear();
        self.trends.clear();
        self.tasks.clear();
        self.all_done = false;
    }
}
