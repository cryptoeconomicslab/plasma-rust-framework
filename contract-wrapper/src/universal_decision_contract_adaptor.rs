use ethabi::Contract as ContractABI;
use ethabi::Token;
use plasma_core::ovm::types::Property;
use web3::contract::tokens::Tokenize;
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::transports::{EventLoopHandle, Http};
use web3::types::{Address, H256};

type Error = ();

pub struct UniversalDecisionContractAdaptor {
    _eloop: EventLoopHandle,
    _web3: web3::Web3<web3::transports::Http>,
    _address: Address,
    inner: Contract<Http>,
}

impl UniversalDecisionContractAdaptor {
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

    pub fn claim_property(&self, from: Address, property: Property) -> Result<H256, Error> {
        let params: Token = property.into();
        let result = self
            .inner
            .call("claimProperty", params, from, Options::default());

        // TODO: error handling
        match result.wait() {
            Ok(r) => Ok(r),
            Err(e) => {
                println!("{}", e);
                Err(())
            }
        }
    }

    pub fn decide_property(
        &self,
        from: Address,
        property: Property,
        decision: bool,
    ) -> Result<H256, Error> {
        let params: Token = property.into();
        let result = self.inner.call(
            "decideProperty",
            (params, Token::Bool(decision)),
            from,
            Options::default(),
        );

        match result.wait() {
            Ok(r) => Ok(r),
            Err(e) => {
                println!("{}", e);
                Err(())
            }
        }
    }
}
