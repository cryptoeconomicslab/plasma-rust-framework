use crate::error::{Error, ErrorKind};
use bytes::Bytes;
use ethabi::Contract as ContractABI;
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::transports::{EventLoopHandle, Http};
use web3::types::{Address, H256};

pub struct CommitmentContractAdaptor {
    _eloop: EventLoopHandle,
    _web3: web3::Web3<web3::transports::Http>,
    _address: Address,
    inner: Contract<Http>,
}

impl CommitmentContractAdaptor {
    pub fn new(host: &str, address: Address, abi: ContractABI) -> Result<Self, Error> {
        let (_eloop, http) = web3::transports::Http::new(host)
            .map_err(|_| Error::from(ErrorKind::FailedToConnect))?;
        let web3 = web3::Web3::new(http);

        let contract = Contract::new(web3.eth(), address, abi);

        Ok(Self {
            _web3: web3,
            _eloop,
            _address: address,
            inner: contract,
        })
    }

    pub fn submit_block(
        &self,
        from: Address,
        block_number: u64,
        root: Bytes,
    ) -> Result<H256, Error> {
        let result = self.inner.call(
            "submit_root",
            (block_number, H256::from_slice(root.to_vec().as_slice())),
            from,
            Options::default(),
        );

        match result.wait() {
            Ok(r) => Ok(r),
            Err(e) => {
                println!("{:?}", e);
                Err(e.into())
            }
        }
    }
}
