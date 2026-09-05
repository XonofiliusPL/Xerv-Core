use xerv_core::api::CoreConfig;
use xerv_core::api::XervCore;

#[test]
fn fresh_core_is_not_shut_down() {
    let dir = tempfile::tempdir().unwrap();
    let c = XervCore::new(CoreConfig::default(), dir.path().join("state.json")).unwrap();
    assert!(!c.is_shutdown());
}

#[test]
fn after_shutdown_is_shut_down() {
    let dir = tempfile::tempdir().unwrap();
    let c = XervCore::new(CoreConfig::default(), dir.path().join("state.json")).unwrap();
    c.shutdown().unwrap();
    assert!(c.is_shutdown());
}

#[test]
fn double_shutdown_still_shut_down() {
    let dir = tempfile::tempdir().unwrap();
    let c = XervCore::new(CoreConfig::default(), dir.path().join("state.json")).unwrap();
    c.shutdown().unwrap();
    let _ = c.shutdown();
    assert!(c.is_shutdown());
}
