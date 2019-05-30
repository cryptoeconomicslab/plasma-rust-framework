use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::cmp::{max, min, Ordering};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    start: u64,
    end: u64,
    value: Vec<u8>,
}

impl Ord for Range {
    fn cmp(&self, other: &Range) -> Ordering {
        self.end.cmp(&other.end)
    }
}

impl PartialOrd for Range {
    fn partial_cmp(&self, other: &Range) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Range {
    pub fn new(start: u64, end: u64, value: &[u8]) -> Self {
        Range {
            start,
            end,
            value: value.to_vec(),
        }
    }
    pub fn get_start(&self) -> u64 {
        self.start
    }
    pub fn get_end(&self) -> u64 {
        self.end
    }
    pub fn get_value(&self) -> &[u8] {
        &self.value
    }
    pub fn validate(&self) -> bool {
        self.start < self.end
    }
    pub fn intersect(&self, start: u64, end: u64) -> bool {
        let max_start = max(self.start, start);
        let max_end = min(self.end, end);
        max_start < max_end
    }
}

impl Encodable for Range {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);
        s.append(&self.start);
        s.append(&self.end);
        s.append(&self.value);
    }
}

impl Decodable for Range {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let start: u64 = rlp.val_at(0)?;
        let end: u64 = rlp.val_at(1)?;
        let value: Vec<u8> = rlp.val_at(2)?;
        Ok(Range::new(start, end, &value))
    }
}

#[cfg(test)]
mod tests {
    use super::Range;

    #[test]
    fn test_intersect() {
        let range = Range::new(0, 100, b"aaa");
        assert_eq!(range.intersect(50, 120), true);
        assert_eq!(range.intersect(100, 200), false);
    }

}
