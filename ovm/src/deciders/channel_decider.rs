use crate::db::TransactionDb;
use crate::deciders::signed_by_decider::Verifier;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, Integer, OwnershipDeciderInput, StateUpdate};
use bytes::Bytes;
use ethereum_types::Address;
use plasma_core::data_structure::Transaction;
use plasma_db::traits::kvs::KeyValueStore;

pub struct PaymentChannelOnPlasmaDecider {}

impl Decider for PaymentChannelOnPlasmaDecider {
    type Input = OwnershipDeciderInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &OwnershipDeciderInput,
    ) -> Result<Decision, Error> {
        let db: TransactionDb<T> = TransactionDb::new(decider.get_range_db());
        let state_update = input.get_state_update();
        let txs =
            db.get_transactions(state_update.get_block_number().0, state_update.get_range())?;
        if txs.is_empty() {
            return Err(Error::from(ErrorKind::CannotDecide));
        }
        for tx in txs.iter() {
            if Verifier::recover(&tx.get_signatures()[0], &Bytes::from(tx.to_body_abi()))
                == PaymentChannelOnPlasmaDecider::get_owner_address(input)
                && Verifier::recover(&tx.get_signatures()[1], &Bytes::from(tx.to_body_abi()))
                    == PaymentChannelOnPlasmaDecider::get_participant_address(input)
            {
                return Ok(Decision::new(true, vec![]));
            }
        }

        Ok(Decision::new(false, vec![]))
    }
}

impl PaymentChannelOnPlasmaDecider {
    pub fn get_owner_address(input: &OwnershipDeciderInput) -> Address {
        Address::from_slice(&input.get_state_update().get_params()[0..20])
    }
    pub fn get_participant_address(input: &OwnershipDeciderInput) -> Address {
        Address::from_slice(&input.get_state_update().get_params()[32..52])
    }
    pub fn execute_state_transition(
        prev_state: &StateUpdate,
        transaction: &Transaction,
        next_block_number: Integer,
    ) -> StateUpdate {
        StateUpdate::new(
            next_block_number,
            transaction.get_range(),
            prev_state.get_property_address(),
            transaction.get_parameters().clone(),
        )
    }
}
