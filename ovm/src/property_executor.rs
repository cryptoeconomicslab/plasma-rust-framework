use crate::db::MessageDb;
use crate::deciders::{
    AndDecider, ForAllSuchThatDecider, HasLowerNonceDecider, NotDecider, OrDecider,
    PreimageExistsDecider, SignedByDecider,
};
use crate::error::Error;
use crate::quantifiers::{
    BlockRangeQuantifier, IntegerRangeQuantifier, NonnegativeIntegerLessThanQuantifier,
    SignedByQuantifier,
};
use crate::types::Decider;
use crate::types::{Decision, Property, Quantifier, QuantifierResult, Witness};
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::RangeDbImpl;

/// Mixin for adding decide method to Property
pub trait DecideMixin<KVS: KeyValueStore> {
    fn decide(
        &self,
        decider: &PropertyExecutor<KVS>,
        witness: Option<Witness>,
    ) -> Result<Decision, Error>;
    fn check_decision(&self, decider: &PropertyExecutor<KVS>) -> Result<Decision, Error>;
}

impl<KVS> DecideMixin<KVS> for Property
where
    KVS: KeyValueStore,
{
    fn decide(
        &self,
        decider: &PropertyExecutor<KVS>,
        witness: Option<Witness>,
    ) -> Result<Decision, Error> {
        decider.decide(self, witness)
    }
    fn check_decision(&self, decider: &PropertyExecutor<KVS>) -> Result<Decision, Error> {
        decider.check_decision(self)
    }
}

/// Core runtime for Property
pub struct PropertyExecutor<KVS: KeyValueStore> {
    db: KVS,
    message_db: MessageDb<KVS>,
    range_db: RangeDbImpl<KVS>,
}

impl<KVS> Default for PropertyExecutor<KVS>
where
    KVS: KeyValueStore + DatabaseTrait,
{
    fn default() -> Self {
        PropertyExecutor {
            db: KVS::open("kvs"),
            message_db: MessageDb::from(KVS::open("message")),
            range_db: RangeDbImpl::from(KVS::open("range")),
        }
    }
}

impl<KVS> PropertyExecutor<KVS>
where
    KVS: KeyValueStore,
{
    pub fn get_db(&self) -> &KVS {
        &self.db
    }
    pub fn get_message_db(&self) -> &MessageDb<KVS> {
        &self.message_db
    }
    pub fn get_range_db(&self) -> &RangeDbImpl<KVS> {
        &self.range_db
    }
    pub fn decide(&self, property: &Property, witness: Option<Witness>) -> Result<Decision, Error> {
        match property {
            Property::AndDecider(input) => AndDecider::decide(self, input, witness),
            Property::NotDecider(input) => NotDecider::decide(self, input, witness),
            Property::PreimageExistsDecider(input) => {
                PreimageExistsDecider::decide(self, input, witness)
            }
            Property::ForAllSuchThatDecider(input) => {
                ForAllSuchThatDecider::decide(self, input, witness)
            }
            Property::OrDecider(input) => OrDecider::decide(self, input, witness),
            Property::SignedByDecider(input) => SignedByDecider::decide(self, input, witness),
            Property::HasLowerNonceDecider(input) => {
                HasLowerNonceDecider::decide(self, input, witness)
            }
            _ => panic!("not implemented!!"),
        }
    }
    pub fn check_decision(&self, property: &Property) -> Result<Decision, Error> {
        match property {
            Property::AndDecider(input) => AndDecider::check_decision(self, input),
            Property::NotDecider(input) => NotDecider::check_decision(self, input),
            Property::PreimageExistsDecider(input) => {
                PreimageExistsDecider::check_decision(self, input)
            }
            Property::ForAllSuchThatDecider(input) => {
                ForAllSuchThatDecider::check_decision(self, input)
            }
            Property::OrDecider(input) => OrDecider::check_decision(self, input),
            Property::SignedByDecider(input) => SignedByDecider::check_decision(self, input),
            Property::HasLowerNonceDecider(input) => {
                HasLowerNonceDecider::check_decision(self, input)
            }
            _ => panic!("not implemented!!"),
        }
    }
    pub fn get_all_quantified(&self, quantifier: &Quantifier) -> QuantifierResult {
        match quantifier {
            Quantifier::IntegerRangeQuantifier(input) => {
                IntegerRangeQuantifier::get_all_quantified(*input)
            }
            Quantifier::NonnegativeIntegerLessThanQuantifier(upper_bound) => {
                NonnegativeIntegerLessThanQuantifier::get_all_quantified(*upper_bound)
            }
            Quantifier::BlockRangeQuantifier(input) => {
                BlockRangeQuantifier::get_all_quantified(self, input)
            }
            Quantifier::SignedByQuantifier(signer) => {
                SignedByQuantifier::get_all_quantified(self, *signer)
            }
        }
    }
}
