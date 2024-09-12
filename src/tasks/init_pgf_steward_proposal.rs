use std::collections::BTreeMap;

use async_trait::async_trait;
use namada_sdk::{
    args::InitProposal,
    error::TxSubmitError,
    governance::{
        cli::onchain::{OnChainProposal, PgfStewardProposal, StewardsUpdate},
        storage::keys::get_counter_key,
    },
    rpc,
    signing::default_sign,
    Namada,
};

use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{Task, TaskError, TaskParam};

pub enum TxInitPgfStewardProposalStorageKeys {
    ProposalId,
    StartEpoch,
    EndEpoch,
    GraceEpoch,
    ProposerAddress,
    StewardAdd,
    StewardRemove,
}

impl ToString for TxInitPgfStewardProposalStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxInitPgfStewardProposalStorageKeys::ProposalId => "proposal-id".to_string(),
            TxInitPgfStewardProposalStorageKeys::StartEpoch => "proposal-start-epoch".to_string(),
            TxInitPgfStewardProposalStorageKeys::EndEpoch => "proposal-end-epoch".to_string(),
            TxInitPgfStewardProposalStorageKeys::GraceEpoch => "proposal-grace-epoch".to_string(),
            TxInitPgfStewardProposalStorageKeys::ProposerAddress => {
                "proposal-proposer-address".to_string()
            }
            TxInitPgfStewardProposalStorageKeys::StewardAdd => "proposal-steward-add".to_string(),
            TxInitPgfStewardProposalStorageKeys::StewardRemove => {
                "proposal-steward-remove".to_string()
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxInitPgfStewardProposal {}

impl TxInitPgfStewardProposal {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxInitPgfStewardProposal {
    type P = TxInitPgfStewardProposalParameters;
    type B = InitProposal;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        settings: TxSettings,
        _state: &Storage,
    ) -> Result<StepResult, TaskError> {
        let signer_address = parameters.signer.to_namada_address(sdk).await;
        let start_epoch = parameters.start_epoch;
        let end_epoch = parameters.end_epoch;
        let grace_epoch = parameters.grace_epoch;

        let mut stewards_to_remove = vec![];
        for source in parameters.steward_remove {
            let address = source.to_namada_address(sdk).await;
            stewards_to_remove.push(address);
        }

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

        let pgf_steward_proposal = PgfStewardProposal {
            proposal: OnChainProposal {
                content: BTreeMap::from_iter([("scenario".to_string(), "tester".to_string())]),
                author: signer_address.clone(),
                voting_start_epoch: start_epoch.into(),
                voting_end_epoch: end_epoch.into(),
                activation_epoch: grace_epoch.into(),
            },
            data: StewardsUpdate {
                add: Some(signer_address.clone()),
                remove: stewards_to_remove.clone(),
            },
        };
        let proposal_json = serde_json::to_string(&pgf_steward_proposal).unwrap();

        let init_proposal_tx_builder = sdk
            .namada
            .new_init_proposal(proposal_json.into_bytes())
            .is_pgf_stewards(true);

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

        let tx = sdk
            .namada
            .submit(init_proposal_tx.clone(), &init_proposal_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&init_proposal_tx, &tx) {
            match tx {
                Ok(tx) => {
                    let errors = Self::get_tx_errors(&init_proposal_tx, &tx).unwrap_or_default();
                    return Ok(StepResult::fail(errors));
                }
                Err(e) => match e {
                    namada_sdk::error::Error::Tx(TxSubmitError::AppliedTimeout) => {
                        return Err(TaskError::Timeout)
                    }
                    _ => return Ok(StepResult::fail(e.to_string())),
                },
            }
        }

        let storage_key = get_counter_key();
        // This returns the next proposal_id, so always subtract 1
        // If multiple proposal in the same block, this would not work
        let proposal_id = rpc::query_storage_value::<_, u64>(sdk.namada.client(), &storage_key)
            .await
            .unwrap()
            - 1;

        storage.add(
            TxInitPgfStewardProposalStorageKeys::ProposalId.to_string(),
            proposal_id.to_string(),
        );
        storage.add(
            TxInitPgfStewardProposalStorageKeys::ProposerAddress.to_string(),
            signer_address.to_string(),
        );
        storage.add(
            TxInitPgfStewardProposalStorageKeys::StartEpoch.to_string(),
            start_epoch.to_string(),
        );
        storage.add(
            TxInitPgfStewardProposalStorageKeys::EndEpoch.to_string(),
            end_epoch.to_string(),
        );
        storage.add(
            TxInitPgfStewardProposalStorageKeys::GraceEpoch.to_string(),
            grace_epoch.to_string(),
        );
        storage.add(
            TxInitPgfStewardProposalStorageKeys::StewardAdd.to_string(),
            signer_address.to_string(),
        );
        storage.add(
            TxInitPgfStewardProposalStorageKeys::StewardRemove.to_string(),
            serde_json::to_string(&stewards_to_remove).unwrap(),
        );

        Ok(StepResult::success(storage))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxInitPgfStewardProposalParametersDto {
    pub signer: Value,
    pub start_epoch: Option<Value>,
    pub end_epoch: Option<Value>,
    pub grace_epoch: Option<Value>,
    pub steward_remove: Vec<Value>,
}

#[derive(Clone, Debug)]
pub struct TxInitPgfStewardProposalParameters {
    signer: AccountIndentifier,
    start_epoch: Option<u64>,
    end_epoch: Option<u64>,
    grace_epoch: Option<u64>,
    steward_remove: Vec<AccountIndentifier>,
}

impl TaskParam for TxInitPgfStewardProposalParameters {
    type D = TxInitPgfStewardProposalParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Option<Self> {
        let steward_remove = dto
            .steward_remove
            .into_iter()
            .map(|value| match value {
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
            })
            .collect::<Vec<AccountIndentifier>>();
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
            steward_remove,
        })
    }
}
