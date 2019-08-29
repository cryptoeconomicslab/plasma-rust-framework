use super::core::{Integer, Property, PropertyFactory, Quantifier};
use super::witness::PlasmaDataBlock;
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
    right: Property,
}

impl AndDeciderInput {
    pub fn new(left: Property, right: Property) -> Self {
        AndDeciderInput { left, right }
    }
    pub fn get_left(&self) -> &Property {
        &self.left
    }
    pub fn get_right(&self) -> &Property {
        &self.right
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct OrDeciderInput {
    left: Property,
    right: Property,
}

impl OrDeciderInput {
    pub fn new(left: Property, right: Property) -> Self {
        OrDeciderInput { left, right }
    }
    pub fn get_left(&self) -> &Property {
        &self.left
    }
    pub fn get_right(&self) -> &Property {
        &self.right
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct NotDeciderInput {
    property: Property,
}

impl NotDeciderInput {
    pub fn new(property: Property) -> Self {
        NotDeciderInput { property }
    }
    pub fn get_property(&self) -> &Property {
        &self.property
    }
}

#[allow(unused_attributes)]
#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct ForAllSuchThatInput {
    quantifier: Quantifier,
    // PropertyFactory and WitnessFactory isn't serializable. Clients don't send these to smart contract directly
    #[ignore]
    property_factory: Option<PropertyFactory>,
}

impl ForAllSuchThatInput {
    pub fn new(quantifier: Quantifier, property_factory: Option<PropertyFactory>) -> Self {
        ForAllSuchThatInput {
            quantifier,
            property_factory,
        }
    }
    pub fn get_quantifier(&self) -> &Quantifier {
        &self.quantifier
    }
    pub fn get_property_factory(&self) -> &Option<PropertyFactory> {
        &self.property_factory
    }
}

#[derive(Clone, Debug, PartialEq, Eq, AbiDecodable, AbiEncodable)]
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
pub struct IncludedAtBlockInput {
    block_number: Integer,
    plasma_data_block: PlasmaDataBlock,
}

impl IncludedAtBlockInput {
    pub fn new(block_number: Integer, plasma_data_block: PlasmaDataBlock) -> Self {
        Self {
            block_number,
            plasma_data_block,
        }
    }
    pub fn get_block_number(&self) -> Integer {
        self.block_number
    }
    pub fn get_plasma_data_block(&self) -> &PlasmaDataBlock {
        &self.plasma_data_block
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

#[derive(Clone, Debug, PartialEq, Eq, AbiDecodable, AbiEncodable)]
pub struct BlockRangeQuantifierInput {
    pub block_number: Integer,
    pub coin_range: Range,
}

impl BlockRangeQuantifierInput {
    pub fn new(block_number: Integer, coin_range: Range) -> Self {
        Self {
            block_number,
            coin_range,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::ChannelUpdateSignatureExistsDeciderInput;
    use super::IncludedAtBlockInput;
    use crate::types::{Integer, PlasmaDataBlock, PreimageExistsInput, Property};
    use bytes::Bytes;
    use ethereum_types::{Address, H256};
    use plasma_core::data_structure::abi::{Decodable, Encodable};
    use plasma_core::data_structure::Range;

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

    #[test]
    fn test_included_in_interval_tree_at_block_input() {
        let plasma_data_block: PlasmaDataBlock = PlasmaDataBlock::new(
            Integer(0),
            Range::new(500, 700),
            true,
            Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(H256::zero()))),
            Bytes::from(&b"root"[..]),
        );
        let input = IncludedAtBlockInput::new(Integer(10), plasma_data_block);
        let encoded = input.to_abi();
        let decoded = IncludedAtBlockInput::from_abi(&encoded).unwrap();
        assert_eq!(decoded.get_block_number(), input.get_block_number());
    }
}
