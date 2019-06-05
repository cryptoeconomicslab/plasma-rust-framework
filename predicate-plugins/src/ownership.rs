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

#[cfg(test)]
mod tests {
    use super::OwnershipPredicate;
    use crate::predicate::PredicatePlugin;
    use ethereum_types::{Address, H256};
    use plasma_core::data_structure::{StateObject, StateUpdate, Transaction, Witness};

    #[test]
    fn test_execute_state_transition() {
        let start = 0;
        let end = 1000;
        let plasma_contract_address = Address::zero();
        let predicate_address = Address::zero();
        let alice_address = Address::zero();
        // make state update
        let state_update = StateUpdate::new(
            &StateObject::new(predicate_address, &b"data"[..]),
            start,
            end,
            10,
            plasma_contract_address,
        );
        // make parameters
        let parameters =
            OwnershipPredicate::make_parameters(predicate_address, alice_address, 10, 20);
        // make transaction
        let transaction = Transaction::new(
            plasma_contract_address,
            start,
            end,
            Transaction::create_method_id(&b"send(address)"[..]),
            &parameters,
            &Witness::new(H256::zero(), H256::zero(), 0),
        );

        let predicate: OwnershipPredicate = Default::default();
        let next_state_update = predicate.execute_state_transition(&state_update, &transaction);
        assert_eq!(next_state_update.get_start(), transaction.get_start());
        assert_eq!(next_state_update.get_end(), transaction.get_end());
    }

}
