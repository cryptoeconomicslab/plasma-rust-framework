use bytes::Bytes;

pub struct QuantifierResult {
    results: Vec<Bytes>,
    all_results_quantified: bool,
}

impl QuantifierResult {
    pub fn new(results: Vec<Bytes>, all_results_quantified: bool) -> Self {
        QuantifierResult {
            results,
            all_results_quantified,
        }
    }
    pub fn get_results(&self) -> &Vec<Bytes> {
        &self.results
    }
    pub fn get_all_results_quantified(&self) -> bool {
        self.all_results_quantified
    }
}

pub trait Quantifier {
    fn get_all_quantified(&self, parameters: Bytes) -> QuantifierResult;
}
