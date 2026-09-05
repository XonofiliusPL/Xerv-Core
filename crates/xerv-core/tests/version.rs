use xerv_core::{api_version, API_VERSION};

#[test]
fn api_version_is_0_2_1() {
    let v = api_version();
    assert_eq!(v.major, 0);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 1);
    assert_eq!(v.pre.as_str(), "");
    assert_eq!(API_VERSION.major, 0);
    assert_eq!(API_VERSION.minor, 2);
}
