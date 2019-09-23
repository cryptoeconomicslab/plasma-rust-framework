extern crate ethereum_types;
extern crate tiny_keccak;

use super::{Metadata, Range};
use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::abi::Encodable;
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use tiny_keccak::Keccak;

#[derive(Clone, Debug, PartialEq, Eq, AbiEncodable, AbiDecodable)]
/// Transaction without signature
pub struct TransactionParams {
    deposit_contract_address: Address,
    range: Range,
    parameters: Bytes,
}

impl TransactionParams {
    pub fn new(deposit_contract_address: Address, range: Range, parameters: Bytes) -> Self {
        TransactionParams {
            deposit_contract_address,
            range,
            parameters,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, AbiEncodable, AbiDecodable)]
/// ## struct Transaction
/// - has a `deposit_contract_address`
/// - has a `start` (A range element)
/// - has a `end` (A range element)
/// - has a `method_id` (like ABI)
/// - has many `parameters`
/// - has a `signature` (for now)
/// - has a `metadata`
/// - Traits
///   - Encodable
///   - Decodable
pub struct Transaction {
    deposit_contract_address: Address,
    range: Range,
    parameters: Bytes,
    signature: Bytes,
    metadata: Metadata,
}

impl Transaction {
    /// ### Transaction.new
    /// A constructor of a Transaction struct
    /// ```ignore
    /// let tx = Transaction.new(deposit_contract_address, start, end ,method_id, &parameters, &witness);
    /// ```
    pub fn new(
        deposit_contract_address: Address,
        range: Range,
        parameters: Bytes,
        signature: Bytes,
        metadata: Metadata,
    ) -> Transaction {
        Transaction {
            deposit_contract_address,
            range,
            parameters,
            signature,
            metadata,
        }
    }

    pub fn from_params(
        transaction_params: TransactionParams,
        signature: Bytes,
        metadata: Metadata,
    ) -> Transaction {
        Transaction::new(
            transaction_params.deposit_contract_address,
            transaction_params.range,
            transaction_params.parameters,
            signature,
            metadata,
        )
    }

    /// ### tx.to_body_abi()
    /// A function to convert the transaction instance to the body abi bytes
    /// ```ignore
    /// let body_abi = tx.to_body_abi()
    /// ```
    pub fn to_body_abi(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Address(self.deposit_contract_address),
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
    pub fn get_deposit_contract_address(&self) -> Address {
        self.deposit_contract_address
    }
    pub fn get_signature(&self) -> &Bytes {
        &self.signature
    }
    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::{Metadata, Range, Transaction};
    use abi_utils::{Decodable, Encodable};
    use bytes::Bytes;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Bytes::from(&b"parameters"[..]);
        let signature_bytes = Bytes::from(&b"signature"[..]);
        let metadata = Metadata::default();
        let transaction = Transaction::new(
            Address::zero(),
            Range::new(0, 100),
            parameters_bytes,
            signature_bytes,
            metadata,
        );
        let encoded = transaction.to_abi();
        let decoded: Transaction = Transaction::from_abi(&encoded).unwrap();
        assert_eq!(
            decoded.get_range().get_start(),
            transaction.get_range().get_start()
        );
    }
}
