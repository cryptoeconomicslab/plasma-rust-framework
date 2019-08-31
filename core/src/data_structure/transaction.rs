extern crate ethereum_types;
extern crate tiny_keccak;

use super::abi::{Decodable, Encodable};
use super::error::{Error, ErrorKind};
use super::Range;
use bytes::Bytes;
use ethabi::Token;
use ethereum_types::Address;
use tiny_keccak::Keccak;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Transaction without signature
pub struct TransactionParams {
    plasma_contract_address: Address,
    range: Range,
    parameters: Bytes,
}

impl TransactionParams {
    pub fn new(plasma_contract_address: Address, range: Range, parameters: Bytes) -> Self {
        TransactionParams {
            plasma_contract_address,
            range,
            parameters,
        }
    }
}

// TODO: use AbiEncodable
impl Encodable for TransactionParams {
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Address(self.plasma_contract_address),
            Token::Tuple(self.range.to_tuple()),
            Token::Bytes(self.parameters.to_vec()),
        ]
    }
}

// TODO: use AbiDecodable
impl Decodable for TransactionParams {
    type Ok = Self;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let plasma_contract_address = tuple[0].clone().to_address();
        let range = tuple[1].clone().to_tuple();
        let parameters = tuple[2].clone().to_bytes();
        if let (Some(plasma_contract_address), Some(range), Some(parameters)) =
            (plasma_contract_address, range, parameters)
        {
            Ok(TransactionParams {
                plasma_contract_address,
                range: Range::from_tuple(&range).ok().unwrap(),
                parameters: Bytes::from(parameters),
            })
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }

    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded: Vec<Token> = ethabi::decode(
            &[
                ethabi::ParamType::Address,
                ethabi::ParamType::Tuple(vec![
                    ethabi::ParamType::Uint(8),
                    ethabi::ParamType::Uint(8),
                ]),
                ethabi::ParamType::Bytes,
            ],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// ## struct Transaction
/// - has a `plasma_contract_address`
/// - has a `start` (A range element)
/// - has a `end` (A range element)
/// - has a `method_id` (like ABI)
/// - has many `parameters`
/// - has a `signature` (for now)
/// - Traits
///   - Encodable
///   - Decodable
pub struct Transaction {
    plasma_contract_address: Address,
    range: Range,
    parameters: Bytes,
    signature: Bytes,
}

impl Transaction {
    /// ### Transaction.new
    /// A constructor of a Transaction struct
    /// ```ignore
    /// let tx = Transaction.new(plasma_contract_address, start, end ,method_id, &parameters, &witness);
    /// ```
    pub fn new(
        plasma_contract_address: Address,
        range: Range,
        parameters: Bytes,
        signature: Bytes,
    ) -> Transaction {
        Transaction {
            plasma_contract_address,
            range,
            parameters,
            signature,
        }
    }

    pub fn from_params(transaction_params: TransactionParams, signature: Bytes) -> Transaction {
        Transaction::new(
            transaction_params.plasma_contract_address,
            transaction_params.range,
            transaction_params.parameters,
            signature,
        )
    }

    /// ### tx.to_body_abi()
    /// A function to convert the transaction instance to the body abi bytes
    /// ```ignore
    /// let body_abi = tx.to_body_abi()
    /// ```
    pub fn to_body_abi(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Address(self.plasma_contract_address),
            Token::Tuple(self.range.to_tuple()),
            Token::Bytes(self.parameters.to_vec()),
        ])
    }
    /// ### Transaction.create_method_id()
    /// A static function to generate method_id bytes from value
    /// ```ignore
    /// let method_id = Transaction.create_method_id(&value)
    /// ```
    pub fn create_method_id(value: &[u8]) -> u8 {
        let mut hasher = Keccak::new_sha3_256();
        hasher.update(value);
        let mut result: [u8; 32] = [0; 32];
        hasher.finalize(&mut result);
        result[0]
    }
    /// ### tx.get_range()
    /// A function to get start of a range of a tx instance
    /// ```ignore
    /// let range = tx.get_range();
    /// ```
    pub fn get_range(&self) -> Range {
        self.range
    }
    pub fn get_parameters(&self) -> &Bytes {
        &self.parameters
    }
    pub fn get_plasma_contract_address(&self) -> Address {
        self.plasma_contract_address
    }
}

impl Encodable for Transaction {
    /// ### tx.to_abi()
    /// A function to convert the transaction instance to the full abi bytes
    /// ```ignore
    /// let abi = tx.to_abi()
    /// ```
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Address(self.plasma_contract_address),
            Token::Tuple(self.range.to_tuple()),
            Token::Bytes(self.parameters.to_vec()),
            Token::Bytes(self.signature.to_vec()),
        ]
    }
}

impl Decodable for Transaction {
    type Ok = Self;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let plasma_contract_address = tuple[0].clone().to_address();
        let range = tuple[1].clone().to_tuple();
        let parameters = tuple[2].clone().to_bytes();
        let signature = tuple[3].clone().to_bytes();
        if let (Some(plasma_contract_address), Some(range), Some(parameters), Some(signature)) =
            (plasma_contract_address, range, parameters, signature)
        {
            Ok(Transaction::new(
                plasma_contract_address,
                Range::from_tuple(&range).ok().unwrap(),
                Bytes::from(parameters),
                Bytes::from(signature),
            ))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    /// ### Transaction.from_abi()
    /// A static function to convert the abi into a tx instance
    /// ```ignore
    /// let tx = Transaction.from_abi(&abi)
    /// ```
    fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded: Vec<Token> = ethabi::decode(
            &[
                ethabi::ParamType::Address,
                ethabi::ParamType::Tuple(vec![
                    ethabi::ParamType::Uint(8),
                    ethabi::ParamType::Uint(8),
                ]),
                ethabi::ParamType::Bytes,
                ethabi::ParamType::Bytes,
            ],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::{Range, Transaction};
    use crate::data_structure::abi::{Decodable, Encodable};
    use bytes::Bytes;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Bytes::from(&b"parameters"[..]);
        let signature_bytes = Bytes::from(&b"signature"[..]);
        let transaction = Transaction::new(
            Address::zero(),
            Range::new(0, 100),
            parameters_bytes,
            signature_bytes,
        );
        let encoded = transaction.to_abi();
        let decoded: Transaction = Transaction::from_abi(&encoded).unwrap();
        assert_eq!(
            decoded.get_range().get_start(),
            transaction.get_range().get_start()
        );
    }
}
