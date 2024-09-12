use async_trait::async_trait;

use namada_sdk::{rpc, Namada};
use serde::{Deserialize, Serialize};

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

        let epoch = rpc::query_epoch(sdk.namada.client()).await.unwrap();

        let bond = rpc::enriched_bonds_and_unbonds(
            sdk.namada.client(),
            epoch,
            &Some(delegator_address.clone()),
            &Some(delegate_address),
        )
        .await;

        if let Ok(bond_amount) = bond {
            let actual_bond_amount = bond_amount
                .bonds_total_active()
                .map(|amount| amount.raw_amount().to_string())
                .unwrap_or(0.to_string());
            let expected_bond_amount = parameters.amount.to_string();

            if actual_bond_amount.eq(&expected_bond_amount) {
                return StepResult::success_empty();
            } else {
                return StepResult::fail_check(actual_bond_amount, expected_bond_amount);
            }
        };
        StepResult::fail_check("unknown".to_string(), parameters.amount.to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BondsCheckParametersDto {
    pub amount: Value,
    pub delegate: Value,
    pub delegator: Value,
}

#[derive(Clone, Debug)]
pub struct BondsCheckParameters {
    amount: u64,
    delegate: AccountIndentifier,
    delegator: AccountIndentifier,
}

impl CheckParam for BondsCheckParameters {
    type D = BondsCheckParametersDto;

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
        let delegate = match dto.delegate {
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
        let delegator = match dto.delegator {
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

        Some(Self {
            amount,
            delegate,
            delegator,
        })
    }
}
