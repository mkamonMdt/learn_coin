use crate::patricia_merkle_trie::state_root;
use crate::primitives::Block;
use crate::wallets::Wallets;

use sha2::{Digest, Sha256};

pub fn get_merkle_proof(wallets: &Wallets, user: &str) -> Option<Vec<(String, bool)>> {
    let (_, tree) = state_root::compute(wallets);
    if tree.is_empty() {
        return None;
    }

    // Find the leaf index for the user
    let leaves = &tree[0];
    let leaf_data = format!(
        "{}{}",
        user,
        serde_json::to_string(wallets.wallets.get(user)?).unwrap()
    );
    let mut hasher = Sha256::new();
    hasher.update(&leaf_data);
    let leaf_hash = format!("{:x}", hasher.finalize());
    let leaf_idx = leaves.iter().position(|h| *h == leaf_hash)?;

    // Build proof by collection siblings
    let mut proof = Vec::new();
    let mut idx = leaf_idx;
    for level in &tree[..tree.len() - 1] {
        let is_left = idx % 2 == 0;
        let sibling_idx = if is_left { idx + 1 } else { idx - 1 };
        if sibling_idx < level.len() {
            proof.push((level[sibling_idx].clone(), is_left))
        }
        idx /= 2; //Move up to parent
    }
    Some(proof)
}

pub fn verify_merkle_proof(
    wallets: &Wallets,
    block: &Block,
    user: &str,
    proof: &[(String, bool)],
) -> bool {
    let wallet = match wallets.wallets.get(user) {
        Some(w) => w,
        None => return false,
    };

    //compute leaf hash
    let leaf_data = format!("{}{}", user, serde_json::to_string(wallet).unwrap());
    let mut hasher = Sha256::new();
    hasher.update(&leaf_data);
    let mut current_hash = format!("{:x}", hasher.finalize());

    //Recompute root using proof
    for (sibling, is_left) in proof {
        let combined = if *is_left {
            format!("{}{}", current_hash, sibling)
        } else {
            format!("{}{}", sibling, current_hash)
        };
        let mut hasher = Sha256::new();
        hasher.update(&combined);
        current_hash = format!("{:x}", hasher.finalize());
    }

    current_hash == block.state_root
}
