use crate::types::{
    BlockRangeQuantifierInput, ForAllSuchThatInput, Integer, Property, PropertyFactory, Quantifier,
    QuantifierResultItem,
};
use plasma_core::data_structure::Range;

pub fn create_plasma_property(specified_block_number: Integer, range: Range) -> Property {
    Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
        Quantifier::NonnegativeIntegerLessThanQuantifier(specified_block_number),
        Some(PropertyFactory::new(Box::new(move |item| {
            if let QuantifierResultItem::Integer(block_number) = item {
                create_coin_range_property(block_number, range)
            } else {
                panic!("invalid type in PropertyFactory");
            }
        }))),
        None,
    )))
}

pub fn create_coin_range_property(block_number: Integer, range: Range) -> Property {
    Property::ForAllSuchThatDecider(Box::new(ForAllSuchThatInput::new(
        Quantifier::BlockRangeQuantifier(BlockRangeQuantifierInput::new(block_number, range)),
        Some(PropertyFactory::new(Box::new(|item| {
            if let QuantifierResultItem::Property(property) = item {
                property
            } else {
                panic!("invalid type in PropertyFactory");
            }
        }))),
        None,
    )))
}
