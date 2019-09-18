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
use std::sync::RwLock;

lazy_static! {
    static ref DECIDER_LIST: Vec<Address> = {
        let mut list = vec![];
        list.push(Address::from_slice(&hex::decode("722d70e765d4ec72719d29fcbefe595480a9a3a0").unwrap()));
        list.push(Address::from_slice(&hex::decode("0888415d7a6b971d6fdb15d62d795f2a909d8065").unwrap()));
        list.push(Address::from_slice(&hex::decode("0326080d0f068c6ab58ab8ee31726da2c92f691f").unwrap()));
        list.push(Address::from_slice(&hex::decode("6bae98b57d444f02b41383434d006d013a7203f0").unwrap()));
        list.push(Address::from_slice(&hex::decode("f73f3ebe9c256e29c9761b6e0668908ffc1639ad").unwrap()));
        list.push(Address::from_slice(&hex::decode("9657997a36fce37b51fb7d99b10ce15f425c54f4").unwrap()));
        list.push(Address::from_slice(&hex::decode("330b5059134444e32c305b2dc20d51057e198ed1").unwrap()));
        list.push(Address::from_slice(&hex::decode("7735e33ecd766357887b2512ff828122174e4f61").unwrap()));
        list.push(Address::from_slice(&hex::decode("d0ac44c34597e9042ee521162560521672eabd18").unwrap()));
        list.push(Address::from_slice(&hex::decode("fa118401b87fad66085764307c72343bdf3b17ac").unwrap()));
        list.push(Address::from_slice(&hex::decode("f0f7daba2c80a15fe17fb0eb0f79c8acee9ac025").unwrap()));
        list.push(Address::from_slice(&hex::decode("5140ac06ade1006cb5f1cab85d96f37b5780eca1").unwrap()));
        list.push(Address::from_slice(&hex::decode("09ea10fff4ee3abce0f4bba57d039d6075d60f5e").unwrap()));
        list.push(Address::from_slice(&hex::decode("a80a778f9ffcdd87302d723da75b64d2b53d6e44").unwrap()));
        list.push(Address::from_slice(&hex::decode("b32f99ebf4bdb4a8734e62398a4594f3e23f7f94").unwrap()));
        list.push(Address::from_slice(&hex::decode("848e53adf2e8dcbb9582a4b11af4cb9245663a23").unwrap()));
        list.push(Address::from_slice(&hex::decode("d58b92479920e05e17ac9a5a250e7da4da083c27").unwrap()));
        list.push(Address::from_slice(&hex::decode("5adad58b266ac03cc77d84dbdf61749f68573728").unwrap()));
        list.push(Address::from_slice(&hex::decode("5a645e3c785477eb4119a32dabae7452a53034e8").unwrap()));
        list.push(Address::from_slice(&hex::decode("55d87e9fd1a712bebe7915e0dce903a470992222").unwrap()));
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
    fn decide(&self, decider: &PropertyExecutor<KVS>) -> Result<Decision, Error>;
}

impl<KVS> DecideMixin<KVS> for Property
where
    KVS: KeyValueStore,
{
    fn decide(&self, decider: &PropertyExecutor<KVS>) -> Result<Decision, Error> {
        decider.decide(self)
    }
}

/// Core runtime for Property
pub struct PropertyExecutor<KVS: KeyValueStore> {
    db: KVS,
    range_db: RangeDbImpl<KVS>,
    variables: RwLock<HashMap<Bytes, QuantifierResultItem>>,
}

impl<KVS> Default for PropertyExecutor<KVS>
where
    KVS: KeyValueStore + DatabaseTrait,
{
    fn default() -> Self {
        PropertyExecutor {
            db: KVS::open("kvs"),
            range_db: RangeDbImpl::from(KVS::open("range")),
            variables: RwLock::new(Default::default()),
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
    pub fn set_variable(&self, placeholder: Bytes, result: QuantifierResultItem) {
        self.variables.write().unwrap().insert(placeholder, result);
    }
    pub fn get_variable(&self, placeholder: &PropertyInput) -> QuantifierResultItem {
        match placeholder {
            PropertyInput::Placeholder(placeholder) => {
                self.variables.read().unwrap().get(placeholder).unwrap().clone()
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
    pub fn decide(&self, property: &Property) -> Result<Decision, Error> {
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
