
/// Plasma Client on OVM.
pub struct PlasmaClient {
}

impl PlasmaClient {
    pub fn new() -> Self {
        PlasmaClient {}
    }

    /// Deposit to plasma contract
    /// Send ethereum transaction to Plasma Deposit Contract.
    pub fn deposit(&self) {}

    /// Handle Deposit Event on Plasma Contract.
    pub fn handle_deposit(&self) {}

    /// Create transaction to update state for specific coin range.
    pub fn create_transaction(&self) {}

    /// Handle incoming transaction from other clients.
    pub fn handle_transaction(&self) {}

    /// Start exit on plasma.
    pub fn start_exit(&self) {}

    /// Handle exit on plasma.
    /// After dispute period, withdraw from Plasma Contract.
    pub fn handle_exit(&self) {}

    /// Challenge to specific exit by claiming contradicting statement.
    pub fn challenge(&self) {}

    /// Handle BlockSubmitted Event on Commitment Contract.
    /// Store inclusion proof or exclusion proof.
    pub fn handle_new_block(&self) {}

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

