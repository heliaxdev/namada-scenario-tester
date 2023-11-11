use async_trait::async_trait;
use namada_sdk::{args::TxBuilder, core::types::token::Amount, Namada};
use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

#[derive(Clone, Debug, Default)]
pub struct Redelegate {}

impl Redelegate {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for Redelegate {
    type P = RedelegateParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        // Params are validator: Address, source: Address, amount: u64
        let source_address = parameters.source.to_namada_address(sdk).await;
        let validator_src = parameters.src_validator.to_namada_address(sdk).await;
        let validator_target = parameters.dest_validator.to_namada_address(sdk).await;

        let source_alias = match parameters.source {
            AccountIndentifier::Alias(alias) => alias,
            AccountIndentifier::Address(_) => panic!(),
            AccountIndentifier::StateAddress(state) => state.alias,
        };
        let bond_amount = Amount::from(parameters.amount);

        let mut wallet = sdk.namada.wallet.write().await;
        let source_secret_key = wallet.find_key(source_alias.clone(), None).unwrap();
        drop(wallet);

        let redelegate_tx_builder = sdk
            .namada
            .new_redelegation(source_address, validator_src, validator_target, bond_amount)
            .signing_keys(vec![source_secret_key]);
        let (mut redelegate_tx, signing_data) = redelegate_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build redelegate tx");
        sdk.namada
            .sign(&mut redelegate_tx, &redelegate_tx_builder.tx, signing_data)
            .await
            .expect("unable to sign redelegate tx");
        let _tx = sdk
            .namada
            .submit(redelegate_tx, &redelegate_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        storage.add("address".to_string(), source_alias.to_string());

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct RedelegateParametersDto {
    source: Value,
    src_validator: Value,
    dest_validator: Value,
    amount: Value,
}

#[derive(Clone, Debug)]
pub struct RedelegateParameters {
    source: AccountIndentifier,
    src_validator: AccountIndentifier,
    dest_validator: AccountIndentifier,
    amount: u64,
}

impl TaskParam for RedelegateParameters {
    type D = RedelegateParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let source = match dto.source {
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
        let src_validator = match dto.src_validator {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                AccountIndentifier::Address(state.get_address(&alias).address)
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
        let dest_validator = match dto.dest_validator {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                AccountIndentifier::Address(state.get_address(&alias).address)
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
        let amount = match dto.amount {
            Value::Ref { value } => {
                let amount = state.get_step_item(&value, "amount");
                amount.parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        };

        Self {
            source,
            src_validator,
            dest_validator,
            amount,
        }
    }
}
