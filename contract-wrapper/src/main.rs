extern crate serde;

use contract_wrapper::universal_decision_contract_adaptor::UniversalDecisionContractAdaptor;
use ethabi::Contract as ContractABI;
use plasma_core::ovm::types::Property;
use std::fs::File;
use std::io::BufReader;
use web3::types::Address;

fn main() {
    let f = File::open("UniversalDecisionContract.json").unwrap();
    let reader = BufReader::new(f);
    let contract_abi = ContractABI::load(reader).unwrap();
    let contract = UniversalDecisionContractAdaptor::new(
        "661E0De345B6AE4848c4Efd7F4094ae1014091F7",
        contract_abi,
    );

    let from: Address = "ce397e30544d737195a341291675ec1ecaf19b13".parse().unwrap();
    let property = Property::new(
        "1a50faDFab6b21AaaED82bb17A541993304786E7".parse().unwrap(),
        b"012345678"[..].into(),
    );

    if let Ok(result) = contract.claim_property(from, property.clone()) {
        println!("claim_property: {}", result);
    };
    if let Ok(result) = contract.decide_property(from, property.clone(), true) {
        println!("decide_property: {}", result);
    };
}
