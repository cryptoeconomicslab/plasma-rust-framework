extern crate ethereum_types;
extern crate rlp;

use super::transaction::Transaction;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone)]
pub struct SignedTransaction {
    pub transactions: Vec<Transaction>,
}

impl Encodable for SignedTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_list(self.transactions.as_slice());
    }
}

impl Decodable for SignedTransaction {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        if !rlp.is_list() {
            return Err(DecoderError::Custom("Provided byte data isn't RLP list."));
        }
        let transactions_result: Result<Vec<Transaction>, DecoderError> = rlp.as_list();
        transactions_result.map(|list| SignedTransaction { transactions: list })
    }
}

#[cfg(test)]
mod tests {
    use super::DecoderError;
    use super::SignedTransaction;
    use super::Transaction;
    use crate::data_structure::state_object::StateObject;
    use crate::data_structure::state_update::StateUpdate;
    use bytes::Bytes;
    use ethereum_types::Address;

    #[test]
    fn test_rlp_encode() {
        let parameters_bytes = Bytes::from(&b"parameters"[..]);
        let witness_bytes = Bytes::from(&b"witness"[..]);
        let state_object = StateObject::new(Address::zero(), parameters_bytes);
        let state_update = StateUpdate::new(0, 0, 0, Address::zero(), state_object);
        let transaction = Transaction::new(state_update, witness_bytes);
        let _signed_transaction = SignedTransaction {
            transactions: vec![transaction],
        };
        let encoded = rlp::encode(&_signed_transaction);
        let _decoded: SignedTransaction = rlp::decode(&encoded).unwrap();
        assert_eq!(
            _decoded.transactions.len(),
            _signed_transaction.transactions.len()
        );
    }

    #[test]
    fn fail_to_decode() {
        let animal = "failtodecode";
        let encoded = rlp::encode(&animal);
        let decoded: Result<SignedTransaction, DecoderError> = rlp::decode(&encoded);
        assert_eq!(decoded.is_err(), true);
    }
}
