use async_trait::async_trait;
use namada_sdk::{core::types::storage::Epoch, rpc, Namada};
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Query, QueryParam};

#[derive(Clone, Debug, Default)]
pub struct BondedStakeQuery {}

impl BondedStakeQuery {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Query for BondedStakeQuery {
    type P = BondedStakeQueryParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let epoch = if let Some(epoch) = parameters.epoch {
            Epoch::from(epoch)
        } else {
            rpc::query_epoch(sdk.namada.client())
                .await
                .expect("Should be able to query for epoch")
        };

        let bonds_and_unbonds =
            rpc::enriched_bonds_and_unbonds(sdk.namada.client(), epoch, &None, &None).await;

        let bonds_and_unbonds = if let Ok(bonds_and_unbonds) = bonds_and_unbonds {
            bonds_and_unbonds
        } else {
            return StepResult::fail();
        };

        let mut storage = StepStorage::default();
        storage.add(
            "bonds_total".to_string(),
            bonds_and_unbonds.bonds_total.to_string_native(),
        );
        storage.add(
            "bonds_total_slashed".to_string(),
            bonds_and_unbonds.bonds_total_slashed.to_string_native(),
        );
        storage.add(
            "unbonds_total".to_string(),
            bonds_and_unbonds.unbonds_total.to_string_native(),
        );
        storage.add(
            "unbonds_total_slashed".to_string(),
            bonds_and_unbonds.unbonds_total_slashed.to_string_native(),
        );
        storage.add(
            "total_withdrawable".to_string(),
            bonds_and_unbonds.total_withdrawable.to_string_native(),
        );
        storage.add("epoch".to_string(), epoch.0.to_string());
        for (bond_id, info) in bonds_and_unbonds.data {
            let source = bond_id.source;
            let validator = bond_id.validator;
            for bond in info.data.bonds {
                storage.add(
                    format!("{}-{}-bond", source.to_string(), validator.to_string()),
                    bond.amount.to_string_native(),
                );
                storage.add(
                    format!(
                        "{}-{}-bond-slash",
                        source.to_string(),
                        validator.to_string()
                    ),
                    bond.slashed_amount
                        .map(|a| a.to_string_native())
                        .unwrap_or("0".to_string()),
                );
                storage.add(
                    format!(
                        "{}-{}-bond-epoch",
                        source.to_string(),
                        validator.to_string()
                    ),
                    bond.start.to_string(),
                );
            }
            for bond in info.data.unbonds {
                storage.add(
                    format!("{}-{}-unbond", source.to_string(), validator.to_string()),
                    bond.amount.to_string_native(),
                );
                storage.add(
                    format!(
                        "{}-{}-unbond-slash",
                        source.to_string(),
                        validator.to_string()
                    ),
                    bond.slashed_amount
                        .map(|a| a.to_string_native())
                        .unwrap_or("0".to_string()),
                );
                storage.add(
                    format!(
                        "{}-{}-unbond-epoch",
                        source.to_string(),
                        validator.to_string()
                    ),
                    bond.start.to_string(),
                );
            }
            // TODO: add slashes
        }
        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BondedStakeQueryParametersDto {
    epoch: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct BondedStakeQueryParameters {
    epoch: Option<u64>,
}

impl QueryParam for BondedStakeQueryParameters {
    type D = BondedStakeQueryParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let epoch = match dto.epoch {
            Some(Value::Ref { value }) => {
                let epoch = state.get_step_item(&value, "epoch");
                epoch.parse::<u64>().ok()
            }
            Some(Value::Value { value }) => value.parse::<u64>().ok(),
            Some(Value::Fuzz {}) => unimplemented!(),
            _ => None,
        };

        Self { epoch }
    }
}
