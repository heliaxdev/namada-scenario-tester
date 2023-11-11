use async_trait::async_trait;
use namada_sdk::{
    args::{InputAmount, TxBuilder},
    core::types::{
        address::Address as NamadaAddress,
        masp::{TransferSource, TransferTarget},
        token::{self, Amount},
    },
    ibc::applications::transfer::amount,
    tendermint::public_key,
    Namada,
};
use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

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
            TxRevealPkStorageKeys::DestValidatorAddress => "dest-validator-address".to_string(),
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
            .new_redelegation(
                source_address.clone(),
                validator_src.clone(),
                validator_target.clone(),
                bond_amount,
            )
            .signing_keys(vec![source_secret_key]);
        let (mut redelegate_tx, signing_data) = redelegate_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build redelegate tx");
        sdk.namada
            .sign(&mut redelegate_tx, &redelegate_tx_builder.tx, signing_data)
            .await
            .expect("unable to sign redelegate tx");
        let tx = sdk
            .namada
            .submit(redelegate_tx, &redelegate_tx_builder.tx)
            .await;

        if let Err(_) = tx {
            return StepResult::fail()
        }

        let mut storage = StepStorage::default();
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

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxRedelegateParametersDto {
    source: Value,
    src_validator: Value,
    dest_validator: Value,
    amount: Value,
}

#[derive(Clone, Debug)]
pub struct TxRedelegateParameters {
    source: AccountIndentifier,
    src_validator: AccountIndentifier,
    dest_validator: AccountIndentifier,
    amount: u64,
}

impl TaskParam for TxRedelegateParameters {
    type D = TxRedelegateParametersDto;

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
