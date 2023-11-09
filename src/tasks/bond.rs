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
pub struct Bond { }

impl Bond {
    pub fn new() -> Self {
        Self { }
    }
}

#[async_trait(?Send)]
impl Task for Bond {
    type P = BondParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        // Params are validator: Address, source: Address, amount: u64
        let source_alias = parameters.source.alias;
        let bond_amount = parameters.amount;
        let validator_target: NamadaAddress = parameters.validator.address.parse().unwrap();
        let est = match validator_target {
            NamadaAddress::Established(established) => established,
            _ => panic!("Invalid validator address"),
        };
        let amount = Amount::from(parameters.amount);

        let mut wallet = sdk.namada.wallet.write().await;
        let source_secret_key = wallet.find_key(source_alias.clone(), None).unwrap();
        let source_public_key = source_secret_key.to_public();
        drop(wallet);
        let reveal_pk_tx_builder = sdk.namada.new_bond(validator_target, amount).signing_keys(source_secret_key);
        let (mut reveal_tx, signing_data, _epoch) = reveal_pk_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build transfer");
        sdk.namada
            .sign(&mut reveal_tx, &reveal_pk_tx_builder.tx, signing_data)
            .await
            .expect("unable to sign reveal pk tx");
        let _tx = sdk.namada.submit(reveal_tx, &reveal_pk_tx_builder.tx).await;

        let mut storage = StepStorage::default();
        storage.add("address".to_string(), source_alias.to_string());

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BondParametersDto {
    source: Value,
    validator: Value,
    amount: Value,
}

#[derive(Clone, Debug)]
pub struct BondParameters {
    source: Address,
    validator: Address,
    amount: u64,
}

impl TaskParam for BondParameters {
    type D = BondParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let source = match dto.source {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias)
            }
            Value::Value { value } => Address::from_alias(value),
            Value::Fuzz {} => unimplemented!(),
        };
        let validator = match dto.validator {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias)
            }
            Value::Value { value } => Address::from_alias(value),
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
            validator,
            amount
        }
    }
}