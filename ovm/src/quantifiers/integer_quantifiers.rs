use crate::types::{Integer, IntegerRangeQuantifierInput, QuantifierResult, QuantifierResultItem};

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
    pub fn get_all_quantified(range: IntegerRangeQuantifierInput) -> QuantifierResult {
        // let integer_range_parameters = IntegerRangeParameters::from_abi(&parameters).unwrap();
        if range.get_end() < range.get_start() {
            panic!("invalid start and end");
        }
        QuantifierResult::new(get_range(range.get_start(), range.get_end()), true)
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
    pub fn get_all_quantified(upper_bound: Integer) -> QuantifierResult {
        if upper_bound < Integer(0) {
            panic!("upper_bound shouldn't negative value.");
        }
        QuantifierResult::new(get_range(0, upper_bound.0), true)
    }
}
