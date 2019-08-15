use super::core::{Integer, Property, PropertyFactory, Quantifier, WitnessFactory};
use super::witness::Witness;
use crate::db::Message;
use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::{Address, H256};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::Range;

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct AndDeciderInput {
    left: Property,
    left_witness: Witness,
    right: Property,
    right_witness: Witness,
}

impl AndDeciderInput {
    pub fn new(
        left: Property,
        left_witness: Witness,
        right: Property,
        right_witness: Witness,
    ) -> Self {
        AndDeciderInput {
            left,
            left_witness,
            right,
            right_witness,
        }
    }
    pub fn get_left(&self) -> &Property {
        &self.left
    }
    pub fn get_right(&self) -> &Property {
        &self.right
    }
    pub fn get_left_witness(&self) -> &Witness {
        &self.left_witness
    }
    pub fn get_right_witness(&self) -> &Witness {
        &self.right_witness
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct OrDeciderInput {
    left: Property,
    left_witness: Witness,
    right: Property,
    right_witness: Witness,
}

impl OrDeciderInput {
    pub fn new(
        left: Property,
        left_witness: Witness,
        right: Property,
        right_witness: Witness,
    ) -> Self {
        OrDeciderInput {
            left,
            left_witness,
            right,
            right_witness,
        }
    }
    pub fn get_left(&self) -> &Property {
        &self.left
    }
    pub fn get_right(&self) -> &Property {
        &self.right
    }
    pub fn get_left_witness(&self) -> &Witness {
        &self.left_witness
    }
    pub fn get_right_witness(&self) -> &Witness {
        &self.right_witness
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct NotDeciderInput {
    property: Property,
    witness: Witness,
}

impl NotDeciderInput {
    pub fn new(property: Property, witness: Witness) -> Self {
        NotDeciderInput { property, witness }
    }
    pub fn get_property(&self) -> &Property {
        &self.property
    }
    pub fn get_witness(&self) -> &Witness {
        &self.witness
    }
}

#[allow(unused_attributes)]
#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct ForAllSuchThatInput {
    quantifier: Quantifier,
    // PropertyFactory and WitnessFactory isn't serializable. Clients don't send these to smart contract directly
    #[ignore]
    property_factory: Option<PropertyFactory>,
    #[ignore]
    witness_factory: Option<WitnessFactory>,
}

impl ForAllSuchThatInput {
    pub fn new(
        quantifier: Quantifier,
        property_factory: Option<PropertyFactory>,
        witness_factory: Option<WitnessFactory>,
    ) -> Self {
        ForAllSuchThatInput {
            quantifier,
            property_factory,
            witness_factory,
        }
    }
    pub fn get_quantifier(&self) -> &Quantifier {
        &self.quantifier
    }
    pub fn get_property_factory(&self) -> &Option<PropertyFactory> {
        &self.property_factory
    }
    pub fn get_witness_factory(&self) -> &Option<WitnessFactory> {
        &self.witness_factory
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct PreimageExistsInput {
    hash: H256,
}

impl PreimageExistsInput {
    pub fn new(hash: H256) -> Self {
        PreimageExistsInput { hash }
    }
    pub fn get_hash(&self) -> H256 {
        self.hash
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct SignedByInput {
    message: Bytes,
    public_key: Address,
}

impl SignedByInput {
    pub fn new(message: Bytes, public_key: Address) -> Self {
        SignedByInput {
            message,
            public_key,
        }
    }
    pub fn get_message(&self) -> &Bytes {
        &self.message
    }
    pub fn get_public_key(&self) -> Address {
        self.public_key
    }
    pub fn hash(&self) -> &Bytes {
        &self.message
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct IncludedInIntervalTreeAtBlockInput {
    block_number: Integer,
    coin_range: Range,
}

impl IncludedInIntervalTreeAtBlockInput {
    pub fn new(block_number: Integer, coin_range: Range) -> Self {
        Self {
            block_number,
            coin_range,
        }
    }
    pub fn get_block_number(&self) -> Integer {
        self.block_number
    }
    pub fn get_coin_range(&self) -> Range {
        self.coin_range
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct HasLowerNonceInput {
    message: Message,
    nonce: Integer,
}

impl HasLowerNonceInput {
    pub fn new(message: Message, nonce: Integer) -> Self {
        HasLowerNonceInput { message, nonce }
    }
    pub fn get_message(&self) -> &Message {
        &self.message
    }
    pub fn get_nonce(&self) -> Integer {
        self.nonce
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct ChannelUpdateSignatureExistsDeciderInput {
    pub channel_id: Bytes,
    pub nonce: Integer,
    pub particilant: Address,
}

impl ChannelUpdateSignatureExistsDeciderInput {
    pub fn new(channel_id: Bytes, nonce: Integer, particilant: Address) -> Self {
        ChannelUpdateSignatureExistsDeciderInput {
            channel_id,
            nonce,
            particilant,
        }
    }
}

pub type IntegerRangeQuantifierInput = Range;
pub type BlockRangeQuantifierInput = IncludedInIntervalTreeAtBlockInput;

#[cfg(test)]
mod tests {

    use super::ChannelUpdateSignatureExistsDeciderInput;
    use crate::types::Integer;
    use bytes::Bytes;
    use ethereum_types::Address;
    use plasma_core::data_structure::abi::{Decodable, Encodable};

    #[test]
    fn test_channel_update_signature_exists_decider_input() {
        let input = ChannelUpdateSignatureExistsDeciderInput::new(
            Bytes::from(&b"parameters"[..]),
            Integer(10),
            Address::zero(),
        );
        let encoded = input.to_abi();
        let decoded = ChannelUpdateSignatureExistsDeciderInput::from_abi(&encoded).unwrap();
        assert_eq!(decoded.channel_id, input.channel_id);
    }

}
