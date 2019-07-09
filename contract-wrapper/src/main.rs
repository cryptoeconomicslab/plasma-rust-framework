extern crate serde;

use contract_wrapper::contract_wrapper::ContractWrapper;
use ethabi::Contract as ContractABI;
use plasma_core::data_structure::StateObject;
use std::fs::File;
use std::io::BufReader;
use web3::types::Address;

fn block_submit() {
    let f = File::open("CommitmentChain.json").unwrap();
    let reader = BufReader::new(f);
    let contract_abi = ContractABI::load(reader).unwrap();
    let contract = ContractWrapper::new("b8EE7cFB77034f882Bb282ffB4e67f7b5a629E2f", contract_abi);
    let address: Address = "ce397e30544d737195a341291675ec1ecaf19b13".parse().unwrap();
    let hash: [u8; 32] = [1; 32];
    let _ = contract.submit_block(address, &hash);
}

fn deposit() {
    let f = File::open("Deposit.json").unwrap();
    let reader = BufReader::new(f);
    let contract_abi = ContractABI::load(reader).unwrap();
    let contract = ContractWrapper::new("F59Ae4F3A76AAC629aC52A98a9193ca32432316E", contract_abi);
    let from: Address = "ce397e30544d737195a341291675ec1ecaf19b13".parse().unwrap();
    let state_object = StateObject::new(
        "F59Ae4F3A76AAC629aC52A98a9193ca32432316E".parse().unwrap(),
        vec![].into(),
    );
    let _ = contract.deposit(from, 5, state_object);
}

fn main() {
    block_submit();
    deposit();
}
