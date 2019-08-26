use crate::deciders::{
    AndDecider, ForAllSuchThatDecider, HasLowerNonceDecider, IncludedAtBlockDecider, NotDecider,
    OrDecider, PreimageExistsDecider, SignedByDecider,
};
use crate::error::Error;
use crate::quantifiers::{
    BlockRangeQuantifier, HashQuantifier, IntegerRangeQuantifier,
    NonnegativeIntegerLessThanQuantifier, SignedByQuantifier,
};
use crate::types::Decider;
use crate::types::{
    Decision, InputType, Property, Quantifier, QuantifierResult, QuantifierResultItem,
};
use bytes::Bytes;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::RangeDbImpl;
use std::collections::HashMap;

/// Mixin for adding decide method to Property
pub trait DecideMixin<KVS: KeyValueStore> {
    fn decide(&self, decider: &mut PropertyExecutor<KVS>) -> Result<Decision, Error>;
}

impl<KVS> DecideMixin<KVS> for Property
where
    KVS: KeyValueStore,
{
    fn decide(&self, decider: &mut PropertyExecutor<KVS>) -> Result<Decision, Error> {
        decider.decide(self)
    }
}

/// Core runtime for Property
pub struct PropertyExecutor<KVS: KeyValueStore> {
    db: KVS,
    range_db: RangeDbImpl<KVS>,
    variables: HashMap<Bytes, QuantifierResultItem>,
}

impl<KVS> Default for PropertyExecutor<KVS>
where
    KVS: KeyValueStore + DatabaseTrait,
{
    fn default() -> Self {
        PropertyExecutor {
            db: KVS::open("kvs"),
            range_db: RangeDbImpl::from(KVS::open("range")),
            variables: Default::default(),
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
    pub fn get_range_db(&self) -> &RangeDbImpl<KVS> {
        &self.range_db
    }
    pub fn set_replace(&mut self, placeholder: Bytes, result: QuantifierResultItem) {
        self.variables.insert(placeholder, result);
    }
    pub fn replace(&self, placeholder: &InputType) -> QuantifierResultItem {
        match placeholder {
            InputType::Placeholder(placeholder) => self.variables.get(placeholder).unwrap().clone(),
            InputType::ConstantAddress(constant) => QuantifierResultItem::Address(*constant),
            InputType::ConstantBytes(constant) => QuantifierResultItem::Bytes(constant.clone()),
            InputType::ConstantH256(constant) => QuantifierResultItem::H256(*constant),
            InputType::ConstantInteger(constant) => QuantifierResultItem::Integer(*constant),
            InputType::ConstantRange(constant) => QuantifierResultItem::Range(*constant),
        }
    }
    pub fn decide(&mut self, property: &Property) -> Result<Decision, Error> {
        match property {
            Property::AndDecider(input) => AndDecider::decide(self, input),
            Property::NotDecider(input) => NotDecider::decide(self, input),
            Property::PreimageExistsDecider(input) => PreimageExistsDecider::decide(self, input),
            Property::ForAllSuchThatDecider(input) => ForAllSuchThatDecider::decide(self, input),
            Property::OrDecider(input) => OrDecider::decide(self, input),
            Property::SignedByDecider(input) => SignedByDecider::decide(self, input),
            Property::HasLowerNonceDecider(input) => HasLowerNonceDecider::decide(self, input),
            Property::IncludedAtBlockDecider(input) => IncludedAtBlockDecider::decide(self, input),
            _ => panic!("not implemented!!"),
        }
    }
    pub fn get_all_quantified(&self, quantifier: &Quantifier) -> QuantifierResult {
        match quantifier {
            Quantifier::HashQuantifier(input) => HashQuantifier::get_all_quantified(self, input),
            Quantifier::IntegerRangeQuantifier(input) => {
                IntegerRangeQuantifier::get_all_quantified(self, input)
            }
            Quantifier::NonnegativeIntegerLessThanQuantifier(upper_bound) => {
                NonnegativeIntegerLessThanQuantifier::get_all_quantified(self, upper_bound)
            }
            Quantifier::BlockRangeQuantifier(input) => {
                BlockRangeQuantifier::get_all_quantified(self, input)
            }
            Quantifier::SignedByQuantifier(signer) => {
                SignedByQuantifier::get_all_quantified(self, signer)
            }
        }
    }
}
