use std::fmt::Display;

use serde::Deserialize;

use crate::{
    checks::{
        balance::{BalanceCheck, BalanceCheckParametersDto},
        tx::{TxCheck, TxCheckParametersDto},
        Check,
    },
    state::state::{Address, StepOutcome, StepStorage, Storage},
    tasks::{
        init_account::{InitAccount, InitAccountParametersDto},
        tx_transparent_transfer::{TxTransparentTransfer, TxTransparentTransferParametersDto},
        wallet_new_key::{WalletNewKey, WalletNewKeyParametersDto},
        Task,
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
}

impl Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepType::WalletNewKey { .. } => write!(f, "wallet-new-key"),
            StepType::InitAccount { .. } => write!(f, "tx-init-account"),
            StepType::TransparentTransfer { .. } => write!(f, "tx-transparent-transfer"),
            StepType::CheckBalance { .. } => write!(f, "check-balance"),
            StepType::CheckTxOutput { .. } => write!(f, "check-tx"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Step {
    pub id: u64,
    pub config: StepType,
}

impl Step {
    pub fn run(&self, storage: &Storage) -> StepResult {
        match self.config.to_owned() {
            StepType::WalletNewKey { parameters: dto } => WalletNewKey::default().run(dto, storage),
            StepType::InitAccount { parameters: dto } => InitAccount::default().run(dto, storage),
            StepType::TransparentTransfer { parameters: dto } => {
                TxTransparentTransfer::default().run(dto, storage)
            }
            StepType::CheckBalance { parameters: dto } => BalanceCheck::default().run(dto, storage),
            StepType::CheckTxOutput { parameters: dto } => TxCheck::default().run(dto, storage),
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
