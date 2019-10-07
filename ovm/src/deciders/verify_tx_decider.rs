use crate::error::Error;
use crate::property_executor::PropertyExecutor;
use crate::types::{Decider, Decision, PropertyInput};
use plasma_db::traits::kvs::KeyValueStore;

pub struct VerifyTxDecider {}

impl Default for VerifyTxDecider {
    fn default() -> Self {
        Self {}
    }
}

impl Decider for VerifyTxDecider {
    fn decide<T: KeyValueStore>(
        decider: &PropertyExecutor<T>,
        inputs: &[PropertyInput],
    ) -> Result<Decision, Error> {
        let _tx_bytes = decider.get_variable(&inputs[0]).to_bytes();
        //let _tx = Transaction::from_abi(&tx_bytes).expect("inputs[0] should be Transaction.");
        // TODO: check tx.params
        Ok(Decision::new(true, vec![]))
    }
}
