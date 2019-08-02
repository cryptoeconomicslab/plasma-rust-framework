use bytes::Bytes;

pub struct QuantifierResult<T> {
    results: Vec<T>,
    all_results_quantified: bool,
}

impl<T> QuantifierResult<T> {
    pub fn new(results: Vec<T>, all_results_quantified: bool) -> Self {
        QuantifierResult {
            results,
            all_results_quantified,
        }
    }
    pub fn get_results(&self) -> &Vec<T> {
        &self.results
    }
    pub fn get_all_results_quantified(&self) -> bool {
        self.all_results_quantified
    }
}

pub trait Quantifier {
    fn get_all_quantified<T>(&self, parameters: Bytes) -> QuantifierResult<T>;
}
