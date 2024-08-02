use std::collections::HashMap;

use clap::Parser;
use itertools::Itertools;

use namada_scenario_tester::tasks::Task;
use scenario_builder::ScenarioBuilder;

use step::TaskType;

use crate::scenario_builder::Weight;

pub mod constants;
pub mod entity;
pub mod hooks;
pub mod scenario_builder;
pub mod state;
pub mod step;
pub mod steps;
pub mod utils;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    steps: u64,
    #[arg(short, long, default_value_t = 1)]
    total: u64,
}

fn main() {
    let args = Args::parse();

    // TODO:
    // change commission
    // activate validator
    // update steward commission

    // TODO:
    // randomize tx settings

    let tasks: HashMap<TaskType, Weight> = HashMap::from_iter([
        (TaskType::NewWalletKey, 1.into()),
        (TaskType::FaucetTransafer, 2.into()),
        (TaskType::TransparentTransfer, 3.into()),
        (TaskType::InitAccount, 4.into()),
        (TaskType::Bond, 4.into()),
        (TaskType::InitDefaultProposal, 3.into()),
        (TaskType::Unbond, 4.into()),
        // (TaskType::Withdraw, 4.into()),
        (TaskType::VoteProposal, 3.into()),
        (TaskType::Redelegate, 5.into()),
        (TaskType::InitPgfStewardProposal, 3.into()),
        (TaskType::InitPgfFundingProposal, 3.into()),
        (TaskType::BecomeValidator, 5.into()),
        (TaskType::UpdateAccount, 3.into()),
        (TaskType::DeactivateValidator, 2.into()),
        (TaskType::ChangeConsensusKey, 5.into()),
        (TaskType::ChangeMetadata, 4.into()),
        (TaskType::ClaimRewards, 5.into())
    ]);

    let mut scenario_builder = ScenarioBuilder::new(
        tasks.keys().cloned().collect_vec(),
        tasks.values().cloned().collect_vec(),
    );

    for _ in 0..=args.steps {
        let next_task = loop {
            let task_type = scenario_builder.choose_next_task();
            if scenario_builder.is_valid_task(task_type) {
                break task_type;
            }
        };
        let step = scenario_builder.build_step(next_task);

        scenario_builder.update_state(step.clone());
        scenario_builder.update_scenario(step.clone());
    }

    scenario_builder.to_file()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_basic() {
        for _ in 0..10000 {
            let tasks: HashMap<TaskType, Weight> = HashMap::from_iter([
                (TaskType::NewWalletKey, 1.into()),
                (TaskType::FaucetTransafer, 2.into()),
                (TaskType::TransparentTransfer, 3.into()),
            ]);

            let mut scenario_builder = ScenarioBuilder::new(
                tasks.keys().cloned().collect_vec(),
                tasks.values().cloned().collect_vec(),
            );

            for _ in 0..=200 {
                let next_task = loop {
                    let task_type = scenario_builder.choose_next_task();
                    if scenario_builder.is_valid_task(task_type) {
                        break task_type;
                    }
                };
                let step = scenario_builder.build_step(next_task);

                scenario_builder.update_state(step.clone());
                scenario_builder.update_scenario(step.clone());
            }
        }
    }

    #[test]
    pub fn test_basic_plus_pos() {
        for _ in 0..10000 {
            let tasks: HashMap<TaskType, Weight> = HashMap::from_iter([
                (TaskType::NewWalletKey, 1.into()),
                (TaskType::FaucetTransafer, 2.into()),
                (TaskType::TransparentTransfer, 3.into()),
                (TaskType::Bond, 4.into()),
                (TaskType::Unbond, 4.into()),
                (TaskType::Withdraw, 8.into()),
                (TaskType::Redelegate, 4.into()),
            ]);

            let mut scenario_builder = ScenarioBuilder::new(
                tasks.keys().cloned().collect_vec(),
                tasks.values().cloned().collect_vec(),
            );

            for _ in 0..=200 {
                let next_task = loop {
                    let task_type = scenario_builder.choose_next_task();
                    if scenario_builder.is_valid_task(task_type) {
                        break task_type;
                    }
                };
                let step = scenario_builder.build_step(next_task);

                scenario_builder.update_state(step.clone());
                scenario_builder.update_scenario(step.clone());
            }
        }
    }

    #[test]
    pub fn test_basic_plus_pos_plus_goverance() {
        for _ in 0..10000 {
            let tasks: HashMap<TaskType, Weight> = HashMap::from_iter([
                (TaskType::NewWalletKey, 1.into()),
                (TaskType::FaucetTransafer, 2.into()),
                (TaskType::TransparentTransfer, 3.into()),
                (TaskType::Bond, 4.into()),
                (TaskType::Unbond, 4.into()),
                (TaskType::Withdraw, 8.into()),
                (TaskType::Redelegate, 4.into()),
                (TaskType::InitDefaultProposal, 8.into()),
                (TaskType::VoteProposal, 8.into()),
                (TaskType::InitPgfStewardProposal, 12.into()),
                (TaskType::InitPgfFundingProposal, 12.into()),
            ]);

            let mut scenario_builder = ScenarioBuilder::new(
                tasks.keys().cloned().collect_vec(),
                tasks.values().cloned().collect_vec(),
            );

            for _ in 0..=200 {
                let next_task = loop {
                    let task_type = scenario_builder.choose_next_task();
                    if scenario_builder.is_valid_task(task_type) {
                        break task_type;
                    }
                };
                let step = scenario_builder.build_step(next_task);

                scenario_builder.update_state(step.clone());
                scenario_builder.update_scenario(step.clone());
            }
        }
    }

    #[test]
    pub fn test_basic_plus_pos_plus_goverance_plus_account() {
        for _ in 0..10000 {
            let tasks: HashMap<TaskType, Weight> = HashMap::from_iter([
                (TaskType::NewWalletKey, 1.into()),
                (TaskType::FaucetTransafer, 2.into()),
                (TaskType::TransparentTransfer, 3.into()),
                (TaskType::Bond, 4.into()),
                (TaskType::Unbond, 4.into()),
                (TaskType::Withdraw, 8.into()),
                (TaskType::Redelegate, 4.into()),
                (TaskType::InitDefaultProposal, 8.into()),
                (TaskType::VoteProposal, 8.into()),
                (TaskType::InitPgfStewardProposal, 12.into()),
                (TaskType::InitPgfFundingProposal, 12.into()),
                (TaskType::InitAccount, 6.into()),
                (TaskType::UpdateAccount, 6.into()),
            ]);

            let mut scenario_builder = ScenarioBuilder::new(
                tasks.keys().cloned().collect_vec(),
                tasks.values().cloned().collect_vec(),
            );

            for _ in 0..=200 {
                let next_task = loop {
                    let task_type = scenario_builder.choose_next_task();
                    if scenario_builder.is_valid_task(task_type) {
                        break task_type;
                    }
                };
                let step = scenario_builder.build_step(next_task);

                scenario_builder.update_state(step.clone());
                scenario_builder.update_scenario(step.clone());
            }
        }
    }
}
