use std::fmt::Display;

use async_trait::async_trait;
use namada_sdk::{args::Bond, signing::default_sign, token::Amount, Namada};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    queries::validators::ValidatorsQueryStorageKeys,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{BuildResult, Task, TaskError, TaskParam};

pub enum TxBondStorageKeys {
    SourceAddress,
    ValidatorAddress,
    Amount,
}

impl ToString for TxBondStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxBondStorageKeys::SourceAddress => "source-address".to_string(),
            TxBondStorageKeys::ValidatorAddress => "validator-address".to_string(),
            TxBondStorageKeys::Amount => "amount".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxBond {}

impl TxBond {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxBond {
    type P = TxBondParameters;
    type B = Bond;

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError> {
        let source_address = parameters.source.to_namada_address(sdk).await;
        let amount = Amount::from(parameters.amount);
        let validator_address = parameters.validator.to_namada_address(sdk).await;

        let bond_tx_builder = sdk
            .namada
            .new_bond(validator_address.clone(), amount)
            .source(source_address.clone());

        let bond_tx_builder = self.add_settings(sdk, bond_tx_builder, settings).await;

        let (mut bond_tx, signing_data) = bond_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut bond_tx,
                &bond_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");

        let mut step_storage = StepStorage::default();
        self.fetch_info(sdk, &mut step_storage).await;

        step_storage.add(
            TxBondStorageKeys::ValidatorAddress.to_string(),
            validator_address.to_string(),
        );
        step_storage.add(
            TxBondStorageKeys::SourceAddress.to_string(),
            source_address.to_string(),
        );
        step_storage.add(
            TxBondStorageKeys::Amount.to_string(),
            amount.raw_amount().to_string(),
        );

        Ok(BuildResult::new(bond_tx, bond_tx_builder.tx, step_storage))
    }
}

impl Display for TxBond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-bond")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxBondParametersDto {
    pub source: Value,
    pub validator: Value,
    pub amount: Value,
}

#[derive(Clone, Debug)]
pub struct TxBondParameters {
    source: AccountIndentifier,
    validator: AccountIndentifier,
    amount: u64,
}

impl TaskParam for TxBondParameters {
    type D = TxBondParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let source = match dto.source {
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
            Value::Fuzz { value } => {
                let step_id = value.expect("Bond task requires fuzz for source to define the step id to a validator query step");
                let total_validators = state
                    .get_step_item(
                        &step_id,
                        ValidatorsQueryStorageKeys::TotalValidator
                            .to_string()
                            .as_str(),
                    )
                    .parse::<u64>()
                    .unwrap();

                let validator_idx = rand::thread_rng().gen_range(0..total_validators);

                let validator_address = state.get_step_item(
                    &step_id,
                    ValidatorsQueryStorageKeys::Validator(validator_idx)
                        .to_string()
                        .as_str(),
                );

                AccountIndentifier::Address(validator_address)
            }
        };
        let validator = match dto.validator {
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
            Value::Fuzz { value } => {
                let step_id = value.expect("Bond task requires fuzz for source to define the step id to a validator query step");
                let total_validators = state
                    .get_step_item(
                        &step_id,
                        ValidatorsQueryStorageKeys::TotalValidator
                            .to_string()
                            .as_str(),
                    )
                    .parse::<u64>()
                    .unwrap();

                let validator_idx = rand::thread_rng().gen_range(0..total_validators);

                let validator_address = state.get_step_item(
                    &step_id,
                    ValidatorsQueryStorageKeys::Validator(validator_idx)
                        .to_string()
                        .as_str(),
                );

                AccountIndentifier::Address(validator_address)
            }
        };
        let amount = match dto.amount {
            Value::Ref { value, field } => {
                let was_step_successful = state.is_step_successful(&value);
                if !was_step_successful {
                    return None;
                }
                let amount = state.get_step_item(&value, &field);
                amount.parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        };

        Some(Self {
            source,
            validator,
            amount,
        })
    }
}
