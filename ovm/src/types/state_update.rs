use crate::property_executor::PropertyExecutor;
use crate::types::{
    core::{Property, QuantifierResultItem},
    PropertyInput,
};
use crate::DecideMixin;
use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::{Decodable, Encodable, Integer};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
use plasma_core::data_structure::{Range, Transaction};
use plasma_db::traits::kvs::KeyValueStore;
use tiny_keccak::Keccak;

#[derive(Clone, Debug, PartialEq, Eq, AbiEncodable, AbiDecodable)]
pub struct StateUpdate {
    block_number: Integer,
    deposit_contract_address: Address,
    range: Range,
    property: Property,
}

impl StateUpdate {
    pub fn new(
        block_number: Integer,
        deposit_contract_address: Address,
        range: Range,
        property: Property,
    ) -> Self {
        Self {
            block_number,
            deposit_contract_address,
            range,
            property,
        }
    }

    pub fn get_deposit_contract_address(&self) -> Address {
        self.deposit_contract_address
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

    pub fn set_start(&mut self, start: u64) {
        self.range = Range::new(start, self.get_range().get_end());
    }

    pub fn set_end(&mut self, end: u64) {
        self.range = Range::new(self.get_range().get_start(), end);
    }

    pub fn is_ownership_state(&self) -> bool {
        self.property.get_type_string()
            == "there_exists_such_that(q_tx(placeholder),bytes,signed_by(address,placeholder))"
    }

    pub fn is_order_state(&self) -> bool {
        self.property.get_type_string() == "there_exists_such_that(q_tx(placeholder),bytes,or(verify_tx(placeholder,address,integer,address),signed_by(address,placeholder)))"
    }

    pub fn is_exchanged_state(&self) -> bool {
        self.property.get_type_string()
            == "there_exists_such_that(q_tx(placeholder),bytes,there_exists_such_that(q_property(integer,address),bytes,there_exists_such_that(q_state_update(placeholder,address,range,placeholder),bytes,or(and(and(for_all_such_that(q_less_than(placeholder),bytes,for_all_such_that(q_block(placeholder,address,range),bytes,is_deprecated(placeholder))),included_at_block(placeholder,placeholder)),signed_by(address,placeholder)),and(not(and(for_all_such_that(q_less_than(placeholder),bytes,for_all_such_that(q_block(placeholder,address,range),bytes,is_deprecated(placeholder))),included_at_block(placeholder,placeholder))),signed_by(address,placeholder))))))"
    }
    pub fn get_owner(&self) -> Address {
        // if property is ownership
        if self.is_ownership_state() {
            return match &self.property.inputs[2] {
                PropertyInput::ConstantProperty(signed_by) => match signed_by.inputs[0] {
                    PropertyInput::ConstantAddress(addr) => addr,
                    _ => panic!("not ownership property."),
                },
                _ => panic!("this is not ownership property."),
            };
        } else if self.is_exchanged_state() {
            let p = &self.get_property().inputs[2];
            if let PropertyInput::ConstantProperty(q_property) = p {
                if let PropertyInput::ConstantProperty(q_su) = &q_property.inputs[2] {
                    if let PropertyInput::ConstantProperty(or) = &q_su.inputs[2] {
                        if let PropertyInput::ConstantProperty(and) = &or.inputs[0] {
                            if let PropertyInput::ConstantProperty(signed_by) = &and.inputs[1] {
                                if let PropertyInput::ConstantAddress(address) = signed_by.inputs[0]
                                {
                                    return address;
                                }
                            }
                        }
                    }
                }
            }
            panic!("Not exchanged property");
        }
        panic!("Not ownership property.");
        // TODO: handle other property.
    }

    pub fn verify_state_transition<T: KeyValueStore>(
        &self,
        decider: &PropertyExecutor<T>,
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

    /// Validates transaction and state update.
    /// Please see https://github.com/cryptoeconomicslab/plasma-rust-framework/issues/241#issuecomment-535820527 for more information.
    pub fn execute_state_transition<T: KeyValueStore>(
        &self,
        decider: &PropertyExecutor<T>,
        transaction: &Transaction,
        next_block_number: Integer,
    ) -> Result<Self, PlasmaCoreError> {
        let next_state = StateUpdate::new(
            next_block_number,
            transaction.get_deposit_contract_address(),
            transaction.get_range(),
            Property::from_abi(transaction.get_parameters()).unwrap(),
        );
        if !self.verify_state_transition(decider, transaction) {
            return Err(PlasmaCoreError::from(
                PlasmaCoreErrorKind::InvalidTransaction,
            ));
        }

        Ok(next_state)
    }
}
