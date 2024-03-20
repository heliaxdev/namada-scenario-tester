use scenario_builder::ScenarioBuilder;

use step::TaskType;

pub mod constants;
pub mod entity;
pub mod hooks;
pub mod scenario_builder;
pub mod state;
pub mod step;
pub mod steps;
pub mod utils;

fn main() {
    let tasks = vec![
        TaskType::NewWalletKey,
        TaskType::FaucetTransafer,
        TaskType::TransparentTransfer,
        TaskType::Bond,
        TaskType::InitAccount,
        TaskType::InitDefaultProposal,
        TaskType::Unbond,
        TaskType::Withdraw,
    ];

    let mut scenario_builder = ScenarioBuilder::new(
        tasks,
        vec![
            1.into(),
            2.into(),
            1.into(),
            2.into(),
            1.into(),
            1.into(),
            3.into(),
            3.into(),
        ],
    );

    for _step_index in 0..=20 {
        let next_task = loop {
            let task_type = scenario_builder.choose_next_task();
            if scenario_builder.is_valid_task(task_type) {
                break task_type;
            }
        };
        println!("next: {:?}", next_task);
        let step = scenario_builder.build_step(next_task);

        scenario_builder.update_state(step.clone());
        scenario_builder.update_scenario(step.clone());
    }

    for (index, step) in scenario_builder.scenario.iter().enumerate() {
        println!("{}: {:?}", index, step);
    }
}
