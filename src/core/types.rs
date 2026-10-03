//! Domain enums and the operator task record.

/// AZ (emergency protection) state machine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AzState {
    /// Automatic protection is armed and will trip on its own.
    Armed,
    /// Protection tripped and latched off until re-armed.
    Tripped,
    /// Operator disabled the automatic protection.
    Disabled,
}

/// Control rod motion speed setting.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RodSpeed {
    Slow,
    Norm,
    Fast,
}

impl RodSpeed {
    #[inline]
    pub fn rate(self) -> f64 {
        match self {
            RodSpeed::Slow => crate::core::constants::ROD_SLOW,
            RodSpeed::Norm => crate::core::constants::ROD_NORM,
            RodSpeed::Fast => crate::core::constants::ROD_FAST,
        }
    }
}

/// BRU-A (steam dump to condenser) mode selector.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BruMode {
    Off,
    Auto,
    Manual,
}

/// Operator log entry severity.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogClass {
    Info,
    Ok,
    Warn,
    Crit,
}

/// Kind of an operator goal — how `check_task` evaluates it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskKind {
    /// Power >= target %.
    PowerAbove,
    /// Power <= target %.
    PowerBelow,
    /// Turbine steam inlet > 0.
    TurbineOn,
    /// Electric power >= target MW.
    MWeAbove,
    /// Mean rod insertion >= target %.
    RodsInserted,
    /// Primary flow >= target (0..1).
    PumpFlow,
    /// Leak repaired / steam line isolated.
    BreakFixed,
    /// Generator breaker closed.
    GridConnected,
    /// Feed water >= target (0..1).
    FeedAbove,
    /// Primary pressure within 15..18 MPa.
    P1InRange,
    /// Fail-kind: must NOT scram.
    NoScram,
    /// Fail-kind: core must NOT be damaged.
    NoDamage,
}

impl TaskKind {
    /// Fail-kinds complete only when every other goal is done and fail on violation.
    #[inline]
    pub fn is_fail_kind(self) -> bool {
        matches!(self, TaskKind::NoScram | TaskKind::NoDamage)
    }
}

/// Operator task (training goal) state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskState {
    Active,
    Done,
    Failed,
}

/// One operator goal of a shift scenario.
#[derive(Clone, Debug)]
pub struct Task {
    pub text: String,
    pub kind: TaskKind,
    pub target: f64,
    /// Seconds the condition must hold.
    pub hold: f64,
    pub held: f64,
    pub state: TaskState,
}

impl Task {
    pub fn new(text: impl Into<String>, kind: TaskKind, target: f64, hold: f64) -> Self {
        Task {
            text: text.into(),
            kind,
            target,
            hold,
            held: 0.0,
            state: TaskState::Active,
        }
    }
}
