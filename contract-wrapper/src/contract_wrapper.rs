use ethabi::Contract as ContractABI;
use ethabi::Token;
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::transports::{EventLoopHandle, Http};
use web3::types::{Address, H256};

type Error = ();

pub struct ContractWrapper {
    _eloop: EventLoopHandle,
    _web3: web3::Web3<web3::transports::Http>,
    _address: Address,
    inner: Contract<Http>, // TODO: make use of generic
}

impl ContractWrapper {

    pub fn new(address: &str, abi: ContractABI) -> Self {
        // TODO: use env to specify url
        let (_eloop, http) = web3::transports::Http::new("http://localhost:9545").unwrap();
        let web3 = web3::Web3::new(http);

        let address: Address = address.parse().unwrap();
        let contract = Contract::new(web3.eth(), address, abi);

        Self {
            _web3: web3,
            _eloop,
            _address: address,
            inner: contract,
        }
    }

    pub fn deposit(&self, from: Address, amount: u64) -> Result<H256, Error> {
        let predicate_address: Address =
            "B3a39641f23aaaAB328B94b0fec83E1E4A08bc2B".parse().unwrap(); // TODO: fix predicate address
        let result = self.inner.call(
            "deposit",
            (
                Token::Uint(amount.into()),
                Token::Tuple(vec![
                    Token::Address(predicate_address),
                    Token::Bytes(vec![]),
                ]),
            ),
            from,
            Options::default(),
        );

        // TODO: Error handling
        match result.wait() {
            Ok(r) => Ok(r),
            Err(_) => Err(()),
        }
    }

    pub fn submit_block(&self, from: Address, root: &[u8; 32]) -> Result<H256, Error> {
        let result = self.inner.call(
            "submit",
            Token::FixedBytes(root.to_vec()),
            from,
            Options::default(),
        );

        // TODO: Error handling
        match result.wait() {
            Ok(r) => Ok(r),
            Err(_) => Err(()),
        }

    }

    // TODO: IMPLEMENT
    pub fn start_checkpoint() {}

    // TODO: IMPLEMENT
    pub fn start_exit() {}

    // TODO: IMPLEMENT
    pub fn finalize_exit() {}

    // TODO: IMPLEMENT
    pub fn challenge() {}
}
