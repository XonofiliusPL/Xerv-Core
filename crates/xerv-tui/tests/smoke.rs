use xerv_core::api;
use xerv_core::api::CoreConfig;
use xerv_tui::app::App;

#[test]
fn api_imports_resolve() {
    let v = api::api_version();
    assert_eq!(v.major, 0);
    let _ = api::API_VERSION;
}

#[test]
fn try_new_smoke() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = CoreConfig {
        data_dir: dir.path().to_path_buf(),
        log_level: "warn".into(),
        state_filename: "state.json".into(),
    };
    let app = App::try_new(cfg, dir.path().join("state.json"));
    assert!(app.is_ok());
}
