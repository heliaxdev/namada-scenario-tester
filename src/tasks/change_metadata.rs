use async_trait::async_trait;
use fake::{
    faker::{
        internet::en::{FreeEmail, Username},
        lorem::en::Words,
    },
    Fake,
};
use namada_sdk::{args::MetaDataChange, signing::default_sign, Namada};
use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{Task, TaskError, TaskParam};

pub enum TxChangeMetadataStorageKeys {
    ValidatorAddress,
}

impl ToString for TxChangeMetadataStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxChangeMetadataStorageKeys::ValidatorAddress => "validator-address".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxChangeMetadata {}

impl TxChangeMetadata {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxChangeMetadata {
    type P = TxChangeMetadataParameters;
    type B = MetaDataChange;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let source_address = parameters.source.to_namada_address(sdk).await;

        let metadata_change_builder = sdk
            .namada
            .new_change_metadata(source_address.clone())
            .email(parameters.email)
            .avatar(parameters.avatar)
            // .commission_rate(commission_rate) // this needs a validator to be active, kind of a diffult check
            .description(parameters.description)
            .discord_handle(parameters.discord_handle)
            .website(parameters.website);

        let metadata_change_builder = self
            .add_settings(sdk, metadata_change_builder, settings)
            .await;

        let (mut metadata_tx, signing_data) = metadata_change_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut metadata_tx,
                &metadata_change_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        let tx = sdk
            .namada
            .submit(metadata_tx.clone(), &metadata_change_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&metadata_tx, &tx) {
            match tx {
                Ok(tx) => {
                    let errors = Self::get_tx_errors(&metadata_tx, &tx).unwrap_or_default();
                    return Ok(StepResult::fail(errors));
                }
                Err(e) => {
                    return Ok(StepResult::fail(e.to_string()));
                }
            }
        }

        storage.add(
            TxChangeMetadataStorageKeys::ValidatorAddress.to_string(),
            source_address.to_string(),
        );

        Ok(StepResult::success(storage))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxChangeMetadataParametersDto {
    pub source: Value,
    pub email: Option<Value>,
    pub avatar: Option<Value>,
    pub commission_rate: Option<Value>,
    pub description: Option<Value>,
    pub discord_handle: Option<Value>,
    pub website: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct TxChangeMetadataParameters {
    source: AccountIndentifier,
    email: String,
    avatar: String,
    description: String,
    discord_handle: String,
    website: String,
}

impl TaskParam for TxChangeMetadataParameters {
    type D = TxChangeMetadataParametersDto;

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
            Value::Fuzz { value: _ } => unimplemented!(),
        };
        let email = match dto.email {
            Some(Value::Ref { .. }) => unimplemented!(),
            Some(Value::Value { value }) => value,
            Some(Value::Fuzz { .. }) => FreeEmail().fake(),
            _ => "".to_string(),
        };
        let avatar = match dto.avatar {
            Some(Value::Ref { .. }) => unimplemented!(),
            Some(Value::Value { value }) => value,
            Some(Value::Fuzz { .. }) => Username().fake(),
            _ => "".to_string(),
        };
        let description = match dto.description {
            Some(Value::Ref { .. }) => unimplemented!(),
            Some(Value::Value { value }) => value,
            Some(Value::Fuzz { .. }) => {
                let words: Vec<String> = Words(0..20).fake();
                words.join(" ")
            }
            _ => "".to_string(),
        };
        let discord_handle = match dto.discord_handle {
            Some(Value::Ref { .. }) => unimplemented!(),
            Some(Value::Value { value }) => value,
            Some(Value::Fuzz { .. }) => Username().fake(),
            _ => "".to_string(),
        };
        let website = match dto.website {
            Some(Value::Ref { .. }) => unimplemented!(),
            Some(Value::Value { value }) => value,
            Some(Value::Fuzz { .. }) => {
                let words: Vec<String> = Words(0..5).fake();
                words.join(" ")
            }
            _ => "".to_string(),
        };

        Some(Self {
            source,
            email,
            avatar,
            description,
            discord_handle,
            website,
        })
    }
}
