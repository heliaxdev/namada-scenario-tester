use std::{collections::BTreeMap, fmt::Display};

use async_trait::async_trait;
use namada_sdk::{
    args::InitProposal,
    governance::{
        cli::onchain::{DefaultProposal, OnChainProposal},
        storage::keys::get_counter_key,
    },
    rpc,
    signing::default_sign,
    Namada,
};

use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{BuildResult, Task, TaskError, TaskParam};

pub enum TxInitDefaultProposalStorageKeys {
    ProposalId,
    StartEpoch,
    EndEpoch,
    GraceEpoch,
    ProposerAddress,
}

impl ToString for TxInitDefaultProposalStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxInitDefaultProposalStorageKeys::ProposalId => "proposal-id".to_string(),
            TxInitDefaultProposalStorageKeys::StartEpoch => "proposal-start-epoch".to_string(),
            TxInitDefaultProposalStorageKeys::EndEpoch => "proposal-end-epoch".to_string(),
            TxInitDefaultProposalStorageKeys::GraceEpoch => "proposal-grace-epoch".to_string(),
            TxInitDefaultProposalStorageKeys::ProposerAddress => {
                "proposal-proposer-address".to_string()
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxInitDefaultProposal {}

impl TxInitDefaultProposal {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxInitDefaultProposal {
    type P = TxInitDefaultProposalParameters;
    type B = InitProposal;

    async fn build(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
    ) -> Result<BuildResult, TaskError> {
        let signer_address = parameters.signer.to_namada_address(sdk).await;
        let start_epoch = parameters.start_epoch;
        let end_epoch = parameters.end_epoch;
        let grace_epoch = parameters.grace_epoch;

        let governance_parameters = rpc::query_governance_parameters(sdk.namada.client()).await;

        let start_epoch = match start_epoch {
            Some(start_epoch) => start_epoch,
            None => {
                let current_epoch = rpc::query_epoch(sdk.namada.client()).await.unwrap();
                governance_parameters.min_proposal_voting_period
                    - (current_epoch.0) % governance_parameters.min_proposal_voting_period
                    + current_epoch.0
                    + governance_parameters.min_proposal_voting_period
            }
        };

        let end_epoch = match end_epoch {
            Some(end_epoch) => end_epoch,
            None => start_epoch + governance_parameters.min_proposal_voting_period,
        };

        let grace_epoch = match grace_epoch {
            Some(grace_epoch) => grace_epoch,
            None => end_epoch + governance_parameters.min_proposal_grace_epochs,
        };

        let _signing_keys = parameters.signer.to_signing_keys(sdk).await;

        let default_proposal = DefaultProposal {
            proposal: OnChainProposal {
                content: BTreeMap::from_iter([("scenario".to_string(), "tester".to_string())]),
                author: signer_address.clone(),
                voting_start_epoch: start_epoch.into(),
                voting_end_epoch: end_epoch.into(),
                activation_epoch: grace_epoch.into(),
            },
            data: None,
        };
        let proposal_json = serde_json::to_string(&default_proposal).unwrap();

        let init_proposal_tx_builder = sdk.namada.new_init_proposal(proposal_json.into_bytes());

        let init_proposal_tx_builder = self
            .add_settings(sdk, init_proposal_tx_builder, settings)
            .await;

        let (mut init_proposal_tx, signing_data) = init_proposal_tx_builder
            .build(&sdk.namada)
            .await
            .map_err(|e| TaskError::Build(e.to_string()))?;

        sdk.namada
            .sign(
                &mut init_proposal_tx,
                &init_proposal_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign tx");

        let mut step_storage = StepStorage::default();
        self.fetch_info(sdk, &mut step_storage).await;

        let proposal_id_storage_key = get_counter_key();
        // This returns the next proposal_id, so always subtract 1
        // If multiple proposal in the same block, this would not work
        let proposal_id =
            rpc::query_storage_value::<_, u64>(sdk.namada.client(), &proposal_id_storage_key)
                .await
                .unwrap()
                - 1;

        step_storage.add(
            TxInitDefaultProposalStorageKeys::ProposalId.to_string(),
            proposal_id.to_string(),
        );
        step_storage.add(
            TxInitDefaultProposalStorageKeys::ProposerAddress.to_string(),
            signer_address.to_string(),
        );
        step_storage.add(
            TxInitDefaultProposalStorageKeys::StartEpoch.to_string(),
            start_epoch.to_string(),
        );
        step_storage.add(
            TxInitDefaultProposalStorageKeys::EndEpoch.to_string(),
            end_epoch.to_string(),
        );
        step_storage.add(
            TxInitDefaultProposalStorageKeys::GraceEpoch.to_string(),
            grace_epoch.to_string(),
        );

        Ok(BuildResult::new(
            init_proposal_tx,
            init_proposal_tx_builder.tx,
            step_storage,
        ))
    }
}

impl Display for TxInitDefaultProposal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-init-default-proposal")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxInitDefaultProposalParametersDto {
    pub signer: Value,
    pub start_epoch: Option<Value>,
    pub end_epoch: Option<Value>,
    pub grace_epoch: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct TxInitDefaultProposalParameters {
    signer: AccountIndentifier,
    start_epoch: Option<u64>,
    end_epoch: Option<u64>,
    grace_epoch: Option<u64>,
}

impl TaskParam for TxInitDefaultProposalParameters {
    type D = TxInitDefaultProposalParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let signer = match dto.signer {
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
            Value::Fuzz { .. } => unimplemented!(),
        };
        let start_epoch = dto.start_epoch.map(|start_epoch| match start_epoch {
            Value::Ref { value: _, field: _ } => {
                unimplemented!() // can't refertence a past epoch as end epoch
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        });
        let end_epoch = dto.end_epoch.map(|end_epoch| match end_epoch {
            Value::Ref { value: _, field: _ } => {
                unimplemented!() // can't refertence a past epoch as end epoch
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        });
        let grace_epoch = dto.grace_epoch.map(|grace_epoch| match grace_epoch {
            Value::Ref { value: _, field: _ } => unimplemented!(), // can't refertence a past epoch as grace epoch
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz { .. } => unimplemented!(),
        });
        Some(Self {
            signer,
            start_epoch,
            end_epoch,
            grace_epoch,
        })
    }
}
