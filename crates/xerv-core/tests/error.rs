use xerv_core::api::{ApiError, CoreState, Result};

#[test]
fn error_config_message_includes_context() {
    let e = ApiError::Config("missing key".into());
    assert_eq!(e.to_string(), "config: missing key");
}

#[test]
fn error_state_message_includes_context() {
    let e = ApiError::State("no state".into());
    assert_eq!(e.to_string(), "state: no state");
}

#[test]
fn error_io_converts_from_io() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
    let e: ApiError = io.into();
    assert!(matches!(e, ApiError::Io(_)));
}

#[test]
fn error_version_mismatch_carries_both() {
    let e = ApiError::VersionMismatch {
        expected: "0.1.0".into(),
        actual: "0.2.0".into(),
    };
    let s = e.to_string();
    assert!(s.contains("0.1.0"));
    assert!(s.contains("0.2.0"));
}

#[test]
fn error_lifecycle_message_includes_context() {
    let e = ApiError::Lifecycle("shutting down".into());
    assert_eq!(e.to_string(), "lifecycle: shutting down");
}

#[test]
fn error_serde_converts_from_serde_json() {
    let json = serde_json::to_string(&serde_json::Value::Null).unwrap();
    let e: Result<CoreState> = serde_json::from_str::<CoreState>(&json).map_err(|e| e.into());
    assert!(matches!(e, Err(ApiError::Serde(_))));
}

#[test]
fn error_other_message_passthrough() {
    let e = ApiError::Other("custom".into());
    assert_eq!(e.to_string(), "custom");
}
