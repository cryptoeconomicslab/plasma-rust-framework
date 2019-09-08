use super::StateUpdate;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::Error as PlasmaCoreError;

#[derive(Clone, Debug)]
pub struct StateUpdateList {
    state_updates: Vec<StateUpdate>,
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
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let state_updates = tuple[0].clone().to_array();
        let state_update_list: Vec<StateUpdate> = state_updates
            .iter()
            .filter_map(|s| StateUpdate::from_tuple(s).ok())
            .collect();
        Ok(StateUpdateList::new(state_update_list))
    }
    fn get_param_types() -> Vec<ParamType> {
        vec![ParamType::Array(Box::new(ParamType::Tuple(
            StateUpdate::get_param_types(),
        )))]
    }
}
