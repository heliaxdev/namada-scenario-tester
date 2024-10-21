use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    checks::{
        balance::{BalanceCheck, BalanceCheckParametersDto},
        bonds::{BondsCheck, BondsCheckParametersDto},
        reveal_pk::{RevealPkCheck, RevealPkCheckParametersDto},
        step::{StepCheck, StepCheckParametersDto},
        storage::{StorageCheck, StorageCheckParametersDto},
        Check,
    },
    queries::{
        account::{AccountQuery, AccountQueryParametersDto},
        balance::{BalanceQuery, BalanceQueryParametersDto},
        bonded_stake::{BondedStakeQuery, BondedStakeQueryParametersDto},
        proposal::{ProposalQuery, ProposalQueryParametersDto},
        proposals::{ProposalsQuery, ProposalsQueryParametersDto},
        validators::{ValidatorsQuery, ValidatorsQueryParametersDto},
        Query,
    },
    sdk::namada::Sdk,
    state::state::{StateAddress, StepOutcome, StepStorage, Storage},
    tasks::{
        become_validator::{BecomeValidatorParametersDto, TxBecomeValidator},
        bond::{TxBond, TxBondParametersDto},
        bond_batch::{TxBondBatch, TxBondBatchParametersDto},
        change_consensus_key::{TxChangeConsensusKey, TxChangeConsensusKeyParametersDto},
        change_metadata::{TxChangeMetadata, TxChangeMetadataParametersDto},
        claim_rewards::{TxClaimRewards, TxClaimRewardsteParametersDto},
        deactivate_validator::{DeactivateValidatorParametersDto, TxDeactivateValidator},
        init_account::{TxInitAccount, TxInitAccountParametersDto},
        init_default_proposal::{TxInitDefaultProposal, TxInitDefaultProposalParametersDto},
        init_pgf_funding_proposal::{
            TxInitPgfFundingProposal, TxInitPgfFundingProposalParametersDto,
        },
        init_pgf_steward_proposal::{
            TxInitPgfStewardProposal, TxInitPgfStewardProposalParametersDto,
        },
        reactivate_validator::{ReactivateValidatorParametersDto, TxReactivateValidator},
        redelegate::{TxRedelegate, TxRedelegateParametersDto},
        redelegate_batch::{TxRedelegateBatch, TxRedelegateBatchParametersDto},
        reveal_pk::{RevealPkParametersDto, TxRevealPk},
        shielded_sync::{ShieldedSync, ShieldedSyncParametersDto},
        transparent_transfer_batch::{
            TxTransparentTransferBatch, TxTransparentTransferBatchParametersDto,
        },
        tx_shielded_transfer::{TxShieldedTransfer, TxShieldedTransferParametersDto},
        tx_shielding_transfer::{TxShieldingTransfer, TxShieldingTransferParametersDto},
        tx_transparent_transfer::{TxTransparentTransfer, TxTransparentTransferParametersDto},
        tx_unshielding_transfer::{TxUnshieldingTransfer, TxUnshieldingTransferParametersDto},
        unbond::{TxUnbond, TxUnbondParametersDto},
        update_account::{TxUpdateAccount, TxUpdateAccountParametersDto},
        vote::{TxVoteProposal, TxVoteProposalParametersDto},
        wallet_new_key::{WalletNewKey, WalletNewKeyParametersDto},
        withdraw::{TxWithdraw, TxWithdrawParametersDto},
        Task,
    },
    utils::settings::TxSettingsDto,
    waits::{
        epoch::{EpochWait, EpochWaitParametersDto},
        height::{HeightWait, HeightWaitParametersDto},
        Wait,
    },
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum StepType {
    #[serde(rename = "shielded-sync")]
    ShieldedSync,
    #[serde(rename = "wallet-new-key")]
    WalletNewKey {
        parameters: WalletNewKeyParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-init-account")]
    InitAccount {
        parameters: TxInitAccountParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-update-account")]
    UpdateAccount {
        parameters: TxUpdateAccountParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-transparent-transfer")]
    TransparentTransfer {
        parameters: TxTransparentTransferParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-shielding-transfer")]
    ShieldingTransfer {
        parameters: TxShieldingTransferParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-shielded-transfer")]
    ShieldedTransfer {
        parameters: TxShieldedTransferParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-unshielding-transfer")]
    UnshieldingTransfer {
        parameters: TxUnshieldingTransferParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "reveal-pk")]
    RevealPk {
        parameters: RevealPkParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-bond")]
    Bond {
        parameters: TxBondParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-unbond")]
    Unbond {
        parameters: TxUnbondParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-withdraw")]
    Withdraw {
        parameters: TxWithdrawParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-become-validator")]
    BecomeValidator {
        parameters: BecomeValidatorParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-change-metadata")]
    ChangeMetadata {
        parameters: TxChangeMetadataParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-change-consesus-key")]
    ChangeConsensusKey {
        parameters: TxChangeConsensusKeyParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-deactivate-validator")]
    DeactivateValidator {
        parameters: DeactivateValidatorParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-reactivate-validator")]
    ReactivateValidator {
        parameters: ReactivateValidatorParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-claim-rewards")]
    ClaimRewards {
        parameters: TxClaimRewardsteParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "check-balance")]
    CheckBalance {
        parameters: BalanceCheckParametersDto,
    },
    #[serde(rename = "check-step")]
    CheckStepOutput { parameters: StepCheckParametersDto },
    #[serde(rename = "wait-epoch")]
    WaitUntillEpoch { parameters: EpochWaitParametersDto },
    #[serde(rename = "wait-height")]
    WaitUntillHeight { parameters: HeightWaitParametersDto },
    #[serde(rename = "query-balance")]
    QueryAccountTokenBalance {
        parameters: BalanceQueryParametersDto,
    },
    #[serde(rename = "query-account")]
    QueryAccount {
        parameters: AccountQueryParametersDto,
    },
    #[serde(rename = "query-bonded-stake")]
    QueryBondedStake {
        parameters: BondedStakeQueryParametersDto,
    },
    #[serde(rename = "tx-redelegate")]
    Redelegate {
        parameters: TxRedelegateParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "check-bonds")]
    CheckBonds { parameters: BondsCheckParametersDto },
    #[serde(rename = "check-reveal-pk")]
    CheckRevealPk {
        parameters: RevealPkCheckParametersDto,
    },
    #[serde(rename = "tx-init-proposal")]
    InitProposal {
        parameters: TxInitDefaultProposalParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-init-pgf-steward-proposal")]
    InitStewardProposal {
        parameters: TxInitPgfStewardProposalParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-init-pgf-funding-proposal")]
    InitFundingProposal {
        parameters: TxInitPgfFundingProposalParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "query-proposal")]
    QueryProposal {
        parameters: ProposalQueryParametersDto,
    },
    #[serde(rename = "tx-vote-proposal")]
    VoteProposal {
        parameters: TxVoteProposalParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "check-storage")]
    CheckStorage {
        parameters: StorageCheckParametersDto,
    },
    #[serde(rename = "query-validators")]
    QueryValidators {
        parameters: ValidatorsQueryParametersDto,
    },
    #[serde(rename = "query-proposals")]
    QueryProposals {
        parameters: ProposalsQueryParametersDto,
    },
    #[serde(rename = "tx-transparent-transfer-batch")]
    TransparentTransferBatch {
        parameters: TxTransparentTransferBatchParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-bond-batch")]
    BondBatch {
        parameters: TxBondBatchParametersDto,
        settings: Option<TxSettingsDto>,
    },
    #[serde(rename = "tx-redelegate-batch")]
    RedelegateBatch {
        parameters: TxRedelegateBatchParametersDto,
        settings: Option<TxSettingsDto>,
    },
}

impl Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepType::ShieldedSync => write!(f, "shielded-sync"),
            StepType::WalletNewKey { .. } => write!(f, "wallet-new-key"),
            StepType::InitAccount { .. } => write!(f, "tx-init-account"),
            StepType::TransparentTransfer { .. } => write!(f, "tx-transparent-transfer"),
            StepType::ShieldingTransfer { .. } => write!(f, "tx-shielding-transfer"),
            StepType::ShieldedTransfer { .. } => write!(f, "tx-shielded-transfer"),
            StepType::UnshieldingTransfer { .. } => write!(f, "tx-unshielding-transfer"),
            StepType::RevealPk { .. } => write!(f, "tx-reveal-pk"),
            StepType::Bond { .. } => write!(f, "tx-bond"),
            StepType::Unbond { .. } => write!(f, "tx-unbond"),
            StepType::Withdraw { .. } => write!(f, "tx-withdraw"),
            StepType::Redelegate { .. } => write!(f, "tx-redelegate"),
            StepType::CheckBalance { .. } => write!(f, "check-balance"),
            StepType::CheckStepOutput { .. } => write!(f, "check-tx"),
            StepType::WaitUntillEpoch { .. } => write!(f, "wait-epoch"),
            StepType::WaitUntillHeight { .. } => write!(f, "wait-height"),
            StepType::QueryAccountTokenBalance { .. } => write!(f, "query-balance"),
            StepType::QueryAccount { .. } => write!(f, "query-account"),
            StepType::QueryBondedStake { .. } => write!(f, "query-bonded-stake"),
            StepType::CheckBonds { .. } => write!(f, "check-bonds"),
            StepType::InitProposal { .. } => write!(f, "tx-init-proposal"),
            StepType::QueryProposal { .. } => write!(f, "query-proposal"),
            StepType::VoteProposal { .. } => write!(f, "tx-vote-proposal"),
            StepType::CheckStorage { .. } => write!(f, "check-storage"),
            StepType::QueryValidators { .. } => write!(f, "query-validators"),
            StepType::QueryProposals { .. } => write!(f, "query-proposals"),
            StepType::InitStewardProposal { .. } => write!(f, "tx-pgf-steward-proposals"),
            StepType::InitFundingProposal { .. } => write!(f, "tx-pgf-funding-proposals"),
            StepType::BecomeValidator { .. } => write!(f, "tx-become-validator"),
            StepType::ChangeMetadata { .. } => write!(f, "tx-change-metadata"),
            StepType::ChangeConsensusKey { .. } => write!(f, "tx-change-consensus-key"),
            StepType::DeactivateValidator { .. } => write!(f, "tx-deactivate-validator"),
            StepType::ReactivateValidator { .. } => write!(f, "tx-reactivate-validator"),
            StepType::ClaimRewards { .. } => write!(f, "tx-claim-rewards"),
            StepType::UpdateAccount { .. } => write!(f, "tx-update-account"),
            StepType::CheckRevealPk { .. } => write!(f, "check-reveal-pk"),
            StepType::TransparentTransferBatch { .. } => write!(f, "transparent-transfer-batch"),
            StepType::BondBatch { .. } => write!(f, "bond-batch"),
            StepType::RedelegateBatch { .. } => write!(f, "redelegate-batch"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Step {
    pub id: u64,
    pub config: StepType,
}

impl Step {
    pub async fn run(&self, storage: &Storage, sdk: &Sdk, avoid_check: bool) -> StepResult {
        match self.config.to_owned() {
            StepType::ShieldedSync => {
                ShieldedSync::default()
                    .run(sdk, ShieldedSyncParametersDto, Default::default(), storage)
                    .await
            }
            StepType::WalletNewKey {
                parameters: dto,
                settings,
            } => {
                WalletNewKey::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::UpdateAccount {
                parameters: dto,
                settings,
            } => {
                TxUpdateAccount::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::InitAccount {
                parameters: dto,
                settings,
            } => {
                TxInitAccount::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::TransparentTransfer {
                parameters: dto,
                settings,
            } => {
                TxTransparentTransfer::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::ShieldingTransfer {
                parameters: dto,
                settings,
            } => {
                TxShieldingTransfer::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::ShieldedTransfer {
                parameters: dto,
                settings,
            } => {
                TxShieldedTransfer::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::UnshieldingTransfer {
                parameters: dto,
                settings,
            } => {
                TxUnshieldingTransfer::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::RevealPk {
                parameters: dto,
                settings,
            } => TxRevealPk::default().run(sdk, dto, settings, storage).await,
            StepType::Bond {
                parameters: dto,
                settings,
            } => TxBond::default().run(sdk, dto, settings, storage).await,
            StepType::Unbond {
                parameters: dto,
                settings,
            } => TxUnbond::default().run(sdk, dto, settings, storage).await,
            StepType::Withdraw {
                parameters: dto,
                settings,
            } => TxWithdraw::default().run(sdk, dto, settings, storage).await,
            StepType::BecomeValidator {
                parameters: dto,
                settings,
            } => {
                TxBecomeValidator::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::ChangeMetadata {
                parameters: dto,
                settings,
            } => {
                TxChangeMetadata::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::ChangeConsensusKey {
                parameters: dto,
                settings,
            } => {
                TxChangeConsensusKey::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::DeactivateValidator {
                parameters: dto,
                settings,
            } => {
                TxDeactivateValidator::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::ReactivateValidator {
                parameters: dto,
                settings,
            } => {
                TxReactivateValidator::default()
                    .run(sdk, dto, settings, storage)
                    .await
            }
            StepType::CheckBalance { parameters: dto } => {
                BalanceCheck::default()
                    .run(sdk, dto, storage, avoid_check)
                    .await
            }
            StepType::CheckStepOutput { parameters: dto } => {
                StepCheck::default()
                    .run(sdk, dto, storage, avoid_check)
                    .await
            }
            StepType::WaitUntillEpoch { parameters: dto } => {
                EpochWait::default().run(sdk, dto, storage).await
            }
            StepType::WaitUntillHeight { parameters: dto } => {
                HeightWait::default().run(sdk, dto, storage).await
            }
            StepType::QueryAccountTokenBalance { parameters: dto } => {
                BalanceQuery::default().run(sdk, dto, storage).await
            }
            StepType::QueryAccount { parameters: dto } => {
                AccountQuery::default().run(sdk, dto, storage).await
            }
            StepType::QueryBondedStake { parameters: dto } => {
                BondedStakeQuery::default().run(sdk, dto, storage).await
            }
            StepType::ClaimRewards {
                parameters,
                settings,
            } => {
                TxClaimRewards::default()
                    .run(sdk, parameters, settings, storage)
                    .await
            }
            StepType::Redelegate {
                parameters,
                settings,
            } => {
                TxRedelegate::default()
                    .run(sdk, parameters, settings, storage)
                    .await
            }
            StepType::CheckBonds { parameters } => {
                BondsCheck::default()
                    .run(sdk, parameters, storage, avoid_check)
                    .await
            }
            StepType::InitProposal {
                parameters,
                settings,
            } => {
                TxInitDefaultProposal::default()
                    .run(sdk, parameters, settings, storage)
                    .await
            }
            StepType::InitStewardProposal {
                parameters,
                settings,
            } => {
                TxInitPgfStewardProposal::default()
                    .run(sdk, parameters, settings, storage)
                    .await
            }
            StepType::InitFundingProposal {
                parameters,
                settings,
            } => {
                TxInitPgfFundingProposal::default()
                    .run(sdk, parameters, settings, storage)
                    .await
            }
            StepType::QueryProposal { parameters } => {
                ProposalQuery::default().run(sdk, parameters, storage).await
            }
            StepType::CheckStorage { parameters } => {
                StorageCheck::default()
                    .run(sdk, parameters, storage, avoid_check)
                    .await
            }
            StepType::VoteProposal {
                parameters,
                settings,
            } => {
                TxVoteProposal::default()
                    .run(sdk, parameters, settings, storage)
                    .await
            }
            StepType::QueryValidators { parameters } => {
                ValidatorsQuery::default()
                    .run(sdk, parameters, storage)
                    .await
            }
            StepType::QueryProposals { parameters } => {
                ProposalsQuery::default()
                    .run(sdk, parameters, storage)
                    .await
            }
            StepType::CheckRevealPk { parameters } => {
                RevealPkCheck::default()
                    .run(sdk, parameters, storage, avoid_check)
                    .await
            }
            StepType::TransparentTransferBatch {
                parameters,
                settings,
            } => {
                TxTransparentTransferBatch::default()
                    .run(sdk, parameters, settings, storage)
                    .await
            }
            StepType::BondBatch {
                parameters,
                settings,
            } => {
                TxBondBatch::default()
                    .run(sdk, parameters, settings, storage)
                    .await
            }
            StepType::RedelegateBatch {
                parameters,
                settings,
            } => {
                TxRedelegateBatch::default()
                    .run(sdk, parameters, settings, storage)
                    .await
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScenarioSettings {
    pub retry_for: Option<u64>,
}

impl Default for ScenarioSettings {
    fn default() -> Self {
        Self { retry_for: Some(1) }
    }
}

#[derive(Clone, Debug)]
pub struct StepResult {
    pub outcome: StepOutcome,
    pub data: StepStorage,
    pub accounts: Vec<StateAddress>,
}

impl Default for StepResult {
    fn default() -> Self {
        Self {
            outcome: StepOutcome::success(),
            data: Default::default(),
            accounts: Default::default(),
        }
    }
}

impl StepResult {
    pub fn is_succesful(&self) -> bool {
        self.outcome.is_succesful()
    }

    pub fn is_strict_succesful(&self) -> bool {
        self.outcome.is_strict_succesful()
    }

    pub fn is_fail(&self) -> bool {
        self.outcome.is_fail()
    }

    pub fn is_noop(&self) -> bool {
        self.outcome.is_noop()
    }

    pub fn is_skip(&self) -> bool {
        self.outcome.is_skip()
    }

    pub fn fail_error(&self) -> String {
        match &self.outcome {
            StepOutcome::Success => panic!(),
            StepOutcome::Fail(err) => err.to_owned(),
            StepOutcome::CheckSkip(_) => panic!(),
            StepOutcome::CheckFail(_, _) => panic!(),
            StepOutcome::NoOp => panic!(),
        }
    }

    pub fn success(data: StepStorage) -> Self {
        Self {
            outcome: StepOutcome::success(),
            data,
            accounts: Vec::new(),
        }
    }

    pub fn skip_check(outcome: bool) -> Self {
        Self {
            outcome: StepOutcome::skip_check(outcome),
            data: StepStorage::default(),
            accounts: Vec::new(),
        }
    }

    pub fn no_op() -> Self {
        Self {
            outcome: StepOutcome::no_op(),
            data: StepStorage::default(),
            accounts: Vec::new(),
        }
    }

    pub fn success_empty() -> Self {
        Self {
            outcome: StepOutcome::success(),
            data: StepStorage::default(),
            accounts: Vec::new(),
        }
    }

    pub fn success_with_accounts(data: StepStorage, accounts: Vec<StateAddress>) -> Self {
        Self {
            outcome: StepOutcome::success(),
            data,
            accounts,
        }
    }

    pub fn fail(error: String) -> Self {
        Self {
            outcome: StepOutcome::fail(error),
            data: StepStorage::default(),
            accounts: Vec::new(),
        }
    }

    pub fn fail_check(actual: String, expected: String) -> Self {
        Self {
            outcome: StepOutcome::check_fail(actual, expected),
            data: StepStorage::default(),
            accounts: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Scenario {
    pub settings: ScenarioSettings,
    pub steps: Vec<Step>,
}
