use crate::error::{Error, ErrorKind};
use bytes::Bytes;
use ethabi::Contract as ContractABI;
use ethabi::Token;
use ovm::types::core::{ImplicationProofElement, Property};
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::transports::{EventLoopHandle, Http};
use web3::types::{Address, H256};

pub struct UniversalDecisionContractAdaptor {
    _eloop: EventLoopHandle,
    _web3: web3::Web3<web3::transports::Http>,
    _address: Address,
    inner: Contract<Http>,
}

impl UniversalDecisionContractAdaptor {
    pub fn new(host: &str, address: &str, abi: ContractABI) -> Result<Self, Error> {
        let (_eloop, http) = web3::transports::Http::new(host)
            .map_err(|_| Error::from(ErrorKind::InvalidInputType))?;
        let web3 = web3::Web3::new(http);

        let address: Address = address
            .parse()
            .map_err(|_| Error::from(ErrorKind::InvalidInputType))?;
        let contract = Contract::new(web3.eth(), address, abi);

        Ok(Self {
            _web3: web3,
            _eloop,
            _address: address,
            inner: contract,
        })
    }

    pub fn claim_property(&self, from: Address, property: Property) -> Result<H256, Error> {
        let params: Token = property.into();
        let result = self
            .inner
            .call("claimProperty", params, from, Options::default());

        match result.wait() {
            Ok(r) => Ok(r),
            Err(e) => { 
                println!("{}", e);
                Err(e.into())
            },
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
            Err(e) => Err(e.into()),
        }
    }

    pub fn verify_implication(
        &self,
        from: Address,
        root_premise: Property,
        implication_proof: Vec<ImplicationProofElement>,
    ) -> Result<H256, Error> {
        let root_premise: Token = root_premise.into();
        let implication_proof: Vec<Token> = implication_proof.into_iter().map(Into::into).collect();

        let result = self.inner.call(
            "verifyImplication",
            (root_premise, implication_proof),
            from,
            Options::default(),
        );
        match result.wait() {
            Ok(r) => Ok(r),
            Err(e) => Err(e.into()),
        }
    }


    pub fn verify_contradicting_implications(
        &self,
        from: Address,
        root1: Property,
        implication_proof1: Vec<ImplicationProofElement>,
        root2: Property,
        implication_proof2: Vec<ImplicationProofElement>,
        contradiction_witness: Bytes,
    ) -> Result<H256, Error> {
        let root1: Token = root1.into();
        let implication_proof1: Vec<Token> =
            implication_proof1.into_iter().map(Into::into).collect();
        let root2: Token = root2.into();
        let implication_proof2: Vec<Token> =
            implication_proof2.into_iter().map(Into::into).collect();
        let contradiction_witness: Token = Token::Bytes(contradiction_witness.to_vec());

        let result = self.inner.call(
            "verifyContradictingImplications",
            (
                root1,
                implication_proof1,
                root2,
                implication_proof2,
                contradiction_witness,
            ),
            from,
            Options::default(),
        );
        match result.wait() {
            Ok(r) => Ok(r),
            Err(e) => Err(e.into()),
        }
    }

    pub fn prove_claim_contradicts_decision(
        &self,
        from: Address,
        decided_property: Property,
        decided_implication_proof: Vec<ImplicationProofElement>,
        contradicting_claim: Property,
        contradiction_implication_proof: Vec<ImplicationProofElement>,
        contradiction_witness: Bytes,
    ) -> Result<H256, Error> {
        let decided_property: Token = decided_property.into();
        let decided_implication_proof: Vec<Token> = decided_implication_proof
            .into_iter()
            .map(Into::into)
            .collect();
        let contradicting_claim: Token = contradicting_claim.into();
        let contradiction_implication_proof: Vec<Token> = contradiction_implication_proof
            .into_iter()
            .map(Into::into)
            .collect();
        let contradiction_witness: Token = Token::Bytes(contradiction_witness.to_vec());

        let result = self.inner.call(
            "verifyContradictingImplications",
            (
                decided_property,
                decided_implication_proof,
                contradicting_claim,
                contradiction_implication_proof,
                contradiction_witness,
            ),
            from,
            Options::default(),
        );
        match result.wait() {
            Ok(r) => Ok(r),
            Err(e) => Err(e.into()),
        }
    }

    pub fn prove_undecided_contradiction(
        &self,
        from: Address,
        contradiction: (Property, Property),
        implication_proof0: Vec<ImplicationProofElement>,
        implication_proof1: Vec<ImplicationProofElement>,
        contradiction_witness: Bytes,
    ) -> Result<H256, Error> {
        let contradiction: (Token, Token) = (contradiction.0.into(), contradiction.1.into());
        let implication_proof0: Vec<Token> =
            implication_proof0.into_iter().map(Into::into).collect();
        let implication_proof1: Vec<Token> =
            implication_proof1.into_iter().map(Into::into).collect();

        let result = self.inner.call(
            "proveUndecidedContradiction",
            (
                Token::Tuple(vec![contradiction.0, contradiction.1]),
                implication_proof0,
                implication_proof1,
                Token::Bytes(contradiction_witness.to_vec()),
            ),
            from,
            Options::default(),
        );

        match result.wait() {
            Ok(r) => Ok(r),
            Err(e) => Err(e.into()),
        }
    }


    pub fn remove_contradiction(
        &self,
        from: Address,
        contradiction: (Property, Property),
        remaining_claim_index: usize,
    ) -> Result<H256, Error> {
        match remaining_claim_index {
            0 | 1 => {
                let contradiction: (Token, Token) =
                    (contradiction.0.into(), contradiction.1.into());
                let result = self.inner.call(
                    "removeContradiction",
                    (
                        Token::Tuple(vec![contradiction.0, contradiction.1]),
                        Token::Uint(remaining_claim_index.into()),
                    ),
                    from,
                    Options::default(),
                );
                match result.wait() {
                    Ok(r) => Ok(r),
                    Err(e) => Err(e.into()),
                }
            }
            _ => Err(Error::from(ErrorKind::InvalidInputType)),
        }
    }
}
