/*
#[macro_use]
extern crate clap;

use bytes::Bytes;
use clap::{App, Arg, SubCommand};
use ethereum_types::Address;
use plasma_clients::plasma::PlasmaClient;
use plasma_core::data_structure::Range;
use plasma_db::impls::kvs::CoreDbMemoryImpl;

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
        println!("Your balance is 500ETH");
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

fn send(to: Address, start: u64, end: u64) {
    let client = PlasmaClient::<CoreDbMemoryImpl>::new(
        Address::zero(),
        "127.0.0.1:8080".to_owned(),
        "659cbb0e2411a44db63778987b1e22153c086a95eb6b18bdf89de078917abc63",
    );
    let tx = client.create_transaction(Range::new(start, end), Bytes::from(to.as_bytes()));
    println!("{:?}", tx);
    client.send_transaction(tx);
}
*/

fn main() {}
