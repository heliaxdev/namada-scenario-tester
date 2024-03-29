use std::collections::BTreeMap;

use async_trait::async_trait;
use namada_sdk::{
    args::{InitProposal, TxBuilder},
    governance::{
        cli::onchain::{OnChainProposal, PgfFunding, PgfFundingProposal},
        storage::{
            keys::get_counter_key,
            proposal::{PGFInternalTarget, PGFTarget},
        },
    },
    rpc,
    signing::default_sign,
    token, Namada,
};

use serde::{Deserialize, Serialize};

use crate::{
    entity::address::{AccountIndentifier, ADDRESS_PREFIX},
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StepStorage, Storage},
    utils::{settings::TxSettings, value::Value},
};

use super::{Task, TaskParam};

pub enum TxInitPgfFundingProposalStorageKeys {
    ProposalId,
    StartEpoch,
    EndEpoch,
    GraceEpoch,
    ProposerAddress,
    ContinousPgf,
    RetroPgf,
}

impl ToString for TxInitPgfFundingProposalStorageKeys {
    fn to_string(&self) -> String {
        match self {
            TxInitPgfFundingProposalStorageKeys::ProposalId => "proposal-id".to_string(),
            TxInitPgfFundingProposalStorageKeys::StartEpoch => "proposal-start-epoch".to_string(),
            TxInitPgfFundingProposalStorageKeys::EndEpoch => "proposal-end-epoch".to_string(),
            TxInitPgfFundingProposalStorageKeys::GraceEpoch => "proposal-grace-epoch".to_string(),
            TxInitPgfFundingProposalStorageKeys::ProposerAddress => {
                "proposal-proposer-address".to_string()
            }
            TxInitPgfFundingProposalStorageKeys::ContinousPgf => "proposal-continous".to_string(),
            TxInitPgfFundingProposalStorageKeys::RetroPgf => "proposal-retro".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TxInitPgfFundingProposal {}

impl TxInitPgfFundingProposal {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl Task for TxInitPgfFundingProposal {
    type P = TxInitPgfFundingProposalParameters;
    type B = InitProposal;

    async fn execute(
        &self,
        sdk: &Sdk,
        parameters: Self::P,
        _settings: TxSettings,
        _state: &Storage,
    ) -> StepResult {
        let signer_address = parameters.signer.to_namada_address(sdk).await;
        let start_epoch = parameters.start_epoch;
        let end_epoch = parameters.end_epoch;
        let grace_epoch = parameters.grace_epoch;

        let mut continous_targets = vec![];
        for source in parameters.continous_funding_target {
            let address = source.to_namada_address(sdk).await;
            continous_targets.push(address);
        }

        let mut retro_targets = vec![];
        for source in parameters.retro_funding_target {
            let address = source.to_namada_address(sdk).await;
            retro_targets.push(address);
        }

        let continous_amounts = parameters.continous_funding_amount.to_vec();
        let retro_amounts = parameters.retro_funding_amount.to_vec();

        let continous_pgf = continous_targets
            .iter()
            .zip(continous_amounts)
            .map(|(target, amount)| {
                PGFTarget::Internal(PGFInternalTarget {
                    target: target.clone(),
                    amount: token::Amount::from_u64(amount),
                })
            })
            .collect::<Vec<PGFTarget>>();

        let retro_pgf = retro_targets
            .iter()
            .zip(retro_amounts)
            .map(|(target, amount)| {
                PGFTarget::Internal(PGFInternalTarget {
                    target: target.clone(),
                    amount: token::Amount::from_u64(amount),
                })
            })
            .collect::<Vec<PGFTarget>>();

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

        let signing_key = parameters.signer.to_public_key(sdk).await;

        let pgf_funding_proposal = PgfFundingProposal {
            proposal: OnChainProposal {
                id: 0,
                content: BTreeMap::from_iter([("scenario".to_string(), "tester".to_string())]),
                author: signer_address.clone(),
                voting_start_epoch: start_epoch.into(),
                voting_end_epoch: end_epoch.into(),
                grace_epoch: grace_epoch.into(),
            },
            data: PgfFunding {
                continuous: continous_pgf.clone(),
                retro: retro_pgf.clone(),
            },
        };
        let proposal_json = serde_json::to_string(&pgf_funding_proposal).unwrap();

        let init_proposal_tx_builder = sdk
            .namada
            .new_init_proposal(proposal_json.into_bytes())
            .is_pgf_funding(true)
            .force(true)
            .signing_keys(vec![signing_key.clone()]);

        let (mut init_proposal_tx, signing_data) = init_proposal_tx_builder
            .build(&sdk.namada)
            .await
            .expect("unable to build init_proposal tx");

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
            .submit(init_proposal_tx, &init_proposal_tx_builder.tx)
            .await;

        let mut storage = StepStorage::default();
        self.fetch_info(sdk, &mut storage).await;

        if Self::is_tx_rejected(&tx) {
            let errors = Self::get_tx_errors(&tx.unwrap()).unwrap_or_default();
            return StepResult::fail(errors);
        }

        let storage_key = get_counter_key();
        // This returns the next proposal_id, so always subtract 1
        // If multiple proposal in the same block, this would not work
        let proposal_id = rpc::query_storage_value::<_, u64>(sdk.namada.client(), &storage_key)
            .await
            .unwrap()
            - 1;

        storage.add(
            TxInitPgfFundingProposalStorageKeys::ProposalId.to_string(),
            proposal_id.to_string(),
        );
        storage.add(
            TxInitPgfFundingProposalStorageKeys::ProposerAddress.to_string(),
            signer_address.to_string(),
        );
        storage.add(
            TxInitPgfFundingProposalStorageKeys::StartEpoch.to_string(),
            start_epoch.to_string(),
        );
        storage.add(
            TxInitPgfFundingProposalStorageKeys::EndEpoch.to_string(),
            end_epoch.to_string(),
        );
        storage.add(
            TxInitPgfFundingProposalStorageKeys::GraceEpoch.to_string(),
            grace_epoch.to_string(),
        );
        storage.add(
            TxInitPgfFundingProposalStorageKeys::ContinousPgf.to_string(),
            serde_json::to_string(&continous_pgf).unwrap(),
        );
        storage.add(
            TxInitPgfFundingProposalStorageKeys::RetroPgf.to_string(),
            serde_json::to_string(&retro_pgf).unwrap(),
        );

        StepResult::success(storage)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxInitPgfFundingProposalParametersDto {
    pub signer: Value,
    pub start_epoch: Option<Value>,
    pub end_epoch: Option<Value>,
    pub grace_epoch: Option<Value>,
    pub continous_funding_target: Vec<Value>,
    pub retro_funding_target: Vec<Value>,
    pub continous_funding_amount: Vec<Value>,
    pub retro_funding_amount: Vec<Value>,
}

#[derive(Clone, Debug)]
pub struct TxInitPgfFundingProposalParameters {
    signer: AccountIndentifier,
    start_epoch: Option<u64>,
    end_epoch: Option<u64>,
    grace_epoch: Option<u64>,
    continous_funding_target: Vec<AccountIndentifier>,
    retro_funding_target: Vec<AccountIndentifier>,
    continous_funding_amount: Vec<u64>,
    retro_funding_amount: Vec<u64>,
}

impl TaskParam for TxInitPgfFundingProposalParameters {
    type D = TxInitPgfFundingProposalParametersDto;

    fn parameter_from_dto(dto: Self::D, state: &Storage) -> Self {
        let continous_funding_target = dto
            .continous_funding_target
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
        let retro_funding_target = dto
            .retro_funding_target
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
        let continous_funding_amount = dto
            .continous_funding_amount
            .into_iter()
            .map(|value| match value {
                Value::Ref { .. } => unimplemented!(),
                Value::Value { value } => value.parse::<u64>().unwrap(),
                Value::Fuzz { .. } => unimplemented!(),
            })
            .collect();
        let retro_funding_amount = dto
            .retro_funding_amount
            .into_iter()
            .map(|value| match value {
                Value::Ref { .. } => unimplemented!(),
                Value::Value { value } => value.parse::<u64>().unwrap(),
                Value::Fuzz { .. } => unimplemented!(),
            })
            .collect();
        let signer = match dto.signer {
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
        Self {
            signer,
            start_epoch,
            end_epoch,
            grace_epoch,
            continous_funding_target,
            retro_funding_target,
            continous_funding_amount,
            retro_funding_amount,
        }
    }
}
