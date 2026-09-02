use std::path::PathBuf;

use xerv_core::{api_version, CoreConfig, XervCore};

#[test]
fn end_to_end_load_config_build_core_shutdown() {
    let dir = tempfile::tempdir().unwrap();
    let cfg_path: PathBuf = dir.path().join("xerv.toml");
    std::fs::write(
        &cfg_path,
        format!(
            r#"
data_dir = "{}"
log_level = "error"
state_filename = "core.json"
"#,
            dir.path().display()
        ),
    )
    .unwrap();

    let cfg = CoreConfig::load(Some(&cfg_path)).unwrap();
    let state_path = cfg.data_dir.join(&cfg.state_filename);

    let core = XervCore::new(cfg, state_path.clone()).unwrap();
    assert_eq!(api_version().major, core.api_version().major);
    core.shutdown().unwrap();
    assert!(state_path.exists());
}
