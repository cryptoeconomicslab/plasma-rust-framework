use super::core::{Integer, Property, PropertyFactory, Quantifier, WitnessFactory};
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

/*
impl Encodable for AndDeciderInput {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Bytes(self.left.to_abi()),
            Token::Bytes(self.left_witness.to_vec()),
            Token::Bytes(self.right.to_abi()),
            Token::Bytes(self.right_witness.to_vec()),
        ]
    }
}

impl Decodable for AndDeciderInput {
    type Ok = AndDeciderInput;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let left = tuple[0].clone().to_bytes();
        let left_witness = tuple[1].clone().to_bytes();
        let right = tuple[2].clone().to_bytes();
        let right_witness = tuple[3].clone().to_bytes();
        if let (Some(left), Some(left_witness), Some(right), Some(right_witness)) =
            (left, left_witness, right, right_witness)
        {
            Ok(AndDeciderInput::new(
                Property::from_abi(&left).unwrap(),
                Bytes::from(left_witness),
                Property::from_abi(&right).unwrap(),
                Bytes::from(right_witness),
            ))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded = ethabi::decode(
            &[
                ParamType::Bytes,
                ParamType::Bytes,
                ParamType::Bytes,
                ParamType::Bytes,
            ],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}
*/

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
    witness_factory: WitnessFactory,
}

impl ForAllSuchThatInput {
    pub fn new(
        quantifier: Quantifier,
        property_factory: PropertyFactory,
        witness_factory: WitnessFactory,
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
    pub fn get_witness_factory(&self) -> &WitnessFactory {
        &self.witness_factory
    }
}

/*
impl Encodable for ForAllSuchThatInput {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Bytes(self.quantifier.as_bytes().to_vec()),
            Token::Bytes(self.quantifier_parameters.to_vec()),
        ]
    }
}

impl Decodable for ForAllSuchThatInput {
    type Ok = ForAllSuchThatInput;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let quantifier = tuple[0].clone().to_address();
        let quantifier_parameters = tuple[1].clone().to_bytes();
        if let (Some(quantifier), Some(quantifier_parameters)) = (quantifier, quantifier_parameters)
        {
            Ok(ForAllSuchThatInput::new(
                quantifier,
                Bytes::from(quantifier_parameters),
            ))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded = ethabi::decode(&[ParamType::Address, ParamType::Bytes], data)
            .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}
*/

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

/*
impl Encodable for PreimageExistsInput {
    fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&self.to_tuple())
    }
    fn to_tuple(&self) -> Vec<Token> {
        vec![Token::Bytes(self.hash.as_bytes().to_vec())]
    }
}

impl Decodable for PreimageExistsInput {
    type Ok = PreimageExistsInput;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let hash = tuple[0].clone().to_bytes();
        if let Some(hash) = hash {
            Ok(PreimageExistsInput::new(H256::from_slice(hash.as_ref())))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded = ethabi::decode(&[ParamType::Bytes], data)
            .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}
*/

#[derive(Clone, Debug)]
pub struct SignedByDeciderInput {
    message: Bytes,
    public_key: Address,
}

impl SignedByDeciderInput {
    pub fn new(message: Bytes, public_key: Address) -> Self {
        SignedByDeciderInput {
            message,
            public_key,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChannelUpdateSignatureExistsDeciderInput {
    channel_id: H256,
    nonce: Integer,
    particilant: Address,
}

impl ChannelUpdateSignatureExistsDeciderInput {
    pub fn new(channel_id: H256, nonce: Integer, particilant: Address) -> Self {
        ChannelUpdateSignatureExistsDeciderInput {
            channel_id,
            nonce,
            particilant,
        }
    }
}
