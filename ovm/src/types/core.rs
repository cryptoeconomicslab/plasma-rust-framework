use super::inputs::{
    AndDeciderInput, BlockRangeQuantifierInput, ChannelUpdateSignatureExistsDeciderInput,
    ForAllSuchThatInput, HasLowerNonceInput, IncludedInIntervalTreeAtBlockInput,
    IntegerRangeQuantifierInput, NotDeciderInput, OrDeciderInput, PreimageExistsInput,
    SignedByInput,
};
use super::witness::Witness;
use crate::db::Message;
use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
use plasma_db::traits::kvs::KeyValueStore;
use std::sync::Arc;

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
    IncludedInIntervalTreeAtBlockDecider(IncludedInIntervalTreeAtBlockInput),
}

#[derive(Clone, Debug)]
pub enum Quantifier {
    // start to end
    IntegerRangeQuantifier(IntegerRangeQuantifierInput),
    // 0 to upperBound
    NonnegativeIntegerLessThanQuantifier(Integer),
    // signer
    SignedByQuantifier(Address),
    // blocknumber and range
    BlockRangeQuantifier(BlockRangeQuantifierInput),
}

impl Property {
    pub fn get_decider_id(&self) -> DeciderId {
        match self {
            Property::AndDecider(_) => DECIDER_LIST[0],
            Property::OrDecider(_) => DECIDER_LIST[1],
            _ => Address::zero(),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Property::AndDecider(input) => input.to_abi(),
            Property::OrDecider(input) => input.to_abi(),
            _ => panic!("unknown decider"),
        }
    }
    fn from_bytes(decider_id: Address, data: &[u8]) -> Result<Self, PlasmaCoreError> {
        if decider_id == DECIDER_LIST[0] {
            AndDeciderInput::from_abi(data).map(|input| Property::AndDecider(Box::new(input)))
        } else if decider_id == DECIDER_LIST[1] {
            OrDeciderInput::from_abi(data).map(|input| Property::OrDecider(Box::new(input)))
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
    fn from_abi(data: &[u8]) -> Result<Self, PlasmaCoreError> {
        let decoded = ethabi::decode(&[ParamType::Address, ParamType::Bytes], data)
            .map_err(|_e| PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

impl Quantifier {
    pub fn get_id(&self) -> u64 {
        match self {
            Quantifier::IntegerRangeQuantifier(_) => 0,
            Quantifier::NonnegativeIntegerLessThanQuantifier(_) => 1,
            Quantifier::SignedByQuantifier(_) => 2,
            Quantifier::BlockRangeQuantifier(_) => 3,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Quantifier::IntegerRangeQuantifier(input) => input.to_abi(),
            Quantifier::NonnegativeIntegerLessThanQuantifier(input) => Bytes::from(*input).to_vec(),
            Quantifier::SignedByQuantifier(input) => input.as_bytes().to_vec(),
            Quantifier::BlockRangeQuantifier(input) => input.to_abi(),
        }
    }
    fn from_bytes(id: u64, data: &[u8]) -> Result<Self, PlasmaCoreError> {
        if id == 0 {
            IntegerRangeQuantifierInput::from_abi(data).map(Quantifier::IntegerRangeQuantifier)
        } else if id == 1 {
            Ok(Quantifier::NonnegativeIntegerLessThanQuantifier(
                Integer::from(Bytes::from(data)),
            ))
        } else if id == 2 {
            Ok(Quantifier::SignedByQuantifier(Address::from_slice(data)))
        } else if id == 3 {
            BlockRangeQuantifierInput::from_abi(data).map(Quantifier::BlockRangeQuantifier)
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
    fn from_abi(data: &[u8]) -> Result<Self, PlasmaCoreError> {
        let decoded = ethabi::decode(&[ParamType::Uint(256), ParamType::Bytes], data)
            .map_err(|_e| PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
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

#[derive(Clone)]
pub struct PropertyFactory(Arc<dyn Fn(QuantifierResultItem) -> Property>);

impl PropertyFactory {
    pub fn new(handler: Box<dyn Fn(QuantifierResultItem) -> Property>) -> Self {
        PropertyFactory(Arc::new(handler))
    }
    pub fn call(&self, item: QuantifierResultItem) -> Property {
        self.0(item)
    }
}

impl std::fmt::Debug for PropertyFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PropertyFactory")
    }
}

#[derive(Clone)]
pub struct WitnessFactory(Arc<dyn Fn(QuantifierResultItem) -> Witness>);

impl WitnessFactory {
    pub fn new(handler: Box<dyn Fn(QuantifierResultItem) -> Witness>) -> Self {
        WitnessFactory(Arc::new(handler))
    }
    pub fn call(&self, item: QuantifierResultItem) -> Witness {
        self.0(item)
    }
}

impl std::fmt::Debug for WitnessFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WitnessFactory")
    }
}

pub trait Decider {
    type Input;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &Self::Input,
        witness: Option<Witness>,
    ) -> Result<Decision, Error>;
    fn check_decision<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &Self::Input,
    ) -> Result<Decision, Error>;
}

#[derive(Clone, Debug)]
pub enum QuantifierResultItem {
    Integer(Integer),
    Bytes(Bytes),
    Message(Message),
    Property(Property),
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
