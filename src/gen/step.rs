use dyn_clone::DynClone;
use namada_scenario_tester::{scenario::StepType, tasks::Task};
use namada_sdk::token::NATIVE_SCALE;

use crate::{
    constants::{MAX_PGF_ACTIONS, MIN_FEE, PROPOSAL_FUNDS},
    entity::{Alias, SpendingKey, TxSettings},
    state::State,
    steps::{
        become_validator::BecomeValidatorBuilder, bonds::BondBuilder,
        change_consensus_key::ChangeConsensusKeyBuilder, change_metadata::ChangeMetadataBuilder,
        claim_rewards::ClaimRewardsBuilder, deactivate_validator::DeactivateValidatorBuilder,
        faucet_transfer::FaucetTransferBuilder, init_account::InitAccountBuilder,
        init_default_proposal::InitDefaultProposalBuilder,
        init_funding_proposal::InitPgfFundingProposalBuilder,
        init_steward_proposal::InitPgfStewardProposalBuilder, new_wallet_key::NewWalletStepBuilder,
        redelegate::RedelegateBuilder, shielding_transfer::ShieldingTransferBuilder,
        transparent_transfer::TransparentTransferBuilder, unbond::UnbondBuilder,
        unshielding_transfer::UnshieldingTransferBuilder, update_account::UpdateAccountBuilder,
        vote::VoteProposalBuilder, withdraw::WithdrawBuilder,
    },
    utils,
};

use std::{
    cmp::min,
    collections::BTreeSet,
    fmt::{Debug, Display},
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum TaskType {
    NewWalletKey,
    FaucetTransafer,
    TransparentTransfer,
    ShieldingTransfer,
    UnshieldingTransfer,
    Bond,
    InitAccount,
    InitDefaultProposal,
    InitPgfStewardProposal,
    InitPgfFundingProposal,
    Unbond,
    Withdraw,
    VoteProposal,
    Redelegate,
    BecomeValidator,
    ChangeMetadata,
    ChangeConsensusKey,
    UpdateAccount,
    DeactivateValidator,
    ClaimRewards,
}

impl TaskType {
    pub fn is_valid(&self, state: &State) -> bool {
        match self {
            TaskType::NewWalletKey => true,
            TaskType::FaucetTransafer => !state.any_address().is_empty(),
            TaskType::TransparentTransfer => {
                !state
                    .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                    .is_empty()
                    && !state
                        .addresses_with_at_least_native_token_balance(MIN_FEE * 2)
                        .is_empty()
                    && state.any_address().len() > 1
            }
            TaskType::ShieldingTransfer => {
                !state
                    .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                    .is_empty()
                    && !state
                        .addresses_with_at_least_native_token_balance(MIN_FEE * 2)
                        .is_empty()
            }
            TaskType::UnshieldingTransfer => {
                !state
                    .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                    .is_empty()
                    && !state
                        .payment_address_with_at_least_native_token_balance(1)
                        .is_empty()
            }
            TaskType::Bond => !state
                .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                .is_empty(),
            TaskType::InitAccount => !state
                .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                .is_empty(),
            TaskType::InitDefaultProposal => !state
                .implicit_addresses_with_at_least_native_token_balance(PROPOSAL_FUNDS + MIN_FEE)
                .is_empty(),
            TaskType::InitPgfStewardProposal => {
                !state
                    .implicit_addresses_with_at_least_native_token_balance(PROPOSAL_FUNDS + MIN_FEE)
                    .is_empty()
                    && state.any_address().len() > 1
            }
            TaskType::InitPgfFundingProposal => {
                !state
                    .implicit_addresses_with_at_least_native_token_balance(PROPOSAL_FUNDS + MIN_FEE)
                    .is_empty()
                    && state.any_address().len() > 1
            }
            TaskType::Unbond => {
                !state.any_bond().is_empty()
                    && !state
                        .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                        .is_empty()
            }
            TaskType::Withdraw => {
                !state.any_unbond().is_empty()
                    && !state
                        .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                        .is_empty()
            }
            TaskType::VoteProposal => {
                !state.any_bond().is_empty()
                    && state.last_proposal_id > 0
                    && !state
                        .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                        .is_empty()
            }
            TaskType::Redelegate => {
                !state.any_bond().is_empty()
                    && !state.any_active_validator_address().is_empty()
                    && !state
                        .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                        .is_empty()
            }
            TaskType::ClaimRewards => {
                !state.any_bond().is_empty()
                    && !state
                        .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                        .is_empty()
            }
            TaskType::BecomeValidator => {
                !state.any_virgin_enstablished_address().is_empty()
                    && !state
                        .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                        .is_empty()
            }
            TaskType::ChangeMetadata => {
                !state.any_validator_address().is_empty()
                    && !state
                        .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                        .is_empty()
            }
            TaskType::ChangeConsensusKey => {
                !state.any_validator_address().is_empty()
                    && !state
                        .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                        .is_empty()
            }
            TaskType::UpdateAccount => {
                !state.any_virgin_enstablished_address().is_empty()
                    && !state
                        .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                        .is_empty()
            }
            TaskType::DeactivateValidator => {
                !state.any_active_validator_address().is_empty()
                    && !state
                        .implicit_addresses_with_at_least_native_token_balance(MIN_FEE)
                        .is_empty()
            }
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

                let amount = utils::random_between(MIN_FEE * 2, 1000 * NATIVE_SCALE);
                let step = FaucetTransferBuilder::default()
                    .target(target.alias)
                    .token(Alias::native_token())
                    .amount(amount)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::ShieldingTransfer => {
                let source = state.random_account_with_at_least_native_token_balance(MIN_FEE * 2);
                let token_balance = state.random_token_balance_for_alias(&source.alias);
                let target = state.random_payment_address();

                let tx_settings = if source.clone().address_type.is_implicit() {
                    let gas_payer = source.alias.clone();
                    TxSettings::default_from_implicit(gas_payer)
                } else {
                    let gas_payer = state
                        .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                        .alias;
                    TxSettings::default_from_enstablished(
                        source.clone().implicit_addresses,
                        gas_payer,
                    )
                };

                let amount = if source.clone().address_type.is_implicit() {
                    utils::random_between(0, token_balance.balance - MIN_FEE)
                } else {
                    utils::random_between(0, token_balance.balance)
                };

                let step = ShieldingTransferBuilder::default()
                    .source(source.alias)
                    .target(target)
                    .token(token_balance.token)
                    .amount(amount)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::UnshieldingTransfer => {
                let source =
                    state.random_payment_address_with_at_least_native_token_balance(MIN_FEE * 2);
                let spending_key_source =
                    format!("{}-masp", source.to_string().strip_suffix("-pa").unwrap());
                let token_balance = state.random_token_balance_for_alias(&source.inner);
                let target = state.random_account(vec![]);

                let gas_payer = state
                    .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                    .alias;
                let tx_settings = TxSettings::default_from_enstablished(BTreeSet::new(), gas_payer);

                let amount = utils::random_between(0, token_balance.balance);

                let step = UnshieldingTransferBuilder::default()
                    .source(spending_key_source.into())
                    .target(target.alias)
                    .token(token_balance.token)
                    .amount(amount)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::TransparentTransfer => {
                let source = state.random_account_with_at_least_native_token_balance(MIN_FEE * 2);
                let target = state.random_account(vec![source.alias.clone()]);
                let token_balance = state.random_token_balance_for_alias(&source.alias);

                let tx_settings = if source.clone().address_type.is_implicit() {
                    let gas_payer = source.alias.clone();
                    TxSettings::default_from_implicit(gas_payer)
                } else {
                    let gas_payer = state
                        .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                        .alias;
                    TxSettings::default_from_enstablished(
                        source.clone().implicit_addresses,
                        gas_payer,
                    )
                };

                let amount = if source.clone().address_type.is_implicit() {
                    utils::random_between(0, token_balance.balance - MIN_FEE)
                } else {
                    utils::random_between(0, token_balance.balance)
                };

                let step = TransparentTransferBuilder::default()
                    .source(source.alias)
                    .target(target.alias)
                    .token(token_balance.token)
                    .amount(amount)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::Bond => {
                let source =
                    state.random_non_validator_address_with_at_least_native_token(MIN_FEE + 1);
                let token_balance = state.random_native_token_balance_for_alias(&source.alias);

                let tx_settings = if source.clone().address_type.is_implicit() {
                    let gas_payer = source.alias.clone();
                    TxSettings::default_from_implicit(gas_payer)
                } else {
                    let gas_payer = state
                        .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                        .alias;
                    TxSettings::default_from_enstablished(
                        source.clone().implicit_addresses,
                        gas_payer,
                    )
                };

                let amount = if source.clone().address_type.is_implicit() {
                    utils::random_between(0, token_balance.balance - MIN_FEE)
                } else {
                    utils::random_between(0, token_balance.balance)
                };

                let step = BondBuilder::default()
                    .source(source.alias)
                    .amount(amount)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::ClaimRewards => {
                let bond = state.random_bond();

                let gas_payer = state
                    .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                    .alias;

                let tx_settings = if bond.source.clone().is_implicit() {
                    TxSettings::default_from_implicit(gas_payer)
                        .ovverride_signers(vec![bond.source.clone()])
                } else {
                    let account = state.get_account_from_alias(&bond.source);
                    TxSettings::default_from_enstablished(account.implicit_addresses, gas_payer)
                };

                let step = ClaimRewardsBuilder::default()
                    .bond_step(bond.step_id)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::InitAccount => {
                let alias = utils::random_enstablished_alias();
                let source =
                    state.random_implicit_account_with_at_least_native_token_balance(MIN_FEE); // pay the fees
                let maybe_treshold = utils::random_between(1, 10);
                let mut accounts =
                    state.random_implicit_accounts(maybe_treshold - 1, vec![source.alias.clone()]);

                accounts.push(source.clone());

                let pks = accounts
                    .into_iter()
                    .map(|account| account.alias)
                    .collect::<BTreeSet<Alias>>();
                let threshold = if pks.len() == 1 {
                    1
                } else {
                    utils::random_between(1, pks.len() as u64)
                };

                let tx_settings =
                    TxSettings::default_from_enstablished(pks.clone(), source.alias.clone());

                let step = InitAccountBuilder::default()
                    .alias(alias.into())
                    .pks(pks)
                    .threshold(threshold)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::InitDefaultProposal => {
                let author = state.random_implicit_account_with_at_least_native_token_balance(
                    PROPOSAL_FUNDS + MIN_FEE,
                );

                let tx_settings = if author.clone().address_type.is_implicit() {
                    let gas_payer = author.alias.clone();
                    TxSettings::default_from_implicit(gas_payer)
                } else {
                    let gas_payer = state
                        .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                        .alias;
                    TxSettings::default_from_enstablished(author.implicit_addresses, gas_payer)
                };

                let step = InitDefaultProposalBuilder::default()
                    .author(author.alias)
                    .start_epoch(None)
                    .end_epoch(None)
                    .grace_epoch(None)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::InitPgfStewardProposal => {
                let author = state.random_implicit_account_with_at_least_native_token_balance(
                    PROPOSAL_FUNDS + MIN_FEE,
                );

                let total_accounts = state.any_address().len();
                let total_stewards_to_remove =
                    utils::random_between(1, min(total_accounts as u64, 14));
                let steward_addresses =
                    state.random_accounts(total_stewards_to_remove, vec![author.alias.clone()]);
                let steward_aliases = steward_addresses
                    .iter()
                    .map(|account| account.alias.clone())
                    .collect();

                let tx_settings = if author.clone().address_type.is_implicit() {
                    let gas_payer = author.alias.clone();
                    TxSettings::default_from_implicit(gas_payer)
                } else {
                    let gas_payer = state
                        .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                        .alias;
                    TxSettings::default_from_enstablished(author.implicit_addresses, gas_payer)
                };

                let step = InitPgfStewardProposalBuilder::default()
                    .author(author.alias)
                    .start_epoch(None)
                    .end_epoch(None)
                    .grace_epoch(None)
                    .tx_settings(tx_settings)
                    .steward_remove(steward_aliases)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::InitPgfFundingProposal => {
                let author = state.random_implicit_account_with_at_least_native_token_balance(
                    PROPOSAL_FUNDS + MIN_FEE,
                );

                let total_accounts = state.any_address().len();
                let total_retro =
                    utils::random_between(0, min(total_accounts as u64, MAX_PGF_ACTIONS));
                let minimum_total_continous = if total_retro > 0 { 0 } else { 1 };
                let maximum_total_continous = MAX_PGF_ACTIONS - total_retro;
                let total_continous = utils::random_between(
                    minimum_total_continous,
                    min(total_accounts as u64, maximum_total_continous),
                );

                let tx_settings = if author.clone().address_type.is_implicit() {
                    let gas_payer = author.alias.clone();
                    TxSettings::default_from_implicit(gas_payer)
                } else {
                    let gas_payer = state
                        .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                        .alias;
                    TxSettings::default_from_enstablished(author.implicit_addresses, gas_payer)
                };

                let retro_addresses =
                    state.random_accounts(total_retro, vec![tx_settings.gas_payer.clone()]);
                let continous_addresses =
                    state.random_accounts(total_continous, vec![tx_settings.gas_payer.clone()]);

                let retro_aliases = retro_addresses
                    .iter()
                    .map(|account| account.alias.clone())
                    .collect();
                let continous_aliases = continous_addresses
                    .iter()
                    .map(|account| account.alias.clone())
                    .collect();

                let retro_amounts = (0..total_retro)
                    .map(|_| utils::random_between(0, 100000))
                    .collect();
                let continous_amounts = (0..total_continous)
                    .map(|_| utils::random_between(0, 100000))
                    .collect();

                let step = InitPgfFundingProposalBuilder::default()
                    .author(author.alias)
                    .start_epoch(None)
                    .end_epoch(None)
                    .grace_epoch(None)
                    .retro_funding_target(retro_aliases)
                    .retro_funding_amount(retro_amounts)
                    .continous_funding_target(continous_aliases)
                    .continous_funding_amount(continous_amounts)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::Unbond => {
                let bond = state.random_bond();
                let amount = utils::random_between(0, bond.amount);

                let gas_payer = state
                    .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                    .alias;

                let tx_settings = if bond.source.clone().is_implicit() {
                    TxSettings::default_from_implicit(gas_payer)
                        .ovverride_signers(vec![bond.source.clone()])
                } else {
                    let account = state.get_account_from_alias(&bond.source);
                    TxSettings::default_from_enstablished(account.implicit_addresses, gas_payer)
                };

                let step = UnbondBuilder::default()
                    .amount(amount)
                    .source(bond.source)
                    .bond_step(bond.step_id)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::Withdraw => {
                let unbond = state.random_unbond();
                let amount = utils::random_between(0, unbond.amount);

                let gas_payer = state
                    .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                    .alias;

                let tx_settings = if unbond.source.clone().is_implicit() {
                    TxSettings::default_from_implicit(gas_payer)
                        .ovverride_signers(vec![unbond.source.clone()])
                } else {
                    let account = state.get_account_from_alias(&unbond.source);
                    TxSettings::default_from_enstablished(account.implicit_addresses, gas_payer)
                };

                let step = WithdrawBuilder::default()
                    .amount(amount)
                    .source(unbond.source)
                    .unbond_step(unbond.step_id)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::VoteProposal => {
                let bond = state.random_bond();
                let bond_source_balance =
                    state.get_alias_token_balance(&bond.source, &Alias::native_token());

                let tx_settings =
                    if bond.source.clone().is_implicit() && bond_source_balance > MIN_FEE {
                        let gas_payer = bond.source.clone();
                        TxSettings::default_from_implicit(gas_payer)
                    } else {
                        let gas_payer = state
                            .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                            .alias;
                        let account = state.get_account_from_alias(&bond.source);
                        TxSettings::default_from_enstablished(account.implicit_addresses, gas_payer)
                    };

                let step = VoteProposalBuilder::default()
                    .voter(bond.source)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::Redelegate => {
                let bond = state.random_bond();
                let source_bond_balance =
                    state.get_alias_token_balance(&bond.source, &Alias::native_token());

                let tx_settings =
                    if bond.source.clone().is_implicit() && source_bond_balance > MIN_FEE {
                        let gas_payer = bond.source.clone();
                        TxSettings::default_from_implicit(gas_payer)
                    } else {
                        let gas_payer = state
                            .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                            .alias;
                        let account = state.get_account_from_alias(&bond.source);
                        TxSettings::default_from_enstablished(account.implicit_addresses, gas_payer)
                    };

                let amount = utils::random_between(1, bond.amount);
                let step = RedelegateBuilder::default()
                    .amount(amount)
                    .source(bond.source)
                    .tx_settings(tx_settings)
                    .source_validator(bond.step_id)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::BecomeValidator => {
                let non_validator_account = state.random_virgin_address();

                let gas_payer = state
                    .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                    .alias;
                let tx_settings = TxSettings::default_from_enstablished(
                    non_validator_account.implicit_addresses,
                    gas_payer,
                );

                let step = BecomeValidatorBuilder::default()
                    .source(non_validator_account.alias)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::ChangeMetadata => {
                let validator_account = state.random_validator_address();

                let gas_payer = state
                    .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                    .alias;
                let tx_settings = TxSettings::default_from_enstablished(
                    validator_account.implicit_addresses,
                    gas_payer,
                );

                let step = ChangeMetadataBuilder::default()
                    .source(validator_account.alias)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::ChangeConsensusKey => {
                let validator_account = state.random_validator_address();

                let gas_payer = state
                    .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                    .alias;
                let tx_settings = TxSettings::default_from_enstablished(
                    validator_account.implicit_addresses,
                    gas_payer,
                );

                let step = ChangeConsensusKeyBuilder::default()
                    .source(validator_account.alias)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::DeactivateValidator => {
                let validator_account = state.random_active_validator_address();

                let gas_payer = state
                    .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                    .alias;
                let tx_settings = TxSettings::default_from_enstablished(
                    validator_account.implicit_addresses,
                    gas_payer,
                );

                let step = DeactivateValidatorBuilder::default()
                    .source(validator_account.alias)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
            TaskType::UpdateAccount => {
                let source_address = state.random_virgin_address();

                let gas_payer = state
                    .random_implicit_account_with_at_least_native_token_balance(MIN_FEE)
                    .alias;
                let tx_settings = TxSettings::default_from_enstablished(
                    source_address.implicit_addresses.clone(),
                    gas_payer,
                );

                let maybe_treshold = utils::random_between(1, 10);
                let accounts = state.random_implicit_accounts(maybe_treshold, vec![]);

                let pks = accounts
                    .into_iter()
                    .map(|account| account.alias)
                    .collect::<BTreeSet<Alias>>();
                let threshold = if pks.len() == 1 {
                    1
                } else {
                    utils::random_between(1, pks.len() as u64)
                };

                let step = UpdateAccountBuilder::default()
                    .source(source_address.alias)
                    .pks(pks)
                    .threshold(threshold)
                    .tx_settings(tx_settings)
                    .build()
                    .unwrap();

                Box::new(step)
            }
        }
    }
}

pub trait Step: DynClone + Debug + Display {
    fn to_step_type(&self, step_index: u64) -> StepType;
    fn update_state(&self, state: &mut State);
    fn post_hooks(&self, step_index: u64, state: &State) -> Vec<Box<dyn Hook>>;
    fn pre_hooks(&self, state: &State) -> Vec<Box<dyn Hook>>;
    fn total_post_hooks(&self) -> u64;
    fn total_pre_hooks(&self) -> u64;
}

dyn_clone::clone_trait_object!(Step);

pub trait Hook: DynClone + Debug + Display {
    fn to_step_type(&self) -> StepType;
}

dyn_clone::clone_trait_object!(Hook);
