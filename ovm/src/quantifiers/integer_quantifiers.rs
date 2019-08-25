use crate::property_executor::PropertyExecutor;
use crate::types::{
    Integer, IntegerRangeQuantifierInput, Placeholder, QuantifierResult, QuantifierResultItem,
};
use plasma_db::traits::kvs::KeyValueStore;

fn get_range(start: u64, end: u64) -> Vec<QuantifierResultItem> {
    (start..end)
        .map(|n| QuantifierResultItem::Integer(Integer::new(n)))
        .collect()
}

/// IntegerRangeQuantifier quantify specific range
pub struct IntegerRangeQuantifier {}

impl Default for IntegerRangeQuantifier {
    fn default() -> Self {
        IntegerRangeQuantifier {}
    }
}

impl IntegerRangeQuantifier {
    pub fn get_all_quantified<KVS: KeyValueStore>(
        decider: &PropertyExecutor<KVS>,
        input: &IntegerRangeQuantifierInput,
    ) -> QuantifierResult {
        if let (QuantifierResultItem::Integer(start), QuantifierResultItem::Integer(end)) = (
            decider.replace(input.get_start()),
            decider.replace(input.get_end()),
        ) {
            if end < start {
                panic!("invalid start and end");
            }
            QuantifierResult::new(get_range(start.0, end.0), true)
        } else {
            panic!("invalid input");
        }
    }
}

/// NonnegativeIntegerLessThanQuantifier quantify 0 to upper bound
pub struct NonnegativeIntegerLessThanQuantifier {}

impl Default for NonnegativeIntegerLessThanQuantifier {
    fn default() -> Self {
        NonnegativeIntegerLessThanQuantifier {}
    }
}

impl NonnegativeIntegerLessThanQuantifier {
    pub fn get_all_quantified<KVS: KeyValueStore>(
        decider: &PropertyExecutor<KVS>,
        placeholder: &Placeholder,
    ) -> QuantifierResult {
        if let QuantifierResultItem::Integer(upper_bound) = decider.replace(&placeholder) {
            if *upper_bound < Integer(0) {
                panic!("upper_bound shouldn't negative value.");
            }
            QuantifierResult::new(get_range(0, upper_bound.0), true)
        } else {
            panic!("invalid input")
        }
    }
}
