use abi_utils::{Decodable, Encodable, Error as AbiError, ErrorKind as AbiErrorKind};
use ethabi::Token;
use std::cmp::{max, min};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    start: u64,
    end: u64,
}

impl Range {
    pub fn new(start: u64, end: u64) -> Self {
        Range { start, end }
    }
    pub fn get_start(&self) -> u64 {
        self.start
    }
    pub fn get_end(&self) -> u64 {
        self.end
    }
    pub fn get_overlapping_range(&self, b: &Range) -> Range {
        if self.start < b.start && b.start <= self.end {
            Range::new(b.start, self.end)
        } else if b.start < self.start && self.start <= b.end {
            Range::new(self.start, b.end)
        } else {
            Range::new(0, 0)
        }
    }
    pub fn overlap(&self, range: &Range) -> bool {
        let over1 = self.start <= range.start && range.start <= self.end;
        let over2 = range.start < self.start && self.start <= range.end;
        over1 || over2
    }
    pub fn is_subrange(&self, b: &Range) -> bool {
        self.get_start() <= b.get_start() && self.get_end() >= b.get_end()
    }
    pub fn is_covered_with(&self, ranges: Vec<Range>) -> bool {
        let merged_ranges = Range::merge_ranges(ranges);
        merged_ranges.iter().any(|range| range.is_subrange(self))
    }
    pub fn merge_range(range1: &Range, range2: &Range) -> Option<Range> {
        if range1.overlap(range2) {
            let start = min(range1.start, range2.start);
            let end = max(range1.end, range2.end);
            Some(Range::new(start, end))
        } else {
            None
        }
    }
    pub fn merge_ranges(ranges: Vec<Range>) -> Vec<Range> {
        ranges.iter().fold(vec![], |vec, range| {
            if vec.is_empty() {
                vec![*range]
            } else {
                let mut res = vec![];
                let mut merged = false;
                for r in vec.iter() {
                    if let Some(merged_range) = Range::merge_range(&r, &range) {
                        merged = true;
                        res.push(merged_range);
                    } else {
                        res.push(*r);
                    }
                }
                if !merged {
                    res.push(*range);
                }
                res
            }
        })
    }
}

impl Encodable for Range {
    fn to_tuple(&self) -> Vec<Token> {
        vec![Token::Uint(self.start.into()), Token::Uint(self.end.into())]
    }
}

impl Decodable for Range {
    type Ok = Self;
    fn from_tuple(tuple: &[Token]) -> Result<Self, AbiError> {
        let start = tuple[0].clone().to_uint();
        let end = tuple[1].clone().to_uint();
        if let (Some(start), Some(end)) = (start, end) {
            Ok(Range::new(start.as_u64(), end.as_u64()))
        } else {
            Err(AbiError::from(AbiErrorKind::AbiDecode))
        }
    }
    fn get_param_types() -> Vec<ethabi::ParamType> {
        vec![ethabi::ParamType::Uint(64), ethabi::ParamType::Uint(64)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_subrange() {
        let range1 = Range::new(0, 10);
        let range2 = Range::new(1, 2);

        assert!(range1.is_subrange(&range2));
    }

    #[test]
    fn test_merge_ranges() {
        let ranges = vec![Range::new(1, 2), Range::new(2, 3), Range::new(3, 12)];
        assert_eq!(Range::merge_ranges(ranges), vec![Range::new(1, 12)]);
    }

    #[test]
    fn test_merge_ranges2() {
        let ranges = vec![Range::new(1, 2), Range::new(3, 12)];
        assert_eq!(
            Range::merge_ranges(ranges),
            vec![Range::new(1, 2), Range::new(3, 12)]
        );
    }

    #[test]
    fn test_is_covered_with() {
        let range1 = Range::new(1, 7);
        let ranges = vec![Range::new(1, 2), Range::new(2, 3), Range::new(3, 12)];
        assert!(range1.is_covered_with(ranges));
    }

    #[test]
    fn test_is_not_covered_with() {
        let range1 = Range::new(1, 7);
        let ranges = vec![Range::new(1, 2), Range::new(3, 12)];
        assert!(!range1.is_covered_with(ranges));
    }
}
