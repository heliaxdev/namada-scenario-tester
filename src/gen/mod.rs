use std::collections::HashMap;

use namada_sdk::{token, uint::Uint};
use proptest::{
    prop_oneof,
    sample::SizeRange,
    strategy::{Just, Strategy, ValueTree},
    test_runner::TestRunner,
};
use proptest_state_machine::ReferenceStateMachine;

use crate::{
    scenario::{Step, StepId, StepType},
    tasks::{
        bond::TxBondParametersDto, tx_transparent_transfer::TxTransparentTransferParametersDto,
        wallet_new_key::WalletNewKeyStorageKeys,
    },
    utils::value::Value,
};

pub fn gen_steps(size: impl Into<SizeRange>) -> Vec<Step> {
    let strategy = StepGen::sequential_strategy(size);
    let mut runner = TestRunner::default();
    let (init_state, steps) = strategy.new_tree(&mut runner).unwrap().current();

    [init_state.init_steps, steps].concat()
}

/// Minimum balance required to send txs set to high enough threshold to cover gas cost.
const MIN_BALANCE_FOR_TX: Uint = Uint::from_u64(token::NATIVE_SCALE); //  1 whole token

#[derive(Clone, Debug, Default)]
struct StepGen {
    last_step_id: Option<StepId>,
    init_steps: Vec<Step>,
    validator_queries: Vec<StepId>,
    keys: Vec<StepId>,
    /// Only non-0 balances
    native_balance: HashMap<StepId, Uint>,
    bonds: HashMap<StepId, TxBondParametersDto>,
}

impl ReferenceStateMachine for StepGen {
    type State = Self;

    type Transition = Step;

    fn init_state() -> proptest::prelude::BoxedStrategy<Self::State> {
        let validator_query = Step {
            id: 0,
            config: StepType::QueryValidators {
                parameters: Default::default(),
            },
        };

        Just(StepGen {
            last_step_id: Some(0),
            validator_queries: vec![0],
            init_steps: vec![validator_query],
            ..StepGen::default()
        })
        .boxed()
    }

    fn transitions(state: &Self::State) -> proptest::prelude::BoxedStrategy<Self::Transition> {
        // We always have 1 validator_query on init
        let rand_validator = proptest::sample::select(state.validator_queries.clone());

        let step_type = prop_oneof![
            1 => Just(StepType::WalletNewKey { parameters: Default::default() }).boxed(),
        ];

        let step_type = if state.keys.is_empty() {
            step_type
        } else {
            let rand_key = proptest::sample::select(state.keys.clone());
            prop_oneof![
                1 => step_type,

                // Faucet withdrawal
                1 => (rand_key, arb_native_token_amount()).prop_map(|(key, amount)| {
                    StepType::TransparentTransfer {
                        parameters: TxTransparentTransferParametersDto {
                            source: Value::Value { value: "faucet".to_string() },
                            target: Value::Ref {value: key, field: WalletNewKeyStorageKeys::Alias.to_string()},
                            amount: Value::Value{value: amount.to_string()},
                            token: Value::Value { value: "nam".to_string() },
                        }
                    }}).boxed(),

            ].boxed()
        };

        let native_balances: Vec<_> = state
            .native_balance
            .iter()
            .filter_map(|(source, bal)| {
                // Only count balances that are greater than a threshold
                if *bal >= MIN_BALANCE_FOR_TX {
                    Some((*source, *bal))
                } else {
                    None
                }
            })
            .collect();

        let step_type = if native_balances.is_empty() {
            step_type.boxed()
        } else {
            let rand_source_with_bal = proptest::sample::select(native_balances);

            prop_oneof![
                    1 => step_type,

                    // PoS bond
                    1 => (rand_validator, rand_source_with_bal).prop_flat_map(|(validator, (source, bal))| {
                        dbg!(source, bal);
                        (1..=bal.as_u64()).prop_map(move |amount|  {
                            dbg!(&amount);
                            StepType::Bond {
                                parameters: TxBondParametersDto {
                                    source: Value::Ref {value: source, field: WalletNewKeyStorageKeys::Alias.to_string()},
                                    validator: Value::Fuzz {value: Some(validator)},
                                    amount: Value::Value{value: amount.to_string()},
                                }
                        }})
                    }).boxed(),
                ].boxed()
        };

        let id = state.last_step_id.map(|id| id + 1).unwrap_or_default();
        step_type
            .prop_map(move |config| Step { id, config })
            .boxed()
    }

    fn apply(mut state: Self::State, transition: &Self::Transition) -> Self::State {
        let id = transition.id;
        match &transition.config {
            StepType::WalletNewKey { parameters } => state.keys.push(id),
            StepType::InitAccount { parameters } => todo!(),
            StepType::TransparentTransfer { parameters } => match parameters {
                TxTransparentTransferParametersDto {
                    source: Value::Value { value: source },
                    target: Value::Ref { value: target, .. },
                    amount: Value::Value { value: amount },
                    token: Value::Value { value: token },
                } => {
                    if source == "faucet" && token == "nam" {
                        let current_bal = state.native_balance.entry(*target).or_default();
                        *current_bal += Uint::from_dec_str(amount).unwrap();
                    }
                }
                _ => {}
            },
            StepType::RevealPk { parameters } => todo!(),
            StepType::Bond { parameters } => {
                state.bonds.insert(id, parameters.clone());
                match parameters {
                    TxBondParametersDto {
                        source: Value::Ref { value: source, .. },
                        validator: _,
                        amount: Value::Value { value: amount },
                    } => {
                        let current_bal = state.native_balance.entry(*source).or_default();
                        *current_bal -= Uint::from_dec_str(&amount).unwrap();
                        if current_bal.is_zero() {
                            state.native_balance.remove(source);
                        }
                    }
                    _ => {}
                }
            }
            StepType::CheckBalance { parameters } => todo!(),
            StepType::CheckStepOutput { parameters } => todo!(),
            StepType::WaitUntillEpoch { parameters } => todo!(),
            StepType::WaitUntillHeight { parameters } => todo!(),
            StepType::QueryAccountTokenBalance { parameters } => todo!(),
            StepType::QueryAccount { parameters } => todo!(),
            StepType::QueryBondedStake { parameters } => todo!(),
            StepType::Redelegate { parameters } => todo!(),
            StepType::CheckBonds { parameters } => todo!(),
            StepType::InitProposal { parameters } => todo!(),
            StepType::QueryProposal { parameters } => todo!(),
            StepType::VoteProposal { parameters } => todo!(),
            StepType::CheckStorage { parameters } => todo!(),
            StepType::QueryValidators { parameters } => state.validator_queries.push(id),
        };
        state.last_step_id = Some(id);
        state
    }
}

fn arb_native_token_amount() -> impl Strategy<Value = Uint> {
    (1_u64..1_000_000_000).prop_map(Uint::from_u64)
}

#[cfg(test)]
mod test {
    use crate::scenario::Scenario;

    use super::*;

    #[test]
    fn test_gen_steps() {
        let steps = gen_steps(10);
        let scenario = Scenario { steps };
        let scenario_str = serde_json::to_string_pretty(&scenario).unwrap();
        println!("{scenario_str}");

        if let Ok(save_to_file) = std::env::var("SAVE_TO_FILE") {
            std::fs::write(&save_to_file, &scenario_str).unwrap();
        }
    }
}
