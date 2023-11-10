use std::path::PathBuf;

use async_trait::async_trait;
use namada_sdk::{
    args::{InputAmount, TxBuilder},
    core::types::{
        address::Address as NamadaAddress,
        masp::{TransferSource, TransferTarget},
        token::{self, Amount},
    },
    Namada, tendermint::public_key, ibc::applications::transfer::amount,
};
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{Address, StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

#[derive(Clone, Debug, Default)]
pub struct Redelegate { }

impl Redelegate {
    pub fn new() -> Self {
        Self { }
    }
}

#[async_trait(?Send)]
impl Task for Redelegate {
    type P = RedelegateParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        // Params are validator: Address, source: Address, amount: u64
        let source_alias = parameters.source.alias;
        let source_address = if parameters.source.address.starts_with("atest") {
            NamadaAddress::decode(parameters.source.address).unwrap()
        } else {
            let wallet = sdk.namada.wallet.read().await;
            wallet.find_address(&parameters.source.address).unwrap().as_ref().clone()
        };

        let validator_src : NamadaAddress = parameters.src_validator.parse().unwrap();
        let validator_target: NamadaAddress = parameters.dest_validator.parse().unwrap();

        let bond_amount = Amount::from(parameters.amount);

        let mut wallet = sdk.namada.wallet.write().await;
        let source_secret_key = wallet.find_key(source_alias.clone(), None).unwrap();
        drop(wallet);
 
        let redelegate_tx_builder = sdk.namada.new_redelegation(source_address, validator_src, validator_target, bond_amount).signing_keys(vec![source_secret_key]);
        let (mut redelegate_tx, signing_data) = redelegate_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build transfer");
        sdk.namada
            .sign(&mut redelegate_tx, &redelegate_tx_builder.tx, signing_data)
            .await
            .expect("unable to sign reveal pk tx");
        let _tx = sdk.namada.submit(redelegate_tx, &redelegate_tx_builder.tx).await;

        println!("Tx is {:?}", _tx);
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
    source: Address,
    src_validator: String,
    dest_validator: String,
    amount: u64,
}

impl TaskParam for RedelegateParameters {
    type D = RedelegateParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let source = match dto.source {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias)
            }
            Value::Value { value } => Address::from_alias(value),
            Value::Fuzz {} => unimplemented!(),
        };
        let src_validator = match dto.src_validator {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias).address
            }
            Value::Value { value } => value,
            Value::Fuzz {} => unimplemented!(),
        };
        let dest_validator = match dto.dest_validator {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias).address
            }
            Value::Value { value } => value,
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
            amount
        }
    }
}