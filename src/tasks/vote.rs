use async_trait::async_trait;
use namada_sdk::{
    args::TxBuilder, governance::utils::ProposalStatus, signing::default_sign, Namada,
};

use rand::Rng;
use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    queries::proposals::ProposalsQueryStorageKeys,
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::value::Value,
};

use super::{Task, TaskParam};

pub enum TxVoteProposalStorageKeys {
    Vote,
    VoterAddress,
}

impl ToString for TxVoteProposalStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxVoteProposalStorageKeys::Vote => "vote".to_string(),
            TxVoteProposalStorageKeys::VoterAddress => "voter-address".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxVoteProposal {}

impl TxVoteProposal {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxVoteProposal {
    type P = TxVoteProposalParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        // Params are validator: Address, source: Address, amount: u64
        let proposal_id = if let Some(id) = parameters.proposal_id {
            id
        } else {
            // no proposal id was specified or fuzzing couldn't find a suitable proposal to vote
            return StepResult::no_op();
        };
        let voter_address = parameters.voter.to_namada_address(sdk).await;
        let vote = parameters.vote;
        let signing_public_key = parameters.voter.to_public_key(sdk).await;

        let vote_proposal_tx_builder = sdk
            .namada
            .new_vote_prposal(vote.clone(), voter_address.clone())
            .proposal_id(proposal_id)
            .signing_keys(vec![signing_public_key]);

        let (mut vote_proposal_tx, signing_data) = vote_proposal_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build vote_proposal tx");

        sdk.namada
            .sign(
                &mut vote_proposal_tx,
                &vote_proposal_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign redelegate tx");
        let tx = sdk
            .namada
            .submit(vote_proposal_tx, &vote_proposal_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();

        if tx.is_err() {
            self.fetch_info(sdk, &mut storage).await;
            return StepResult::fail();
        }

        storage.add(TxVoteProposalStorageKeys::Vote.to_string(), vote);
        storage.add(
            TxVoteProposalStorageKeys::VoterAddress.to_string(),
            voter_address.to_string(),
        );

        self.fetch_info(sdk, &mut storage).await;
        if tx.is_ok() {
            return StepResult::success(storage);
        }
        StepResult::fail()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxVoteProposalParametersDto {
    pub proposal_id: Value,
    pub voter: Value,
    pub vote: Value,
}

#[derive(Clone, Debug)]
pub struct TxVoteProposalParameters {
    proposal_id: Option<u64>,
    voter: AccountIndentifier,
    vote: String,
}

impl TaskParam for TxVoteProposalParameters {
    type D = TxVoteProposalParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let proposal_id = match dto.proposal_id {
            Value::Ref { value, field } => {
                let id_string = state.get_step_item(&value, &field);
                id_string.parse::<u64>().ok()
            }
            Value::Value { value } => value.parse::<u64>().ok(),
            Value::Fuzz { value } => {
                let step_id = value.expect(
                    "Fuzzing vote proposal require a step id pointing to a QueryProposal step.",
                );
                let last_proposal_id = state
                    .get_step_item(
                        &step_id,
                        ProposalsQueryStorageKeys::Total.to_string().as_str(),
                    )
                    .parse::<u64>()
                    .unwrap();
                let mut proposal_id = None;
                for id in 0..last_proposal_id {
                    let maybe_proposal_status = state.get_step_item(
                        &step_id,
                        ProposalsQueryStorageKeys::ProposalStatus(id)
                            .to_string()
                            .as_str(),
                    );
                    if maybe_proposal_status.eq(&ProposalStatus::OnGoing.to_string()) {
                        proposal_id = Some(id);
                    }
                    if proposal_id.is_some() && rand::thread_rng().gen_range(0..=1) >= 1 {
                        break;
                    }
                }
                proposal_id
            }
        };
        let voter = match dto.voter {
            Value::Ref { value, field } => {
                let alias = state.get_step_item(&value, &field);
                AccountIndentifier::Address(state.get_address(&alias).address)
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
        let vote = match dto.vote {
            Value::Ref { value: _, field: _ } => {
                unimplemented!()
            }
            Value::Value { value } => value,
            Value::Fuzz { .. } => match rand::thread_rng().gen_range(0..3) {
                0 => "yay".to_string(),
                1 => "nay".to_string(),
                2 => "abstain".to_string(),
                _ => "abstain".to_string(),
            },
        };

        Self {
            proposal_id,
            voter,
            vote,
        }
    }
}
