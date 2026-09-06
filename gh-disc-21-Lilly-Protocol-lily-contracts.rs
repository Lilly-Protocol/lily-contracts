// contracts/lily/src/contract.rs
use cosmwasm_std::{to_binary, Binary, Deps, Env, MessageInfo, Response, StdResult};
use crate::state::{CONFIG, IS_INITIALIZED};

pub fn instantiate(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    _msg: crate::msg::InstantiateMsg,
) -> StdResult<Response> {
    // Initialize is_initialized to false explicitly
    IS_INITIALIZED.save(deps.storage, &false)?;
    
    // Initialize config with default values
    CONFIG.save(deps.storage, &crate::state::Config::default())?;
    
    Ok(Response::new().add_attribute("action", "instantiate"))
}

pub fn is_initialized(deps: Deps) -> StdResult<bool> {
    IS_INITIALIZED.load(deps.storage)
}

// contracts/lily/src/state.rs
use cosmwasm_std::Storage;
use cw_storage_plus::Item;

pub struct Config {
    pub owner: String,
    // other fields...
}

impl Default for Config {
    fn default() -> Self {
        Config {
            owner: String::new(),
        }
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const IS_INITIALIZED: Item<bool> = Item::new("is_initialized");

// tests/integration.rs (test snippet)
#[test]
fn test_is_initialized_returns_false_before_initialization() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Before instantiate, is_initialized should return false
    let result = is_initialized(deps.as_ref()).unwrap();
    assert!(!result);
    
    // After instantiate, is_initialized should still be false
    // (actual initialization happens via execute messages)
    let msg = InstantiateMsg {};
    let info = mock_info("creator", &[]);
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 1);
    
    // Verify is_initialized is still false after instantiate
    let result = is_initialized(deps.as_ref()).unwrap();
    assert!(!result);
}