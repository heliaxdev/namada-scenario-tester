use async_trait::async_trait;
use namada_sdk::{args::TxBuilder, signing::default_sign, Namada};

use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
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
        let proposal_id = parameters.proposal_id;
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

        if tx.is_err() || tx.unwrap().is_applied_and_valid().is_none() {
            self.fetch_info(sdk, &mut storage).await;
            return StepResult::fail();
        }

        storage.add(TxVoteProposalStorageKeys::Vote.to_string(), vote);
        storage.add(
            TxVoteProposalStorageKeys::VoterAddress.to_string(),
            voter_address.to_string(),
        );

        self.fetch_info(sdk, &mut storage).await;
        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxVoteProposalParametersDto {
    proposal_id: Value,
    voter: Value,
    vote: Value,
}

#[derive(Clone, Debug)]
pub struct TxVoteProposalParameters {
    proposal_id: u64,
    voter: AccountIndentifier,
    vote: String,
}

impl TaskParam for TxVoteProposalParameters {
    type D = TxVoteProposalParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let proposal_id = match dto.proposal_id {
            Value::Ref { value, field } => {
                let id_string = state.get_step_item(&value, &field);
                id_string.parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
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
            Value::Fuzz {} => unimplemented!(),
        };
        let vote = match dto.vote {
            Value::Ref { value: _, field: _ } => {
                unimplemented!()
            }
            Value::Value { value } => value,
            Value::Fuzz {} => unimplemented!(),
        };

        Self {
            proposal_id,
            voter,
            vote,
        }
    }
}
