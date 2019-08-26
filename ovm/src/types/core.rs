use super::inputs::{
    AndDeciderInput, BlockRangeQuantifierInput, ChannelUpdateSignatureExistsDeciderInput,
    ForAllSuchThatInput, HasLowerNonceInput, IncludedAtBlockInput, IntegerRangeQuantifierInput,
    NotDeciderInput, OrDeciderInput, PreimageExistsInput, SignedByInput,
};
use super::witness::{PlasmaDataBlock, Witness};
use crate::db::Message;
use crate::error::Error;
use crate::property_executor::PropertyExecutor;
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

lazy_static! {
    static ref DECIDER_LIST: Vec<Address> = {
        let mut list = vec![];
        for _ in 0..10 {
            list.push(Address::random())
        }
        list
    };
}

/// The property which will be decided by Decider
#[derive(Clone, Debug)]
pub enum Property {
    // left, left_witness, right, right_witness
    AndDecider(Box<AndDeciderInput>),
    // property, witness
    NotDecider(Box<NotDeciderInput>),
    // quantifier, quantifier_parameters, property_factory, witness_factory?
    ForAllSuchThatDecider(Box<ForAllSuchThatInput>),
    // hash
    PreimageExistsDecider(Box<PreimageExistsInput>),
    // message, public_key
    SignedByDecider(SignedByInput),
    // left, right
    OrDecider(Box<OrDeciderInput>),
    // message, nonce
    HasLowerNonceDecider(HasLowerNonceInput),
    // channelId, nonce, participant
    ChannelUpdateSignatureExistsDecider(ChannelUpdateSignatureExistsDeciderInput),
    IncludedAtBlockDecider(Box<IncludedAtBlockInput>),
}

#[derive(Clone, Debug)]
pub enum Quantifier {
    HashQuantifier(InputType),
    // start to end
    IntegerRangeQuantifier(IntegerRangeQuantifierInput),
    // 0 to upperBound
    NonnegativeIntegerLessThanQuantifier(InputType),
    // signer
    SignedByQuantifier(InputType),
    // blocknumber and range
    BlockRangeQuantifier(BlockRangeQuantifierInput),
}

impl Property {
    pub fn get_decider_id(&self) -> DeciderId {
        match self {
            Property::AndDecider(_) => DECIDER_LIST[0],
            Property::OrDecider(_) => DECIDER_LIST[1],
            Property::ForAllSuchThatDecider(_) => DECIDER_LIST[2],
            Property::NotDecider(_) => DECIDER_LIST[3],
            Property::PreimageExistsDecider(_) => DECIDER_LIST[4],
            Property::SignedByDecider(_) => DECIDER_LIST[5],
            Property::HasLowerNonceDecider(_) => DECIDER_LIST[6],
            Property::ChannelUpdateSignatureExistsDecider(_) => DECIDER_LIST[7],
            Property::IncludedAtBlockDecider(_) => DECIDER_LIST[8],
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Property::AndDecider(input) => input.to_abi(),
            Property::OrDecider(input) => input.to_abi(),
            Property::ForAllSuchThatDecider(input) => input.to_abi(),
            Property::NotDecider(input) => input.to_abi(),
            Property::PreimageExistsDecider(input) => input.to_abi(),
            Property::SignedByDecider(input) => input.to_abi(),
            Property::HasLowerNonceDecider(input) => input.to_abi(),
            Property::ChannelUpdateSignatureExistsDecider(input) => input.to_abi(),
            Property::IncludedAtBlockDecider(input) => input.to_abi(),
        }
    }
    fn from_bytes(decider_id: Address, data: &[u8]) -> Result<Self, PlasmaCoreError> {
        if decider_id == DECIDER_LIST[0] {
            AndDeciderInput::from_abi(data).map(|input| Property::AndDecider(Box::new(input)))
        } else if decider_id == DECIDER_LIST[1] {
            OrDeciderInput::from_abi(data).map(|input| Property::OrDecider(Box::new(input)))
        } else if decider_id == DECIDER_LIST[2] {
            ForAllSuchThatInput::from_abi(data)
                .map(|input| Property::ForAllSuchThatDecider(Box::new(input)))
        } else if decider_id == DECIDER_LIST[3] {
            NotDeciderInput::from_abi(data).map(|input| Property::NotDecider(Box::new(input)))
        } else if decider_id == DECIDER_LIST[4] {
            PreimageExistsInput::from_abi(data)
                .map(|input| Property::PreimageExistsDecider(Box::new(input)))
        } else if decider_id == DECIDER_LIST[5] {
            SignedByInput::from_abi(data).map(Property::SignedByDecider)
        } else if decider_id == DECIDER_LIST[6] {
            HasLowerNonceInput::from_abi(data).map(Property::HasLowerNonceDecider)
        } else if decider_id == DECIDER_LIST[7] {
            ChannelUpdateSignatureExistsDeciderInput::from_abi(data)
                .map(Property::ChannelUpdateSignatureExistsDecider)
        } else if decider_id == DECIDER_LIST[8] {
            IncludedAtBlockInput::from_abi(data)
                .map(|input| Property::IncludedAtBlockDecider(Box::new(input)))
        } else {
            panic!("unknown decider")
        }
    }
}

impl Encodable for Property {
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Address(self.get_decider_id()),
            Token::Bytes(self.to_bytes()),
        ]
    }
}

impl Decodable for Property {
    type Ok = Property;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let decider_id = tuple[0].clone().to_address();
        let input_data = tuple[1].clone().to_bytes();
        if let (Some(decider_id), Some(input_data)) = (decider_id, input_data) {
            Ok(Property::from_bytes(decider_id, &input_data).unwrap())
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn get_param_types() -> Vec<ParamType> {
        vec![ParamType::Address, ParamType::Bytes]
    }
}

impl From<Property> for Token {
    fn from(property: Property) -> Token {
        Token::Tuple(property.to_tuple())
    }
}

impl Quantifier {
    pub fn get_id(&self) -> u64 {
        match self {
            Quantifier::HashQuantifier(_) => 4,
            Quantifier::IntegerRangeQuantifier(_) => 0,
            Quantifier::NonnegativeIntegerLessThanQuantifier(_) => 1,
            Quantifier::SignedByQuantifier(_) => 2,
            Quantifier::BlockRangeQuantifier(_) => 3,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Quantifier::HashQuantifier(input) => input.to_abi(),
            Quantifier::IntegerRangeQuantifier(input) => input.to_abi(),
            Quantifier::NonnegativeIntegerLessThanQuantifier(input) => input.to_abi(),
            Quantifier::SignedByQuantifier(input) => input.to_abi(),
            Quantifier::BlockRangeQuantifier(input) => input.to_abi(),
        }
    }
    fn from_bytes(id: u64, data: &[u8]) -> Result<Self, PlasmaCoreError> {
        if id == 0 {
            IntegerRangeQuantifierInput::from_abi(data).map(Quantifier::IntegerRangeQuantifier)
        } else if id == 1 {
            InputType::from_abi(data).map(Quantifier::NonnegativeIntegerLessThanQuantifier)
        } else if id == 2 {
            InputType::from_abi(data).map(Quantifier::SignedByQuantifier)
        } else if id == 3 {
            BlockRangeQuantifierInput::from_abi(data).map(Quantifier::BlockRangeQuantifier)
        } else if id == 4 {
            InputType::from_abi(data).map(Quantifier::HashQuantifier)
        } else {
            panic!("unknown decider")
        }
    }
}

impl Encodable for Quantifier {
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Uint(self.get_id().into()),
            Token::Bytes(self.to_bytes()),
        ]
    }
}

impl Decodable for Quantifier {
    type Ok = Quantifier;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let id = tuple[0].clone().to_uint();
        let input_data = tuple[1].clone().to_bytes();
        if let (Some(id), Some(input_data)) = (id, input_data) {
            Ok(Quantifier::from_bytes(id.as_u64(), &input_data).unwrap())
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn get_param_types() -> Vec<ethabi::ParamType> {
        vec![ethabi::ParamType::Uint(256), ethabi::ParamType::Bytes]
    }
}

/// Implication proof element has the property which is decided by Decider
#[derive(Clone, Debug)]
pub struct ImplicationProofElement {
    implication: Property,
    implication_witness: Option<Witness>,
}

impl ImplicationProofElement {
    pub fn new(implication: Property, implication_witness: Option<Witness>) -> Self {
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
                Some(v) => v.to_abi(),
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Placeholder(Bytes);
impl Placeholder {
    pub fn new(placeholder: &str) -> Self {
        Self(Bytes::from(placeholder))
    }
}

impl Encodable for Placeholder {
    fn to_tuple(&self) -> Vec<Token> {
        vec![Token::Bytes(self.0.to_vec())]
    }
}

impl Decodable for Placeholder {
    type Ok = Placeholder;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let bytes = tuple[0].clone().to_bytes();
        if let Some(bytes) = bytes {
            Ok(Placeholder(Bytes::from(bytes)))
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn get_param_types() -> Vec<ethabi::ParamType> {
        vec![ethabi::ParamType::Bytes]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputType {
    Placeholder(Bytes),
    ConstantAddress(Address),
    ConstantBytes(Bytes),
    ConstantH256(H256),
    ConstantInteger(Integer),
    ConstantRange(Range),
}

impl InputType {
    pub fn placeholder(placeholder: &str) -> Self {
        InputType::Placeholder(Bytes::from(placeholder))
    }
}

impl Encodable for InputType {
    fn to_tuple(&self) -> Vec<Token> {
        match self {
            InputType::Placeholder(placeholder) => {
                vec![Token::Uint(0.into()), Token::Bytes(placeholder.to_vec())]
            }
            InputType::ConstantAddress(address) => vec![
                Token::Uint(1.into()),
                Token::Bytes(address.as_bytes().to_vec()),
            ],
            InputType::ConstantBytes(bytes) => {
                vec![Token::Uint(2.into()), Token::Bytes(bytes.to_vec())]
            }
            InputType::ConstantH256(h256) => vec![
                Token::Uint(3.into()),
                Token::Bytes(h256.as_bytes().to_vec()),
            ],
            InputType::ConstantInteger(integer) => {
                let b: Bytes = (*integer).into();
                vec![Token::Uint(4.into()), Token::Bytes(b.to_vec())]
            }
            InputType::ConstantRange(range) => {
                vec![Token::Uint(5.into()), Token::Bytes(range.to_abi())]
            }
        }
    }
}

impl Decodable for InputType {
    type Ok = InputType;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let id = tuple[0].clone().to_uint();
        let bytes = tuple[1].clone().to_bytes();
        if let (Some(id), Some(bytes)) = (id, bytes) {
            let id_num = id.as_u64();
            if id_num == 0 {
                Ok(InputType::Placeholder(Bytes::from(bytes)))
            } else if id_num == 1 {
                Ok(InputType::ConstantAddress(Address::from_slice(&bytes)))
            } else if id_num == 2 {
                Ok(InputType::ConstantBytes(Bytes::from(bytes)))
            } else if id_num == 3 {
                Ok(InputType::ConstantH256(H256::from_slice(&bytes)))
            } else if id_num == 3 {
                Ok(InputType::ConstantInteger(Bytes::from(bytes).into()))
            } else {
                Range::from_abi(&bytes).map(|item| InputType::ConstantRange(item))
            }
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }
    fn get_param_types() -> Vec<ParamType> {
        vec![ParamType::Uint(256), ParamType::Bytes]
    }
}

pub trait Decider {
    type Input;
    fn decide<T: KeyValueStore>(
        decider: &mut PropertyExecutor<T>,
        input: &Self::Input,
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
    Range(Range),
    H256(H256),
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
    use crate::types::{InputType, PreimageExistsInput};
    use plasma_core::data_structure::abi::{Decodable, Encodable};

    #[test]
    fn test_encode_and_decode_property() {
        let property = Property::PreimageExistsDecider(Box::new(PreimageExistsInput::new(
            InputType::placeholder("hash"),
        )));
        let encoded = property.to_abi();
        let decoded = Property::from_abi(&encoded).unwrap();
        if let Property::PreimageExistsDecider(input) = decoded {
            assert_eq!(
                input,
                Box::new(PreimageExistsInput::new(InputType::placeholder("hash")))
            );
        } else {
            panic!()
        }
    }
}
