use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub to: String,
    pub message: Vec<u8>,
}

impl Message {
    pub fn new(to: String, message: Vec<u8>) -> Self {
        Message { to, message }
    }
}
