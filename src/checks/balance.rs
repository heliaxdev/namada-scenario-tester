use std::str::FromStr;

use async_trait::async_trait;

use namada_sdk::rpc;
use namada_sdk::token::Amount;
use serde::{Deserialize, Serialize};

use crate::entity::address::{AccountIndentifier, ADDRESS_PREFIX};
use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage, utils::value::Value};

use super::{Check, CheckParam};

#[derive(Clone, Debug, Default)]
pub struct BalanceCheck {}

impl BalanceCheck {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Check for BalanceCheck {
    type P = BalanceCheckParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let owner_address = parameters.address.to_namada_address(sdk).await;
        let token_address = parameters.token.to_namada_address(sdk).await;

        let balance = rpc::get_token_balance(
            &sdk.namada.clone_client(),
            &token_address,
            &owner_address,
            None,
        )
        .await;

        let previous_balance = Amount::from_u64(parameters.amount);
        let current_balance = match balance {
            Ok(res) => res,
            Err(e) => {
                println!("{}", e);
                return StepResult::fail_check(0.to_string(), previous_balance.to_string());
            }
        };

        let result = match parameters.op {
            Operation::Ge => current_balance.ge(&previous_balance),
            Operation::Le => current_balance.le(&previous_balance),
        };

        if result {
            StepResult::success_empty()
        } else {
            StepResult::fail_check(current_balance.to_string(), previous_balance.to_string())
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Operation {
    Ge,
    Le,
}

impl FromStr for Operation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ge" => Ok(Self::Ge),
            "le" => Ok(Self::Le),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BalanceCheckParametersDto {
    pub amount: Value,
    pub address: Value,
    pub token: Value,
    pub op: Value,
}

#[derive(Clone, Debug)]
pub struct BalanceCheckParameters {
    amount: u64,
    address: AccountIndentifier,
    token: AccountIndentifier,
    op: Operation,
}

impl CheckParam for BalanceCheckParameters {
    type D = BalanceCheckParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let amount = match dto.amount {
            Value::Ref { value, field } => {
                let was_step_successful = state.is_step_successful(&value);
                if !was_step_successful {
                    return None;
                }
                state.get_step_item(&value, &field).parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        };
        let address = match dto.address {
            Value::Ref { value, field } => {
                let was_step_successful = state.is_step_successful(&value);
                if !was_step_successful {
                    return None;
                }
                let data = state.get_step_item(&value, &field);
                match field.to_lowercase().as_str() {
                    "alias" => AccountIndentifier::Alias(data),
                    "public-key" => AccountIndentifier::PublicKey(data),
                    "state" => AccountIndentifier::StateAddress(state.get_address(&data)),
                    _ => AccountIndentifier::Address(data),
                }
            }
            Value::Value { value } => {
                if value.starts_with(ADDRESS_PREFIX) {
                    AccountIndentifier::Address(value)
                } else {
                    AccountIndentifier::Alias(value)
                }
            }
            Value::Fuzz { .. } => unimplemented!(),
        };
        let token = match dto.token {
            Value::Ref { value, field } => {
                let was_step_successful = state.is_step_successful(&value);
                if !was_step_successful {
                    return None;
                }
                let data = state.get_step_item(&value, &field);
                match field.to_lowercase().as_str() {
                    "alias" => AccountIndentifier::Alias(data),
                    "public-key" => AccountIndentifier::PublicKey(data),
                    "state" => AccountIndentifier::StateAddress(state.get_address(&data)),
                    _ => AccountIndentifier::Address(data),
                }
            }
            Value::Value { value } => {
                if value.starts_with(ADDRESS_PREFIX) {
                    AccountIndentifier::Address(value)
                } else {
                    AccountIndentifier::Alias(value)
                }
            }
            Value::Fuzz { .. } => unimplemented!(),
        };
        let op = match dto.op {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => Operation::from_str(&value).unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        };

        Some(Self {
            amount,
            address,
            token,
            op,
        })
    }
}
