use super::StateUpdate;
use abi_utils::{Decodable, Encodable, Error as AbiError};
use ethabi::{ParamType, Token};

#[derive(Clone, Debug)]
pub struct StateUpdateList {
    pub state_updates: Vec<StateUpdate>,
}

impl StateUpdateList {
    pub fn new(state_updates: Vec<StateUpdate>) -> Self {
        Self { state_updates }
    }
}

impl Encodable for StateUpdateList {
    fn to_tuple(&self) -> Vec<Token> {
        vec![Token::Array(
            self.state_updates
                .iter()
                .map(|s| Token::Tuple(s.to_tuple()))
                .collect(),
        )]
    }
}

impl Decodable for StateUpdateList {
    type Ok = StateUpdateList;
    fn from_tuple(tuple: &[Token]) -> Result<Self, AbiError> {
        let state_updates = tuple[0].clone().to_array().unwrap();
        let state_update_list: Vec<StateUpdate> = state_updates
            .iter()
            .filter_map(|s| StateUpdate::from_tuple(&s.clone().to_tuple().unwrap()).ok())
            .collect();
        Ok(StateUpdateList::new(state_update_list))
    }
    fn get_param_types() -> Vec<ParamType> {
        vec![ParamType::Array(Box::new(ParamType::Tuple(
            StateUpdate::get_param_types(),
        )))]
    }
}

#[cfg(test)]
mod tests {

    use super::StateUpdateList;
    use crate::types::PropertyInput;
    use crate::types::{Integer, StateUpdate};
    use crate::DeciderManager;
    use abi_utils::{Decodable, Encodable};
    use ethereum_types::H256;
    use plasma_core::data_structure::Range;

    #[test]
    fn test_encode_and_decode() {
        let property =
            DeciderManager::preimage_exists_decider(vec![
                PropertyInput::ConstantH256(H256::zero()),
            ]);
        let state_update = StateUpdate::new(Integer(10), Range::new(0, 100), property);
        let state_update_list = StateUpdateList::new(vec![state_update]);
        let encoded = state_update_list.to_abi();
        let decoded = StateUpdateList::from_abi(&encoded).unwrap();
        assert_eq!(
            decoded.state_updates.len(),
            state_update_list.state_updates.len()
        );
    }
}
