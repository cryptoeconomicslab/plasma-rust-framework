#[macro_use]
extern crate clap;

use clap::{App, Arg, SubCommand};
use ethereum_types::Address;
use futures::future;
use plasma_clients::plasma::PlasmaClientShell;

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
            SubCommand::with_name("init")
                .about("initialize")
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

    let commitment_contract_address_hex =
        hex::decode("9FBDa871d559710256a2502A2517b794B482Db40").unwrap();
    let commitment_contract_address = Address::from_slice(&commitment_contract_address_hex);
    let mut shell = PlasmaClientShell::new(
        "127.0.0.1:8080".to_string(),
        commitment_contract_address,
        "659cbb0e2411a44db63778987b1e22153c086a95eb6b18bdf89de078917abc63",
    );

    if matches.subcommand_matches("balance").is_some() {
        tokio::run(future::lazy(move || {
            shell.connect();
            shell.initialize();
            println!("Your balance is {:?} ETH", shell.get_balance());
            Ok(())
        }));
    } else if matches.subcommand_matches("init").is_some() {
        tokio::run(future::lazy(move || {
            shell.connect();
            shell.initialize();
            Ok(())
        }));
    } else if let Some(matches) = matches.subcommand_matches("send") {
        let to_address = value_t!(matches, "to", String).unwrap();
        let start = value_t!(matches, "start", u64).unwrap();
        let end = value_t!(matches, "end", u64).unwrap();
        println!("Send {:?}-{:?} ETH to {:?} ", start, end, to_address);
        tokio::run(future::lazy(move || {
            shell.connect();
            shell.send_transaction(&to_address, start, end);
            println!("Sent!!!");
            Ok(())
        }));
    }
}
