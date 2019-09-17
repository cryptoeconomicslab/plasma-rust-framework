use crate::deciders::{
    AndDecider, ForAllSuchThatDecider, HasLowerNonceDecider, IncludedAtBlockDecider,
    IsDeprecatedDecider, NotDecider, OrDecider, OwnershipDecider, PreimageExistsDecider,
    SignedByDecider,
};
use crate::error::Error;
use crate::quantifiers::{
    BlockRangeQuantifier, HashQuantifier, IntegerRangeQuantifier,
    NonnegativeIntegerLessThanQuantifier, SignedByQuantifier,
};
use crate::types::{
    Decider, Decision, Property, PropertyInput, QuantifierResult, QuantifierResultItem,
};
use bytes::Bytes;
use ethereum_types::Address;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::RangeDbImpl;
use std::collections::HashMap;

lazy_static! {
    static ref DECIDER_LIST: Vec<Address> = {
        let mut list = vec![];
        for _ in 0..20 {
            list.push(Address::random())
        }
        list
    };
}

pub struct DeciderManager {}
impl DeciderManager {
    pub fn get_decider_address(i: usize) -> Address {
        DECIDER_LIST[i]
    }
    pub fn preimage_exists_decider(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(2), inputs)
    }
    pub fn and_decider(left: Property, right: Property) -> Property {
        Property::new(
            Self::get_decider_address(0),
            vec![
                PropertyInput::ConstantProperty(left),
                PropertyInput::ConstantProperty(right),
            ],
        )
    }
    pub fn or_decider(left: Property, right: Property) -> Property {
        Property::new(
            Self::get_decider_address(4),
            vec![
                PropertyInput::ConstantProperty(left),
                PropertyInput::ConstantProperty(right),
            ],
        )
    }
    pub fn not_decider(p: Property) -> Property {
        Property::new(
            Self::get_decider_address(1),
            vec![PropertyInput::ConstantProperty(p)],
        )
    }
    pub fn has_lower_nonce_decider(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(6), inputs)
    }
    pub fn for_all_such_that_decider(
        quantifier: Property,
        placeholder: Bytes,
        property: Property,
    ) -> Property {
        Self::for_all_such_that_decider_raw(&[
            PropertyInput::ConstantProperty(quantifier),
            PropertyInput::ConstantBytes(placeholder),
            PropertyInput::ConstantProperty(property),
        ])
    }
    pub fn for_all_such_that_decider_raw(inputs: &[PropertyInput]) -> Property {
        Property::new(Self::get_decider_address(3), inputs.to_vec())
    }
    pub fn signed_by_decider(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(5), inputs)
    }
    pub fn included_at_block_decider(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(7), inputs)
    }
    pub fn is_deprecated(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(8), inputs)
    }
    pub fn ownership(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(9), inputs)
    }
    pub fn q_range(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(10), inputs)
    }
    pub fn q_uint(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(11), inputs)
    }
    pub fn q_block(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(12), inputs)
    }
    pub fn q_signed_by(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(13), inputs)
    }
    pub fn q_hash(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(14), inputs)
    }
}

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
    pub fn set_variable(&mut self, placeholder: Bytes, result: QuantifierResultItem) {
        self.variables.insert(placeholder, result);
    }
    pub fn get_variable(&self, placeholder: &PropertyInput) -> QuantifierResultItem {
        match placeholder {
            PropertyInput::Placeholder(placeholder) => {
                self.variables.get(placeholder).unwrap().clone()
            }
            PropertyInput::ConstantAddress(constant) => QuantifierResultItem::Address(*constant),
            PropertyInput::ConstantBytes(constant) => QuantifierResultItem::Bytes(constant.clone()),
            PropertyInput::ConstantH256(constant) => QuantifierResultItem::H256(*constant),
            PropertyInput::ConstantInteger(constant) => QuantifierResultItem::Integer(*constant),
            PropertyInput::ConstantRange(constant) => QuantifierResultItem::Range(*constant),
            PropertyInput::ConstantProperty(constant) => {
                QuantifierResultItem::Property(constant.clone())
            }
            PropertyInput::ConstantMessage(constant) => {
                QuantifierResultItem::Message(constant.clone())
            }
        }
    }
    pub fn decide(&mut self, property: &Property) -> Result<Decision, Error> {
        let decider_id = property.decider;
        if decider_id == DECIDER_LIST[0] {
            AndDecider::decide(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[1] {
            NotDecider::decide(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[2] {
            PreimageExistsDecider::decide(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[3] {
            ForAllSuchThatDecider::decide(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[4] {
            OrDecider::decide(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[5] {
            SignedByDecider::decide(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[6] {
            HasLowerNonceDecider::decide(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[7] {
            IncludedAtBlockDecider::decide(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[8] {
            IsDeprecatedDecider::decide(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[9] {
            OwnershipDecider::decide(self, &property.inputs)
        } else {
            panic!("unknown decider")
        }
    }
    pub fn get_all_quantified(&self, property: &Property) -> QuantifierResult {
        let decider_id = property.decider;
        if decider_id == DECIDER_LIST[10] {
            IntegerRangeQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[11] {
            NonnegativeIntegerLessThanQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[12] {
            BlockRangeQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[13] {
            SignedByQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[14] {
            HashQuantifier::get_all_quantified(self, &property.inputs)
        } else {
            panic!("unknown quantifier")
        }
    }
}
