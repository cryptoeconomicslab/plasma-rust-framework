use crate::property_executor::PropertyExecutor;
use crate::types::core::{Integer, Property};
use crate::types::PlasmaDataBlock;
use crate::DecideMixin;
use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
use plasma_core::data_structure::{Range, Transaction};
use plasma_db::impls::kvs::CoreDbMemoryImpl;
use tiny_keccak::Keccak;

#[derive(Clone, Debug, AbiEncodable, AbiDecodable)]
pub struct StateUpdate {
    block_number: Integer,
    range: Range,
    property_address: Address,
    params: Bytes,
}

impl StateUpdate {
    pub fn new(
        block_number: Integer,
        range: Range,
        property_address: Address,
        params: Bytes,
    ) -> Self {
        Self {
            block_number,
            range,
            property_address,
            params,
        }
    }

    pub fn get_range(&self) -> Range {
        self.range
    }

    pub fn set_range(&mut self, range: Range) {
        self.range = range
    }

    pub fn get_property_address(&self) -> Address {
        self.property_address
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

    pub fn get_params(&self) -> Bytes {
        self.params.clone()
    }

    pub fn verify_state_transition(&self, transaction: &Transaction) -> bool {
        let decider = PropertyExecutor::<CoreDbMemoryImpl>::default();
        let address = self.get_property_address();
        let property = Property::get_generalized_plasma_property(address, self.clone());
        let decided = property.decide(&decider);
        println!("decided: {:?}", decided);
        decided.is_ok()
    }

    /// validate transaction and state update.
    pub fn execute_state_transition(
        &self,
        transaction: &Transaction,
    ) -> Result<Self, PlasmaCoreError> {
        if !self.verify_state_transition(transaction) {
            return Err(PlasmaCoreError::from(
                PlasmaCoreErrorKind::InvalidTransaction,
            ));
        }

        Ok(Self {
            block_number: Integer::new(self.block_number.0 + 1),
            range: transaction.get_range().clone(),
            property_address: self.property_address.clone(),
            params: transaction.get_parameters().clone(),
        })
    }
}

impl From<PlasmaDataBlock> for StateUpdate {
    fn from(plasma_data_block: PlasmaDataBlock) -> Self {
        StateUpdate::new(
            plasma_data_block.get_block_number(),
            plasma_data_block.get_updated_range(),
            plasma_data_block.get_property().clone().get_decider_id(),
            Bytes::new(), // TODO: save params in PlasmaDataBlock
        )
    }
}

impl Default for StateUpdate {
    fn default() -> Self {
        Self {
            block_number: Integer::new(0),
            range: Range::new(0, 0),
            property_address: Address::zero(),
            params: Bytes::new(),
        }
    }
}
