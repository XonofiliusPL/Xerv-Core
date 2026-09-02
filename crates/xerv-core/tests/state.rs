use xerv_core::CoreState;

#[test]
fn fresh_state_has_schema_v1_and_boot_1() {
    let s = CoreState::fresh();
    assert_eq!(s.schema_version, 1);
    assert_eq!(s.boot_count, 1);
    // Rok 2023+; test nie jest kruchy co do dokładnej daty.
    assert!(s.started_at_unix > 1_700_000_000);
}

#[test]
fn load_missing_returns_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("nope.json");
    let s = CoreState::load(&p).unwrap();
    assert_eq!(s.schema_version, 1);
}

#[test]
fn save_then_load_roundtrips() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("state.json");
    let mut s = CoreState::fresh();
    s.boot_count = 7;
    s.save(&p).unwrap();
    let loaded = CoreState::load(&p).unwrap();
    assert_eq!(loaded.boot_count, 7);
    assert_eq!(loaded.schema_version, 1);
}
