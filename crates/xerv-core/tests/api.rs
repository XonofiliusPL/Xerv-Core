use std::any::TypeId;

use xerv_core::api::{self as api_mod, ApiError, ApiResult};
use xerv_core::CoreState;
use xerv_core::{config, core, error, state};

fn tid<T: 'static>(_: &T) -> TypeId {
    TypeId::of::<T>()
}

#[test]
fn api_reexports_match_crate_root_types() {
    // Tworzymy instancje przez konstruktory/fresh/default, bo typy mają prywatne pola.
    let a: api_mod::CoreConfig = api_mod::CoreConfig::default();
    let b: config::CoreConfig = config::CoreConfig::default();
    assert_eq!(tid(&a), tid(&b));

    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join("s.json");
    let a = api_mod::XervCore::new(api_mod::CoreConfig::default(), state_path.clone()).unwrap();
    let b = core::XervCore::new(config::CoreConfig::default(), state_path).unwrap();
    assert_eq!(tid(&a), tid(&b));

    // Error — singleton-enum z wariantami; ten sam wariant = ten sam typ.
    let a: api_mod::Error = api_mod::Error::Config("x".into());
    let b: error::Error = error::Error::Config("x".into());
    assert_eq!(tid(&a), tid(&b));

    let a: api_mod::CoreState = api_mod::CoreState::fresh();
    let b: CoreState = state::CoreState::fresh();
    assert_eq!(tid(&a), tid(&b));
}

#[test]
fn api_aliases_match_crate_root_aliases() {
    // TypeId nie działa na type-aliasach, więc sprawdzamy przez konwersję.
    let e: ApiError = ApiError::Config("x".into());
    let e2: error::Error = e;
    assert_eq!(e2.to_string(), "config: x");
    let r: ApiResult<i32> = Ok(7);
    // Konwersja aliasu ApiResult -> error::Result tożsamość typów.
    fn _assert_same<T>(_: error::Result<T>, _: ApiResult<T>) {}
    _assert_same(Ok(7), r);
}

#[test]
fn api_version_constants_match() {
    assert_eq!(api_mod::api_version().major, api_mod::API_VERSION.major);
    assert_eq!(api_mod::api_version().minor, api_mod::API_VERSION.minor);
    assert_eq!(api_mod::api_version().patch, api_mod::API_VERSION.patch);
}
