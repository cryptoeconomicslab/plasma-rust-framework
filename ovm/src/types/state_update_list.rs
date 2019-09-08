use super::StateUpdate;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::Error as PlasmaCoreError;
use serde::{Deserialize, Serialize};
use bincode::{deserialize, serialize};

#[derive(Clone, Debug)]
pub struct StateUpdateList 
{
    pub state_updates: Vec<StateUpdate>,
}

impl StateUpdateList {
    pub fn new(state_updates: Vec<StateUpdate>) -> Self {
        Self {
            state_updates
        }
    }
    pub fn serialize(&self) -> Vec<u8> {
        let bytes: Vec<Vec<u8>> = self.state_updates.iter().map(|s| s.to_abi().to_vec()).collect();
        serialize(&bytes).unwrap()
    }
    pub fn deserialize(message: Vec<u8>) -> Self {
        let deserialized: Vec<Vec<u8>> = deserialize(&message).unwrap();
        StateUpdateList::new(deserialized.iter().map(|s| StateUpdate::from_abi(&s).unwrap()).collect())
    }

}
