use crate::db::SignedByDb;
use crate::error::{Error, ErrorKind};
use crate::property_executor::PropertyExecutor;
use crate::types::{
    Decider, Decision, DecisionValue, ImplicationProofElement, Property, SignedByInput, Witness,
};
use bytes::Bytes;
use ethereum_types::{Address, H256};
use ethsign::{SecretKey, Signature};
use plasma_core::data_structure::abi::Decodable;
use plasma_db::traits::kvs::{BaseDbKey, KeyValueStore};
use tiny_keccak::Keccak;

pub fn signature_to_bytes(signature: &Signature) -> Bytes {
    let mut bytes = vec![signature.v];
    bytes.extend([signature.r, signature.s].concat());
    bytes.to_vec().into()
}

pub fn bytes_to_signature(bytes: &Bytes) -> Signature {
    let buf = bytes.to_vec();
    let mut r = [0u8; 32];
    let mut s = [0u8; 32];
    let v = buf[0];
    r.copy_from_slice(&buf[1..33]);
    s.copy_from_slice(&buf[33..65]);
    Signature { v, r, s }
}

pub fn hash(preimage: &Bytes) -> H256 {
    let mut sha3 = Keccak::new_sha3_256();

    sha3.update(preimage.as_ref());

    let mut res: [u8; 32] = [0; 32];
    sha3.finalize(&mut res);
    H256::from(res)
}

pub struct Verifier {}

impl Verifier {
    pub fn recover(sig_bytes: &Bytes, message: &Bytes) -> Address {
        let signature: Signature = bytes_to_signature(sig_bytes);
        signature
            .recover(hash(message).as_bytes())
            .unwrap()
            .address()
            .into()
    }
    pub fn sign(key: &SecretKey, message: &Bytes) -> Bytes {
        signature_to_bytes(&key.sign(hash(message).as_bytes()).unwrap())
    }
}

pub struct SignedByDecider {}

impl Default for SignedByDecider {
    fn default() -> Self {
        SignedByDecider {}
    }
}

impl Decider for SignedByDecider {
    type Input = SignedByInput;
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &SignedByInput,
        _witness: Option<Witness>,
    ) -> Result<Decision, Error> {
        let db: SignedByDb<T> = SignedByDb::new(decider.get_db());
        let witness = db.get_witness(input)?;
        if let Witness::Bytes(signature) = witness {
            if Verifier::recover(&signature, input.get_message()) != input.get_public_key() {
                return Err(Error::from(ErrorKind::InvalidPreimage));
            }

            Ok(Decision::new(
                true,
                vec![ImplicationProofElement::new(
                    Property::SignedByDecider(input.clone()),
                    Some(Witness::Bytes(signature)),
                )],
            ))
        } else {
            panic!("invalid witness");
        }
    }
    fn check_decision<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        input: &SignedByInput,
    ) -> Result<Decision, Error> {
        let decision_key = input.hash();
        let result = decider
            .get_db()
            .bucket(&BaseDbKey::from(&b"signed_by_decider"[..]))
            .get(&BaseDbKey::from(decision_key.to_vec().as_slice()))
            .map_err::<Error, _>(Into::into)?;
        if let Some(decision_value_bytes) = result {
            let decision_value = DecisionValue::from_abi(&decision_value_bytes).unwrap();
            return Ok(Decision::new(
                decision_value.get_decision(),
                vec![ImplicationProofElement::new(
                    Property::SignedByDecider(input.clone()),
                    Some(decision_value.get_witness().clone()),
                )],
            ));
        }

        Err(Error::from(ErrorKind::Undecided))
    }
}

#[cfg(test)]
mod tests {
    use super::Verifier;
    use crate::db::SignedByDb;
    use crate::property_executor::PropertyExecutor;
    use crate::types::{Decision, Property, SignedByInput, Witness};
    use bytes::Bytes;
    use ethsign::SecretKey;
    use plasma_db::impls::kvs::CoreDbLevelDbImpl;

    #[test]
    fn test_decide() {
        let raw_key =
            hex::decode("c87509a1c067bbde78beb793e6fa76530b6382a4c0241e5e4a9ec0a0f44dc0d3")
                .unwrap();
        let secret_key = SecretKey::from_raw(&raw_key).unwrap();
        let message = Bytes::from("message");
        let signature = Verifier::sign(&secret_key, &message);
        let witness = Witness::Bytes(signature);
        let input = SignedByInput::new(message, secret_key.public().address().into());
        let property = Property::SignedByDecider(input.clone());
        let decider: PropertyExecutor<CoreDbLevelDbImpl> = Default::default();
        let db = SignedByDb::new(decider.get_db());
        assert!(db.store_witness(&input, &witness).is_ok());
        let decided: Decision = decider.decide(&property, Some(witness)).unwrap();
        assert_eq!(decided.get_outcome(), true);
    }
}
