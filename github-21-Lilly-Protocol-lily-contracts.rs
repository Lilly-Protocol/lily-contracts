// contracts/lily/src/contract.rs

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use crate::state::{CONFIG, STATE, Config, State};

const INITIALIZE_MSG: &str = "initialize";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    // Initialize state with is_initialized = false
    STATE.save(deps.storage, &State::Uninitialized)?;
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Initialize { .. } => execute_initialize(deps, env, info),
        _ => Err(StdError::generic_err("Unsupported execute message")),
    }
}

pub fn execute_initialize(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    // Only owner can initialize
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // Set initialized state
    STATE.save(deps.storage, &State::Initialized)?;
    
    Ok(Response::new().add_attribute("action", "initialize"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsInitialized {} => to_binary(&query_is_initialized(deps)?),
        _ => Err(StdError::generic_err("Unsupported query message")),
    }
}

pub fn query_is_initialized(deps: Deps) -> StdResult<bool> {
    match STATE.load(deps.storage)? {
        State::Initialized => Ok(true),
        State::Uninitialized => Ok(false),
    }
}

// contracts/lily/src/state.rs

use cosmwasm_std::{Addr, Storage};
use cw_storage_plus::Item;

pub enum State {
    Uninitialized,
    Initialized,
}

pub const STATE: Item<State> = Item::new("state");
pub const CONFIG: Item<Config> = Item::new("config");

pub struct Config {
    pub owner: Addr,
}