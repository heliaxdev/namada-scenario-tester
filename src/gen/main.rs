use std::collections::HashMap;

use clap::Parser;
use itertools::Itertools;

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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    steps: u64,
    #[arg(long, default_value_t = 1)]
    total: u64,
    #[arg(long, default_value_t = 5)]
    transparent_transfers: u64,
    #[arg(long, default_value_t = 3)]
    shielding_transfer: u64,
    #[arg(long, default_value_t = 3)]
    unshielding_transfer: u64,
    #[arg(long, default_value_t = 3)]
    init_account: u64,
    #[arg(long, default_value_t = 4)]
    bond: u64,
    #[arg(long, default_value_t = 4)]
    unbond: u64,
    #[arg(long, default_value_t = 0)]
    withdraw: u64,
    #[arg(long, default_value_t = 10)]
    vote_proposal: u64,
    #[arg(long, default_value_t = 4)]
    redelegate: u64,
    #[arg(long, default_value_t = 15)]
    init_default_proposal: u64,
    #[arg(long, default_value_t = 15)]
    init_pgf_steward_proposal: u64,
    #[arg(long, default_value_t = 15)]
    init_pgf_funding_proposal: u64,
    #[arg(long, default_value_t = 2)]
    become_validator: u64,
    #[arg(long, default_value_t = 3)]
    update_account: u64,
    #[arg(long, default_value_t = 1)]
    deactivate_validator: u64,
    #[arg(long, default_value_t = 2)]
    change_metadata: u64,
    #[arg(long, default_value_t = 4)]
    claim_rewards: u64,
}

fn main() {
    let args = Args::parse();

    // TODO:
    // change commission
    // activate validator

    // TODO:
    // randomize tx settings

    let tasks: HashMap<TaskType, Weight> = HashMap::from_iter([
        (TaskType::NewWalletKey, 2.into()),
        (TaskType::FaucetTransafer, 2.into()),
        (
            TaskType::TransparentTransfer,
            args.transparent_transfers.into(),
        ),
        (TaskType::ShieldingTransfer, args.shielding_transfer.into()),
        (
            TaskType::UnshieldingTransfer,
            args.unshielding_transfer.into(),
        ),
        (TaskType::InitAccount, args.init_account.into()),
        (TaskType::Bond, args.bond.into()),
        (
            TaskType::InitDefaultProposal,
            args.init_default_proposal.into(),
        ),
        (TaskType::Unbond, args.unbond.into()),
        (TaskType::Withdraw, args.withdraw.into()),
        (TaskType::VoteProposal, args.vote_proposal.into()),
        (TaskType::Redelegate, args.redelegate.into()),
        (
            TaskType::InitPgfStewardProposal,
            args.init_pgf_steward_proposal.into(),
        ),
        (
            TaskType::InitPgfFundingProposal,
            args.init_pgf_funding_proposal.into(),
        ),
        (TaskType::BecomeValidator, args.become_validator.into()),
        (TaskType::UpdateAccount, args.update_account.into()),
        (
            TaskType::DeactivateValidator,
            args.deactivate_validator.into(),
        ),
        (TaskType::ChangeMetadata, args.change_metadata.into()),
        (TaskType::ClaimRewards, args.claim_rewards.into()),
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
