use async_trait::async_trait;
use namada_sdk::{
    args::{InputAmount, TxBuilder},
    core::types::{
        address::Address as NamadaAddress,
        masp::{TransferSource, TransferTarget},
        token::{self, DenominatedAmount},
    },
    Namada, tendermint::public_key,
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
pub struct TxRevealPk { }

impl TxRevealPk {
    pub fn new() -> Self {
        Self { }
    }
}

#[async_trait(?Send)]
impl Task for TxRevealPk {
    type P = RevealPkParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let source_alias = parameters.source.alias;
        let mut wallet = sdk.namada.wallet.write().await;
        let source_secret_key = wallet.find_key(source_alias.clone(), None).unwrap();
        let source_public_key = source_secret_key.to_public();
        drop(wallet);
        let reveal_pk_tx_builder = sdk.namada.new_reveal_pk(source_public_key).signing_keys(vec![source_secret_key]);
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
pub struct RevealPkParametersDto {
    source: Value
}

#[derive(Clone, Debug)]
pub struct RevealPkParameters {
    source: Address
}

impl TaskParam for RevealPkParameters {
    type D = RevealPkParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let source = match dto.source {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
                state.get_address(&alias)
            }
            Value::Value { value } => Address::from_alias(value),
            Value::Fuzz {} => unimplemented!(),
        };
        Self {
            source,
        }
    }
}