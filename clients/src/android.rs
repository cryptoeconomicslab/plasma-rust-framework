use crate::plasma::{utils::*, PlasmaClientShell};
use ethereum_types::Address;

lazy_static! {
    pub static ref GLOBAL_CLIENT: PlasmaClientShell = {
        let mut client = PlasmaClientShell::new(
            "client", // db name
            "192.168.0.9:8080".to_owned(),
            string_to_address("9FBDa871d559710256a2502A2517b794B482Db40"),
        );
        client.connect();
        client.initialize();
        client
    };
}

pub struct AndroidClient {}
impl AndroidClient {
    pub fn create_account() -> String {
        let (session, _) = GLOBAL_CLIENT.create_account();
        encode_session(session)
    }

    pub fn get_balance(session: String) -> u64 {
        let eth_address = Address::zero();
        *GLOBAL_CLIENT
            .get_balance(&decode_session(session.clone()).unwrap())
            .get(&eth_address)
            .unwrap_or(&0)
    }
}
