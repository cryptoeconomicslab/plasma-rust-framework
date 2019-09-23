#[macro_use]
extern crate clap;

use bytes::Bytes;
use clap::{App, Arg, SubCommand};
use ethereum_types::Address;
use futures::future;
use plasma_clients::plasma::PlasmaClientShell;

fn main() {
    let matches = App::new("OVM Wallet!!!")
        .version("1.0")
        .author("CryptoeconomicsLab. Inc")
        .about("Does awesome things")
        .arg(
            Arg::with_name("session")
                .short("ss")
                .value_name("session")
                .takes_value(true)
                .help("session string"),
        )
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
            SubCommand::with_name("import")
                .about("import account")
                .version("1.0")
                .arg(
                    Arg::with_name("secret_key")
                        .short("sk")
                        .long("secret")
                        .value_name("secret_key")
                        .takes_value(true)
                        .help("hex secret key"),
                ),
        )
        .subcommand(
            SubCommand::with_name("send")
                .about("send money")
                .version("1.0")
                .arg(
                    Arg::with_name("token")
                        .short("d")
                        .value_name("token")
                        .takes_value(true)
                        .help("deposit contract address"),
                )
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
    let session_str = value_t!(matches, "session", String).unwrap();
    let mut shell =
        PlasmaClientShell::new("127.0.0.1:8080".to_string(), commitment_contract_address);

    if matches.subcommand_matches("balance").is_some() {
        let eth_address = Address::zero();
        tokio::run(future::lazy(move || {
            shell.connect();
            println!(
                "Your balance is {:?} ETH",
                shell
                    .get_balance(&string_to_session(session_str))
                    .get(&eth_address)
                    .unwrap_or(&0)
            );
            Ok(())
        }));
    } else if matches.subcommand_matches("init").is_some() {
        tokio::run(future::lazy(move || {
            shell.connect();
            shell.initialize();
            Ok(())
        }));
    } else if let Some(matches) = matches.subcommand_matches("import") {
        let secret_key = value_t!(matches, "secret_key", String).unwrap();
        tokio::run(future::lazy(move || {
            shell.connect();
            let (session, _key) = shell.import_account(&secret_key);
            println!("session: {}", hex::encode(session.to_vec()));
            Ok(())
        }));
    } else if let Some(matches) = matches.subcommand_matches("send") {
        let token_address_opt = value_t!(matches, "token", String)
            .map(|a| Address::from_slice(&hex::decode(&a).unwrap()))
            .ok();
        let to_address =
            Address::from_slice(&hex::decode(&value_t!(matches, "to", String).unwrap()).unwrap());
        let start = value_t!(matches, "start", u64).unwrap();
        let end = value_t!(matches, "end", u64).unwrap();
        println!(
            "Send {:?}-{:?} token={:?} to {:?} ",
            start, end, token_address_opt, to_address
        );
        tokio::run(future::lazy(move || {
            shell.connect();
            shell.send_transaction(
                &string_to_session(session_str),
                to_address,
                token_address_opt,
                start,
                end,
            );
            println!("Sent!!!");
            Ok(())
        }));
    }
}

fn string_to_session(session: String) -> Bytes {
    Bytes::from(hex::decode(session).unwrap())
}
