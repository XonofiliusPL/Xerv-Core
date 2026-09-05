use xerv_core::api::CoreConfig;

#[test]
fn config_default_has_expected_values() {
    let cfg = CoreConfig::default();
    assert_eq!(cfg.log_level, "info");
    assert_eq!(cfg.state_filename, "state.json");
}

#[test]
fn config_load_missing_returns_default() {
    let cfg =
        CoreConfig::load(Some(std::path::Path::new("/nonexistent/path/config.toml"))).unwrap();
    assert_eq!(cfg.log_level, "info");
}
