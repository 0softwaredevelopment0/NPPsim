//! Port of the original Java `PhysicsTest` self-test run (39 checks).
//! Used both by `NPPsim --selftest` and by `cargo test`.

use super::{Plant, Scenario};
use super::types::TaskState;

pub struct SelfTestReport {
    pub passed: usize,
    pub failed: usize,
}

impl SelfTestReport {
    fn check(&mut self, name: &str, ok: bool) {
        println!("{} {name}", if ok { "PASS" } else { "FAIL" });
        if ok {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
    }
}

fn fmt(s: &Plant) -> String {
    format!(
        "P={:.1}% T={:.1} P1={:.2} P2={:.2} MWe={:.0} xe={:.4} dmgT={:.0} dmgP={:.0} dead={}",
        s.p, s.t_core, s.p1, s.p2, s.mwe, s.xe, s.dmg_t, s.dmg_p, s.dead
    )
}

fn run(s: &mut Plant, sec: f64) {
    let n = (sec * 10.0) as i32;
    for _ in 0..n {
        s.physics(0.1);
        s.time += 0.1;
    }
}

/// Run the full suite, printing PASS/FAIL lines like the Java original.
pub fn run_all() -> SelfTestReport {
    let mut r = SelfTestReport { passed: 0, failed: 0 };
    let mut s = Plant::new();

    // 1. Stability at 100 % for 2 h
    s.load_scenario(Scenario::Full);
    run(&mut s, 7200.0);
    println!("steady@100% after 2h: {}", fmt(&s));
    r.check("power stays near 100%", (s.p - 100.0).abs() < 15.0);
    r.check("Tcore sane", s.t_core > 300.0 && s.t_core < 345.0);
    r.check("P1 sane", s.p1 > 15.2 && s.p1 < 18.0);
    r.check("P2 sane", s.p2 > 5.5 && s.p2 < 7.6);
    r.check("MWe sane", s.mwe > 850.0 && s.mwe < 1020.0);

    // 2. AZ: power drop, iodine pit, no damage
    let xe_before = s.xe;
    s.scram();
    run(&mut s, 900.0);
    println!("after scram 15min: {}", fmt(&s));
    r.check("power dropped after scram", s.p < 12.0);
    r.check("xenon pit appears (xe rises)", s.xe > xe_before * 1.05);
    r.check("no damage on clean scram", !s.dead);

    // 3. Xenon decay after the pit
    run(&mut s, 3600.0 * 20.0);
    println!("20h after scram: {}", fmt(&s));
    r.check("xenon decaying after pit", s.xe < 4.0);

    // 4. LOCA: pressure drop, damage
    s.load_scenario(Scenario::Full);
    s.loca = true;
    run(&mut s, 300.0);
    println!("LOCA after 5min: {}", fmt(&s));
    r.check("LOCA drops P1", s.p1 < 1.0);
    r.check("LOCA leads to damage", s.dead);

    // 5. Loss of flow: automatic AZ, no immediate damage
    s.load_scenario(Scenario::Full);
    s.gcn = [false; 4];
    run(&mut s, 900.0);
    println!("pumps off 15min: {}", fmt(&s));
    r.check("pump loss triggers auto scram", s.scram);
    r.check("pump loss does NOT immediately damage", !s.dead);

    // 6. Feed water loss: AZ, survivable
    s.load_scenario(Scenario::Full);
    s.fen = [false; 3];
    run(&mut s, 1800.0);
    println!("feed loss 30min: {}", fmt(&s));
    r.check("feed loss auto scram", s.scram);
    r.check("feed loss recoverable (no dead)", !s.dead);

    // 7. Reactivity excursion: AZ protects
    s.load_scenario(Scenario::Full);
    s.ark = false;
    s.b = 4.0;
    s.set_all_rods(0.0);
    run(&mut s, 1500.0);
    println!("reactivity excursion 25min: {}", fmt(&s));
    r.check("reactivity excursion triggers auto AZ", s.scram);
    r.check("reactivity excursion does NOT damage (AZ protects)", !s.dead);

    // 8. Startup: rod withdrawal + boron dilution raises power
    s.load_scenario(Scenario::Start);
    s.b = 5.5;
    s.set_all_rods(0.0);
    run(&mut s, 3600.0);
    println!("startup rods out + dilute 1h: {}", fmt(&s));
    r.check("startup power rose", s.p > 1.0);
    r.check("startup no damage", !s.dead);

    // 9. Steam line break: recovery after isolation
    s.load_scenario(Scenario::Full);
    s.steam_break = true;
    run(&mut s, 20.0);
    println!("steam break 20s: {}", fmt(&s));
    r.check("P2 collapsed", s.p2 < 5.5);
    s.fen = [true, true, false];
    s.close_break();
    run(&mut s, 1200.0);
    println!("after closing break 20min: {}", fmt(&s));
    r.check("system recovers after closing break", s.p > 50.0 && s.p2 > 5.5 && !s.dead);

    // 10. AZ: tripped -> latched off until re-armed
    s.load_scenario(Scenario::Full);
    r.check("AZ armed initially", s.az_armed());
    s.scram();
    r.check("AZ latched after trip", s.az_state == super::types::AzState::Tripped && s.scram);
    s.reset_scram();
    r.check("AZ re-armed after arm-up", s.az_armed() && !s.scram);

    // 11. Disabled AZ does not auto-trip, manual button still works
    s.load_scenario(Scenario::Full);
    s.toggle_az();
    r.check("AZ disabled by operator", s.az_state == super::types::AzState::Disabled);
    s.gcn = [true, true, false, false];
    run(&mut s, 120.0);
    r.check("disabled AZ does NOT auto-trip", !s.scram);
    r.check("no melt without extreme conditions", s.melted_count() == 0 && !s.dead);
    s.scram();
    r.check("manual scram works when AZ disabled", s.scram && s.az_state == super::types::AzState::Tripped);

    // 12. Tasks: engine completes operator goals
    s.load_scenario(Scenario::Start);
    s.p = 60.0;
    s.gov = 0.9;
    s.fen = [true, true, false];
    s.gcn = [true, true, true, true];
    s.set_all_rods(2.0);
    s.b = 5.5;
    for _ in 0..400 {
        s.advance(0.1); // 40 s of simulation
    }
    let mut p_done = false;
    let mut gov_done = false;
    for t in &s.tasks {
        if t.text.starts_with("Поднимите") && t.state == TaskState::Done {
            p_done = true;
        }
        if t.text.contains("турбину") && t.state == TaskState::Done {
            gov_done = true;
        }
    }
    r.check("task: power goal completed", p_done);
    r.check("task: turbine goal completed", gov_done);

    // 13. Individual rod withdrawal
    s.load_scenario(Scenario::Full);
    s.set_rods(&[0, 4], 100.0);
    r.check(
        "per-rod set changes mean insertion",
        (s.ins - (7.0 * 3.0 + 2.0 * 100.0) / 9.0).abs() < 1e-9,
    );
    s.set_all_rods(0.0);
    r.check("select-all extraction -> I=0", s.ins == 0.0);
    r.check("extraction readback 100%", (s.rod_extract_mean() - 100.0).abs() < 1e-9);
    s.set_rods(&[2], 100.0);
    r.check("single rod inserted -> mean 100/9", (s.ins - 100.0 / 9.0).abs() < 1e-9);

    // 14. At 100 % power fuel does NOT melt
    s.load_scenario(Scenario::Full);
    run(&mut s, 7200.0);
    println!(
        "fuel after 2h: max={:.1}°C melted={}",
        s.fuel_max(),
        s.melted_count()
    );
    r.check("fuel temp controlled at rated power", s.fuel_max() < 2800.0 && s.melted_count() == 0);

    // 15. Melting: AZ disabled + excursion -> fuel melt, accident
    s.load_scenario(Scenario::Full);
    s.toggle_az();
    s.ark = false;
    s.b = 3.5;
    s.set_all_rods(0.0);
    s.p = 220.0;
    run(&mut s, 150.0);
    println!(
        "excursion melt: max={:.1} melted={} dead={}",
        s.fuel_max(),
        s.melted_count(),
        s.dead
    );
    r.check("fuel melts on uncontrolled excursion", s.melted_count() >= 1 && s.dead);

    // 16. Power-trip AZ protects from melting
    s.load_scenario(Scenario::Full);
    s.ark = false;
    s.b = 4.0;
    s.set_all_rods(0.0);
    run(&mut s, 1500.0);
    println!(
        "excursion AZ on: P={:.1} scram={} dead={} melted={}",
        s.p,
        s.scram,
        s.dead,
        s.melted_count()
    );
    r.check("power-trip AZ prevents melt", s.scram && !s.dead && s.melted_count() == 0);

    // 17. MCP: individual switching, flow from pump count, AZ on two pumps off
    s.load_scenario(Scenario::Full);
    r.check("4 ГЦН -> расход 100%", (s.pump_flow() - 1.0).abs() < 1e-9);
    r.check("все ПЭН вкл -> пит. вода 100%", (s.feed_flow() - 1.0).abs() < 1e-9);
    s.gcn[3] = false;
    r.check("3 ГЦН -> расход 75%", (s.pump_flow() - 0.75).abs() < 1e-9);
    s.gcn[2] = false;
    s.gcn[1] = false;
    run(&mut s, 30.0);
    println!("2 GCN off: {}", fmt(&s));
    r.check("отключение 2 ГЦН на мощности -> АЗ", s.scram);

    // 18. BRU-A: AUTO holds SG pressure, OFF leads to GPZ (survivable)
    s.load_scenario(Scenario::Full);
    s.gov = 0.3;
    s.bru_mode = super::types::BruMode::Off;
    run(&mut s, 600.0);
    println!(
        "BRU OFF: P2={:.1} gpz={} dead={}",
        s.p2, s.gpz_count, s.dead
    );
    r.check("БРУ ОТКЛ -> ГПЗ перехватили сброс", s.gpz_count >= 1 && !s.dead);
    s.load_scenario(Scenario::Full);
    s.gov = 0.3;
    s.bru_mode = super::types::BruMode::Auto;
    run(&mut s, 600.0);
    println!(
        "BRU AUTO: P2={:.1} gpz={} bruOpen={}",
        s.p2, s.gpz_count, s.bru_open_readonly()
    );
    r.check("БРУ АВТ держит давление (без ГПЗ)", s.gpz_count == 0 && s.p2 > 6.0);
    // MANUAL: with the button open dumps the same as AUTO
    s.load_scenario(Scenario::Full);
    s.gov = 0.3;
    s.bru_mode = super::types::BruMode::Manual;
    s.bru_manual = true;
    run(&mut s, 600.0);
    r.check("БРУ РУЧ+открыта держит давление", s.gpz_count == 0 && s.p2 > 6.0);

    // 19. Pressurizer charging: off -> slow P1 bleed, unit survives
    s.load_scenario(Scenario::Full);
    s.makeup = false;
    run(&mut s, 3600.0);
    println!("no makeup 1h: P1={:.1}", s.p1);
    r.check("отключённая подпитка снижает P1", s.p1 < 15.5 && !s.dead);

    r
}

/// `--selftest` entry: prints the summary and returns process exit code.
pub fn main_selftest() -> i32 {
    let r = run_all();
    if r.failed == 0 {
        println!("\nALL TESTS PASSED");
        0
    } else {
        println!("\n{} TEST(S) FAILED", r.failed);
        1
    }
}
