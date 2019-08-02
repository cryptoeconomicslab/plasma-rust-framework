use super::Decision;

pub trait Decider {
    type Input;
    type Witness;
    fn decide(&self, input: &Self::Input, witness: Self::Witness) -> Decision;
    fn check_decision(&self, input: &Self::Input) -> Decision;
}
