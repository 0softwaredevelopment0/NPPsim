//! Core fuel-assembly temperatures and melting.

use super::constants::{clamp, CORE_N, MELT_T};
use super::state::Plant;

impl Plant {
    /// Assembly (FA) coolant temperature at grid cell (i, j): centre hotter than edges, with fixed noise.
    pub fn fa_temp(&self, i: usize, j: usize) -> f64 {
        let c = (CORE_N - 1) as f64 / 2.0;
        let d = ((i as f64 - c).hypot(j as f64 - c)) / c;
        let hot = 40.0 * clamp(self.p / 100.0, 0.0, 1.0) * (1.0 - d * d * 1.15).max(0.0);
        let noise = (((i * 7 + j * 13) % 5) as f64 - 2.0) * 1.4;
        (self.t_core - 8.0 + hot + noise).max(120.0)
    }

    /// Fuel temperature inside assembly (i, j): coolant + power/cooling-dependent rise.
    pub fn fuel_temp(&self, i: usize, j: usize) -> f64 {
        let flow_eff = self.pump + if self.pump < 0.02 { 0.1 } else { 0.0 };
        let leak_f = if self.loca { 0.55 } else { 1.0 };
        let cool = clamp(0.35 + 0.65 * flow_eff * leak_f, 0.35, 1.6);
        let c = (CORE_N - 1) as f64 / 2.0;
        let d = ((i as f64 - c).hypot(j as f64 - c)) / c;
        let radial = (1.0 - d * d * 1.15).max(0.0);
        let heat = 1400.0 * clamp(self.p / 100.0, 0.0, 2.5).powf(1.5) / cool;
        self.fa_temp(i, j) + heat * radial
    }

    /// Max fuel temperature over all assemblies, °C.
    pub fn fuel_max(&self) -> f64 {
        let mut m: f64 = 0.0;
        for i in 0..CORE_N {
            for j in 0..CORE_N {
                m = m.max(self.fuel_temp(i, j));
            }
        }
        m
    }

    /// Fully melted assembly count.
    pub fn melted_count(&self) -> usize {
        self.melt
            .iter()
            .flatten()
            .filter(|&&m| m >= 1.0)
            .count()
    }

    /// Total melted fuel fraction (0..N).
    pub fn melt_total(&self) -> f64 {
        self.melt.iter().flatten().sum()
    }

    /// Melt threshold getter for the UI.
    #[inline]
    pub fn melt_t() -> f64 {
        MELT_T
    }
}
