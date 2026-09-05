use xerv_core::api::CoreConfig;
use xerv_core::api::XervCore;

#[test]
fn debug_impl_contains_xervcore() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = CoreConfig {
        data_dir: dir.path().to_path_buf(),
        log_level: "warn".into(),
        state_filename: "s.json".into(),
    };
    let c = XervCore::new(cfg, dir.path().join("s.json")).unwrap();
    let s = format!("{c:?}");
    assert!(s.contains("XervCore"));
}
