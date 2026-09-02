#![cfg(test)]

use lily_test_support::{soroban_string, test_env};

use super::{
    require_compatible_schema, require_non_empty, require_non_whitespace, require_valid_bps,
    DEFAULT_SCHEMA_VERSION, MAX_BPS,
};

#[test]
fn validates_valid_strings_with_whitespace_guard() {
    let env = test_env();
    let valid_str = soroban_string(&env, "ipfs://agent-lily/profile-v1");
    require_non_whitespace(&env, &valid_str);

    let spaced_str = soroban_string(&env, "  valid memo  ");
    require_non_whitespace(&env, &spaced_str);
}

#[test]
#[should_panic]
fn rejects_empty_string_with_whitespace_guard() {
    let env = test_env();
    let empty_str = soroban_string(&env, "");
    require_non_whitespace(&env, &empty_str);
}

#[test]
#[should_panic]
fn rejects_spaces_only_string() {
    let env = test_env();
    let space_str = soroban_string(&env, "    ");
    require_non_whitespace(&env, &space_str);
}

#[test]
#[should_panic]
fn rejects_tabs_and_newlines_only_string() {
    let env = test_env();
    let ws_str = soroban_string(&env, "\t\n\r  ");
    require_non_whitespace(&env, &ws_str);
}

#[test]
fn validates_fee_bps_and_schema_compatibility() {
    let env = test_env();
    require_valid_bps(&env, 0);
    require_valid_bps(&env, 500);
    require_valid_bps(&env, MAX_BPS);

    require_compatible_schema(&env, DEFAULT_SCHEMA_VERSION, 1);
    require_non_empty(&env, 5);
}

#[test]
#[should_panic]
fn rejects_fee_bps_above_max() {
    let env = test_env();
    require_valid_bps(&env, MAX_BPS + 1);
}

#[test]
#[should_panic]
fn rejects_mismatched_schema_version() {
    let env = test_env();
    require_compatible_schema(&env, 2, 1);
}
