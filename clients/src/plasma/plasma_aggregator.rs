use super::block_manager::BlockManager;
use super::command::NewTransactionEvent;
use super::error::{Error, ErrorKind};
use super::plasma_block::PlasmaBlock;
use super::plasma_client::PlasmaClientShell;
use super::state_db::StateDb;
use super::token::Token;
use super::utils::*;
use bytes::Bytes;
use ethereum_types::Address;
use ethsign::SecretKey;
use ovm::db::{SignedByDb, TransactionDb};
use ovm::deciders::SignVerifier;
use ovm::property_executor::PropertyExecutor;
use ovm::types::Integer;
use ovm::types::{StateUpdate, StateUpdateList};
use plasma_core::data_structure::{Range, Transaction};
use plasma_db::prelude::*;

pub struct PlasmaAggregator<KVS: KeyValueStore> {
    aggregator_address: Address,
    commitment_contract_address: Address,
    deposit_contract_address: Address,
    //_secret_key: SecretKey,
    _raw_key: Vec<u8>,
    _my_address: Address,
    block_manager: BlockManager<KVS>,
    decider: PropertyExecutor<KVS>,
    //_secret_key: SecretKey,
    //    state_update_queue:
}

impl<KVS: KeyValueStore + DatabaseTrait> PlasmaAggregator<KVS> {
    pub fn new(
        aggregator_address: Address,
        deposit_contract_address: Address,
        commitment_contract_address: Address,
        private_key: &str,
    ) -> Self {
        let raw_key = hex::decode(private_key).unwrap();
        let secret_key = SecretKey::from_raw(&raw_key).unwrap();
        let my_address: Address = secret_key.public().address().into();
        let block_manager = BlockManager::new(aggregator_address, commitment_contract_address);

        PlasmaAggregator {
            aggregator_address,
            deposit_contract_address,
            commitment_contract_address,
            _raw_key: raw_key,
            //_secret_key: secret_key,
            _my_address: my_address,
            block_manager,
            decider: Default::default(),
        }
    }

    // 1. query all state_updates overlapping with given range.
    // 2. check if the range of transaction is covered by queried state_updates. If not, return
    //    InvalidTransaction Error.
    // 3. for all state_updates, check state transition using state_update.property.decide(transaction).
    //    any of these throw error, return InvalidTransaction Error.
    // 4. if all transitions are verified, add new state_update to a queue.
    //
    // TODO:
    // - handle multi prev_states case.
    // - fix decide logic for state transition.
    pub fn ingest_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<NewTransactionEvent, Error> {
        let transaction_db = TransactionDb::new(self.decider.get_range_db());
        let signed_by_db = SignedByDb::new(self.decider.get_db());
        let next_block_number = self.block_manager.get_current_block_number();
        let mut state_db = StateDb::new(self.decider.get_range_db());
        let state_updates = state_db
            .get_verified_state_updates(
                transaction.get_deposit_contract_address(),
                transaction.get_range().get_start(),
                transaction.get_range().get_end(),
            )
            .unwrap();
        if state_updates.is_empty() {
            return Err(Error::from(ErrorKind::InvalidTransaction));
        }
        // Store witness
        // TODO: if one of these Database operation failed, need to roll back all of them.
        for prev_state in state_updates.clone() {
            transaction_db.put_transaction(prev_state.get_block_number().0, transaction.clone());
        }
        let message = Bytes::from(transaction.to_body_abi());
        assert!(signed_by_db
            .store_witness(
                SignVerifier::recover(transaction.get_signature(), &message),
                message,
                transaction.get_signature().clone(),
            )
            .is_ok());
        // Check that the transaction deprecate all previous state_updates within same coin range.
        for prev_state in state_updates.clone() {
            // Current execute_state_transition returns next state_update which has the same range as transaction.
            // It means same next_state is added to storage multiple times and it's overwrite.
            if let Ok(next_state) = prev_state.execute_state_transition(
                &self.decider,
                &transaction,
                Integer(next_block_number),
            ) {
                self.block_manager.enqueue_state_update(&next_state)?;
                state_db.put_verified_state_update(&next_state)?;
            } else {
                return Err(Error::from(ErrorKind::InvalidTransaction));
            }
        }
        let prev_block_numbers = state_updates.iter().map(|s| s.get_block_number()).collect();
        let new_tx = NewTransactionEvent::new(prev_block_numbers, transaction.clone());
        self.block_manager.enqueue_tx(new_tx.clone())?;
        Ok(new_tx)
    }

    pub fn submit_next_block(&mut self) -> Result<(), Error> {
        // dequeue all state_update stored in range db
        // generate block using that data.
        let block_manager = &mut self.block_manager;
        block_manager.submit_next_block()
    }

    pub fn get_aggregator_addres(&self) -> Address {
        self.aggregator_address
    }

    pub fn get_commitment_contract_address(&self) -> Address {
        self.commitment_contract_address
    }

    pub fn get_deposit_contract_address(&self) -> Address {
        self.deposit_contract_address
    }

    pub fn show_queued_state_updates(&self) {
        println!("{:?}", self.block_manager.get_queued_state_updates());
    }

    pub fn insert_test_ranges(&mut self) {
        let mut state_db = StateDb::new(self.decider.get_range_db());
        let eth_token_address = Address::zero();
        let dai_token_address = string_to_address("0000000000000000000000000000000000000001");
        for i in 0..3 {
            let state_update = StateUpdate::new(
                Integer::new(0),
                eth_token_address,
                Range::new(i * 20, (i + 1) * 20),
                PlasmaClientShell::create_ownership_state_object(string_to_address(
                    "627306090abab3a6e1400e9345bc60c78a8bef57",
                )),
            );
            assert!(state_db.put_verified_state_update(&state_update).is_ok());
        }
        for i in 0..3 {
            let state_update = StateUpdate::new(
                Integer::new(0),
                dai_token_address,
                Range::new(i * 100, (i + 1) * 100),
                PlasmaClientShell::create_ownership_state_object(string_to_address(
                    "627306090abab3a6e1400e9345bc60c78a8bef57",
                )),
            );
            assert!(state_db.put_verified_state_update(&state_update).is_ok());
        }
    }

    pub fn get_all_state_updates(&self) -> Vec<StateUpdate> {
        let state_db = StateDb::new(self.decider.get_range_db());
        state_db.get_all_state_updates().unwrap_or_else(|_| vec![])
    }

    pub fn get_state_updates_of_block(
        &self,
        block_number: Integer,
    ) -> Result<StateUpdateList, Error> {
        self.block_manager
            .get_block_range(block_number)
            .map(|b| StateUpdateList::new(b.get_state_updates().to_vec()))
    }

    pub fn get_plasma_block_of_block(&self, block_number: Integer) -> Result<PlasmaBlock, Error> {
        self.block_manager.get_block_range(block_number)
    }

    pub fn register_token(_token: Token) {
        // TODO: implement
        unimplemented!("Register Token is not impemented yet");
    }

    // TODO: get dynamically using token map?
    pub fn get_all_tokens(&self) -> Vec<Token> {
        vec![
            Token::new("ETH", Address::zero()),
            Token::new(
                "DAI",
                string_to_address("0000000000000000000000000000000000000001"),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi_utils::abi::Encodable;
    use ethereum_types::Address;
    use ethsign::SecretKey;
    use ovm::deciders::SignVerifier;
    use plasma_core::data_structure::{Metadata, Range, Transaction, TransactionParams};
    use plasma_db::impls::kvs::CoreDbMemoryImpl;

    #[test]
    fn test_ingest() {
        let mut aggregator: PlasmaAggregator<CoreDbMemoryImpl> = PlasmaAggregator::new(
            Address::zero(),
            Address::zero(),
            Address::zero(),
            "c87509a1c067bbde78beb793e6fa76530b6382a4c0241e5e4a9ec0a0f44dc0d3",
        );
        let secret_key_raw =
            hex::decode("c87509a1c067bbde78beb793e6fa76530b6382a4c0241e5e4a9ec0a0f44dc0d3")
                .unwrap();
        let secret_key = SecretKey::from_raw(&secret_key_raw).unwrap();
        let test_range = Range::new(5, 15);
        let parameters = PlasmaClientShell::create_ownership_state_object(Address::zero()).to_abi();
        aggregator.insert_test_ranges();
        let transaction_params =
            TransactionParams::new(Address::zero(), test_range, Bytes::from(parameters));
        let signature = SignVerifier::sign(&secret_key, &Bytes::from(transaction_params.to_abi()));
        let transaction =
            Transaction::from_params(transaction_params, signature, Metadata::default());
        let result = aggregator.ingest_transaction(transaction);
        assert!(result.is_ok());
    }
}
