

use crate::{
    state::State,
    step::{Step, TaskType},
};
use namada_scenario_tester::scenario::StepType;
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

    pub fn generate_scenario(&self) -> Vec<StepType> {
        let mut steps = Vec::new();
        for (index, step) in self.steps.iter().enumerate() {
            let step_pre_hooks = step.pre_hooks(index as u64, &self.state);
            let step_post_hooks = step.post_hooks(index as u64, &self.state);
            let pre_hooks_json = step_pre_hooks
                .into_iter()
                .map(|step| step.to_json())
                .collect::<Vec<StepType>>();
            let post_hooks_json = step_post_hooks
                .into_iter()
                .map(|step| step.to_json())
                .collect::<Vec<StepType>>();
            let step_json = step.to_json();
            steps.extend(pre_hooks_json);
            steps.push(step_json);
            steps.extend(post_hooks_json);
        }

        steps
    }
}
