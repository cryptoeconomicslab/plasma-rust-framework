extern crate ethereum_types;
extern crate tiny_keccak;

use super::abi::{Decodable, Encodable};
use super::error::{Error, ErrorKind};
use super::Range;
use bytes::Bytes;
use ethabi::Token;
use ethereum_types::{Address, H256};
use tiny_keccak::Keccak;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Witness {
    v: H256,
    r: H256,
    s: u64,
}

impl Witness {
    pub fn new(v: H256, r: H256, s: u64) -> Self {
        Witness { v, r, s }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// ## struct Transaction
/// - has a `plasma_contract_address`
/// - has a `start` (A range element)
/// - has a `end` (A range element)
/// - has a `method_id` (like ABI)
/// - has many `parameters`
/// - has a `witness` (signature, proof or some)
/// - Traits
///   - Encodable
///   - Decodable
pub struct Transaction {
    plasma_contract_address: Address,
    range: Range,
    parameters: Bytes,
    witness: Witness,
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
        witness: &Witness,
    ) -> Transaction {
        Transaction {
            plasma_contract_address,
            range,
            parameters,
            witness: witness.clone(),
        }
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
    pub fn get_range(&self) -> &Range {
        &self.range
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
            Token::FixedBytes(self.witness.v.as_bytes().to_vec()),
            Token::FixedBytes(self.witness.r.as_bytes().to_vec()),
            Token::Uint(self.witness.s.into()),
        ]
    }
}

impl Decodable for Transaction {
    type Ok = Self;
    fn from_tuple(tuple: &[Token]) -> Result<Self, Error> {
        let plasma_contract = tuple[0].clone().to_address();
        let range = tuple[1].clone().to_tuple();
        let parameters = tuple[2].clone().to_bytes();
        let v = tuple[3].clone().to_fixed_bytes();
        let r = tuple[4].clone().to_fixed_bytes();
        let s = tuple[5].clone().to_uint();
        if let (Some(plasma_contract), Some(range), Some(parameters), Some(v), Some(r), Some(s)) =
            (plasma_contract, range, parameters, v, r, s)
        {
            Ok(Transaction::new(
                plasma_contract,
                Range::from_tuple(&range).ok().unwrap(),
                Bytes::from(parameters),
                &Witness::new(H256::from_slice(&v), H256::from_slice(&r), s.as_u64()),
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
                ethabi::ParamType::FixedBytes(32),
                ethabi::ParamType::FixedBytes(32),
                ethabi::ParamType::Uint(1),
            ],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        Self::from_tuple(&decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::{Range, Transaction, Witness};
    use crate::data_structure::abi::{Decodable, Encodable};
    use bytes::Bytes;
    use ethereum_types::{Address, H256};

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Bytes::from(&b"parameters"[..]);
        let transaction = Transaction::new(
            Address::zero(),
            Range::new(0, 100),
            parameters_bytes,
            &Witness::new(H256::zero(), H256::zero(), 0),
        );
        let encoded = transaction.to_abi();
        let decoded: Transaction = Transaction::from_abi(&encoded).unwrap();
        assert_eq!(
            decoded.get_range().get_start(),
            transaction.get_range().get_start()
        );
    }

}
