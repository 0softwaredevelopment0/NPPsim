//! Unit tests for domain edge cases not covered by the scenario selftest:
//! BRU-A hysteresis, task engine dedup/fail-kinds, AZ state machine,
//! withdrawal % parsing.

use nppsim::core::rods::parse_withdrawal_pct;
use nppsim::core::types::{AzState, BruMode, Task, TaskKind, TaskState};
use nppsim::core::{Plant, Scenario};

fn fresh_full() -> Plant {
    let mut p = Plant::new();
    p.load_scenario(Scenario::Full);
    p
}

// ================= BRU-A hysteresis =================

#[test]
fn bru_auto_opens_at_high_p2_and_stays_open_between_745_and_759() {
    let mut p = fresh_full();
    p.bru_mode = BruMode::Auto;
    p.p2 = 7.59;
    assert!(p.bru_open(), "must open at P2 >= 7.59");
    // drop into the hysteresis band: latch keeps it open
    p.p2 = 7.50;
    assert!(p.bru_open(), "hysteresis band keeps the latch open");
    p.p2 = 7.46;
    assert!(p.bru_open(), "still open at 7.45 < P2 < 7.59");
}

#[test]
fn bru_auto_closes_below_745_after_being_open() {
    let mut p = fresh_full();
    p.bru_mode = BruMode::Auto;
    p.p2 = 7.60;
    assert!(p.bru_open());
    p.p2 = 7.44;
    assert!(!p.bru_open(), "below 7.45 the latch releases");
}

#[test]
fn bru_auto_stays_closed_while_p2_below_open_threshold() {
    let mut p = fresh_full();
    p.bru_mode = BruMode::Auto;
    p.p2 = 7.50; // never reached 7.59
    assert!(!p.bru_open(), "in the band but was never open -> stays closed");
}

#[test]
fn bru_manual_follows_the_button_and_off_is_always_closed() {
    let mut p = fresh_full();
    p.bru_mode = BruMode::Manual;
    p.bru_manual = true;
    assert!(p.bru_open());
    p.bru_manual = false;
    assert!(!p.bru_open());
    p.bru_manual = true;
    p.bru_mode = BruMode::Off;
    assert!(!p.bru_open(), "OFF mode ignores the manual latch");
}

// ================= task engine =================

#[test]
fn add_task_deduplicates_by_text() {
    let mut p = fresh_full();
    p.add_task(Task::new("Test goal", TaskKind::GridConnected, 0.0, 0.0));
    p.add_task(Task::new("Test goal", TaskKind::GridConnected, 0.0, 0.0));
    assert_eq!(p.tasks.iter().filter(|t| t.text == "Test goal").count(), 1);
}

#[test]
fn no_scram_task_fails_on_scram() {
    let mut p = fresh_full();
    p.tasks.clear();
    p.add_task(Task::new("Не допускайте срабатывания АЗ", TaskKind::NoScram, 0.0, 0.0));
    p.scram();
    p.update_tasks(0.1);
    assert_eq!(p.tasks[0].state, TaskState::Failed);
}

#[test]
fn no_damage_task_waits_for_others_and_fails_on_death() {
    // fail-kind completes only when every other goal is done
    let mut p = fresh_full();
    p.tasks.clear();
    p.p = 10.0; // keeps the power goal unmet for a while
    p.add_task(Task::new("Не допускайте повреждения", TaskKind::NoDamage, 0.0, 0.0));
    p.add_task(Task::new("Мощность >= 50%", TaskKind::PowerAbove, 50.0, 2.0));
    for _ in 0..20 {
        p.update_tasks(0.1);
    }
    assert_eq!(
        p.tasks[0].state,
        TaskState::Active,
        "NoDamage must not complete while another goal is still active"
    );
    p.p = 60.0; // now the power goal can complete (2 s of holding)
    for _ in 0..30 {
        p.update_tasks(0.1);
    }
    assert_eq!(p.tasks[0].state, TaskState::Done, "completes once the others are done");

    // and fails if the core dies first
    let mut q = fresh_full();
    q.tasks.clear();
    q.add_task(Task::new("Не допускайте повреждения", TaskKind::NoDamage, 0.0, 0.0));
    q.dead = true;
    q.update_tasks(0.1);
    assert_eq!(q.tasks[0].state, TaskState::Failed);
}

#[test]
fn power_goal_needs_holding_for_the_required_time() {
    let mut p = fresh_full();
    p.tasks.clear();
    p.add_task(Task::new("Мощность >= 50%", TaskKind::PowerAbove, 50.0, 5.0));
    p.p = 60.0;
    p.update_tasks(1.0);
    assert_eq!(p.tasks[0].state, TaskState::Active);
    assert!((p.tasks[0].held - 1.0).abs() < 1e-9);
    for _ in 0..4 {
        p.update_tasks(1.0);
    }
    assert_eq!(p.tasks[0].state, TaskState::Done, "5 s of holding completes the goal");
    // condition lost -> held resets
    let mut q = fresh_full();
    q.tasks.clear();
    q.add_task(Task::new("Мощность >= 50%", TaskKind::PowerAbove, 50.0, 5.0));
    q.p = 60.0;
    q.update_tasks(3.0);
    q.p = 10.0;
    q.update_tasks(1.0);
    assert!((q.tasks[0].held - 0.0).abs() < 1e-9, "held resets when the condition breaks");
}

// ================= AZ state machine =================

#[test]
fn toggle_az_is_blocked_while_tripped() {
    let mut p = fresh_full();
    p.scram();
    assert_eq!(p.az_state, AzState::Tripped);
    p.toggle_az();
    assert_eq!(p.az_state, AzState::Tripped, "tripped AZ must be re-armed, not toggled");
}

#[test]
fn reset_scram_rearms_from_tripped_and_disabled() {
    let mut p = fresh_full();
    p.scram();
    p.reset_scram();
    assert_eq!(p.az_state, AzState::Armed);
    assert!(!p.scram);

    p.toggle_az(); // Armed -> Disabled
    assert_eq!(p.az_state, AzState::Disabled);
    p.reset_scram();
    assert_eq!(p.az_state, AzState::Armed);
}

#[test]
fn disabled_az_stays_silent_while_the_protection_condition_fires() {
    let mut p = fresh_full();
    p.toggle_az();
    // toggle_az registers the "AZ-OFF" alarm, so repeated protect() hits while
    // AZ is disabled stay silent (exact port of the Java guards).
    p.t_core = 100.0;
    let log_len = p.log.len();
    for _ in 0..100 {
        p.physics(0.1);
        p.time += 0.1;
    }
    assert!(p.p1 < 5.0, "precondition: the low-pressure condition fires");
    assert!(!p.scram, "disabled AZ does not trip");
    assert_eq!(p.az_state, AzState::Disabled);
    let silent: Vec<_> = p.log[log_len..]
        .iter()
        .filter(|e| e.msg.contains("НЕ сработала"))
        .collect();
    assert!(
        silent.is_empty(),
        "no extra protection-failure spam after the toggle alarm"
    );
}

// ================= withdrawal % parsing =================

#[test]
fn withdrawal_parsing_accepts_0_100_and_rejects_the_rest() {
    assert_eq!(parse_withdrawal_pct("50"), Ok(50));
    assert_eq!(parse_withdrawal_pct(" 0 "), Ok(0));
    assert_eq!(parse_withdrawal_pct("100"), Ok(100));
    assert!(parse_withdrawal_pct("abc").is_err());
    assert!(parse_withdrawal_pct("-1").is_err());
    assert!(parse_withdrawal_pct("101").is_err());
    assert!(parse_withdrawal_pct("12.5").is_err());
}

// ================= scenario reset =================

#[test]
fn reset_state_clears_alarms_log_tasks_and_melt() {
    let mut p = fresh_full();
    p.loca = true;
    p.alarm("X", "test");
    p.add_task(Task::new("X-unique-goal", TaskKind::GridConnected, 0.0, 0.0));
    p.load_scenario(Scenario::Start); // triggers reset_state
    assert!(p.alarms.is_empty());
    assert!(!p.tasks.iter().any(|t| t.text == "X-unique-goal"), "old tasks are dropped");
    assert_eq!(p.melted_count(), 0);
    assert!(!p.loca);
    assert_eq!(p.az_state, AzState::Armed);
}
