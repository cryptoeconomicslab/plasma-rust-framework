extern crate crypto;
extern crate ethereum_types;
extern crate rlp;

use crypto::digest::Digest;
use crypto::sha3::Sha3;
use ethabi::Token;
use ethereum_types::Address;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Debug)]
pub enum Error {
    VerifyError,
}

#[derive(Clone, Debug)]
pub struct Transaction {
    plasma_contract_address: Address,
    block: u64,
    start: u64,
    end: u64,
    method_id: Vec<u8>,
    parameters: Vec<u8>,
}

impl Transaction {
    pub fn new(
        plasma_contract_address: Address,
        block: u64,
        start: u64,
        end: u64,
        method_id: &[u8],
        parameters: &[u8],
    ) -> Transaction {
        Transaction {
            plasma_contract_address,
            block,
            start,
            end,
            method_id: method_id.to_vec(),
            parameters: parameters.to_vec(),
        }
    }
    pub fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Address(self.plasma_contract_address),
            Token::Uint(self.block.into()),
            Token::Uint(self.start.into()),
            Token::Uint(self.end.into()),
            Token::FixedBytes(self.method_id.clone()),
            Token::Bytes(self.parameters.clone()),
        ])
    }
    pub fn from_abi(data: &[u8]) -> Result<Self, ethabi::Error> {
        let decoded = ethabi::decode(
            &[
                ethabi::ParamType::Address,
                ethabi::ParamType::Uint(32),
                ethabi::ParamType::Uint(32),
                ethabi::ParamType::Uint(32),
                ethabi::ParamType::FixedBytes(32),
                ethabi::ParamType::Bytes,
            ],
            data,
        )?;
        Ok(Transaction::new(
            decoded[0].clone().to_address().unwrap(),
            decoded[1].clone().to_uint().unwrap().as_u64(),
            decoded[2].clone().to_uint().unwrap().as_u64(),
            decoded[3].clone().to_uint().unwrap().as_u64(),
            &decoded[4].clone().to_fixed_bytes().unwrap(),
            &decoded[5].clone().to_bytes().unwrap(),
        ))
    }
    pub fn create_method_id(value: &[u8]) -> Vec<u8> {
        let mut hasher = Sha3::keccak256();
        let mut result = vec![0u8; hasher.output_bits() / 8];
        hasher.reset();
        hasher.input(value);
        hasher.result(result.as_mut_slice());
        result.clone()
    }
}

impl Encodable for Transaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(1);
        s.append(&self.to_abi());
    }
}

impl Decodable for Transaction {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let abi: Vec<u8> = rlp.list_at(0)?;
        Ok(Transaction::from_abi(&abi).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::Transaction;
    use ethereum_types::Address;

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Vec::from(&b"parameters"[..]);
        let transaction = Transaction::new(
            Address::zero(),
            0,
            0,
            100,
            &Transaction::create_method_id(&b"send(address)"[..]),
            &parameters_bytes,
        );
        let encoded = transaction.to_abi();
        let decoded: Transaction = Transaction::from_abi(&encoded).unwrap();
        assert_eq!(decoded.block, transaction.block);
    }

}
