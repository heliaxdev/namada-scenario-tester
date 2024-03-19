use std::collections::BTreeSet;

use crate::{
    state::State,
    step::{Step, TaskType},
};
use weighted_rand::{builder::*, table::WalkerTable};

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
        step.update_state(&mut self.state)
    }

    pub fn update_scenario(&mut self, step: Box<dyn Step>) {
        self.steps.push(step);
        self.state.last_proposal_id += 1;
    }

    pub fn generate_scenario(&self, _step: Box<dyn Step>) {
        let mut steps = BTreeSet::new();
        for (index, step) in self.steps.iter().enumerate() {
            let step_pre_hooks = step.pre_hooks(index as u64);
            let step_post_hooks = step.post_hooks(index as u64);
            let pre_hooks_json = step_pre_hooks
                .into_iter()
                .map(|step| step.to_json())
                .collect::<Vec<String>>();
            let post_hooks_json = step_post_hooks
                .into_iter()
                .map(|step| step.to_json())
                .collect::<Vec<String>>();
            let step_json = step.to_json();
            steps.extend(pre_hooks_json);
            steps.insert(step_json);
            steps.extend(post_hooks_json);
        }
    }
}
