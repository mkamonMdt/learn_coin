use ed25519_dalek::Keypair;
use ed25519_dalek::PublicKey;
use ed25519_dalek::Signature;
use ed25519_dalek::Signer;
use ed25519_dalek::Verifier;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    sender: String,
    receiver: String,
    amount: f64,
    timestamp: u64,
    signature: String,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: f64) -> Self {
        Transaction {
            sender,
            receiver,
            amount,
            timestamp: 0,
            signature: String::new(),
        }
    }

    pub fn sign_transaction(&mut self, keypair: &Keypair) {
        let message = format!("{}{}{}", self.sender, self.receiver, self.amount);
        let signature = keypair.sign(message.as_bytes());
        self.signature = hex::encode(signature.to_bytes());
    }

    pub fn verify_signature(&self, public_key: &PublicKey) -> bool {
        let message = format!("{}{}{}", self.sender, self.receiver, self.amount);
        let signature_bytes = hex::decode(&self.signature).unwrap();
        let signature = Signature::from_bytes(&signature_bytes).unwrap();
        public_key.verify(message.as_bytes(), &signature).is_ok()
    }

    pub fn is_valid(&self, public_key: &PublicKey, sender_balance: f64) -> bool {
        if !self.verify_signature(public_key) {
            return false;
        }
        if self.amount > sender_balance {
            return false;
        }
        true
    }
}
