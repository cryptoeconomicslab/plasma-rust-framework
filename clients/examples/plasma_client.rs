use bytes::Bytes;
use ethereum_types::Address;
use plasma_clients::plasma::PlasmaClient;
use plasma_core::data_structure::Range;
use plasma_db::impls::kvs::CoreDbMemoryImpl;

fn main() {
    let client = PlasmaClient::<CoreDbMemoryImpl>::new(
        Address::zero(),
        "127.0.0.1:8080".to_owned(),
        "659cbb0e2411a44db63778987b1e22153c086a95eb6b18bdf89de078917abc63",
    );

    let tx = client.create_transaction(Range::new(0, 8), Bytes::from(&b""[..]));

    println!("{:?}", tx);
    client.send_transaction(tx);
}
