use bytes::Bytes;
use ethereum_types::Address;
use ovm::types::{Property, PropertyInput};
use ovm::DeciderManager;
use plasma_clients::plasma::PlasmaClient;
use plasma_core::data_structure::abi::Encodable;
use plasma_core::data_structure::Range;
use plasma_db::impls::kvs::CoreDbMemoryImpl;

fn create_ownership_state_object(to_address: Address) -> Property {
    let ownership_decider_id = DeciderManager::get_decider_address(9);
    Property::new(
        ownership_decider_id,
        vec![
            PropertyInput::Placeholder(Bytes::from("state_update")),
            PropertyInput::ConstantAddress(to_address),
        ],
    )
}

fn main() {
    let client = PlasmaClient::<CoreDbMemoryImpl>::new(
        Address::zero(),
        "127.0.0.1:8080".to_owned(),
        "659cbb0e2411a44db63778987b1e22153c086a95eb6b18bdf89de078917abc63",
    );
    let my_address =
        Address::from_slice(&hex::decode("2932b7a2355d6fecc4b5c0b6bd44cc31df247a2e").unwrap());

    let tx = client.create_transaction(
        Range::new(0, 8),
        Bytes::from(create_ownership_state_object(my_address).to_abi()),
    );

    println!("{:?}", tx);
    client.send_transaction(tx);
}
