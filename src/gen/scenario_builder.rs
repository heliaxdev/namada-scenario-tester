use std::fs;

use namada_scenario_tester::scenario::{
    Scenario, ScenarioSettings, Step as ScenarioStep, StepType,
};
use weighted_rand::{builder::*, table::WalkerTable};

use crate::{
    state::State,
    step::{Step, TaskType},
    utils,
};

#[derive(Clone)]
pub struct Weight {
    pub inner: u32,
}

impl From<u64> for Weight {
    fn from(value: u64) -> Self {
        Self {
            inner: value as u32,
        }
    }
}

impl From<Weight> for u32 {
    fn from(value: Weight) -> Self {
        value.inner
    }
}

pub struct ScenarioBuilder {
    pub state: State,
    pub tasks_types: Vec<TaskType>,
    pub steps: Vec<Box<dyn Step>>,
    pub scenario: Vec<StepType>,
    inner: WalkerTable,
}

impl ScenarioBuilder {
    pub fn new(tasks: Vec<TaskType>, weights: Vec<Weight>) -> Self {
        let weights_indexes = weights
            .into_iter()
            .map(|weight| weight.into())
            .collect::<Vec<u32>>();
        let builder = WalkerTableBuilder::new(&weights_indexes);
        let table = builder.build();

        Self {
            state: State::default(),
            tasks_types: tasks,
            steps: Vec::default(),
            scenario: Vec::default(),
            inner: table,
        }
    }

    pub fn choose_next_task(&self) -> TaskType {
        self.tasks_types[self.inner.next()]
    }

    pub fn is_valid_task(&self, task_type: TaskType) -> bool {
        task_type.is_valid(&self.state)
    }

    pub fn build_step(&self, task_type: TaskType) -> Box<dyn Step> {
        task_type.build(&self.state)
    }

    pub fn update_state(&mut self, step: Box<dyn Step>) {
        self.state.last_step_id += step.total_pre_hooks();
        step.update_state(&mut self.state);
        self.state.last_step_id += 1 + step.total_post_hooks();
    }

    pub fn update_scenario(&mut self, step: Box<dyn Step>) {
        self.steps.push(step.clone());

        let current_scenario_index = self.scenario.len() as u64;
        let step_index = current_scenario_index + step.total_pre_hooks();

        let step_pre_hooks = step.pre_hooks(&self.state);
        let step_post_hooks = step.post_hooks(step_index, &self.state);

        let pre_hooks_json = step_pre_hooks
            .into_iter()
            .map(|step| step.to_step_type())
            .collect::<Vec<StepType>>();
        let post_hooks_json = step_post_hooks
            .into_iter()
            .map(|step| step.to_step_type())
            .collect::<Vec<StepType>>();
        let step_json = step.to_step_type(step_index);

        self.scenario.extend(pre_hooks_json);
        self.scenario.push(step_json);
        self.scenario.extend(post_hooks_json);
    }

    pub fn to_file(&self) {
        let scenario = Scenario {
            settings: ScenarioSettings { retry_for: None },
            steps: self
                .scenario
                .clone()
                .into_iter()
                .enumerate()
                .map(|(index, step_type)| ScenarioStep {
                    id: index as u64,
                    config: step_type,
                })
                .collect(),
        };
        let scenario_json = serde_json::to_string(&scenario).unwrap();
        let scenario_name = utils::random_with_namespace(scenario.steps.len().to_string().as_str());
        fs::write(format!("scenarios/{}.json", scenario_name), scenario_json)
            .expect("Unable to write file");
        println!("Scenario {} generate and saved to file.", scenario_name)
    }
}
