use async_trait::async_trait;
use namada_sdk::{rpc, Namada};
use namada_sdk::core::types::address::Address as NamadaAddress;
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    state::state::{Address, Storage},
    utils::value::Value, sdk::namada::Sdk,
};

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

    async fn execute(&self, sdk: &Sdk, paramaters: Self::P, _state: &Storage) -> StepResult {
        let wallet = sdk.namada.wallet.read().await;

        let owner_address = wallet.find_address(&paramaters.address.alias);
        let owner_address = if let Some(address) = owner_address {
            address
        } else {
            return StepResult::fail() 
        };

        let balance = rpc::get_token_balance(
            sdk.namada.client(),
            &NamadaAddress::decode(&paramaters.token).unwrap(),
            owner_address,
        )
        .await;

        let balance = balance.unwrap().to_string_native();

        if paramaters.amount.to_string().eq(&balance) {
            StepResult::success_empty()
        } else {
            StepResult::fail()
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BalanceCheckParametersDto {
    amount: Value,
    address: Value,
    token: Value,
}

#[derive(Clone, Debug)]
pub struct BalanceCheckParameters {
    amount: u64,
    address: Address,
    token: String,
}

impl CheckParam for BalanceCheckParameters {
    type D = BalanceCheckParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let amount = match dto.amount {
            Value::Ref { value } => state
                .get_step_item(&value, "amount")
                .parse::<u64>()
                .unwrap(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        };
        let address = match dto.address {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias)
            }
            Value::Value { value } => Address::from_alias(value),
            Value::Fuzz {} => unimplemented!(),
        };
        let token = match dto.token {
            Value::Ref { value } => state.get_step_item(&value, "token"),
            Value::Value { value } => value.to_owned(),
            Value::Fuzz {} => unimplemented!(),
        };

        Self {
            amount,
            address,
            token,
        }
    }
}
