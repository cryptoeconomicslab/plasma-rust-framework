#[macro_use]
extern crate clap;

use bytes::Bytes;
use clap::{App, Arg, SubCommand};
use ethereum_types::Address;
use plasma_clients::plasma::PlasmaClient;
use plasma_db::impls::kvs::CoreDbLevelDbImpl;
use pubsub_messaging::{connect, ClientHandler, Message, Sender};
use plasma_core::data_structure::{Range, Transaction, TransactionParams};
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use ovm::types::{Integer, StateUpdate, StateUpdateList};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use std::sync::{Arc, Mutex};
use bincode::{deserialize, serialize};

#[derive(Clone)]
struct Handle {
    pub plasma_client: Arc<Mutex<PlasmaClient<CoreDbLevelDbImpl>>>,
}

impl ClientHandler for Handle {
    fn handle_message(&self, msg: Message, _sender: Sender) {
        let result: StateUpdateList = StateUpdateList::deserialize(msg.message);
        self.plasma_client.lock().unwrap().update_state_updates(result.state_updates);
        let res = self.plasma_client.lock().unwrap().get_state_updates();
    }
}

fn main() {
    let matches = App::new("OVM Wallet!!!")
        .version("1.0")
        .author("CryptoeconomicsLab. Inc")
        .about("Does awesome things")
        .subcommand(
            SubCommand::with_name("balance")
                .about("get balance")
                .version("1.0"),
        )
        .subcommand(
            SubCommand::with_name("send")
                .about("send money")
                .version("1.0")
                .arg(
                    Arg::with_name("start")
                        .short("s")
                        .value_name("start")
                        .takes_value(true)
                        .help("start of range"),
                )
                .arg(
                    Arg::with_name("end")
                        .short("e")
                        .value_name("end")
                        .takes_value(true)
                        .help("end of range"),
                )
                .arg(
                    Arg::with_name("to")
                        .short("t")
                        .value_name("to")
                        .takes_value(true)
                        .help("to address"),
                ),
        )
        .get_matches();
    if matches.subcommand_matches("balance").is_some() {
        println!("Your balance is {:?} ETH", get_balance());
    } else if let Some(matches) = matches.subcommand_matches("send") {
        let to_address =
            Address::from_slice(&hex::decode(matches.value_of("to").unwrap()).unwrap());
        let start = value_t!(matches, "start", u64).unwrap();
        let end = value_t!(matches, "end", u64).unwrap();
        println!("Send {:?}-{:?} ETH to {:?} ", start, end, to_address);
        send(to_address, start, end);
        println!("Sent!!!");
    }
}

fn get_balance() -> u64 {
    let client = PlasmaClient::<CoreDbLevelDbImpl>::new(
        Address::zero(),
        "127.0.0.1:8080".to_owned(),
        "659cbb0e2411a44db63778987b1e22153c086a95eb6b18bdf89de078917abc63",
    );

    let add = Bytes::from(hex::decode("2932b7a2355d6fecc4b5c0b6bd44cc31df247a2e").unwrap());
    client.get_state_updates().iter().filter(|s| s.get_params() == add).fold(0, |sum, r| sum + r.get_amount()) as u64
}


fn send(to: Address, start: u64, end: u64) {
    let client = PlasmaClient::<CoreDbLevelDbImpl>::new(
        Address::zero(),
        "127.0.0.1:8080".to_owned(),
        "c87509a1c067bbde78beb793e6fa76530b6382a4c0241e5e4a9ec0a0f44dc0d3",
    );
    let handler = Handle { plasma_client: Arc::new(Mutex::new(client)) };

    let tx = handler.plasma_client.lock().unwrap().create_transaction(Range::new(start, end), Bytes::from(to.as_bytes()));
    let endpoint = handler.plasma_client.lock().unwrap().aggregator_endpoint.clone();
    let mut connection = connect(endpoint, handler.clone()).unwrap();
    let msg = Message::new("Aggregator".to_string(), tx.to_abi());
    connection.send(msg);
    assert!(connection.handle.join().is_ok());

}
