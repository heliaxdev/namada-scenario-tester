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

#[derive(Clone, Debug, Default)]
pub struct VoteProposal {}

impl VoteProposal {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for VoteProposal {
    type P = VoteProposalParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        // Params are validator: Address, source: Address, amount: u64
        let proposal_id = parameters.proposal_id;
        let voter_address = parameters.voter.to_namada_address(sdk).await;
        let vote = parameters.vote;
        let signing_key = parameters.voter.to_secret_key(sdk).await;

        let vote_proposal_tx_builder = sdk
            .namada
            .new_vote_prposal(vote.clone(), voter_address)
            .proposal_id(proposal_id)
            .signing_keys(vec![signing_key]);
        let (mut vote_proposal_tx, signing_data, _option_epoch) = vote_proposal_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build vote_proposal tx");
        sdk.namada
            .sign(
                &mut vote_proposal_tx,
                &vote_proposal_tx_builder.tx,
                signing_data,
                default_sign,
            )
            .await
            .expect("unable to sign redelegate tx");
        let tx = sdk
            .namada
            .submit(vote_proposal_tx, &vote_proposal_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        storage.add("vote".to_string(), vote);

        self.fetch_info(sdk, &mut storage).await;
        if tx.is_ok() {
            return StepResult::success(storage);
        }
        StepResult::fail()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct VoteProposalParametersDto {
    proposal_id: Value,
    voter: Value,
    vote: Value,
}

#[derive(Clone, Debug)]
pub struct VoteProposalParameters {
    proposal_id: u64,
    voter: AccountIndentifier,
    vote: String,
}

impl TaskParam for VoteProposalParameters {
    type D = VoteProposalParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let proposal_id = match dto.proposal_id {
            Value::Ref { value } => {
                let id_string = state.get_step_item(&value, "proposal-id");
                id_string.parse::<u64>().unwrap()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        };
        let voter = match dto.voter {
            Value::Ref { value } => {
                let alias = state.get_step_item(&value, "address-alias");
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
            Value::Ref { value: _ } => {
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
