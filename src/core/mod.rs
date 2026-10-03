//! NPPsim domain layer: pure physics/logic, no GUI dependencies.

pub mod constants;
pub mod events;
pub mod fuel;
pub mod physics;
pub mod protection;
pub mod rods;
pub mod scenarios;
pub mod selftest;
pub mod state;
pub mod types;

pub use events::{EventKind, Rng};
pub use scenarios::Scenario;
pub use state::{Alarm, LogEntry, Plant, TrendPoint};
pub use types::{AzState, BruMode, LogClass, RodSpeed, Task, TaskKind, TaskState};

/// Progress the whole simulation by `dt` (already rate-adjusted) seconds.
impl Plant {
    pub fn advance(&mut self, dt: f64) {
        if !self.dead {
            self.physics(dt);
        }
        self.time += dt;
        self.samp_t += dt;
        if self.samp_t >= 1.0 {
            self.samp_t = 0.0;
            self.trends.push([self.p, self.t_core, self.p1, self.mwe]);
            if self.trends.len() > 1500 {
                self.trends.remove(0);
            }
        }
        self.update_tasks(dt);
    }
}

/// Audible alert. Windows: system MessageBeep (no external crates); else silent.
pub fn beep() {
    #[cfg(windows)]
    {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn MessageBeep(utype: u32) -> i32;
        }
        const MB_ICONASTERISK: u32 = 0x0000_0030;
        unsafe {
            MessageBeep(MB_ICONASTERISK);
        }
    }
}
