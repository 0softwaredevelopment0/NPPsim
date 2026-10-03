//! Integration test: full physics suite (same checks as `NPPsim --selftest`).

#[test]
fn physics_selftest() {
    let report = nppsim::core::selftest::run_all();
    assert_eq!(report.failed, 0, "self-test failures: {}", report.failed);
}
