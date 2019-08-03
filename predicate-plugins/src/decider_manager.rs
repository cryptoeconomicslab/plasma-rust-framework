use crate::deciders::{AndDecider, PreimageExistsDecider};
use ethereum_types::Address;
use plasma_core::ovm::{Decider, DeciderId};

pub struct DeciderManager {
    preimage_exists_decider_id: DeciderId,
    _and_decider_id: DeciderId,
}

impl Default for DeciderManager {
    fn default() -> Self {
        DeciderManager {
            preimage_exists_decider_id: Address::random(),
            _and_decider_id: Address::random(),
        }
    }
}

impl DeciderManager {
    pub fn get_decider(&self, decider_id: DeciderId) -> Box<dyn Decider> {
        if decider_id == self.preimage_exists_decider_id {
            let decider: PreimageExistsDecider = Default::default();
            Box::new(decider)
        } else {
            let decider: AndDecider = Default::default();
            Box::new(decider)
        }
    }

    pub fn get_preimage_exists_decider_id(&self) -> DeciderId {
        self.preimage_exists_decider_id
    }
}
