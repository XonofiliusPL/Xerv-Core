use std::path::PathBuf;

use xerv_core::CoreConfig;

#[test]
fn default_config_is_usable() {
    let c = CoreConfig::default();
    assert_eq!(c.log_level, "info");
    assert_eq!(c.state_filename, "state.json");
    assert!(!c.data_dir.as_os_str().is_empty());
}

#[test]
fn load_none_returns_default() {
    let c = CoreConfig::load(None).unwrap();
    assert_eq!(c.log_level, "info");
}

#[test]
fn load_missing_path_returns_default() {
    let c = CoreConfig::load(Some(&PathBuf::from("/nope/nope.toml"))).unwrap();
    assert_eq!(c.log_level, "info");
}

#[test]
fn load_valid_toml_overrides_fields() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.toml");
    std::fs::write(
        &p,
        r#"
data_dir = "/tmp/x"
log_level = "debug"
state_filename = "x.json"
"#,
    )
    .unwrap();
    let c = CoreConfig::load(Some(&p)).unwrap();
    assert_eq!(c.log_level, "debug");
    assert_eq!(c.state_filename, "x.json");
    assert_eq!(c.data_dir, PathBuf::from("/tmp/x"));
}

#[test]
fn load_invalid_toml_errors() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.toml");
    std::fs::write(&p, "this is = not valid [toml").unwrap();
    assert!(CoreConfig::load(Some(&p)).is_err());
}
