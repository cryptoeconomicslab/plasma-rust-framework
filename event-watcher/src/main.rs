extern crate ethabi;
extern crate futures;
extern crate tokio;

use ethabi::{Event, EventParam, ParamType};
use ethereum_types::Address;
use event_watcher::event_db::DefaultEventDb;
use event_watcher::event_watcher::EventWatcher;
use futures::future;

fn main() {
    println!("Watcher started");
    let address: Address = match "aF712Cc731F120d5f6c7dA8CF1D09b5fB7dCd38c".parse() {
        Ok(v) => v,
        Err(e) => panic!(e),
    };

    let abi: Vec<Event> = vec![Event {
        name: "StoreValue".to_owned(),
        inputs: vec![EventParam {
            name: "value".to_owned(),
            kind: ParamType::Uint(256),
            indexed: false,
        }],
        anonymous: false,
    }];

    let db = DefaultEventDb::new();
    let mut watcher = EventWatcher::new("http://localhost:9545", address, abi, db);

    watcher.subscribe(Box::new(|log| {
        println!("{:?}", log);
    }));

    tokio::run(future::lazy(|| {
        tokio::spawn(watcher);
        Ok(())
    }));
}
