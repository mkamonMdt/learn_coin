pub mod bchain_error;
pub mod message;
pub mod primitives;

mod blockchain;
mod chain;
mod config;
mod patricia_merkle_trie;
mod validators;
mod wallets;

pub use crate::config::static_config::GENESIS;
pub use blockchain::Blockchain;
