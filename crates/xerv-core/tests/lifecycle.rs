use xerv_core::api::CoreConfig;
use xerv_core::api::XervCore;

#[test]
fn new_then_shutdown_writes_state_file() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = CoreConfig {
        data_dir: dir.path().to_path_buf(),
        log_level: "warn".into(),
        state_filename: "s.json".into(),
    };
    let state_path = cfg.data_dir.join(&cfg.state_filename);

    let core = XervCore::new(cfg, state_path.clone()).unwrap();
    assert_eq!(core.state().boot_count, 1);
    assert_eq!(core.api_version().minor, 2);

    core.shutdown().unwrap();
    assert!(state_path.exists());
}

#[test]
fn double_shutdown_is_error() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = CoreConfig {
        data_dir: dir.path().to_path_buf(),
        log_level: "warn".into(),
        state_filename: "s.json".into(),
    };
    let state_path = cfg.data_dir.join(&cfg.state_filename);

    let core = XervCore::new(cfg, state_path).unwrap();
    core.shutdown().unwrap();
    let r = core.shutdown();
    assert!(r.is_err());
}
