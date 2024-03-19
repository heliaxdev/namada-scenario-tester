use std::{cmp::min, collections::HashMap};

use crate::entity::{Account, Alias};
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
        self.balances
            .iter()
            .fold(vec![], |mut acc, (alias, token_balances)| {
                if token_balances.values().any(|balance| *balance > 0) {
                    let account = self.get_account_from_alias(alias);
                    acc.push(account);
                    acc
                } else {
                    acc
                }
            })
    }

    pub fn addresses_with_native_token_balance(&self) -> Vec<Account> {
        self.balances
            .iter()
            .fold(vec![], |mut acc, (alias, token_balances)| {
                if let Some(balance) = token_balances.get(&Alias::native_token()) {
                    if *balance > 0 {
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

    pub fn random_account_with_balance(&self) -> Account {
        let all_addresses_with_balance = self.addresses_with_any_token_balance();
        all_addresses_with_balance
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
        *self.balances.get(owner).unwrap().get(token).unwrap()
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

    pub fn insert_bond(&mut self, source_alias: &Alias, amount: u64) {
        let default = HashMap::from_iter([(self.last_step_id, 0u64)]);
        *self
            .bonds
            .entry(source_alias.clone())
            .or_insert(default)
            .entry(self.last_step_id)
            .or_insert(amount) += amount;
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
            .or_insert(amount) += amount;
    }

    pub fn insert_new_key(&mut self, alias: Alias) {
        self.sks.push(alias.clone());
        self.pks.push(alias.clone());
        self.implicit_addresses
            .insert(alias.clone(), Account::new_implicit_address(alias));
    }

    pub fn add_new_account(&mut self, alias: Alias, pks: Vec<Alias>, threshold: u64) {
        self.enstablished_addresses.insert(
            alias.clone(),
            Account::new_enstablished_address(alias, pks, threshold),
        );
    }
}
