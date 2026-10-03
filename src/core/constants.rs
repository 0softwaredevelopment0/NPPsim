//! Physical constants of the VVER-1000 model (units match the original JS/Java model).

/// Rated thermal power, MW(th).
pub const RATED: f64 = 3000.0;
/// Power time constant, s.
pub const TAU: f64 = 1.5;
/// Full negative reactivity worth of all control rods.
pub const ROD_WORTH: f64 = 0.020;
/// Temperature (reactivity) coefficient, 1/°C.
pub const ALPHA_T: f64 = 1.5e-5;
/// Boron reactivity per 1 g/kg above the 6 g/kg nominal.
pub const KB: f64 = 1.0e-3;
/// Heat removal at full flow, MW/°C.
pub const KHX: f64 = 100.0;
/// Primary circuit heat capacity, MW·s/°C.
pub const C1: f64 = 15000.0;
/// Secondary circuit heat capacity, MW·s/°C.
pub const C2: f64 = 2200.0;
/// Nominal primary pressure / pressurizer relief setpoint, MPa.
pub const P1_SET: f64 = 15.7;
pub const P1_RELIEF: f64 = 18.6;
/// Core temperature alarm / damage thresholds, °C.
pub const T_WARN: f64 = 350.0;
pub const T_DAMAGE: f64 = 368.0;
/// Xenon/iodine chain and burnup rates.
pub const LI: f64 = 2.87e-5;
pub const LX: f64 = 2.1e-5;
pub const KI: f64 = 3.0e-6;
pub const KX: f64 = 7.0e-7;
pub const BURN: f64 = 2.0e-6;
/// Rod motion speeds, %/s (slow / normal / fast) and AZ scram rate.
pub const ROD_SLOW: f64 = 0.6;
pub const ROD_NORM: f64 = 2.5;
pub const ROD_FAST: f64 = 10.0;
pub const SCRAM_RATE: f64 = 45.0;
/// Thermal-to-electric efficiency.
pub const EFF: f64 = 0.333;
/// Nominal grid frequency, Hz.
pub const F0: f64 = 50.0;
/// Number of CPS control rods (individual withdrawal).
pub const ROD_N: usize = 9;
/// Fuel melting temperature, °C.
pub const MELT_T: f64 = 2800.0;
/// Core grid size (CORE_N × CORE_N fuel assemblies).
pub const CORE_N: usize = 9;

#[inline]
pub fn clamp(v: f64, a: f64, b: f64) -> f64 {
    v.max(a).min(b)
}

/// Rod worth as a function of mean insertion `I` (0..100 %).
#[inline]
pub fn rod_worth_at(i: f64) -> f64 {
    ROD_WORTH * (i / 100.0).powf(1.2)
}
