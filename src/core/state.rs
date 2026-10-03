//! Plant state: reactor + primary + secondary circuits, alarms, log, tasks, trends.

use super::constants::{CORE_N, ROD_N};
use super::types::{AzState, BruMode, LogClass, RodSpeed, Task};

/// One acknowledged/unacknowledged alarm.
#[derive(Clone, Debug)]
pub struct Alarm {
    pub t: f64,
    pub text: String,
    pub warn: bool,
    pub ack: bool,
}

/// One operator journal line.
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub t: f64,
    pub msg: String,
    pub cls: LogClass,
}

/// Trend sample: [P %, Tcore °C, P1 MPa, MWe].
pub type TrendPoint = [f64; 4];

/// Full simulation state of the unit. Plain data + behaviour split across
/// the sibling modules (`rods`, `physics`, `fuel`, `protection`, `scenarios`,
/// `events`, `tasks`).
pub struct Plant {
    // ---- main process variables ----
    /// Reactor power, % nominal.
    pub p: f64,
    /// Core (coolant) temperature, °C.
    pub t_core: f64,
    /// Steam generator temperature, °C.
    pub t_sg: f64,
    /// Primary circuit pressure, MPa.
    pub p1: f64,
    /// Secondary circuit (SG steam) pressure, MPa.
    pub p2: f64,
    /// Mean rod insertion, % (0 — fully withdrawn, 100 — fully in core).
    pub ins: f64,
    /// Primary circuit flow, 0..1 (derived from MCP count).
    pub pump: f64,
    /// Feed water flow, 0..1 (derived from feed pump switches).
    pub feed: f64,
    /// Turbine steam inlet valve, 0..1.
    pub gov: f64,
    /// Boron concentration, g/kg.
    pub b: f64,
    /// Xenon / iodine inventories (model units).
    pub xe: f64,
    pub io: f64,
    /// Steam generator level, %.
    pub level: f64,
    /// Grid frequency, Hz.
    pub f: f64,
    /// Electric power, MW(e).
    pub mwe: f64,

    // ---- pressurizer / leaks ----
    pub leak_p: f64,
    /// APR (automatic power regulator) setpoint, %.
    pub p_set: f64,
    /// Accumulated pressure bleed with charging off.
    pub kd_slip: f64,

    // ---- operator controls ----
    /// Rod motion direction: -1 insert, 0 stop, +1 withdraw.
    pub rod_dir: i32,
    pub rod_sp: RodSpeed,
    /// Time acceleration factor.
    pub rate: f64,
    pub breaker: bool,
    pub scram: bool,
    pub loca: bool,
    /// Primary relief valve open.
    pub rel_p: bool,
    /// Atmospheric steam dump (GPZ) open.
    pub rel_s: bool,
    pub steam_break: bool,
    /// APR (automatic regulator) enabled.
    pub ark: bool,
    /// Unit lost (core damage / fuel melt).
    pub dead: bool,
    /// MCP-1..4 switches.
    pub gcn: [bool; 4],
    /// FP-1, FP-2, EFP switches.
    pub fen: [bool; 3],
    pub bru_mode: BruMode,
    pub bru_manual: bool,
    /// BRU-A hysteresis latch (open).
    pub bru_was_open: bool,
    /// Pressurizer charging.
    pub makeup: bool,
    /// How many times GPZ opened.
    pub gpz_count: u32,
    pub az_state: AzState,
    /// Moment of the last scram, -1 if none.
    pub t_scram: f64,
    /// Damage accumulators: overheat / low pressure / loss of flow.
    pub dmg_t: f64,
    pub dmg_p: f64,
    pub dmg_f: f64,
    /// Simulation time, s.
    pub time: f64,

    /// Per-rod insertion, % (0 — withdrawn, 100 — fully in core).
    pub rod_ins: [f64; ROD_N],
    /// Per-assembly melt fraction, 0..1.
    pub melt: [[f64; CORE_N]; CORE_N],

    // ---- alarms / journal / sound ----
    pub alarms: Vec<(String, Alarm)>,
    pub log: Vec<LogEntry>,
    pub sound: bool,

    // ---- operator tasks / trends ----
    pub tasks: Vec<Task>,
    pub(crate) all_done: bool,
    pub trends: Vec<TrendPoint>,
    pub(crate) samp_t: f64,
}

impl Default for Plant {
    fn default() -> Self {
        Plant::new()
    }
}

impl Plant {
    /// Fresh plant in the same state as the original Java `new Reactor()`.
    pub fn new() -> Self {
        Plant {
            p: 0.05,
            t_core: 280.0,
            t_sg: 275.0,
            p1: 15.7,
            p2: 4.4,
            ins: 4.0,
            pump: 1.0,
            feed: 0.5,
            gov: 0.0,
            b: 6.0,
            xe: 0.0,
            io: 0.0,
            level: 50.0,
            f: 50.0,
            mwe: 0.0,
            leak_p: 0.0,
            p_set: 100.0,
            kd_slip: 0.0,
            rod_dir: 0,
            rod_sp: RodSpeed::Norm,
            rate: 1.0,
            breaker: true,
            scram: false,
            loca: false,
            rel_p: false,
            rel_s: false,
            steam_break: false,
            ark: true,
            dead: false,
            gcn: [false; 4],
            fen: [false; 3],
            bru_mode: BruMode::Auto,
            bru_manual: false,
            bru_was_open: false,
            makeup: true,
            gpz_count: 0,
            az_state: AzState::Armed,
            t_scram: -1.0,
            dmg_t: 0.0,
            dmg_p: 0.0,
            dmg_f: 0.0,
            time: 0.0,
            rod_ins: [4.0; ROD_N],
            melt: [[0.0; CORE_N]; CORE_N],
            alarms: Vec::new(),
            log: Vec::new(),
            sound: true,
            tasks: Vec::new(),
            all_done: false,
            trends: Vec::new(),
            samp_t: 0.0,
        }
    }

    // ================= journal / alarms =================

    pub fn add_log(&mut self, msg: impl Into<String>, cls: LogClass) {
        self.log.push(LogEntry {
            t: self.time,
            msg: msg.into(),
            cls,
        });
        if self.log.len() > 400 {
            self.log.remove(0);
        }
    }

    pub fn alarm(&mut self, key: &str, text: impl Into<String>) {
        self.alarm_warn(key, text, false);
    }

    pub fn alarm_warn(&mut self, key: &str, text: impl Into<String>, warn: bool) {
        if self.alarm_get(key).is_some() {
            return;
        }
        self.alarms.push((
            key.to_string(),
            Alarm {
                t: self.time,
                text: text.into(),
                warn,
                ack: false,
            },
        ));
        self.beep();
    }

    pub fn alarm_get(&self, key: &str) -> Option<&Alarm> {
        self.alarms.iter().find(|(k, _)| k == key).map(|(_, a)| a)
    }

    pub fn ack_all(&mut self) {
        for (_, a) in &mut self.alarms {
            a.ack = true;
        }
    }

    /// Console/GUI beep — Windows: system MessageBeep, other platforms: silent.
    pub(crate) fn beep(&self) {
        if self.sound {
            super::beep();
        }
    }
}
