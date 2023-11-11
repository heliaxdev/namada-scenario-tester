use async_trait::async_trait;

use namada_sdk::{rpc, Namada};
use serde::Deserialize;

use crate::entity::address::{AccountIndentifier, ADDRESS_PREFIX};
use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage, utils::value::Value};

use super::{Check, CheckParam};

#[derive(Clone, Debug, Default)]
pub struct BondsCheck {}

impl BondsCheck {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Check for BondsCheck {
    type P = BondsCheckParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let delegate_address = parameters.delegate.to_namada_address(sdk).await;
        let delegator_address = parameters.delegator.to_namada_address(sdk).await;

        println!("delegate_address: {:?}", delegate_address);
        println!("delegator_address: {:?}", delegator_address);

        let epoch = None;
        let bond = rpc::query_bond(
            sdk.namada.client(),
            &delegator_address,
            &delegate_address,
            epoch,
        )
        .await
        .ok();

        if let Some(bond_amount) = bond {
            if parameters
                .amount
                .to_string()
                .eq(&bond_amount.raw_amount().to_string())
            {
                return StepResult::success_empty();
            }
        }
        println!("bond: {:?}", bond);
        println!(
            "parameters amount to string: {:?}",
            parameters.amount.to_string()
        );
        StepResult::fail()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BondsCheckParametersDto {
    amount: Value,
    delegate: Value,
    delegator: Value,
}

#[derive(Clone, Debug)]
pub struct BondsCheckParameters {
    amount: u64,
    delegate: AccountIndentifier,
    delegator: AccountIndentifier,
}

impl CheckParam for BondsCheckParameters {
    type D = BondsCheckParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let amount = match dto.amount {
            Value::Ref { value } => state
                .get_step_item(&value, "amount")
                .parse::<u64>()
                .unwrap(),
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        };
        let delegate = match dto.delegate {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                AccountIndentifier::StateAddress(state.get_address(&alias))
            }
            Value::Value { value } => {
                if value.starts_with(ADDRESS_PREFIX) {
                    AccountIndentifier::Address(value)
                } else {
                    AccountIndentifier::Alias(value)
                }
            }
            Value::Fuzz {} => unimplemented!(),
        };
        let delegator = match dto.delegator {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                AccountIndentifier::StateAddress(state.get_address(&alias))
            }
            Value::Value { value } => {
                if value.starts_with(ADDRESS_PREFIX) {
                    AccountIndentifier::Address(value)
                } else {
                    AccountIndentifier::Alias(value)
                }
            }
            Value::Fuzz {} => unimplemented!(),
        };

        Self {
            amount,
            delegate,
            delegator,
        }
    }
}
