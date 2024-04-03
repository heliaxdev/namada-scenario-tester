use std::fmt::Display;

use derive_builder::Builder;
use namada_scenario_tester::{
    scenario::StepType, tasks::reveal_pk::RevealPkParametersDto, utils::value::Value,
};

use crate::{entity::{Alias, TxSettings}, step::Hook};

#[derive(Clone, Debug, PartialEq, Eq, Builder)]
pub struct RevealPk {
    pub alias: Alias,
    pub tx_settings: TxSettings
}

impl RevealPk {
    pub fn new(alias: Alias) -> Self {
        Self { alias: alias.clone(), tx_settings: TxSettings::default_from_implicit(alias) }
    }
}

impl Hook for RevealPk {
    fn to_step_type(&self) -> StepType {
        StepType::RevealPk {
            parameters: RevealPkParametersDto {
                source: Value::v(self.alias.to_string()),
            },
            settings: Some(self.tx_settings.clone().into()),
        }
    }
}

impl Display for RevealPk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "reveal pk for alias {}", self.alias,)
    }
}
