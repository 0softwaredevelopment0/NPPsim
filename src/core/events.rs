//! Accident/event injection (training buttons) and the operator task engine.

use super::constants::{clamp, ROD_N};
use super::state::Plant;
use super::types::{LogClass, Task, TaskKind, TaskState};

/// Random source without external crates (xorshift64*), seeded from the clock.
pub struct Rng(u64);

impl Rng {
    pub fn new() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15);
        Rng(nanos | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventKind {
    Pump,
    Loca,
    Steam,
    Scram,
    Feed,
    Grid,
    Rod,
    Random,
}

impl Plant {
    pub fn fire_event(&mut self, k: EventKind, rng: &mut Rng) {
        let ev = match k {
            EventKind::Random => {
                let pool = [
                    EventKind::Pump,
                    EventKind::Loca,
                    EventKind::Steam,
                    EventKind::Scram,
                    EventKind::Feed,
                    EventKind::Grid,
                    EventKind::Rod,
                ];
                pool[rng.below(pool.len())]
            }
            other => other,
        };
        if self.dead {
            self.add_log("Блок в аварийном состоянии — загрузите сценарий", LogClass::Warn);
            return;
        }
        self.add_log("!!! ВОЗНИКЛО СОБЫТИЕ: НАСТУПИЛО", LogClass::Warn);
        match ev {
            EventKind::Pump => {
                for i in 0..4 {
                    if self.gcn[i] {
                        self.gcn[i] = false;
                        self.add_log(format!("Отказ ГЦН-{} — расход снизился", i + 1), LogClass::Crit);
                        break;
                    }
                }
                self.alarm("EV", "Отказ ГЦН");
                self.add_task(Task::new("Включите все ГЦН (4 из 4)", TaskKind::PumpFlow, 0.8, 20.0));
            }
            EventKind::Loca => {
                self.loca = true;
                self.add_log("Обнаружена течь 1-го контура! Давление падает", LogClass::Crit);
                self.alarm("EV", "Течь 1-го контура");
                self.add_task(Task::new(
                    "Устраните течь 1-го контура",
                    TaskKind::BreakFixed,
                    0.0,
                    0.0,
                ));
            }
            EventKind::Steam => {
                self.steam_break = true;
                self.add_log("Разрыв паропровода! Давление 2-го контура падает", LogClass::Crit);
                self.alarm("EV", "Разрыв паропровода");
                self.add_task(Task::new(
                    "Изолируйте разорванный паропровод",
                    TaskKind::BreakFixed,
                    0.0,
                    0.0,
                ));
            }
            EventKind::Scram => {
                self.do_scram("Ложное срабатывание АЗ (событие)");
            }
            EventKind::Feed => {
                for i in 0..3 {
                    if self.fen[i] {
                        self.fen[i] = false;
                        let name = if i == 2 { "МПНА".to_string() } else { format!("ПЭН-{}", i + 1) };
                        self.add_log(
                            format!("Отказ {name} — питательная вода снизилась"),
                            LogClass::Crit,
                        );
                        break;
                    }
                }
                self.alarm("EV", "Отказ питательного насоса");
                self.add_task(Task::new(
                    "Восстановите питательную воду ≥ 80%",
                    TaskKind::FeedAbove,
                    0.8,
                    20.0,
                ));
            }
            EventKind::Grid => {
                if self.breaker {
                    self.breaker = false;
                    self.add_log("Отключение от сети — генератор отпал", LogClass::Warn);
                    self.add_task(Task::new(
                        "Подключите генератор к сети",
                        TaskKind::GridConnected,
                        0.0,
                        0.0,
                    ));
                }
            }
            EventKind::Rod => {
                let rk = rng.below(ROD_N);
                self.rod_ins[rk] = clamp(self.rod_ins[rk] + 30.0, 0.0, 100.0);
                let s: f64 = self.rod_ins.iter().sum();
                self.ins = s / ROD_N as f64;
                self.add_log(
                    format!(
                        "Падение кассеты стержня {} — ввод {}%, реактивность снизилась",
                        rk + 1,
                        self.rod_ins[rk].round()
                    ),
                    LogClass::Warn,
                );
                self.alarm("EV", "Падение кассеты");
            }
            EventKind::Random => unreachable!(),
        }
    }

    // ================= operator task engine =================

    pub fn add_task(&mut self, t: Task) {
        if self.tasks.iter().any(|x| x.text == t.text) {
            return;
        }
        self.tasks.push(t);
        self.all_done = false;
    }

    /// Re-evaluate operator tasks; call every tick with the (rate-adjusted) dt.
    pub fn update_tasks(&mut self, dt: f64) {
        if self.tasks.is_empty() || self.all_done {
            return;
        }
        let others_done = self
            .tasks
            .iter()
            .all(|t| t.state != TaskState::Active || t.kind.is_fail_kind());

        // Phase 1: evaluate while borrowing tasks only.
        // Phase 1: evaluate conditions (immutable), then mutate task bookkeeping.
        let checks: Vec<bool> = self
            .tasks
            .iter()
            .map(|t| self.check_task(t.kind, t.target))
            .collect();
        let mut done: Vec<(usize, String)> = Vec::new();
        let mut failed: Vec<(usize, String)> = Vec::new();
        for (n, t) in self.tasks.iter_mut().enumerate() {
            if t.state != TaskState::Active {
                continue;
            }
            if t.kind == TaskKind::NoScram && self.scram {
                failed.push((n, t.text.clone()));
                continue;
            }
            if t.kind == TaskKind::NoDamage && self.dead {
                failed.push((n, t.text.clone()));
                continue;
            }
            if checks[n] {
                t.held += dt;
                let complete = if t.kind.is_fail_kind() {
                    others_done
                } else {
                    t.held >= t.hold
                };
                if complete {
                    done.push((n, t.text.clone()));
                }
            } else {
                t.held = 0.0;
            }
        }

        // Phase 2: apply state changes and journal entries.
        for (n, text) in failed {
            self.tasks[n].state = TaskState::Failed;
            self.add_log(format!("✘ ПРОВАЛ ЗАДАНИЯ: {text}"), LogClass::Crit);
        }
        for (n, text) in done {
            self.tasks[n].state = TaskState::Done;
            self.add_log(format!("✔ Задание выполнено: {text}"), LogClass::Ok);
        }
        if self.tasks.iter().all(|t| t.state == TaskState::Done) {
            self.all_done = true;
            self.add_log(
                "=== СМЕНА УСПЕШНО ЗАВЕРШЕНА — все задания выполнены ===",
                LogClass::Ok,
            );
        }
    }

    fn check_task(&self, kind: TaskKind, target: f64) -> bool {
        match kind {
            TaskKind::PowerAbove => self.p >= target,
            TaskKind::PowerBelow => self.p <= target,
            TaskKind::TurbineOn => self.gov > 0.0,
            TaskKind::MWeAbove => self.mwe >= target,
            TaskKind::RodsInserted => self.ins >= target,
            TaskKind::PumpFlow => self.pump >= target,
            TaskKind::BreakFixed => !self.loca && !self.steam_break,
            TaskKind::GridConnected => self.breaker,
            TaskKind::FeedAbove => self.feed >= target,
            TaskKind::P1InRange => self.p1 >= 15.0 && self.p1 <= 18.0,
            TaskKind::NoScram => !self.scram,
            TaskKind::NoDamage => !self.dead,
        }
    }
}
