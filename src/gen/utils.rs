use rand::{
    distributions::{Alphanumeric, DistString},
    Rng,
};

pub fn random_alias() -> String {
    format!(
        "load-tester-{}",
        Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
    )
}

pub fn random_pa_alias() -> String {
    format!(
        "load-tester-pa-{}",
        Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
    )
}

pub fn random_with_namespace(ns: &str) -> String {
    format!(
        "scenario-{}-{}",
        ns,
        Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
    )
}

pub fn random_enstablished_alias() -> String {
    format!(
        "load-tester-enst-{}",
        Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
    )
}

pub fn random_between(start: u64, to: u64) -> u64 {
    rand::thread_rng().gen_range(start..to)
}
