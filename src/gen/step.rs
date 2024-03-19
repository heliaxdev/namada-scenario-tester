use dyn_clone::DynClone;
use namada_scenario_tester::scenario::StepType;

use crate::{
    entity::Alias,
    state::State,
    steps::{
        bonds::BondBuilder, faucet_transfer::FaucetTransferBuilder,
        init_account::InitAccountBuilder, new_wallet_key::NewWalletStepBuilder,
        transparent_transfer::TransparentTransferBuilder,
    },
    utils,
};

use std::fmt::{Debug, Display};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum TaskType {
    NewWalletKey,
    FaucetTransafer,
    TransparentTransfer,
    Bond,
    InitAccount,
}

impl TaskType {
    pub fn is_valid(&self, state: &State) -> bool {
        match self {
            TaskType::NewWalletKey => true,
            TaskType::FaucetTransafer => !state.any_address().is_empty(),
            TaskType::TransparentTransfer => {
                !state.addresses_with_any_token_balance().is_empty()
                    && state.any_address().len() > 1
            }
            TaskType::Bond => !state.addresses_with_native_token_balance().is_empty(),
            TaskType::InitAccount => !state.addresses_with_native_token_balance().is_empty(), // we need to pay for fees
        }
    }

    pub fn build(&self, state: &State) -> Box<dyn Step> {
        match self {
            TaskType::NewWalletKey => {
                let alias = utils::random_alias();
                let step = NewWalletStepBuilder::default()
                    .alias(alias.into())
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::FaucetTransafer => {
                let target = state.random_account(vec![]);

                let amount = utils::random_between(1, 1000);
                let step = FaucetTransferBuilder::default()
                    .target(target.alias)
                    .token(Alias::native_token())
                    .amount(amount)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::TransparentTransfer => {
                let source = state.random_account_with_balance();
                let target = state.random_account(vec![source.clone()]);
                let token_balance = state.random_token_balance_for_alias(&source.alias);

                let amount = utils::random_between(0, token_balance.balance);
                let step = TransparentTransferBuilder::default()
                    .source(source.alias)
                    .target(target.alias)
                    .token(token_balance.token)
                    .amount(amount)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::Bond => {
                let source = state.random_account_with_native_token_balance();
                let token_balance = state.random_native_token_balance_for_alias(&source.alias);

                let amount = utils::random_between(0, token_balance.balance);
                let step = BondBuilder::default()
                    .source(source.alias)
                    .amount(amount)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::InitAccount => {
                let alias = utils::random_alias();
                let source = state.random_account_with_native_token_balance(); // pay the fees
                let maybe_treshold = utils::random_between(1, 10);
                let mut accounts = state.random_accounts(maybe_treshold - 1, vec![source.clone()]);

                accounts.push(source);
                accounts.reverse(); // source should be the fee payer and so must be the first one in the array

                let pks = accounts
                    .into_iter()
                    .map(|account| account.alias)
                    .collect::<Vec<Alias>>();
                let threshold = if pks.len() == 1 {
                    1
                } else {
                    utils::random_between(1, pks.len() as u64)
                };

                let step = InitAccountBuilder::default()
                    .alias(alias.into())
                    .pks(pks)
                    .threshold(threshold)
                    .build()
                    .unwrap();

                Box::new(step)
            }
        }
    }
}

pub trait Step: DynClone + Debug + Display {
    fn to_json(&self) -> StepType;
    fn update_state(&self, state: &mut State);
    fn post_hooks(&self, step_index: u64, state: &State) -> Vec<Box<dyn Hook>>;
    fn pre_hooks(&self, state: &State) -> Vec<Box<dyn Hook>>;
    fn total_post_hooks(&self) -> u64 {
        self.post_hooks(0, &State::default()).len() as u64
    }
    fn total_pre_hooks(&self) -> u64 {
        self.pre_hooks(&State::default()).len() as u64
    }
}

dyn_clone::clone_trait_object!(Step);

pub trait Hook: DynClone + Debug + Display {
    fn to_json(&self) -> StepType;
}

dyn_clone::clone_trait_object!(Hook);
