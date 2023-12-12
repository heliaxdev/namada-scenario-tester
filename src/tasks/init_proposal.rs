use std::path::PathBuf;

use async_trait::async_trait;
use namada_sdk::{
    args::TxBuilder, core::ledger::governance::storage::keys::get_counter_key, rpc,
    signing::default_sign, Namada,
};

use serde::Deserialize;

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{
        valid_proposal::{ProposalType, ValidProposal},
        value::Value,
    },
};

use super::{Task, TaskParam};

pub enum TxInitProposalStorageKeys {
    ProposalId,
    StartEpoch,
    EndEpoch,
    GraceEpoch,
    ProposerAddress,
}

impl ToString for TxInitProposalStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxInitProposalStorageKeys::ProposalId => "proposal-id".to_string(),
            TxInitProposalStorageKeys::StartEpoch => "proposal-start-epoch".to_string(),
            TxInitProposalStorageKeys::EndEpoch => "proposal-end-epoch".to_string(),
            TxInitProposalStorageKeys::GraceEpoch => "proposal-grace-epoch".to_string(),
            TxInitProposalStorageKeys::ProposerAddress => "proposal-proposer-address".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxInitProposal {}

impl TxInitProposal {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxInitProposal {
    type P = TxInitProposalParameters;

    async fn execute(&self, sdk: &Sdk, parameters: Self::P, _state: &Storage) -> StepResult {
        // Params are validator: Address, source: Address, amount: u64
        let proposal_type = parameters.proposal_type;
        let signer_address = parameters.signer.to_namada_address(sdk).await;
        let start_epoch = parameters.start_epoch;
        let end_epoch = parameters.end_epoch;
        let grace_epoch = parameters.grace_epoch;

        let start_epoch = match start_epoch {
            Some(start_epoch) => start_epoch,
            None => {
                let current_epoch = rpc::query_epoch(sdk.namada.client()).await.unwrap();
                3 - (current_epoch.0) % 3 + current_epoch.0 + 3
            }
        };

        let end_epoch = match end_epoch {
            Some(end_epoch) => end_epoch,
            None => start_epoch + 12,
        };

        let grace_epoch = match grace_epoch {
            Some(grace_epoch) => grace_epoch,
            None => end_epoch + 6,
        };

        let signing_key = parameters.signer.to_public_key(sdk).await;

        let proposal = ValidProposal::new(
            signer_address.to_string(),
            start_epoch,
            end_epoch,
            grace_epoch,
            proposal_type,
        );
        let proposal_json = proposal.generate_proposal();

        // Eventually use the generate proposal.json file and then load it
        let proposal_data = proposal_json.to_string().as_bytes().to_vec();
        let init_proposal_tx_builder = sdk
            .namada
            .new_init_proposal(proposal_data)
            .signing_keys(vec![signing_key]);

        let (mut init_proposal_tx, signing_data, _option_epoch) = init_proposal_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build init_proposal tx");

        sdk.namada
            .sign(
                &mut init_proposal_tx,
                &init_proposal_tx_builder.tx,
                signing_data,
                default_sign,
                ()
            )
            .await
            .expect("unable to sign redelegate tx");
        let _tx = sdk
            .namada
            .submit(init_proposal_tx, &init_proposal_tx_builder.tx)
            .await;
        let mut storage = StepStorage::default();

        let storage_key = get_counter_key();
        // This returns the next proposal_id, so always subtract 1
        let proposal_id = rpc::query_storage_value::<_, u64>(sdk.namada.client(), &storage_key)
            .await
            .unwrap()
            - 1;

        storage.add(
            TxInitProposalStorageKeys::ProposalId.to_string(),
            proposal_id.to_string(),
        );
        storage.add(
            TxInitProposalStorageKeys::ProposerAddress.to_string(),
            signer_address.to_string(),
        );
        storage.add(
            TxInitProposalStorageKeys::StartEpoch.to_string(),
            start_epoch.to_string(),
        );
        storage.add(
            TxInitProposalStorageKeys::EndEpoch.to_string(),
            end_epoch.to_string(),
        );
        storage.add(
            TxInitProposalStorageKeys::GraceEpoch.to_string(),
            grace_epoch.to_string(),
        );

        self.fetch_info(sdk, &mut storage).await;

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TxInitProposalParametersDto {
    proposal_type: Value,
    signer: Value,
    start_epoch: Option<Value>,
    end_epoch: Option<Value>,
    grace_epoch: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct TxInitProposalParameters {
    proposal_type: ProposalType,
    signer: AccountIndentifier,
    start_epoch: Option<u64>,
    end_epoch: Option<u64>,
    grace_epoch: Option<u64>,
}

impl TaskParam for TxInitProposalParameters {
    type D = TxInitProposalParametersDto;

    fn from_dto(dto: Self::D, state: &Storage) -> Self {
        let proposal_type = match dto.proposal_type {
            Value::Ref { value: _ } => {
                unimplemented!()
            }
            Value::Value { value } => {
                if value.to_lowercase().eq("empty") {
                    println!("Generating empty proposal");
                    ProposalType::Empty
                } else if value.to_lowercase().eq("pgf_steward_proposal") {
                    ProposalType::PgfStewardProposal
                } else if value.to_lowercase().eq("pgf_funding_proposal") {
                    ProposalType::PgfFundingProposal
                } else {
                    ProposalType::Wasm(PathBuf::from(value))
                }
            }
            Value::Fuzz {} => unimplemented!(),
        };
        let signer = match dto.signer {
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
        let start_epoch = dto.start_epoch.map(|start_epoch| match start_epoch {
            Value::Ref { value } => {
                let epoch_string = state.get_step_item(&value, "epoch");
                let epoch_value = epoch_string.parse::<u64>().unwrap();
                (3 - epoch_value % 3) + epoch_value + 3
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        });
        let end_epoch = dto.end_epoch.map(|end_epoch| match end_epoch {
            Value::Ref { value: _ } => {
                unimplemented!()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        });
        let grace_epoch = dto.grace_epoch.map(|grace_epoch| match grace_epoch {
            Value::Ref { value: _ } => {
                unimplemented!()
            }
            Value::Value { value } => value.parse::<u64>().unwrap(),
            Value::Fuzz {} => unimplemented!(),
        });
        Self {
            proposal_type,
            signer,
            start_epoch,
            end_epoch,
            grace_epoch,
        }
    }
}
