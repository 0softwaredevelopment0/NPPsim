//! CPS control rods (individual withdrawal) and pump/feed flow bookkeeping.

use super::constants::{clamp, ROD_N};
use super::state::Plant;
use super::types::{BruMode, RodSpeed};

impl Plant {
    /// Set insertion of the selected rods, % (0 — withdrawn, 100 — fully in core).
    pub fn set_rods(&mut self, idx: &[usize], pct: f64) {
        let v = clamp(pct, 0.0, 100.0);
        for &k in idx {
            if k < ROD_N {
                self.rod_ins[k] = v;
            }
        }
        self.recompute_i();
    }

    /// Set insertion of ALL rods, %.
    pub fn set_all_rods(&mut self, pct: f64) {
        let v = clamp(pct, 0.0, 100.0);
        self.rod_ins = [v; ROD_N];
        self.ins = v;
    }

    /// Shift all rods by `d_i` percent of insertion (for ▲/▼, APR, AZ).
    pub fn move_all_rods(&mut self, d_i: f64) {
        for r in &mut self.rod_ins {
            *r = clamp(*r + d_i, 0.0, 100.0);
        }
        self.recompute_i();
    }

    fn recompute_i(&mut self) {
        self.ins = self.rod_ins.iter().sum::<f64>() / ROD_N as f64;
    }

    /// Withdrawal of rod `k`, % (100 — fully withdrawn).
    #[inline]
    pub fn rod_extract(&self, k: usize) -> f64 {
        100.0 - self.rod_ins[k]
    }

    /// Mean withdrawal of all rods, %.
    #[inline]
    pub fn rod_extract_mean(&self) -> f64 {
        100.0 - self.ins
    }

    /// Number of running MCPs (main circulation pumps).
    pub fn gcn_count(&self) -> usize {
        self.gcn.iter().filter(|&&on| on).count()
    }

    /// Primary flow 0..1 from the number of running MCPs.
    #[inline]
    pub fn pump_flow(&self) -> f64 {
        self.gcn_count() as f64 / 4.0
    }

    /// Feed water 0..1: FP-1 50 %, FP-2 50 %, EFP 30 %.
    pub fn feed_flow(&self) -> f64 {
        let f: f64 = (if self.fen[0] { 0.5 } else { 0.0 })
            + (if self.fen[1] { 0.5 } else { 0.0 })
            + (if self.fen[2] { 0.3 } else { 0.0 });
        f.min(1.0)
    }

    /// Is BRU-A open? AUTO — by SG pressure (with hysteresis), MANUAL — by the button.
    /// Mutates the hysteresis latch, like the original model.
    pub fn bru_open(&mut self) -> bool {
        match self.bru_mode {
            BruMode::Manual => self.bru_manual,
            BruMode::Off => false,
            BruMode::Auto => {
                if self.p2 >= 7.59 {
                    self.bru_was_open = true;
                } else if self.p2 < 7.45 {
                    self.bru_was_open = false;
                }
                self.bru_was_open
            }
        }
    }

    /// Non-mutating readback of the BRU-A lamp state (for UI refresh).
    pub fn bru_open_readonly(&self) -> bool {
        match self.bru_mode {
            BruMode::Manual => self.bru_manual,
            BruMode::Off => false,
            BruMode::Auto => self.bru_was_open || self.p2 >= 7.59,
        }
    }

    #[inline]
    pub fn rod_speed(sp: RodSpeed) -> f64 {
        sp.rate()
    }
}

/// Parse a "% withdrawal" text input: 0..100 integer. Port of the validation
/// part of the Java GUI `applyRods()` (error strings match the original).
pub fn parse_withdrawal_pct(text: &str) -> Result<i32, &'static str> {
    let pct: i32 = text.trim().parse().map_err(|_| "Стержни: некорректный % извлечения")?;
    if !(0..=100).contains(&pct) {
        return Err("Стержни: % извлечения вне диапазона 0–100");
    }
    Ok(pct)
}
