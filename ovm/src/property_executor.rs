use crate::db::MessageDb;
use crate::deciders::{
    AndDecider, ForAllSuchThatDecider, HasLowerNonceDecider, NotDecider, PreimageExistsDecider,
    SignedByDecider,
};
use crate::error::Error;
use crate::quantifiers::{
    IntegerRangeQuantifier, NonnegativeIntegerLessThanQuantifier, SignedByQuantifier,
};
use crate::types::Decider;
use crate::types::{Decision, Property, Quantifier, QuantifierResult};
use bytes::Bytes;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::RangeDbImpl;

/// Mixin for adding decide method to Property
pub trait DecideMixin<KVS: KeyValueStore> {
    fn decide(
        &self,
        decider: &PropertyExecutor<KVS>,
        witness: Option<Bytes>,
    ) -> Result<Decision, Error>;
}

impl<KVS> DecideMixin<KVS> for Property
where
    KVS: KeyValueStore,
{
    fn decide(
        &self,
        decider: &PropertyExecutor<KVS>,
        witness: Option<Bytes>,
    ) -> Result<Decision, Error> {
        decider.decide(self, witness)
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
    pub fn decide(&self, property: &Property, witness: Option<Bytes>) -> Result<Decision, Error> {
        match property {
            Property::AndDecider(input) => AndDecider::decide(self, input, witness),
            Property::NotDecider(input) => NotDecider::decide(self, input, witness),
            Property::PreimageExistsDecider(input) => {
                PreimageExistsDecider::decide(self, input, witness)
            }
            Property::ForAllSuchThatDecider(input) => {
                ForAllSuchThatDecider::decide(self, input, witness)
            }
            Property::SignedByDecider(input) => SignedByDecider::decide(self, input, witness),
            Property::HasLowerNonceDecider(input) => {
                HasLowerNonceDecider::decide(self, input, witness)
            }
            _ => panic!("not implemented!!"),
        }
    }
    pub fn get_all_quantified(&self, quantifier: &Quantifier) -> QuantifierResult {
        match quantifier {
            Quantifier::IntegerRangeQuantifier(start, end) => {
                IntegerRangeQuantifier::get_all_quantified(*start, *end)
            }
            Quantifier::NonnegativeIntegerLessThanQuantifier(upper_bound) => {
                NonnegativeIntegerLessThanQuantifier::get_all_quantified(*upper_bound)
            }
            Quantifier::SignedByQuantifier(signer) => {
                SignedByQuantifier::get_all_quantified(self, *signer)
            }
            _ => panic!("not implemented!!"),
        }
    }
}
