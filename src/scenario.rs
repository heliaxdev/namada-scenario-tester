use std::{collections::HashMap, fmt::Display};

use serde::Deserialize;

use crate::{
    checks::balance::BalanceParametersDto,
    tasks::{
        TaskParam,
        init_account::{InitAccountParametersDto, InitAccountParameters, InitAccount},
        tx_transfer::TxTransferParametersDto,
        wallet_new_key::{WalletNewKeyParameters, WalletNewKeyParametersDto, WalletNewKey}, Task,
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
    TransparentTransfer { parameters: TxTransferParametersDto },
    #[serde(rename = "check-balance")]
    CheckBalance { parameters: BalanceParametersDto },
}

impl Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepType::WalletNewKey { .. } => write!(f, "wallet-new-key"),
            StepType::InitAccount { .. } => write!(f, "tx-init-account"),
            StepType::TransparentTransfer { .. } => write!(f, "tx-transparent-transfer"),
            StepType::CheckBalance { .. } => write!(f, "check-balance"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Step {
    pub id: u64,
    pub config: StepType,
}

impl Step {
    pub fn run(&self, data: &HashMap<u64, StepData>) -> StepResult {
        match self.config.to_owned() {
            StepType::WalletNewKey { parameters: dto } => {
                WalletNewKey::run(dto, data)
            }
            StepType::InitAccount { parameters: dto } => {
                InitAccount::run(dto, data)
            }
            StepType::TransparentTransfer { parameters: _ } => {
                let data = StepData {
                    data: HashMap::default(),
                };
                let outcome = StepOutcome { success: true };
                StepResult { data, outcome }
            }
            StepType::CheckBalance { parameters: _ } => {
                let data = StepData {
                    data: HashMap::default(),
                };
                let outcome = StepOutcome { success: true };
                StepResult { data, outcome }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct StepResult {
    pub outcome: StepOutcome,
    pub data: StepData,
}

impl StepResult {
    pub fn is_succesful(&self) -> bool {
        self.outcome.success
    }
}

#[derive(Clone, Debug)]
pub struct StepOutcome {
    pub success: bool,
}

impl StepOutcome {
    pub fn successful() -> Self {
        Self { success: true }
    }

    pub fn fail() -> Self {
        Self { success: false }
    }
}

#[derive(Clone, Debug)]
pub struct StepData {
    pub data: HashMap<String, String>,
}

impl StepData {
    pub fn add(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get_field(&self, field: &str) -> String {
        self.data
            .get(field)
            .expect("Field should be present in data.")
            .to_owned()
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Scenario {
    pub steps: Vec<Step>,
}
