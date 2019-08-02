use super::{DeciderId, Decision};

pub trait Decider {
    type Input;
    type Witness;
    fn get_address(&self) -> DeciderId;
    fn decide(&self, input: &Self::Input, witness: Self::Witness) -> Decision;
    fn check_decision(&self, input: &Self::Input) -> Decision;
}
