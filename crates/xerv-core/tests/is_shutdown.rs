use xerv_core::{CoreConfig, XervCore};

fn core(dir: &std::path::Path) -> XervCore {
    let cfg = CoreConfig {
        data_dir: dir.to_path_buf(),
        log_level: "warn".into(),
        state_filename: "s.json".into(),
    };
    XervCore::new(cfg, dir.join("s.json")).unwrap()
}

#[test]
fn fresh_core_is_not_shut_down() {
    let dir = tempfile::tempdir().unwrap();
    let c = core(dir.path());
    assert!(!c.is_shutdown());
}

#[test]
fn after_shutdown_is_shut_down() {
    let dir = tempfile::tempdir().unwrap();
    let c = core(dir.path());
    c.shutdown().unwrap();
    assert!(c.is_shutdown());
}

#[test]
fn double_shutdown_still_shut_down() {
    let dir = tempfile::tempdir().unwrap();
    let c = core(dir.path());
    c.shutdown().unwrap();
    let _ = c.shutdown(); // błąd, ale flaga zostaje
    assert!(c.is_shutdown());
}
