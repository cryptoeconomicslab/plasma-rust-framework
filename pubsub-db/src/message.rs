use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message<T> {
    pub to: String,
    pub message: T,
}

impl<T> Message<T> {
    pub fn new(to: String, message: T) -> Self {
        Message { to, message }
    }
}
