use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, WasmMsg, CosmosMsg, Addr, StdError,
};
use cw2::set_contract_version;
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

// Define the contract name and version
const CONTRACT_NAME: &str = "crates.io:cosmwasm-distribution";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Allocation {
    pub address: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Claim {},
    Withdraw {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Allocation { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllocationResponse {
    pub amount: Uint128,
}

pub const OWNER: Item<Addr> = Item::new("owner");
pub const ALLOCATIONS: Map<&Addr, Uint128> = Map::new("allocations");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let owner = deps.api.addr_validate(&msg.owner)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    OWNER.save(deps.storage, &owner)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::Withdraw {} => execute_withdraw(deps, env, info),
    }
}

fn execute_claim(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let amount = ALLOCATIONS.load(deps.storage, &info.sender)?;
    if amount == Uint128::zero() {
        return Err(ContractError::NotClaimable {});
    }

    ALLOCATIONS.save(deps.storage, &info.sender, &Uint128::zero())?;
    let send_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: "token".to_string(),
            amount,
        }],
    };

    Ok(Response::new()
        .add_message(send_msg)
        .add_attribute("method", "claim")
        .add_attribute("amount", amount.to_string()))
}

fn execute_withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut msgs: Vec<CosmosMsg> = vec![];
    let all_allocations = ALLOCATIONS.range(deps.storage, None, None, cosmwasm_std::Order::Ascending);
    
    for alloc in all_allocations {
        let (addr, amount) = alloc?;
        let send_msg = BankMsg::Send {
            to_address: addr.into_string(),
            amount: vec![Coin {
                denom: "token".into(),
                amount,
            }],
        };
        msgs.push(send_msg.into());
        ALLOCATIONS.save(deps.storage, &addr, &Uint128::zero())?;
    }

    Ok(Response::new().add_messages(msgs).add_attribute("method", "withdraw"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Allocation { address } => query_allocation(deps, address),
    }
}

fn query_allocation(deps: Deps, address: String) -> StdResult<Binary> {
    let address = deps.api.addr_validate(&address)?;
    let amount = ALLOCATIONS.may_load(deps.storage, &address)?.unwrap_or_default();
    let response = AllocationResponse { amount };
    to_binary(&response)
}

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("Unauthorized")]
    Unauthorized {},

    #[error("NotClaimable")]
    NotClaimable {},

    #[error("{0}")]
    Std(#[from] StdError),
}
