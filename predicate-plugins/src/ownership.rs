use crate::{PredicateParameters, PredicatePlugin};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::{StateObject, StateUpdate, Transaction};

/// Parameters of ownership predicate
pub struct OwnershipPredicateParameters {
    state_object: StateObject,
    origin_block: u64,
    max_block: u64,
}

impl OwnershipPredicateParameters {
    /// Creates new parameters object
    pub fn new(state_object: StateObject, origin_block: u64, max_block: u64) -> Self {
        OwnershipPredicateParameters {
            state_object,
            origin_block,
            max_block,
        }
    }
    pub fn get_state_object(&self) -> &StateObject {
        &self.state_object
    }
    pub fn get_origin_block(&self) -> u64 {
        self.origin_block
    }
    pub fn get_max_block(&self) -> u64 {
        self.max_block
    }
    /// Parse parameters from ABI
    pub fn decode(data: &[u8]) -> Option<Self> {
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
                    .map(|state_object| {
                        OwnershipPredicateParameters::new(
                            state_object,
                            origin_block.as_u64(),
                            max_block.as_u64(),
                        )
                    })
            } else {
                None
            }
        })
    }
}

impl PredicateParameters for OwnershipPredicateParameters {
    /// Make parameters for ownership predicate
    fn encode(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Tuple(vec![
                Token::Address(self.get_state_object().get_predicate()),
                Token::Bytes(self.get_state_object().get_data().to_vec()),
            ]),
            Token::Uint(self.origin_block.into()),
            Token::Uint(self.max_block.into()),
        ])
    }
}

/// Simple ownership predicate
pub struct OwnershipPredicate {}

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
        let parameters =
            OwnershipPredicateParameters::decode(transaction.get_parameters()).unwrap();
        // Where does the function get pending block number from?
        let pending_block_number = parameters.get_max_block();
        assert!(input.get_block_number() <= parameters.get_origin_block());
        assert!(pending_block_number <= parameters.get_max_block());
        StateUpdate::new(
            parameters.get_state_object().clone(),
            transaction.get_start(),
            transaction.get_end(),
            pending_block_number,
            transaction.get_plasma_contract_address(),
        )
    }

    fn query_state(&self, state_update: &StateUpdate, _parameters: &[u8]) -> Vec<Bytes> {
        let owner = state_update.get_state_object().get_data();
        vec![owner.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::{OwnershipPredicate, OwnershipPredicateParameters};
    use crate::{PredicateParameters, PredicatePlugin};
    use bytes::Bytes;
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
            StateObject::new(predicate_address, Bytes::from(&b"data"[..])),
            start,
            end,
            10,
            plasma_contract_address,
        );
        // make parameters
        let parameters = OwnershipPredicateParameters::new(
            StateObject::new(predicate_address, Bytes::from(alice_address.as_bytes())),
            10,
            20,
        );
        let parameters_bytes = parameters.encode();
        // make transaction
        let transaction = Transaction::new(
            plasma_contract_address,
            start,
            end,
            Transaction::create_method_id(&b"send(address)"[..]),
            &parameters_bytes,
            &Witness::new(H256::zero(), H256::zero(), 0),
        );

        let predicate: OwnershipPredicate = Default::default();
        let next_state_update = predicate.execute_state_transition(&state_update, &transaction);
        assert_eq!(next_state_update.get_start(), transaction.get_start());
        assert_eq!(next_state_update.get_end(), transaction.get_end());
    }

}
