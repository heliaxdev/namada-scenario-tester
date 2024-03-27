use async_trait::async_trait;
use namada_sdk::{rpc, Namada};
use serde::{Deserialize, Serialize};

use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
};

use super::{Query, QueryParam};

pub enum ProposalsQueryStorageKeys {
    ProposalId(u64),
    StartEpoch(u64),
    EndEpoch(u64),
    GraceEpoch(u64),
    ProposerAddress(u64),
    ProposalStatus(u64),
    Total,
}

impl ToString for ProposalsQueryStorageKeys {
    fn to_string(&self) -> String {
        match self {
            ProposalsQueryStorageKeys::ProposalId(id) => format!("proposal-id-{}", id),
            ProposalsQueryStorageKeys::StartEpoch(id) => format!("proposal-start-epoch-{}", id),
            ProposalsQueryStorageKeys::EndEpoch(id) => format!("proposal-end-epoch-{}", id),
            ProposalsQueryStorageKeys::GraceEpoch(id) => format!("proposal-grace-epoch-{}", id),
            ProposalsQueryStorageKeys::ProposerAddress(id) => {
                format!("proposal-proposer-address-{}", id)
            }
            ProposalsQueryStorageKeys::ProposalStatus(id) => format!("proposal-status-{}", id),
            ProposalsQueryStorageKeys::Total => "total".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProposalsQuery {}

impl ProposalsQuery {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Query for ProposalsQuery {
    type P = ProposalsQueryParameters;

    async fn execute(&self, sdk: &Sdk, _parameters: Self::P, _state: &Storage) -> StepResult {
        let epoch = rpc::query_epoch(sdk.namada.client())
            .await
            .expect("Should be able to query for epoch");

        let last_proposal_id_key = namada_sdk::governance::storage::keys::get_counter_key();
        let last_proposal_id: u64 =
            rpc::query_storage_value(sdk.namada.client(), &last_proposal_id_key)
                .await
                .expect("counter key must be present");

        let mut storage = StepStorage::default();

        storage.add(
            ProposalsQueryStorageKeys::Total.to_string(),
            last_proposal_id.to_string(),
        );

        for id in (0..last_proposal_id).rev() {
            let active_proposals = rpc::query_proposal_by_id(sdk.namada.client(), id)
                .await
                .expect("Should be able to query for proposal");

            if let Some(storage_proposal) = active_proposals {
                let proposal_status = storage_proposal.get_status(epoch);
                let start_epoch = storage_proposal.voting_start_epoch;
                let end_epoch = storage_proposal.voting_end_epoch;
                let grace_epoch = storage_proposal.grace_epoch;
                let author = storage_proposal.author;
                storage.add(
                    ProposalsQueryStorageKeys::ProposalStatus(id).to_string(),
                    proposal_status.to_string(),
                );
                storage.add(
                    ProposalsQueryStorageKeys::StartEpoch(id).to_string(),
                    start_epoch.to_string(),
                );
                storage.add(
                    ProposalsQueryStorageKeys::EndEpoch(id).to_string(),
                    end_epoch.to_string(),
                );
                storage.add(
                    ProposalsQueryStorageKeys::GraceEpoch(id).to_string(),
                    grace_epoch.to_string(),
                );
                storage.add(
                    ProposalsQueryStorageKeys::ProposerAddress(id).to_string(),
                    author.to_string(),
                );
            }
        }

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProposalsQueryParametersDto {}

#[derive(Clone, Debug)]
pub struct ProposalsQueryParameters {}

impl QueryParam for ProposalsQueryParameters {
    type D = ProposalsQueryParametersDto;

    fn from_dto(_dto: Self::D, _state: &Storage) -> Self {
        Self {}
    }
}
