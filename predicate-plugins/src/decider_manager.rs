use crate::deciders::PreimageExistsDecider;
use plasma_core::ovm::{Decider, DeciderId};

pub struct DeciderManager {}

impl DeciderManager {
    pub fn get_decider(&self, decider_id: DeciderId) -> Box<impl Decider> {
        let decider: PreimageExistsDecider = Default::default();
        Box::new(decider)
    }
}
