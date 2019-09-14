extern crate serde;

use contract_wrapper::universal_decision_contract_adaptor::UniversalDecisionContractAdaptor;
use ethabi::Contract as ContractABI;
use ovm::types::InputType;
use ovm::DeciderManager;
use std::fs::File;
use std::io::BufReader;
use web3::types::Address;

fn main() {
    let f = File::open("UniversalDecisionContract.json").unwrap();
    let reader = BufReader::new(f);
    let contract_abi = ContractABI::load(reader).unwrap();
    let contract = UniversalDecisionContractAdaptor::new(
        "http://127.0.0.1:9545/",
        "661E0De345B6AE4848c4Efd7F4094ae1014091F7",
        contract_abi,
    )
    .unwrap();

    let from: Address = "ce397e30544d737195a341291675ec1ecaf19b13".parse().unwrap();
    let property = DeciderManager::signed_by_decider(vec![
        InputType::ConstantBytes(b"012345678"[..].into()),
        InputType::ConstantAddress("1a50faDFab6b21AaaED82bb17A541993304786E7".parse().unwrap()),
    ]);

    if let Ok(res) = contract.claim_property(from, property.clone()) {
        println!("{:?}", res);
    };

    if let Ok(result) = contract.decide_property(from, property.clone(), true) {
        println!("decide_property: {}", result);
    };

    if let Ok(result) = contract.verify_implication(from, property.clone(), vec![]) {
        println!("verify_implication: {}", result);
    };

    if let Ok(result) = contract.verify_contradicting_implications(
        from,
        property.clone(),
        vec![],
        property.clone(),
        vec![],
        b""[..].into(),
    ) {
        println!("verify_contradicting_implications: {}", result);
    };

    if let Ok(result) = contract.prove_claim_contradicts_decision(
        from,
        property.clone(),
        vec![],
        property.clone(),
        vec![],
        b""[..].into(),
    ) {
        println!("prove_claim_contradicts_decision: {}", result);
    };

    if let Ok(result) = contract.prove_undecided_contradiction(
        from,
        (property.clone(), property.clone()),
        vec![],
        vec![],
        b""[..].into(),
    ) {
        println!("prove_undecided_contradiction: {}", result);
    };

    if let Ok(result) = contract.remove_contradiction(from, (property.clone(), property.clone()), 1)
    {
        println!("remove_contradiction: {}", result);
    };
}
