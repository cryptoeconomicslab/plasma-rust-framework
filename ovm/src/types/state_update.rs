use crate::deciders::OwnershipDecider;
use crate::property_executor::PropertyExecutor;
use crate::types::core::{Integer, Property, QuantifierResultItem};
use crate::types::PlasmaDataBlock;
use crate::DecideMixin;
use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
use plasma_core::data_structure::{Range, Transaction};
use plasma_db::traits::kvs::KeyValueStore;
use tiny_keccak::Keccak;

#[derive(Clone, Debug, AbiEncodable, AbiDecodable)]
pub struct StateUpdate {
    block_number: Integer,
    range: Range,
    property: Property,
}

impl StateUpdate {
    pub fn new(block_number: Integer, range: Range, property: Property) -> Self {
        Self {
            block_number,
            range,
            property,
        }
    }

    pub fn get_range(&self) -> Range {
        self.range
    }

    pub fn set_range(&mut self, range: Range) {
        self.range = range
    }

    pub fn get_property(&self) -> &Property {
        &self.property
    }

    pub fn get_block_number(&self) -> Integer {
        self.block_number
    }

    pub fn get_hash(&self) -> Bytes {
        let mut sha3 = Keccak::new_sha3_256();
        sha3.update(&self.to_abi());
        let mut res: [u8; 32] = [0; 32];
        sha3.finalize(&mut res);

        Bytes::from(&res[..])
    }

    pub fn get_amount(&self) -> u64 {
        self.range.get_end() - self.range.get_start()
    }

    pub fn verify_state_transition<T: KeyValueStore>(
        &self,
        decider: &mut PropertyExecutor<T>,
        transaction: &Transaction,
    ) -> bool {
        let property = self.get_property();
        decider.set_variable(
            Bytes::from("state_update"),
            QuantifierResultItem::StateUpdate(self.clone()),
        );
        let decided = property.decide(decider);
        println!(
            "decide local deprecation claim {:?}. decision = {:?}",
            transaction.get_range(),
            decided.is_ok()
        );
        decided.is_ok()
    }

    /// validate transaction and state update.
    pub fn execute_state_transition<T: KeyValueStore>(
        &self,
        decider: &mut PropertyExecutor<T>,
        transaction: &Transaction,
        next_block_number: Integer,
    ) -> Result<Self, PlasmaCoreError> {
        let next_state =
            OwnershipDecider::execute_state_transition(self, transaction, next_block_number);
        if !self.verify_state_transition(decider, transaction) {
            return Err(PlasmaCoreError::from(
                PlasmaCoreErrorKind::InvalidTransaction,
            ));
        }

        Ok(next_state)
    }
}

impl From<PlasmaDataBlock> for StateUpdate {
    fn from(plasma_data_block: PlasmaDataBlock) -> Self {
        StateUpdate::from_abi(plasma_data_block.get_data()).unwrap()
    }
}
