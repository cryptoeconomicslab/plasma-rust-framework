/// Parameters of predicate, parameters must be ABI encoded
pub trait PredicateParameters {
    /// Encodes to ABI
    fn encode(&self) -> Vec<u8>;
}
