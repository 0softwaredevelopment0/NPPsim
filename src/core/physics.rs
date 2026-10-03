//! Physics integration: point kinetics, xenon, both circuits, protections,
//! damage and fuel melting. Exact port of the original `step()`.

use super::constants::*;
use super::state::Plant;
use super::types::{AzState, BruMode, LogClass, RodSpeed};

impl Plant {
    /// Advance physics by `dt` seconds (internally split into <=0.5 s substeps).
    pub fn physics(&mut self, dt: f64) {
        let n = ((dt / 0.5).ceil() as usize).max(1);
        let h = dt / n as f64;
        for _ in 0..n {
            self.step(h);
        }
    }

    fn step(&mut self, h: f64) {
        // --- flows from real pump switches ---
        self.pump = self.pump_flow();
        self.feed = self.feed_flow();

        // --- rods (individual positions, I = mean insertion) ---
        if self.scram {
            self.move_all_rods(SCRAM_RATE * h);
        } else if self.rod_dir != 0 {
            let d = self.rod_dir as f64 * self.rod_sp.rate() * h;
            self.move_all_rods(d);
        }
        if self.ark && !self.scram && self.rod_dir == 0 {
            let sp = RodSpeed::Norm.rate();
            if self.p < self.p_set - 0.3 {
                self.move_all_rods(-sp * h);
            } else if self.p > self.p_set + 0.3 {
                self.move_all_rods(sp * h);
            }
        }

        let rho_rod = -rod_worth_at(self.ins);
        let rho_t = -ALPHA_T * (self.t_core - 310.0);
        let rho_xe = -self.xe * 1.792e-4;
        let rho_b = -KB * (self.b - 6.0);
        let rho = rho_rod + rho_t + rho_xe + rho_b;

        // --- power kinetics ---
        let dp = self.p * rho / TAU * h;
        self.p = clamp(self.p + dp, 0.05, 250.0);
        if self.scram && self.t_scram >= 0.0 {
            let e = (self.time - self.t_scram) / 2000.0;
            self.p = self.p.max(6.5 * (-e).exp() + 0.5);
        }
        let pth = self.p / 100.0 * RATED;

        // --- xenon / iodine ---
        self.io += (KI * self.p - LI * self.io) * h;
        self.xe += (LI * self.io + KX * self.p - LX * self.xe - BURN * self.xe * self.p) * h;
        if self.xe < 0.0 {
            self.xe = 0.0;
        }

        // --- primary circuit: core temperature ---
        let flow_eff = self.pump + if self.pump < 0.02 { 0.1 } else { 0.0 };
        let leak_f = if self.loca { 0.55 } else { 1.0 };
        let q_rem = KHX * flow_eff * leak_f * (self.t_core - self.t_sg);
        self.t_core += (pth - q_rem) / C1 * h;

        // --- primary pressure ---
        if self.loca {
            self.leak_p += 3.0 * h;
        } else {
            self.leak_p = (self.leak_p - 0.2 * h).max(0.0);
        }
        if self.makeup {
            self.kd_slip = (self.kd_slip - 0.002 * h).max(0.0);
        } else {
            self.kd_slip += 0.0015 * h;
        }
        let mut p1 = 15.7 + 0.35 * (self.t_core - 310.0) - self.leak_p - self.kd_slip;
        if self.rel_p {
            p1 = p1.min(18.6);
            p1 -= 0.3 * h;
            if p1 <= 18.6 && !(self.t_core > 342.0) {
                self.rel_p = false;
            }
        }
        self.p1 = clamp(p1, 0.1, 21.0);
        if self.p1 > P1_RELIEF {
            self.rel_p = true;
            self.alarm(
                "P-1",
                format!(
                    "Сработал предохранительный клапан 1-го контура (P={:.1} МПа)",
                    self.p1
                ),
            );
        }
        if self.p1 > 18.0 {
            self.protect("P1>18 МПа", "Срабатывание АЗ по давлению 1-го контура");
        }
        if self.p1 < 5.0 {
            self.protect("P1<5 МПа", "Срабатывание АЗ по низкому давлению 1-го контура");
        }
        if self.p1 < 0.8 {
            self.dmg_p += h;
        } else {
            self.dmg_p = 0.0;
        }

        // --- secondary circuit ---
        let mut q_steam = 3000.0 * self.p2_steam_frac();
        if self.steam_break {
            q_steam += 3500.0;
        }
        let bru_open = self.bru_open();
        if bru_open {
            q_steam += 3000.0;
        }
        if self.rel_s {
            q_steam += 3000.0;
        }
        self.t_sg += (q_rem - q_steam) / C2 * h;
        self.p2 = self.sat_p2(self.t_sg);
        if self.steam_break {
            self.p2 = self.p2.min(1.0);
            self.level = (self.level - 1.5 * h).max(0.0);
        }
        // GPZ trips only if BRU-A is not dumping steam (actual P2)
        let bru_now =
            (self.bru_mode == BruMode::Auto && self.p2 >= 7.59) || (self.bru_mode == BruMode::Manual && self.bru_manual);
        if self.p2 >= 7.6 && !bru_now && !self.rel_s {
            self.rel_s = true;
            self.gpz_count += 1;
            self.alarm("S-1", "Сработали ГПЗ (сброс пара в атмосферу)");
        }
        if self.rel_s && self.p2 < 7.4 {
            self.rel_s = false;
        }
        if bru_now && self.alarm_get("BRU").is_none() {
            self.alarm_warn("BRU", "БРУ-А открыт: сброс пара в конденсатор", true);
        }

        // --- SG level ---
        let sf = self.p2_steam_frac().clamp(0.0, 1.5);
        self.level = clamp(self.level + (self.feed - sf) * 3.0 * h, 0.0, 120.0);
        if self.level < 15.0 {
            if self.gov > 0.0 {
                self.trip_turbine("Защита ПГ: низкий уровень");
            }
            if self.p > 30.0 {
                self.protect("уровень ПГ<15%", "АЗ по низкому уровню ПГ");
            }
        }
        if self.level < 10.0 {
            self.protect("уровень ПГ<10%", "АЗ по низкому уровню ПГ");
        }
        if self.p2 < 1.2 && self.gov > 0.0 {
            self.trip_turbine("Падение давления пара");
        }
        if self.level < 25.0 || self.level > 85.0 {
            self.alarm("L-1", format!("Уровень ПГ за пределами: {}%", self.level.round()));
        }

        // --- turbine and generator ---
        let mwe_avail = pth * EFF;
        self.mwe = if self.gov > 0.0 && self.breaker {
            mwe_avail.min(q_steam * EFF).max(0.0)
        } else {
            0.0
        };
        self.f = F0 + if self.breaker {
            clamp((self.mwe - 1000.0) / 30000.0, -0.3, 0.3)
        } else {
            0.0
        };

        // --- protections ---
        if self.p1 > P1_RELIEF {
            self.alarm("P-2", "Предельное давление 1-го контура!");
        }
        if self.p > 115.0 {
            self.protect("P>115%", "Срабатывание АЗ по превышению мощности");
        }
        if self.t_core > T_WARN {
            self.alarm(
                "T-1",
                format!("Превышение температуры активной зоны: {}°C", self.t_core.round()),
            );
            if self.t_core > 350.0 {
                self.protect("Tcore>350°C", "АЗ по температуре активной зоны");
            }
        }
        if self.t_core > T_DAMAGE {
            self.dmg_t += h;
        } else {
            self.dmg_t = 0.0;
        }
        if flow_eff < 0.25 && self.t_core > 350.0 {
            self.dmg_f += h;
        } else {
            self.dmg_f = 0.0;
        }
        if flow_eff < 0.25 && self.p > 30.0 {
            self.protect("потеря расхода", "АЗ по потере расхода 1-го контура");
        }
        let n_gcn = self.gcn_count();
        if n_gcn <= 2 && self.p > 10.0 {
            self.protect("отключение 2 ГЦН", "АЗ по отключению двух ГЦН");
        }
        if n_gcn <= 3 && self.p > 75.0 {
            self.protect("отключение ГЦН на мощности", "АЗ: отключение ГЦН при мощности > 75%");
        }

        // --- core damage ---
        if (self.dmg_t > 30.0 || self.dmg_p > 30.0 || self.dmg_f > 90.0) && !self.dead {
            self.dead = true;
            self.scram = true;
            self.gov = 0.0;
            self.mwe = 0.0;
            let cause = if self.dmg_t > 30.0 {
                "расплавление активной зоны из-за перегрева"
            } else if self.dmg_p > 30.0 {
                "обнажение активной зоны из-за падения давления"
            } else {
                "перегрев активной зоны при потере расхода"
            };
            self.add_log(format!("КРИТ: повреждение активной зоны — {cause}"), LogClass::Crit);
            self.alarm("R-1", "РАДИАЦИОННАЯ АВАРИЯ! Повреждение активной зоны");
        }

        // --- fuel temperature and assembly melting ---
        let mut fmax: f64 = 0.0;
        for i in 0..CORE_N {
            for j in 0..CORE_N {
                let tf = self.fuel_temp(i, j);
                fmax = fmax.max(tf);
                if tf > MELT_T {
                    self.melt[i][j] =
                        (self.melt[i][j] + (tf - MELT_T) / MELT_T * h / 60.0).min(1.0);
                    if self.melt[i][j] >= 1.0 && !self.dead {
                        self.dead = true;
                        self.scram = true;
                        self.gov = 0.0;
                        self.mwe = 0.0;
                        self.add_log(
                            format!(
                                "КРИТ: РАСПЛАВЛЕНИЕ ТОПЛИВА в ТК ({},{}) — темп. {}°C > {}°C",
                                i + 1,
                                j + 1,
                                tf.round(),
                                MELT_T.round()
                            ),
                            LogClass::Crit,
                        );
                        self.alarm("R-2", "РАДИАЦИОННАЯ АВАРИЯ: расплавление топлива активной зоны");
                    }
                }
            }
        }
        if fmax > 2600.0 {
            self.alarm("T-3", format!("Предельная температура топлива: {}°C", fmax.round()));
        }
    }

    // ==== helpers mirroring the original private methods ====

    /// SG saturation pressure approximation, MPa.
    fn sat_p2(&self, t: f64) -> f64 {
        let x = t - 280.0;
        clamp(6.4 + 0.5 * x + 0.02 * x * x, 0.05, 7.6)
    }

    fn p2_steam_frac(&self) -> f64 {
        (self.gov / 0.9) * (self.p2 / 6.4)
    }

    /// True if the AZ state machine has the protection armed.
    pub fn az_armed(&self) -> bool {
        self.az_state == AzState::Armed
    }
}
