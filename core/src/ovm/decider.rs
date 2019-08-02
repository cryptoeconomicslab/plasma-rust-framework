use super::Decision;

pub trait Decider {
    type Input;
    type Witness;
    fn decide<D>(&self, input: &Self::Input, witness: Self::Witness) -> Decision<D>
    where
        D: Decider;
    fn check_decision<D>(&self, input: &Self::Input) -> Decision<D>
    where
        D: Decider;
}
