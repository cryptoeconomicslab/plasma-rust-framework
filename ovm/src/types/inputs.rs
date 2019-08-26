use super::core::{InputType, Integer, Property, Quantifier};
use abi_derive::{AbiDecodable, AbiEncodable};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use plasma_core::data_structure::abi::{Decodable, Encodable};

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

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct ForAllSuchThatInput {
    quantifier: Quantifier,
    placeholder: Bytes,
    property: Property,
}

impl ForAllSuchThatInput {
    pub fn new(quantifier: Quantifier, placeholder: Bytes, property: Property) -> Self {
        ForAllSuchThatInput {
            quantifier,
            placeholder,
            property,
        }
    }
    pub fn get_quantifier(&self) -> &Quantifier {
        &self.quantifier
    }
    pub fn get_placeholder(&self) -> &Bytes {
        &self.placeholder
    }

    pub fn get_property(&self) -> &Property {
        &self.property
    }
}

#[derive(Clone, Debug, PartialEq, Eq, AbiDecodable, AbiEncodable)]
pub struct PreimageExistsInput {
    hash: InputType,
}

impl PreimageExistsInput {
    pub fn new(hash: InputType) -> Self {
        PreimageExistsInput { hash }
    }
    pub fn get_hash(&self) -> &InputType {
        &self.hash
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct SignedByInput {
    message: InputType,
    public_key: InputType,
}

impl SignedByInput {
    pub fn new(message: InputType, public_key: InputType) -> Self {
        SignedByInput {
            message,
            public_key,
        }
    }
    pub fn get_message(&self) -> &InputType {
        &self.message
    }
    pub fn get_public_key(&self) -> &InputType {
        &self.public_key
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct IncludedAtBlockInput {
    block_number: InputType,
    plasma_data_block: InputType,
}

impl IncludedAtBlockInput {
    pub fn new(block_number: InputType, plasma_data_block: InputType) -> Self {
        Self {
            block_number,
            plasma_data_block,
        }
    }
    pub fn get_block_number(&self) -> &InputType {
        &self.block_number
    }
    pub fn get_plasma_data_block(&self) -> &InputType {
        &self.plasma_data_block
    }
}

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct HasLowerNonceInput {
    message: InputType,
    nonce: InputType,
}

impl HasLowerNonceInput {
    pub fn new(message: InputType, nonce: InputType) -> Self {
        HasLowerNonceInput { message, nonce }
    }
    pub fn get_message(&self) -> &InputType {
        &self.message
    }
    pub fn get_nonce(&self) -> &InputType {
        &self.nonce
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

#[derive(Clone, Debug, AbiDecodable, AbiEncodable)]
pub struct IntegerRangeQuantifierInput {
    start: InputType,
    end: InputType,
}

impl IntegerRangeQuantifierInput {
    pub fn new(start: InputType, end: InputType) -> Self {
        Self { start, end }
    }
    pub fn get_start(&self) -> &InputType {
        &self.start
    }
    pub fn get_end(&self) -> &InputType {
        &self.end
    }
}

#[derive(Clone, Debug, PartialEq, Eq, AbiDecodable, AbiEncodable)]
pub struct BlockRangeQuantifierInput {
    pub block_number: InputType,
    pub coin_range: InputType,
}

impl BlockRangeQuantifierInput {
    pub fn new(block_number: InputType, coin_range: InputType) -> Self {
        Self {
            block_number,
            coin_range,
        }
    }
}

/*
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
            Bytes::from(&b"root"[..]),
            true,
            Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(InputType::new(
                "hash",
            )))),
        );
        let input = IncludedAtBlockInput::new(Integer(10), plasma_data_block);
        let encoded = input.to_abi();
        let decoded = IncludedAtBlockInput::from_abi(&encoded).unwrap();
        assert_eq!(decoded.get_block_number(), input.get_block_number());
    }
}
*/
