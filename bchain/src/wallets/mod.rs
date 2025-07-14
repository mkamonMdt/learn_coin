use crate::primitives::Wallet;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Wallets {
    pub wallets: HashMap<String, Wallet>,
}
