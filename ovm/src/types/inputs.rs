use super::core::{Integer, Property, PropertyFactory, Quantifier, WitnessFactory};
use crate::db::Message;
use bytes::Bytes;
use ethereum_types::{Address, H256};

#[derive(Clone, Debug)]
pub struct AndDeciderInput {
    left: Property,
    left_witness: Bytes,
    right: Property,
    right_witness: Bytes,
}

impl AndDeciderInput {
    pub fn new(left: Property, left_witness: Bytes, right: Property, right_witness: Bytes) -> Self {
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
    pub fn get_left_witness(&self) -> &Bytes {
        &self.left_witness
    }
    pub fn get_right_witness(&self) -> &Bytes {
        &self.right_witness
    }
}

#[derive(Clone, Debug)]
pub struct NotDeciderInput {
    property: Property,
    witness: Bytes,
}

impl NotDeciderInput {
    pub fn new(property: Property, witness: Bytes) -> Self {
        NotDeciderInput { property, witness }
    }
    pub fn get_property(&self) -> &Property {
        &self.property
    }
    pub fn get_witness(&self) -> &Bytes {
        &self.witness
    }
}

#[derive(Clone, Debug)]
pub struct ForAllSuchThatInput {
    quantifier: Quantifier,
    // PropertyFactory and WitnessFactory isn't serializable. Clients don't send these to smart contract directly
    property_factory: PropertyFactory,
    witness_factory: Option<WitnessFactory>,
}

impl ForAllSuchThatInput {
    pub fn new(
        quantifier: Quantifier,
        property_factory: PropertyFactory,
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
    pub fn get_property_factory(&self) -> &PropertyFactory {
        &self.property_factory
    }
    pub fn get_witness_factory(&self) -> &Option<WitnessFactory> {
        &self.witness_factory
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
