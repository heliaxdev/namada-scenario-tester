use async_trait::async_trait;
use namada_sdk::args::Bond;
use namada_sdk::masp::{find_valid_diversifier, PaymentAddress};
use namada_sdk::masp_primitives::zip32;
use namada_sdk::{address::Address, key::SchemeType};
use rand::rngs::OsRng;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use super::{Task, TaskParam};
use crate::utils::settings::TxSettings;
use crate::utils::value::Value;
use crate::{
    scenario::StepResult,
    sdk::namada::Sdk,
    state::state::{StateAddress, StepStorage, Storage},
};
use namada_sdk::key::RefTo;

pub enum WalletNewKeyStorageKeys {
    Alias,
    PrivateKey,
    PublicKey,
    Address,
    PaymentAddress,
    SpendingKey
}

impl ToString for WalletNewKeyStorageKeys {
    fn to_string(&self) -> String {
        match self {
            WalletNewKeyStorageKeys::Address => "address".to_string(),
            WalletNewKeyStorageKeys::Alias => "alias".to_string(),
            WalletNewKeyStorageKeys::PublicKey => "public-key".to_string(),
            WalletNewKeyStorageKeys::PrivateKey => "private-key".to_string(),
            WalletNewKeyStorageKeys::PaymentAddress => "payment-address".to_string(),
            WalletNewKeyStorageKeys::SpendingKey => "spending-key".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct WalletNewKey {}

impl WalletNewKey {
    pub fn new() -> Self {
        Self {}
    }
}

impl WalletNewKey {
    pub fn generate_random_alias() -> String {
        let random_suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();

        format!("lt-addr-{}", random_suffix)
    }
}

#[async_trait(?Send)]
impl Task for WalletNewKey {
    type P = WalletNewKeyParameters;
    type B = Bond; // just a placeholder

    async fn execute(
        &self,
        sdk: &Sdk,
        dto: Self::P,
        _settings: TxSettings,
        _state: &Storage,
    ) -> StepResult {
        let alias = dto.alias;

        let mut wallet = sdk.namada.wallet.write().await;

        let keypair =
            wallet.gen_store_secret_key(SchemeType::Ed25519, Some(alias), true, None, &mut OsRng);

        let (alias, sk) = if let Some((alias, sk)) = keypair {
            wallet.save().expect("unable to save wallet");
            (alias, sk)
        } else {
            return StepResult::fail("Failed saving wallet pk".to_string());
        };

        let address = Address::from(&sk.ref_to()).to_string();

         // this also generates and store in the wallet the viewing key (witht he same alias)
         let spending_key_alias = format!("{}-masp", alias);
        let spending_key = wallet.gen_store_spending_key(spending_key_alias.clone(), None, true, &mut OsRng);

        let (alias, spending_key) = if let Some((alias, sk)) = spending_key {
            wallet.save().expect("unable to save wallet");
            (alias, sk)
        } else {
            return StepResult::fail("Failed saving wallet spending key".to_string());
        };

        let viewing_key = zip32::ExtendedFullViewingKey::from(&spending_key.into()).fvk.vk;
        let (div, _g_d) = find_valid_diversifier(&mut OsRng);
        let masp_payment_addr: namada_sdk::masp_primitives::sapling::PaymentAddress = viewing_key
            .to_payment_address(div)
            .expect("a PaymentAddress");
        let payment_addr = PaymentAddress::from(masp_payment_addr);

        let payment_address_alias = format!("{}-pa", alias);
        wallet.insert_payment_addr(payment_address_alias, payment_addr, true);
        wallet.save().expect("unable to save wallet");

        let mut storage = StepStorage::default();
        storage.add(
            WalletNewKeyStorageKeys::Alias.to_string(),
            alias.to_string(),
        );
        storage.add(
            WalletNewKeyStorageKeys::PublicKey.to_string(),
            sk.ref_to().to_string(),
        );
        storage.add(
            WalletNewKeyStorageKeys::Address.to_string(),
            address.clone(),
        );
        storage.add(
            WalletNewKeyStorageKeys::PrivateKey.to_string(),
            sk.to_string(),
        );
        storage.add(
            WalletNewKeyStorageKeys::PaymentAddress.to_string(),
            payment_addr.to_string(),
        );
        storage.add(
            WalletNewKeyStorageKeys::SpendingKey.to_string(),
            spending_key.to_string(),
        );

        let address = StateAddress::new_implicit(alias, address);

        StepResult::success_with_accounts(storage, vec![address])
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WalletNewKeyParametersDto {
    pub alias: Value,
}

#[derive(Clone, Debug)]
pub struct WalletNewKeyParameters {
    alias: String,
}

impl TaskParam for WalletNewKeyParameters {
    type D = WalletNewKeyParametersDto;

    fn parameter_from_dto(dto: Self::D, _state: &Storage) -> Self {
        let alias = match dto.alias {
            Value::Ref { .. } => unimplemented!(),
            Value::Value { value } => value.to_string(),
            Value::Fuzz { .. } => WalletNewKey::generate_random_alias(),
        };

        WalletNewKeyParameters { alias }
    }
}
