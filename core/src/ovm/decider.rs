use super::Decision;
use bytes::Bytes;

pub trait Decider {
    fn decide(&self, input: &Bytes, witness: &Bytes) -> Decision;
    fn check_decision(&self, input: &Bytes) -> Decision;
}
