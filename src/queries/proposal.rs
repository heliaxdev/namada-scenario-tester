use async_trait::async_trait;
use namada_sdk::{types::storage::Epoch, rpc, Namada};
use serde::Deserialize;

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Query, QueryParam};

pub enum ProposalQueryStorageKeys {
    ProposalId,
    StartEpoch,
    EndEpoch,
    GraceEpoch,
    ProposerAddress,
    ProposalStatus,
}

impl ToString for ProposalQueryStorageKeys {
    fn to_string(&self) -> String {
        match self {
            ProposalQueryStorageKeys::ProposalId => "proposal-id".to_string(),
            ProposalQueryStorageKeys::StartEpoch => "proposal-start-epoch".to_string(),
            ProposalQueryStorageKeys::EndEpoch => "proposal-end-epoch".to_string(),
            ProposalQueryStorageKeys::GraceEpoch => "proposal-grace-epoch".to_string(),
            ProposalQueryStorageKeys::ProposerAddress => "proposal-proposer-address".to_string(),
            ProposalQueryStorageKeys::ProposalStatus => "proposal-status".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProposalQuery {}

impl ProposalQuery {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Query for ProposalQuery {
    type P = ProposalQueryParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        let epoch = if let Some(epoch) = parameters.epoch {
            Epoch::from(epoch)
        } else {
            rpc::query_epoch(sdk.namada.client())
                .await
                .expect("Should be able to query for epoch")
        };

        let proposal_id = parameters.proposal_id;

        let active_proposals = rpc::query_proposal_by_id(sdk.namada.client(), proposal_id)
            .await
            .expect("Should be able to query for proposal");

        let mut storage = StepStorage::default();

        if let Some(storage_proposal) = active_proposals {
            let proposal_status = storage_proposal.get_status(epoch);
            let start_epoch = storage_proposal.voting_start_epoch;
            let end_epoch = storage_proposal.voting_end_epoch;
            let grace_epoch = storage_proposal.grace_epoch;
            storage.add(
                ProposalQueryStorageKeys::ProposalStatus.to_string(),
                proposal_status.to_string(),
            );
            storage.add(
                ProposalQueryStorageKeys::StartEpoch.to_string(),
                start_epoch.to_string(),
            );
            storage.add(
                ProposalQueryStorageKeys::EndEpoch.to_string(),
                end_epoch.to_string(),
            );
            storage.add(
                ProposalQueryStorageKeys::GraceEpoch.to_string(),
                grace_epoch.to_string(),
            );
            return StepResult::success(storage);
        }
        StepResult::fail()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProposalQueryParametersDto {
    proposal_id: Value,
    epoch: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct ProposalQueryParameters {
    proposal_id: u64,
    epoch: Option<u64>,
}

impl QueryParam for ProposalQueryParameters {
    type D = ProposalQueryParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let epoch = match dto.epoch {
            Some(Value::Ref { value, field }) => {
                let epoch = state.get_step_item(&value, &field);
                epoch.parse::<u64>().ok()
            }
            Some(Value::Value { value }) => value.parse::<u64>().ok(),
            Some(Value::Fuzz {}) => unimplemented!(),
            _ => None,
        };
        let proposal_id = match dto.proposal_id {
            Value::Ref { value, field } => {
                let proposal_id = state.get_step_item(&value, &field);
                proposal_id.parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        };

        Self { proposal_id, epoch }
    }
}
