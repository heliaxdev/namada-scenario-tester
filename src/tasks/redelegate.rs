use async_trait::async_trait;
use namada_sdk::{
    args::{Redelegate, TxBuilder},
    signing::default_sign,
    token::Amount,
    Namada,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Task, TaskParam};
use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    queries::validators::ValidatorsQueryStorageKeys,
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

pub enum TxRevealPkStorageKeys {
    SourceValidatorAddress,
    DestValidatorAddress,
    SourceAddress,
    Amount,
}

impl ToString for TxRevealPkStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxRevealPkStorageKeys::SourceValidatorAddress => "source-validator-address".to_string(),
            TxRevealPkStorageKeys::DestValidatorAddress => "validator-address".to_string(), // keep this the same as bonds.rs so we can reuse the bond check
            TxRevealPkStorageKeys::SourceAddress => "source-address".to_string(),
            TxRevealPkStorageKeys::Amount => "amount".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxRedelegate {}

impl TxRedelegate {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxRedelegate {
    type P = TxRedelegateParameters;
    type B = Redelegate;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> StepResult {
        // Params are validator: Address, source: Address, amount: u64
        let source_address = parameters.source.to_namada_address(sdk).await;
        let validator_src = parameters.src_validator.to_namada_address(sdk).await;
        let validator_target = if let Some(address) = parameters.dest_validator {
            address.to_namada_address(sdk).await
        } else {
            return StepResult::no_op();
        };

        let bond_amount = Amount::from(parameters.amount);

        let redelegate_tx_builder = sdk
            .namada
            .new_redelegation(
                source_address.clone(),
                validator_src.clone(),
                validator_target.clone(),
                bond_amount,
            )
            .force(true);

        let redelegate_tx_builder = self
            .add_settings(sdk, redelegate_tx_builder, settings)
            .await;

        let (mut redelegate_tx, signing_data) = redelegate_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build tx");

        sdk.namada
            .sign(
                &mut redelegate_tx,
                &redelegate_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");
        let tx = sdk
            .namada
            .submit(redelegate_tx.clone(), &redelegate_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&redelegate_tx, &tx) {
            let errors = Self::get_tx_errors(&tx.unwrap()).unwrap_or_default();
            return StepResult::fail(errors);
        }

        storage.add(
            TxRevealPkStorageKeys::SourceValidatorAddress.to_string(),
            validator_src.to_string(),
        );
        storage.add(
            TxRevealPkStorageKeys::DestValidatorAddress.to_string(),
            validator_target.to_string(),
        );
        storage.add(
            TxRevealPkStorageKeys::SourceAddress.to_string(),
            source_address.to_string(),
        );
        storage.add(
            TxRevealPkStorageKeys::Amount.to_string(),
            bond_amount.to_string_native(),
        );

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxRedelegateParametersDto {
    pub source: Value,
    pub src_validator: Value,
    pub dest_validator: Value,
    pub amount: Value,
}

#[derive(Clone, Debug)]
pub struct TxRedelegateParameters {
    source: AccountIndentifier,
    src_validator: AccountIndentifier,
    dest_validator: Option<AccountIndentifier>,
    amount: u64,
}

impl TaskParam for TxRedelegateParameters {
    type D = TxRedelegateParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Self {
        let source = match dto.source {
            Value::Ref { value, field } => {
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
        let src_validator = match dto.src_validator {
            Value::Ref { value, field } => {
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
        let dest_validator = match dto.dest_validator {
            Value::Ref { value, field } => {
                let data = state.get_step_item(&value, &field);
                match field.to_lowercase().as_str() {
                    "alias" => Some(AccountIndentifier::Alias(data)),
                    "public-key" => Some(AccountIndentifier::PublicKey(data)),
                    "state" => Some(AccountIndentifier::StateAddress(state.get_address(&data))),
                    _ => Some(AccountIndentifier::Address(data)),
                }
            }
            Value::Value { value } => {
                if value.starts_with(ADDRESS_PREFIX) {
                    Some(AccountIndentifier::Address(value))
                } else {
                    Some(AccountIndentifier::Alias(value))
                }
            }
            Value::Fuzz { value } => {
                let step_id = value.expect("Redelgate task requires fuzz for dest valdidator to define the step id to a validator query step");
                let total_validators = state
                    .get_step_item(
                        &step_id,
                        ValidatorsQueryStorageKeys::TotalValidator
                            .to_string()
                            .as_str(),
                    )
                    .parse::<u64>()
                    .unwrap();

                if total_validators < 2 {
                    None
                } else {
                    loop {
                        let validator_idx = rand::thread_rng().gen_range(0..total_validators);
                        let validator_address = state.get_step_item(
                            &step_id,
                            ValidatorsQueryStorageKeys::Validator(validator_idx)
                                .to_string()
                                .as_str(),
                        );
                        // let validator_state = state.get_step_item(
                        //     &step_id,
                        //     ValidatorsQueryStorageKeys::State(validator_idx)
                        //         .to_string()
                        //         .as_str(),
                        // );

                        let dest_validator = AccountIndentifier::Address(validator_address);

                        if dest_validator != src_validator {
                            break Some(dest_validator);
                        }

                        // if dest_validator != src_validator
                        //     && validator_state != ValidatorState::Inactive.to_string()
                        //     && validator_state != ValidatorState::Jailed.to_string()
                        // {
                        //     break Some(dest_validator);
                        // }
                    }
                }
            }
        };
        let amount = match dto.amount {
            Value::Ref { value, field } => {
                let amount = state.get_step_item(&value, &field);
                amount.parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        };

        Self {
            source,
            src_validator,
            dest_validator,
            amount,
        }
    }
}
