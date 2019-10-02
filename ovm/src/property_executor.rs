use crate::deciders::{
    AndDecider, ForAllSuchThatDecider, HasLowerNonceDecider, IncludedAtBlockDecider,
    IsDeprecatedDecider, NotDecider, OrDecider, OwnershipDecider, PreimageExistsDecider,
    SignedByDecider, ThereExistsSuchThatDecider, VerifyTxDecider,
};
use crate::error::Error;
use crate::quantifiers::{
    BlockRangeQuantifier, HashQuantifier, IntegerRangeQuantifier,
    NonnegativeIntegerLessThanQuantifier, PropertyQuantifier, SignedByQuantifier,
    StateUpdateQuantifier, TxQuantifier,
};
use crate::types::{
    Decider, Decision, Property, PropertyInput, QuantifierResult, QuantifierResultItem,
};
use bytes::Bytes;
use ethereum_types::Address;
use plasma_db::prelude::*;
use std::collections::HashMap;
use std::sync::RwLock;

fn get_address(address: &str) -> Address {
    Address::from_slice(&hex::decode(address).unwrap())
}

lazy_static! {
    pub static ref DECIDER_LIST: Vec<Address> = {
        let mut list = vec![];
        list.push(get_address("722d70e765d4ec72719d29fcbefe595480a9a3a0"));
        list.push(get_address("0888415d7a6b971d6fdb15d62d795f2a909d8065"));
        list.push(get_address("0326080d0f068c6ab58ab8ee31726da2c92f691f"));
        list.push(get_address("6bae98b57d444f02b41383434d006d013a7203f0"));
        list.push(get_address("f73f3ebe9c256e29c9761b6e0668908ffc1639ad"));
        list.push(get_address("9657997a36fce37b51fb7d99b10ce15f425c54f4"));
        list.push(get_address("330b5059134444e32c305b2dc20d51057e198ed1"));
        list.push(get_address("7735e33ecd766357887b2512ff828122174e4f61"));
        list.push(get_address("d0ac44c34597e9042ee521162560521672eabd18"));
        list.push(get_address("fa118401b87fad66085764307c72343bdf3b17ac"));
        list.push(get_address("f0f7daba2c80a15fe17fb0eb0f79c8acee9ac025"));
        list.push(get_address("5140ac06ade1006cb5f1cab85d96f37b5780eca1"));
        list.push(get_address("09ea10fff4ee3abce0f4bba57d039d6075d60f5e"));
        list.push(get_address("a80a778f9ffcdd87302d723da75b64d2b53d6e44"));
        list.push(get_address("b32f99ebf4bdb4a8734e62398a4594f3e23f7f94"));
        list.push(get_address("848e53adf2e8dcbb9582a4b11af4cb9245663a23"));
        list.push(get_address("d58b92479920e05e17ac9a5a250e7da4da083c27"));
        list.push(get_address("5adad58b266ac03cc77d84dbdf61749f68573728"));
        list.push(get_address("5a645e3c785477eb4119a32dabae7452a53034e8"));
        list.push(get_address("55d87e9fd1a712bebe7915e0dce903a470992222"));
        list.push(get_address("afac41bff4f07bae909f4451c833d272e6d3e517"));
        list.push(get_address("e850050a1d1f7ff310e596292642e9db2b05c15b"));
        list.push(get_address("38711bcdc98739f455c1572229174d9305aaeab6"));
        list.push(get_address("d7d790356e856c56f2da12d72a85f0f23f3c3efc"));
        list.push(get_address("91f014215cb599e5f558b8c87d1a829a7ea91778"));
        list.push(get_address("daee7898f47fa216714d40ae561a7911c7d5b32c"));
        list.push(get_address("d9d2fe08bfeb7ea0031cebc5c67fb537e037ff70"));
        list.push(get_address("1e5f550e8fe2c59e9af4aea40a2e972f430be600"));
        list.push(get_address("d5728ae21dc0c87ab08a5c764218622061a4e7ea"));
        list.push(get_address("0921d46a4e60091107ff8060952576c3c03511ce"));
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
    pub fn there_exists_such_that(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(10), inputs)
    }
    pub fn verify_tx(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(11), inputs)
    }
    pub fn q_range(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(20), inputs)
    }
    pub fn q_less_than(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(21), inputs)
    }
    pub fn q_block(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(22), inputs)
    }
    pub fn q_signed_by(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(23), inputs)
    }
    pub fn q_hash(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(24), inputs)
    }
    pub fn q_tx(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(25), inputs)
    }
    pub fn q_property(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(26), inputs)
    }
    pub fn q_state_update(inputs: Vec<PropertyInput>) -> Property {
        Property::new(Self::get_decider_address(27), inputs)
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

pub struct PropertyExecuterOptions {
    pub is_aggregator: bool,
    pub db_name: String,
}

impl Default for PropertyExecuterOptions {
    fn default() -> Self {
        Self {
            is_aggregator: false,
            db_name: "property_executer".to_string(),
        }
    }
}

/// Core runtime for Property
pub struct PropertyExecutor<KVS: KeyValueStore> {
    db: KVS,
    range_db: RangeDbImpl<KVS>,
    variables: RwLock<HashMap<Bytes, QuantifierResultItem>>,
    pub options: PropertyExecuterOptions,
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
            options: Default::default(),
        }
    }
}

impl<KVS> PropertyExecutor<KVS>
where
    KVS: KeyValueStore + DatabaseTrait,
{
    pub fn new(options: PropertyExecuterOptions) -> Self {
        let db_name = options.db_name.clone();
        PropertyExecutor {
            db: KVS::open(&format!("{}-{}", db_name, "kvs")),
            range_db: RangeDbImpl::from(KVS::open(&format!("{}-{}", db_name, "range"))),
            variables: RwLock::new(Default::default()),
            options,
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
            PropertyInput::Placeholder(placeholder) => self
                .variables
                .read()
                .unwrap()
                .get(placeholder)
                .unwrap()
                .clone(),
            PropertyInput::ConstantAddress(constant) => QuantifierResultItem::Address(*constant),
            PropertyInput::ConstantBytes(constant) => QuantifierResultItem::Bytes(constant.clone()),
            PropertyInput::ConstantH256(constant) => QuantifierResultItem::H256(*constant),
            PropertyInput::ConstantInteger(constant) => QuantifierResultItem::Integer(*constant),
            PropertyInput::ConstantRange(constant) => QuantifierResultItem::Range(*constant),
            PropertyInput::ConstantProperty(constant) => {
                QuantifierResultItem::Property(constant.clone())
            }
            PropertyInput::ConstantStateUpdate(constant) => {
                QuantifierResultItem::StateUpdate(constant.clone())
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
        } else if decider_id == DECIDER_LIST[10] {
            ThereExistsSuchThatDecider::decide(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[11] {
            VerifyTxDecider::decide(self, &property.inputs)
        } else {
            panic!("unknown decider")
        }
    }
    pub fn get_all_quantified(&self, property: &Property) -> QuantifierResult {
        let decider_id = property.decider;
        if decider_id == DECIDER_LIST[20] {
            IntegerRangeQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[21] {
            NonnegativeIntegerLessThanQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[22] {
            BlockRangeQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[23] {
            SignedByQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[24] {
            HashQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[25] {
            TxQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[26] {
            PropertyQuantifier::get_all_quantified(self, &property.inputs)
        } else if decider_id == DECIDER_LIST[27] {
            StateUpdateQuantifier::get_all_quantified(self, &property.inputs)
        } else {
            panic!("unknown quantifier")
        }
    }
}
