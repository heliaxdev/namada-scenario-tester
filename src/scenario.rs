use std::fmt::Display;

use rand::seq::SliceRandom;
use serde::Deserialize;

use crate::{
    checks::{
        balance::{BalanceCheck, BalanceCheckParametersDto},
        tx::{TxCheck, TxCheckParametersDto},
        Check,
    },
    queries::{
        account::{AccountQuery, AccountQueryParametersDto},
        balance::{BalanceQuery, BalanceQueryParametersDto},
        Query,
    },
    state::state::{Address, StepOutcome, StepStorage, Storage},
    tasks::{
        init_account::{InitAccount, InitAccountParametersDto},
        tx_transparent_transfer::{TxTransparentTransfer, TxTransparentTransferParametersDto},
        wallet_new_key::{WalletNewKey, WalletNewKeyParametersDto},
        Task,
    },
    waits::{
        epoch::{EpochWait, EpochWaitParametersDto},
        height::{HeightWait, HeightWaitParametersDto},
        Wait,
    }, utils::settings::Settings, sdk::namada::Sdk,
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
        parameters: InitAccountParametersDto,
    },
    #[serde(rename = "tx-transparent-transfer")]
    TransparentTransfer {
        parameters: TxTransparentTransferParametersDto,
    },
    #[serde(rename = "check-balance")]
    CheckBalance {
        parameters: BalanceCheckParametersDto,
    },
    #[serde(rename = "check-tx")]
    CheckTxOutput { parameters: TxCheckParametersDto },
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
}

impl Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepType::WalletNewKey { .. } => write!(f, "wallet-new-key"),
            StepType::InitAccount { .. } => write!(f, "tx-init-account"),
            StepType::TransparentTransfer { .. } => write!(f, "tx-transparent-transfer"),
            StepType::CheckBalance { .. } => write!(f, "check-balance"),
            StepType::CheckTxOutput { .. } => write!(f, "check-tx"),
            StepType::WaitUntillEpoch { .. } => write!(f, "wait-epoch"),
            StepType::WaitUntillHeight { .. } => write!(f, "wait-height"),
            StepType::QueryAccountTokenBalance { .. } => write!(f, "query-balance"),
            StepType::QueryAccount { .. } => write!(f, "query-account"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Step {
    pub id: u64,
    pub config: StepType,
    pub settings: Option<Settings>,
}

impl Step {
    pub async fn run(&self, storage: &Storage, sdk: &Sdk<'_>) -> StepResult {
        match self.config.to_owned() {
            StepType::WalletNewKey { parameters: dto } => {
                WalletNewKey::new(&sdk).run(sdk, dto, storage).await
            }
            StepType::InitAccount { parameters: dto } => {
                InitAccount::new(&sdk).run(sdk, dto, storage).await
            }
            StepType::TransparentTransfer { parameters: dto } => {
                TxTransparentTransfer::new(&sdk).run(sdk, dto, storage).await
            }
            StepType::CheckBalance { parameters: dto } => {
                BalanceCheck::new(&sdk).run(sdk, dto, storage).await
            }
            StepType::CheckTxOutput { parameters: dto } => {
                TxCheck::new(&sdk).run(sdk, dto, storage).await
            }
            StepType::WaitUntillEpoch { parameters: dto } => {
                EpochWait::new(&sdk).run(dto, storage)
            }
            StepType::WaitUntillHeight { parameters: dto } => {
                HeightWait::new(&sdk).run(dto, storage)
            }
            StepType::QueryAccountTokenBalance { parameters: dto } => {
                BalanceQuery::new(&sdk).run(dto, storage)
            }
            StepType::QueryAccount { parameters: dto } => {
                AccountQuery::new(&sdk).run(dto, storage)
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StepResult {
    pub outcome: StepOutcome,
    pub data: StepStorage,
    pub accounts: Vec<Address>,
}

impl StepResult {
    pub fn is_succesful(&self) -> bool {
        self.outcome.is_succesful()
    }

    pub fn success(data: StepStorage) -> Self {
        Self {
            outcome: StepOutcome::success(),
            data,
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

    pub fn success_with_accounts(data: StepStorage, accounts: Vec<Address>) -> Self {
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
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Scenario {
    pub steps: Vec<Step>,
}
