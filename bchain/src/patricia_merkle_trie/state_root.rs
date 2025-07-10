use crate::wallets::Wallets;
use sha2::{Digest, Sha256};

pub fn compute(wallets: &Wallets) -> (String, Vec<Vec<String>>) {
    if wallets.wallets.is_empty() {
        let zero_hash = format!("{:x}", Sha256::new().finalize());
        return (zero_hash, vec![]);
    }

    let mut leaves: Vec<(String, String)> = wallets
        .wallets
        .iter()
        .map(|(user, wallet)| {
            let data = format!("{}{}", user, serde_json::to_string(wallet).unwrap());
            let mut hasher = Sha256::new();
            hasher.update(data);
            (user.clone(), format!("{:x}", hasher.finalize()))
        })
        .collect();
    leaves.sort_by(|a, b| a.0.cmp(&b.0));
    let mut tree: Vec<Vec<String>> = vec![leaves.iter().map(|(_, h)| h.clone()).collect()];

    let mut current_level = tree[0].clone();
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        for chunk in current_level.chunks(2) {
            let combined = if chunk.len() == 2 {
                format!("{}{}", chunk[0], chunk[1])
            } else {
                chunk[0].to_string()
            };
            let mut hasher = Sha256::new();
            hasher.update(&combined);
            next_level.push(format!("{:x}", hasher.finalize()));
        }
        tree.push(next_level.clone());
        current_level = next_level;
    }
    (current_level[0].clone(), tree)
}
