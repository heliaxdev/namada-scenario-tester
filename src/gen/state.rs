use std::{cmp::min, collections::{BTreeSet, HashMap}};

use crate::{
    constants::{DEFAULT_GAS_LIMIT, DEFAULT_GAS_PRICE},
    entity::{Account, Alias, Bond, TxSettings, Unbond},
};

use namada_sdk::token::NATIVE_SCALE;
use rand::prelude::SliceRandom;

pub type StepId = u64;
pub type ProposalId = u64;

#[derive(Clone, Debug, Default)]
pub struct State {
    pub sks: Vec<Alias>,
    pub pks: Vec<Alias>,
    pub implicit_addresses: HashMap<Alias, Account>,
    pub enstablished_addresses: HashMap<Alias, Account>,
    pub balances: HashMap<Alias, HashMap<Alias, u64>>,
    pub bonds: HashMap<Alias, HashMap<StepId, u64>>,
    pub unbonds: HashMap<Alias, HashMap<StepId, u64>>,
    pub proposals: HashMap<StepId, Vec<ProposalId>>,
    pub last_proposal_id: ProposalId,
    pub last_step_id: StepId,
}

#[derive(Clone, Debug, Default)]
pub struct AccountBalance {
    pub token: Alias,
    pub balance: u64,
}

impl State {
    pub fn addresses_with_any_token_balance(&self) -> Vec<Account> {
        self.addresses_with_at_least_token_balance(0)
    }

    pub fn addresses_with_at_least_token_balance(&self, amount: u64) -> Vec<Account> {
        self.balances
            .iter()
            .fold(vec![], |mut acc, (alias, token_balances)| {
                if token_balances.values().any(|balance| *balance > amount) {
                    let account = self.get_account_from_alias(alias);
                    acc.push(account);
                    acc
                } else {
                    acc
                }
            })
    }

    pub fn addresses_with_native_token_balance(&self) -> Vec<Account> {
        self.addresses_with_at_least_native_token_balance(0)
    }

    pub fn addresses_with_at_least_native_token_balance(&self, amount: u64) -> Vec<Account> {
        self.balances
            .iter()
            .fold(vec![], |mut acc, (alias, token_balances)| {
                if let Some(balance) = token_balances.get(&Alias::native_token()) {
                    if *balance > amount {
                        let account = self.get_account_from_alias(alias);
                        acc.push(account);
                    }
                    acc
                } else {
                    acc
                }
            })
    }

    pub fn implicit_addresses_with_at_least_native_token_balance(
        &self,
        amount: u64,
    ) -> Vec<Account> {
        self.balances
            .iter()
            .filter(|(alias, _)| !alias.to_string().starts_with("load-tester-enst"))
            .fold(vec![], |mut acc, (alias, token_balances)| {
                if let Some(balance) = token_balances.get(&Alias::native_token()) {
                    if *balance > amount {
                        let account = self.get_account_from_alias(alias);
                        acc.push(account);
                    }
                    acc
                } else {
                    acc
                }
            })
    }

    pub fn get_account_from_alias(&self, alias: &Alias) -> Account {
        let is_implicit = self.implicit_addresses.get(alias);
        let is_enstablished = self.enstablished_addresses.get(alias);

        if let Some(implicit) = is_implicit {
            implicit.clone()
        } else if let Some(enstablished) = is_enstablished {
            enstablished.clone()
        } else {
            panic!()
        }
    }

    pub fn any_address(&self) -> Vec<Account> {
        let implicit_accounts = self
            .implicit_addresses
            .values()
            .cloned()
            .collect::<Vec<Account>>();
        let enstablished_accounts = self.enstablished_addresses.values().cloned().collect();

        [implicit_accounts, enstablished_accounts].concat()
    }

    pub fn any_implicit_address(&self) -> Vec<Account> {
        self.implicit_addresses
            .values()
            .cloned()
            .collect::<Vec<Account>>()
    }

    pub fn any_non_validator_address(&self) -> Vec<Account> {
        self.enstablished_addresses
            .values()
            .filter(|account| {
                !account.is_validator
                    && account
                        .address_type
                        .eq(&crate::entity::AddressType::Enstablished)
            })
            .cloned()
            .collect()
    }

    pub fn any_validator_address(&self) -> Vec<Account> {
        self.enstablished_addresses
            .values()
            .filter(|account| account.is_validator)
            .cloned()
            .collect()
    }

    pub fn random_non_validator_address(&self) -> Account {
        self.any_non_validator_address()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone()
    }

    pub fn random_validator_address(&self) -> Account {
        self.any_validator_address()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone()
    }

    pub fn random_account(&self, blacklist: Vec<Account>) -> Account {
        let all_addresses = self.any_address();

        let account = loop {
            let maybe_account = all_addresses
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone();

            if !blacklist.contains(&maybe_account) {
                break maybe_account;
            }
        };

        account
    }

    pub fn random_accounts(&self, total: u64, blacklist: Vec<Account>) -> Vec<Account> {
        let all_addresses = self.any_address();
        let total = min(total as usize, all_addresses.len() - blacklist.len());

        if total == 0 {
            return vec![];
        }

        let mut accounts = vec![];

        loop {
            let maybe_account = all_addresses
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone();

            if !blacklist.contains(&maybe_account) {
                accounts.push(maybe_account);
            }

            if accounts.len() == total {
                return accounts;
            }
        }
    }

    pub fn random_implicit_accounts(&self, total: u64, blacklist: Vec<Account>) -> Vec<Account> {
        let all_addresses = self.any_implicit_address();
        let total = min(total as usize, all_addresses.len() - blacklist.len());

        if total == 0 {
            return vec![];
        }

        let mut accounts = vec![];

        loop {
            let maybe_account = all_addresses
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone();

            if !blacklist.contains(&maybe_account) {
                accounts.push(maybe_account);
            }

            if accounts.len() == total {
                return accounts;
            }
        }
    }

    pub fn random_account_with_balance(&self) -> Account {
        let all_addresses_with_balance = self.addresses_with_any_token_balance();
        all_addresses_with_balance
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone()
    }

    pub fn random_account_with_at_least_native_token_balance(&self, amount: u64) -> Account {
        let all_addresses_with_native_token_balance =
            self.addresses_with_at_least_native_token_balance(amount);
        all_addresses_with_native_token_balance
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone()
    }

    pub fn random_implicit_account_with_at_least_native_token_balance(
        &self,
        amount: u64,
    ) -> Account {
        let all_implicit_addresses_with_native_token_balance =
            self.implicit_addresses_with_at_least_native_token_balance(amount);
        all_implicit_addresses_with_native_token_balance
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone()
    }

    pub fn random_account_with_native_token_balance(&self) -> Account {
        let all_addresses_with_native_token_balance = self.addresses_with_native_token_balance();
        all_addresses_with_native_token_balance
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone()
    }

    pub fn random_token_balance_for_alias(&self, alias: &Alias) -> AccountBalance {
        let balances = self.balances.get(alias).unwrap();
        balances
            .iter()
            .next()
            .map(|(token, balance)| AccountBalance {
                token: token.clone(),
                balance: *balance,
            })
            .unwrap()
    }

    pub fn random_native_token_balance_for_alias(&self, alias: &Alias) -> AccountBalance {
        let balances = self.balances.get(alias).unwrap();
        balances
            .get(&Alias::native_token())
            .map(|balance| AccountBalance {
                token: Alias::native_token(),
                balance: *balance,
            })
            .unwrap()
    }

    pub fn get_alias_token_balance(&self, owner: &Alias, token: &Alias) -> u64 {
        if let Some(balances) = self.balances.get(owner) {
            *balances.get(token).unwrap_or(&0u64)
        } else {
            0u64
        }
    }

    pub fn decrease_account_token_balance(
        &mut self,
        address_alias: &Alias,
        token_alias: &Alias,
        amount: u64,
    ) {
        *self
            .balances
            .get_mut(address_alias)
            .unwrap()
            .get_mut(token_alias)
            .unwrap() -= amount;
    }

    pub fn decrease_account_fees(
        &mut self,
        address_alias: &Alias,
        tx_settings: &Option<TxSettings>,
    ) {
        let gas_limit = if let Some(tx_settings) = tx_settings {
            tx_settings.gas_limit
        } else {
            DEFAULT_GAS_LIMIT
        };
        let gas_price = (gas_limit as f64 * DEFAULT_GAS_PRICE * NATIVE_SCALE as f64).ceil() as u64;
        self.decrease_account_token_balance(address_alias, &Alias::native_token(), gas_price)
    }

    pub fn any_bond(&self) -> Vec<Bond> {
        let mut bonds = vec![];
        for alias in self.bonds.keys() {
            for (step_id, amount) in self.bonds.get(alias).unwrap() {
                if *amount == 0 {
                    continue;
                }
                let bond = Bond {
                    source: alias.clone(),
                    amount: *amount,
                    step_id: *step_id,
                };
                bonds.push(bond)
            }
        }

        bonds
    }

    pub fn any_unbond(&self) -> Vec<Unbond> {
        let mut unbonds = vec![];
        for alias in self.unbonds.keys() {
            for (step_id, amount) in self.unbonds.get(alias).unwrap() {
                if *amount == 0 {
                    continue;
                }
                let bond = Unbond {
                    source: alias.clone(),
                    amount: *amount,
                    step_id: *step_id,
                };
                unbonds.push(bond)
            }
        }

        unbonds
    }

    pub fn random_bond(&self) -> Bond {
        self.any_bond()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone()
    }

    pub fn random_unbond(&self) -> Unbond {
        self.any_unbond()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone()
    }

    pub fn insert_bond(&mut self, source_alias: &Alias, amount: u64) {
        let default = HashMap::from_iter([(self.last_step_id, 0u64)]);
        *self
            .bonds
            .entry(source_alias.clone())
            .or_insert(default)
            .entry(self.last_step_id)
            .or_insert(0) += amount;
    }

    pub fn insert_unbond(&mut self, source_alias: &Alias, amount: u64, bond_step: u64) {
        // decrease bond
        *self
            .bonds
            .get_mut(source_alias)
            .unwrap()
            .get_mut(&bond_step)
            .unwrap() -= amount;

        // increase unbonds
        let default = HashMap::from_iter([(self.last_step_id, 0u64)]);
        *self
            .unbonds
            .entry(source_alias.clone())
            .or_insert(default)
            .entry(self.last_step_id)
            .or_insert(0) += amount;
    }

    pub fn get_account_total_bonded(&self, source: &Alias) -> u64 {
        self.bonds.get(source).unwrap().values().sum()
    }

    pub fn update_bonds_by_redelegation(
        &mut self,
        source_alias: &Alias,
        bond_step_id: u64,
        amount: u64,
    ) {
        // decrease bond
        *self
            .bonds
            .get_mut(source_alias)
            .unwrap()
            .get_mut(&bond_step_id)
            .unwrap() -= amount;

        self.insert_bond(source_alias, amount)
    }

    pub fn insert_withdraw(&mut self, source_alias: &Alias, amount: u64, unbond_step: u64) {
        // decrease unbonds
        *self
            .unbonds
            .get_mut(source_alias)
            .unwrap()
            .get_mut(&unbond_step)
            .unwrap() -= amount;
    }

    pub fn increase_account_token_balance(
        &mut self,
        address_alias: &Alias,
        token_alias: Alias,
        amount: u64,
    ) {
        *self
            .balances
            .entry(address_alias.clone())
            .or_insert(HashMap::from_iter([(token_alias.clone(), 0)]))
            .entry(token_alias)
            .or_insert(0) += amount;
    }

    pub fn insert_new_key(&mut self, alias: Alias) {
        self.sks.push(alias.clone());
        self.pks.push(alias.clone());
        self.implicit_addresses
            .insert(alias.clone(), Account::new_implicit_address(alias));
    }

    pub fn add_new_account(&mut self, alias: Alias, pks: BTreeSet<Alias>, threshold: u64) {
        self.enstablished_addresses.insert(
            alias.clone(),
            Account::new_enstablished_address(alias, pks, threshold),
        );
    }

    pub fn set_account_as_validator(&mut self, alias: &Alias) {
        let old_account = self.enstablished_addresses.get(alias).unwrap().clone();

        let new_account = Account {
            is_validator: true,
            ..old_account
        };

        self.enstablished_addresses
            .insert(alias.clone(), new_account);
    }
}
