use xerv_core::api::{api_version, API_VERSION};

#[test]
fn api_version_constants_match() {
    assert_eq!(api_version().major, API_VERSION.major);
    assert_eq!(api_version().minor, API_VERSION.minor);
    assert_eq!(api_version().patch, API_VERSION.patch);
}

#[test]
fn test_version_is_zero_two_one() {
    assert_eq!(api_version().major, 0);
    assert_eq!(api_version().minor, 2);
    assert_eq!(api_version().patch, 1);
}
