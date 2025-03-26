use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Opcode {
    Push(f64),                // Push a value onto the stack
    Add,                      // Pop two values, add them, push result
    Sub,                      // Pop two values, subtract, push result
    Eq,                       // Pop two values, push 1 if equal, 0 if not
    Store(String),            // Pop a value, store it in contract storage under key
    Load(String),             // Load a value from contract storage, push to stack
    Balance(String),          // Push the balance of a user to the stack
    Transfer(String, String), // Pop amount, transfer from sender to receiver
}
