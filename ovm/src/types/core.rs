use super::plasma_data_block::PlasmaDataBlock;
use super::state_update::StateUpdate;
use crate::db::Message;
use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::PropertyInput;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use ethabi::{ParamType, Token};
use ethereum_types::{Address, H256};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
use plasma_core::data_structure::Range;
use plasma_db::traits::kvs::KeyValueStore;

pub type DeciderId = Address;
pub type QuantifierId = Address;
pub trait Input {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub struct Integer(pub u64);

impl Integer {
    pub fn new(n: u64) -> Self {
        Integer(n)
    }
}

impl From<Integer> for Bytes {
    fn from(i: Integer) -> Self {
        let mut buf = BytesMut::with_capacity(64);
        buf.put_u64_le(i.0);
        Bytes::from(buf.to_vec())
    }
}

impl From<Bytes> for Integer {
    fn from(bytes: Bytes) -> Self {
        let mut buf = std::io::Cursor::new(bytes.to_vec());
        Integer(buf.get_u64_le())
    }
}

/// The property which will be decided by Decider
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub decider: Address,
    pub inputs: Vec<PropertyInput>,
}

impl Property {
    pub fn new(decider: Address, inputs: Vec<PropertyInput>) -> Self {
        Self { decider, inputs }
    }
}

impl Encodable for Property {
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Address(self.decider),
            Token::Array(
                self.inputs
                    .iter()
                    .map(|i| Token::Bytes(i.to_abi()))
                    .collect(),
            ),
        ]
    }
}

impl Decodable for Property {
    type Ok = Property;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let decider_id = tuple[0].clone().to_address();
        let inputs = tuple[1].clone().to_array();
        if let (Some(decider_id), Some(inputs)) = (decider_id, inputs) {
            Ok(Property {
                decider: decider_id,
                inputs: inputs
                    .iter()
                    .map(|i| PropertyInput::from_abi(&i.clone().to_bytes().unwrap()).unwrap())
                    .collect(),
            })
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn get_param_types() -> Vec<ParamType> {
        vec![
            ParamType::Address,
            ParamType::Array(Box::new(ParamType::Bytes)),
        ]
    }
}

impl From<Property> for Token {
    fn from(property: Property) -> Token {
        Token::Tuple(property.to_tuple())
    }
}

/// Implication proof element has the property which is decided by Decider
#[derive(Clone, Debug)]
pub struct ImplicationProofElement {
    implication: Property,
    implication_witness: Option<Bytes>,
}

impl ImplicationProofElement {
    pub fn new(implication: Property, implication_witness: Option<Bytes>) -> Self {
        ImplicationProofElement {
            implication,
            implication_witness,
        }
    }
}

impl From<ImplicationProofElement> for Token {
    fn from(element: ImplicationProofElement) -> Token {
        Token::Tuple(vec![
            element.implication.into(),
            Token::Bytes(match element.implication_witness {
                Some(v) => v.to_vec(),
                None => vec![],
            }),
        ])
    }
}

/// Decision made by Decider
#[derive(Clone, Debug)]
pub struct Decision {
    outcome: bool,
    implication_proof: Vec<ImplicationProofElement>,
}

impl Decision {
    pub fn new(outcome: bool, implication_proof: Vec<ImplicationProofElement>) -> Self {
        Decision {
            outcome,
            implication_proof,
        }
    }
    pub fn get_outcome(&self) -> bool {
        self.outcome
    }
    pub fn get_implication_proof(&self) -> &Vec<ImplicationProofElement> {
        &self.implication_proof
    }
}

pub trait Decider {
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error>;
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum QuantifierResultItem {
    Address(Address),
    Integer(Integer),
    Bytes(Bytes),
    Message(Message),
    Property(Property),
    PlasmaDataBlock(PlasmaDataBlock),
    StateUpdate(StateUpdate),
    Range(Range),
    H256(H256),
}

impl QuantifierResultItem {
    pub fn to_bytes(&self) -> Bytes {
        if let QuantifierResultItem::Bytes(bytes) = self {
            bytes.clone()
        } else {
            panic!("QuantifierResultItem isn't Bytes!")
        }
    }
    pub fn to_integer(&self) -> Integer {
        if let QuantifierResultItem::Integer(integer) = self {
            *integer
        } else {
            panic!("QuantifierResultItem isn't Integer!")
        }
    }
    pub fn to_address(&self) -> Address {
        if let QuantifierResultItem::Address(address) = self {
            *address
        } else {
            panic!("QuantifierResultItem isn't Address!")
        }
    }
    pub fn to_h256(&self) -> H256 {
        if let QuantifierResultItem::H256(h256) = self {
            *h256
        } else {
            panic!("QuantifierResultItem isn't H256!")
        }
    }
    pub fn to_range(&self) -> Range {
        if let QuantifierResultItem::Range(range) = self {
            *range
        } else {
            panic!("QuantifierResultItem isn't Range!")
        }
    }
    pub fn to_property(&self) -> Property {
        if let QuantifierResultItem::Property(property) = self {
            property.clone()
        } else {
            panic!("QuantifierResultItem isn't Property!")
        }
    }
    pub fn to_message(&self) -> Message {
        if let QuantifierResultItem::Message(message) = self {
            message.clone()
        } else {
            panic!("QuantifierResultItem isn't Message!")
        }
    }
    pub fn to_plasma_data_block(&self) -> PlasmaDataBlock {
        if let QuantifierResultItem::PlasmaDataBlock(plasma_data_block) = self {
            plasma_data_block.clone()
        } else {
            panic!("QuantifierResultItem isn't PlasmaDataBlock!")
        }
    }
    pub fn to_state_update(&self) -> StateUpdate {
        if let QuantifierResultItem::StateUpdate(state_update) = self {
            state_update.clone()
        } else {
            panic!("QuantifierResultItem isn't StateUpdate!")
        }
    }
}

pub struct QuantifierResult {
    results: Vec<QuantifierResultItem>,
    all_results_quantified: bool,
}

impl QuantifierResult {
    pub fn new(results: Vec<QuantifierResultItem>, all_results_quantified: bool) -> Self {
        QuantifierResult {
            results,
            all_results_quantified,
        }
    }
    pub fn get_results(&self) -> &Vec<QuantifierResultItem> {
        &self.results
    }
    pub fn get_all_results_quantified(&self) -> bool {
        self.all_results_quantified
    }
}

#[cfg(test)]
mod tests {

    use super::Property;
    use crate::types::PropertyInput;
    use crate::DeciderManager;
    use ethereum_types::H256;
    use plasma_core::data_structure::abi::{Decodable, Encodable};

    #[test]
    fn test_encode_and_decode_property() {
        let property =
            DeciderManager::preimage_exists_decider(vec![
                PropertyInput::ConstantH256(H256::zero()),
            ]);
        let encoded = property.to_abi();
        let decoded = Property::from_abi(&encoded).unwrap();
        assert_eq!(decoded, property);
    }
}
