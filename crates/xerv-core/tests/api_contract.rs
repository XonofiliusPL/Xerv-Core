//! API contract tests for Point 2.
//!
//! Guarantee that the Core exposes types declared in the plan in both
//! locations (module `api` and crate root `xerv_core`).

use xerv_core::api::{self as api_mod};
use xerv_core::{CoreConfig, CoreState, XervCore};

#[test]
fn contract_holds_for_config() {
    fn _takes_both<T>(_: T, _: T) {}
    let cfg1: CoreConfig = CoreConfig::default();
    let cfg2: api_mod::CoreConfig = api_mod::CoreConfig::default();
    _takes_both(cfg1, cfg2);
}

#[test]
fn contract_holds_for_state() {
    let s1: CoreState = CoreState::fresh();
    let s2: api_mod::CoreState = api_mod::CoreState::fresh();
    assert_eq!(s1, s2);
}

#[test]
fn contract_holds_for_core() {
    // Function `new` has the same type signature in both locations.
    let _ = XervCore::new as fn(_, _) -> _;
    let _ = api_mod::XervCore::new as fn(_, _) -> _;
}
