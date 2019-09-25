use ethereum_types::Address;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Token {
    name: String,
    address: Address,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            name: "token".to_string(),
            address: Address::zero(),
        }
    }
}

impl Token {
    pub fn new(name: &str, address: Address) -> Self {
        Self {
            name: name.to_string(),
            address,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_address(&self) -> Address {
        self.address
    }
}
