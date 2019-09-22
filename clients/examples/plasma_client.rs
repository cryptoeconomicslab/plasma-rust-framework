use bytes::Bytes;
use ethereum_types::Address;
use futures::future;
use plasma_clients::plasma::PlasmaClientShell;

fn main() {
    let commitment_contract_address_hex =
        hex::decode("9FBDa871d559710256a2502A2517b794B482Db40").unwrap();
    let commitment_contract_address = Address::from_slice(&commitment_contract_address_hex);
    let mut shell =
        PlasmaClientShell::new("127.0.0.1:8080".to_string(), commitment_contract_address);
    tokio::run(future::lazy(move || {
        shell.connect();
        println!("{:?}", shell.get_balance(&Bytes::from("")));
        shell.send_transaction(
            &Bytes::from(""),
            "2932b7a2355d6fecc4b5c0b6bd44cc31df247a2e",
            None,
            0,
            10,
        );
        Ok(())
    }));
}
