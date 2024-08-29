use async_trait::async_trait;

use namada_sdk::rpc;
use serde::{Deserialize, Serialize};

use crate::entity::address::{AccountIndentifier, ADDRESS_PREFIX};
use crate::{scenario::StepResult, sdk::namada::Sdk, state::state::Storage, utils::value::Value};

use super::{Check, CheckParam};

#[derive(Clone, Debug, Default)]
pub struct RevealPkCheck {}

impl RevealPkCheck {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Check for RevealPkCheck {
    type P = RevealPkCheckParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let source_address = parameters.source.to_namada_address(sdk).await;

        let is_pk_revealed = rpc::is_public_key_revealed(&sdk.namada.client, &source_address)
            .await;

        if let Err(e) = is_pk_revealed {
            println!("{}", e);
            StepResult::fail_check(false.to_string(), true.to_string())
        } else {
            StepResult::success_empty()
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RevealPkCheckParametersDto {
    pub source: Value,
}

#[derive(Clone, Debug)]
pub struct RevealPkCheckParameters {
    source: AccountIndentifier,
}

impl CheckParam for RevealPkCheckParameters {
    type D = RevealPkCheckParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
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

        Self { source }
    }
}
