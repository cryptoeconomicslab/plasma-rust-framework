#[macro_use]
extern crate clap;

use clap::{App, Arg, SubCommand};
use ethereum_types::Address;
use futures::future;
use plasma_clients::plasma::{utils::*, PlasmaClientShell};

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

    let session_str = value_t!(matches, "session", String).unwrap();
    let mut shell = PlasmaClientShell::new(
        "127.0.0.1:8080".to_string(),
        string_to_address("9FBDa871d559710256a2502A2517b794B482Db40"),
    );

    if matches.subcommand_matches("balance").is_some() {
        let eth_address = Address::zero();
        let dai_address = string_to_address("0000000000000000000000000000000000000001");
        tokio::run(future::lazy(move || {
            shell.connect();
            println!(
                "Balance\n\t{:?} ETH\n\t{:?} DAI",
                shell
                    .get_balance(&decode_session(session_str.clone()).unwrap())
                    .get(&eth_address)
                    .unwrap_or(&0),
                shell
                    .get_balance(&decode_session(session_str).unwrap())
                    .get(&dai_address)
                    .unwrap_or(&0),
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
            .map(|a| string_to_address(&a))
            .ok();
        let to_address = string_to_address(&value_t!(matches, "to", String).unwrap());
        let start = value_t!(matches, "start", u64).unwrap();
        let end = value_t!(matches, "end", u64).unwrap();
        println!(
            "Send {:?}-{:?} token={:?} to {:?} ",
            start, end, token_address_opt, to_address
        );
        tokio::run(future::lazy(move || {
            shell.connect();
            let session = &decode_session(session_str).unwrap();
            let (property, metadata) = shell.ownership_property(session, to_address);
            shell.send_transaction(session, token_address_opt, start, end, property, metadata);
            println!("Sent!!!");
            Ok(())
        }));
    }
}
