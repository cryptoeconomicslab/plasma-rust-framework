use crate::predicate::PredicatePlugin;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use plasma_core::data_structure::{StateObject, StateUpdate, Transaction};

/// Simple ownership predicate
pub struct OwnershipPredicate {}

impl OwnershipPredicate {
    /// Make parameters for ownership predicate
    pub fn make_parameters(
        predicate: Address,
        owner: Address,
        origin_block: u64,
        max_block: u64,
    ) -> Vec<u8> {
        ethabi::encode(&[
            Token::Tuple(vec![
                Token::Address(predicate),
                Token::Bytes(owner.as_bytes().to_vec()),
            ]),
            Token::Uint(origin_block.into()),
            Token::Uint(max_block.into()),
        ])
    }
    /// Parse parameters of ownership predicate
    pub fn parse_parameters(data: &[u8]) -> Option<(StateObject, u64, u64)> {
        ethabi::decode(
            &[
                ParamType::Tuple(vec![ParamType::Address, ParamType::Bytes]),
                ParamType::Uint(16),
                ParamType::Uint(16),
            ],
            data,
        )
        .ok()
        .and_then(|decoded| {
            let state_object_tuple = decoded[0].clone().to_tuple();
            let origin_block = decoded[1].clone().to_uint();
            let max_block = decoded[2].clone().to_uint();
            if let (Some(state_object_tuple), Some(origin_block), Some(max_block)) =
                (state_object_tuple, origin_block, max_block)
            {
                StateObject::from_tuple(&state_object_tuple)
                    .ok()
                    .map(|state_object| (state_object, origin_block.as_u64(), max_block.as_u64()))
            } else {
                None
            }
        })
    }
}

impl Default for OwnershipPredicate {
    fn default() -> Self {
        OwnershipPredicate {}
    }
}

impl PredicatePlugin for OwnershipPredicate {
    fn execute_state_transition(
        &self,
        input: &StateUpdate,
        transaction: &Transaction,
    ) -> StateUpdate {
        // should parse transaction.parameters
        // make new state update
        let (state_object, origin_block, max_block) =
            Self::parse_parameters(transaction.get_parameters()).unwrap();
        // Where does the function get pending block number from?
        let pending_block_number = max_block;
        assert!(input.get_block_number() <= origin_block);
        assert!(pending_block_number <= max_block);
        StateUpdate::new(
            &state_object,
            transaction.get_start(),
            transaction.get_end(),
            pending_block_number,
            transaction.get_plasma_contract_address(),
        )
    }
}
