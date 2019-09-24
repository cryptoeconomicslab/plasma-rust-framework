use bytes::Bytes;
use futures::future;
use plasma_clients::plasma::{utils::*, PlasmaClientShell};

fn main() {
    let mut shell = PlasmaClientShell::new(
        "127.0.0.1:8080".to_string(),
        string_to_address("9FBDa871d559710256a2502A2517b794B482Db40"),
    );
    tokio::run(future::lazy(move || {
        shell.connect();
        println!("{:?}", shell.get_balance(&Bytes::from("")));
        let session = &Bytes::from("");
        let (property, metadata) = shell.ownership_property(
            session,
            string_to_address("2932b7a2355d6fecc4b5c0b6bd44cc31df247a2e"),
        );
        shell.send_transaction(session, None, 0, 10, property, metadata);
        Ok(())
    }));
}
