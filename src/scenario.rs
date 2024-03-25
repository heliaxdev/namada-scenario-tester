use std::fmt::Display;

use namada_sdk::proof_of_stake::BecomeValidator;
use serde::Deserialize;

use crate::{
    checks::{
        balance::{BalanceCheck, BalanceCheckParametersDto},
        bonds::{BondsCheck, BondsCheckParametersDto},
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
        become_validator::{BecomeValidatorParametersDto, TxBecomeValidator}, bond::{TxBond, TxBondParametersDto}, init_account::{TxInitAccount, TxInitAccountParametersDto}, init_default_proposal::{TxInitDefaultProposal, TxInitDefaultProposalParametersDto}, init_pgf_funding_proposal::{
            TxInitPgfFundingProposal, TxInitPgfFundingProposalParametersDto,
        }, init_pgf_steward_proposal::{
            TxInitPgfStewardProposal, TxInitPgfStewardProposalParametersDto,
        }, redelegate::{TxRedelegate, TxRedelegateParametersDto}, reveal_pk::{RevealPkParametersDto, TxRevealPk}, tx_transparent_transfer::{TxTransparentTransfer, TxTransparentTransferParametersDto}, unbond::{TxUnbond, TxUnbondParametersDto}, vote::{TxVoteProposal, TxVoteProposalParametersDto}, wallet_new_key::{WalletNewKey, WalletNewKeyParametersDto}, withdraw::{TxWithdraw, TxWithdrawParametersDto}, Task
    },
    waits::{
        epoch::{EpochWait, EpochWaitParametersDto},
        height::{HeightWait, HeightWaitParametersDto},
        Wait,
    },
};

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum StepType {
    #[serde(rename = "wallet-new-key")]
    WalletNewKey {
        parameters: WalletNewKeyParametersDto,
    },
    #[serde(rename = "tx-init-account")]
    InitAccount {
        parameters: TxInitAccountParametersDto,
    },
    #[serde(rename = "tx-transparent-transfer")]
    TransparentTransfer {
        parameters: TxTransparentTransferParametersDto,
    },
    #[serde(rename = "reveal-pk")]
    RevealPk { parameters: RevealPkParametersDto },
    #[serde(rename = "tx-bond")]
    Bond { parameters: TxBondParametersDto },
    #[serde(rename = "tx-unbond")]
    Unbond { parameters: TxUnbondParametersDto },
    #[serde(rename = "tx-withdraw")]
    Withdraw { parameters: TxWithdrawParametersDto },
    #[serde(rename = "tx-become-validator")]
    BecomeValidator { parameters: BecomeValidatorParametersDto },
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
    },
    #[serde(rename = "check-bonds")]
    CheckBonds { parameters: BondsCheckParametersDto },
    #[serde(rename = "tx-init-proposal")]
    InitProposal {
        parameters: TxInitDefaultProposalParametersDto,
    },
    #[serde(rename = "tx-init-pgf-steward-proposal")]
    InitStewardProposal {
        parameters: TxInitPgfStewardProposalParametersDto,
    },
    #[serde(rename = "tx-init-pgf-funding-proposal")]
    InitFundingProposal {
        parameters: TxInitPgfFundingProposalParametersDto,
    },
    #[serde(rename = "query-proposal")]
    QueryProposal {
        parameters: ProposalQueryParametersDto,
    },
    #[serde(rename = "tx-vote-proposal")]
    VoteProposal {
        parameters: TxVoteProposalParametersDto,
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
}

impl Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepType::WalletNewKey { .. } => write!(f, "wallet-new-key"),
            StepType::InitAccount { .. } => write!(f, "tx-init-account"),
            StepType::TransparentTransfer { .. } => write!(f, "tx-transparent-transfer"),
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
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Step {
    pub id: u64,
    pub config: StepType,
}

impl Step {
    pub async fn run(&self, storage: &Storage, sdk: &Sdk) -> StepResult {
        match self.config.to_owned() {
            StepType::WalletNewKey { parameters: dto } => {
                WalletNewKey::default().run(sdk, dto, storage).await
            }
            StepType::InitAccount { parameters: dto } => {
                TxInitAccount::default().run(sdk, dto, storage).await
            }
            StepType::TransparentTransfer { parameters: dto } => {
                TxTransparentTransfer::default()
                    .run(sdk, dto, storage)
                    .await
            }
            StepType::RevealPk { parameters: dto } => {
                TxRevealPk::default().run(sdk, dto, storage).await
            }
            StepType::Bond { parameters: dto } => TxBond::default().run(sdk, dto, storage).await,
            StepType::Unbond { parameters: dto } => {
                TxUnbond::default().run(sdk, dto, storage).await
            }
            StepType::Withdraw { parameters: dto } => {
                TxWithdraw::default().run(sdk, dto, storage).await
            }
            StepType::BecomeValidator { parameters: dto } => {
                TxBecomeValidator::default().run(sdk, dto, storage).await
            }
            StepType::CheckBalance { parameters: dto } => {
                BalanceCheck::default().run(sdk, dto, storage).await
            }
            StepType::CheckStepOutput { parameters: dto } => {
                StepCheck::default().run(sdk, dto, storage).await
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
            StepType::Redelegate { parameters } => {
                TxRedelegate::default().run(sdk, parameters, storage).await
            }
            StepType::CheckBonds { parameters } => {
                BondsCheck::default().run(sdk, parameters, storage).await
            }
            StepType::InitProposal { parameters } => {
                TxInitDefaultProposal::default()
                    .run(sdk, parameters, storage)
                    .await
            }
            StepType::InitStewardProposal { parameters } => {
                TxInitPgfStewardProposal::default()
                    .run(sdk, parameters, storage)
                    .await
            }
            StepType::InitFundingProposal { parameters } => {
                TxInitPgfFundingProposal::default()
                    .run(sdk, parameters, storage)
                    .await
            }
            StepType::QueryProposal { parameters } => {
                ProposalQuery::default().run(sdk, parameters, storage).await
            }
            StepType::CheckStorage { parameters } => {
                StorageCheck::default().run(sdk, parameters, storage).await
            }
            StepType::VoteProposal { parameters } => {
                TxVoteProposal::default()
                    .run(sdk, parameters, storage)
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
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScenarioSettings {
    pub retry_for: Option<u64>,
}

impl Default for ScenarioSettings {
    fn default() -> Self {
        Self { retry_for: Some(1) }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StepResult {
    pub outcome: StepOutcome,
    pub data: StepStorage,
    pub accounts: Vec<StateAddress>,
}

impl StepResult {
    pub fn is_succesful(&self) -> bool {
        self.outcome.is_succesful()
    }

    pub fn is_fail(&self) -> bool {
        self.outcome.is_fail()
    }

    pub fn success(data: StepStorage) -> Self {
        Self {
            outcome: StepOutcome::no_op(),
            data,
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

    pub fn fail() -> Self {
        Self {
            outcome: StepOutcome::fail(),
            data: StepStorage::default(),
            accounts: Vec::new(),
        }
    }

    pub fn fail_check() -> Self {
        Self {
            outcome: StepOutcome::check_fail(),
            data: StepStorage::default(),
            accounts: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Scenario {
    pub settings: ScenarioSettings,
    pub steps: Vec<Step>,
}
