use xerv_core::Error;

#[test]
fn error_display_includes_context() {
    let e = Error::Config("missing key".into());
    assert_eq!(e.to_string(), "config: missing key");
}

#[test]
fn error_io_converts_from_io() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
    let e: Error = io.into();
    assert!(matches!(e, Error::Io(_)));
}

#[test]
fn error_version_mismatch_carries_both() {
    let e = Error::VersionMismatch {
        expected: "0.1.0".into(),
        actual: "0.2.0".into(),
    };
    let s = e.to_string();
    assert!(s.contains("0.1.0"));
    assert!(s.contains("0.2.0"));
}
