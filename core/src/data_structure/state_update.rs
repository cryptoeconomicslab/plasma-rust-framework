extern crate ethereum_types;
extern crate rlp;

use super::state_object::StateObject;
use ethereum_types::Address;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone)]
pub struct StateUpdate {
    start: u64,
    end: u64,
    block: u64,
    plasma_contract: Address,
    new_state: StateObject,
}

impl StateUpdate {
    pub fn new(
        start: u64,
        end: u64,
        block: u64,
        plasma_contract: Address,
        new_state: StateObject,
    ) -> StateUpdate {
        StateUpdate {
            start,
            end,
            block,
            plasma_contract,
            new_state,
        }
    }
}

impl Encodable for StateUpdate {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(5);
        s.append(&self.start);
        s.append(&self.end);
        s.append(&self.block);
        s.append(&self.plasma_contract);
        s.append(&self.new_state);
    }
}

impl Decodable for StateUpdate {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let new_state_result: Result<StateObject, DecoderError> = rlp.val_at(4);
        new_state_result.map(|new_state| StateUpdate {
            start: rlp.val_at(0).unwrap_or(0),
            end: rlp.val_at(1).unwrap_or(0),
            block: rlp.val_at(2).unwrap_or(0),
            plasma_contract: rlp.val_at(3).unwrap_or_else(|_| Address::zero()),
            new_state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DecoderError;
    use super::StateObject;
    use super::StateUpdate;
    use bytes::Bytes;
    use ethereum_types::Address;

    #[test]
    fn test_rlp_encode() {
        let message_bytes = Bytes::from(&b"parameters"[..]);
        let state_object = StateObject::new(Address::zero(), message_bytes);
        let state_update = StateUpdate::new(0, 0, 0, Address::zero(), state_object);
        let encoded = rlp::encode(&state_update);
        let _decoded: StateUpdate = rlp::decode(&encoded).unwrap();
        assert_eq!(_decoded.start, state_update.start);
    }

    #[test]
    fn fail_to_decode() {
        let failtodecode = "failtodecode";
        let encoded = rlp::encode(&failtodecode);
        let decoded: Result<StateUpdate, DecoderError> = rlp::decode(&encoded);
        assert_eq!(decoded.is_err(), true);
    }
}
