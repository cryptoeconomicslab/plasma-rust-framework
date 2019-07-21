extern crate ethabi;
extern crate futures;
extern crate tokio;

use ethabi::{Event, EventParam, ParamType};
use ethereum_types::Address;
use event_watcher::event_db::EventDbImpl;
use event_watcher::event_watcher::EventWatcher;
use futures::Future;
use plasma_db::impls::kvs::memory::CoreDbMemoryImpl;
use plasma_db::traits::DatabaseTrait;

fn main() {
    println!("Watcher started");
    // Use MockDeposit.sol
    // https://github.com/cryptoeconomicslab/plasma-predicates/blob/master/contracts/deposit/MockDeposit.sol
    let address: Address = match "8f0483125FCb9aaAEFA9209D8E9d7b9C8B9Fb90F".parse() {
        Ok(v) => v,
        Err(e) => panic!(e),
    };

    let abi: Vec<Event> = vec![Event {
        name: "CheckpointFinalized".to_owned(),
        inputs: vec![EventParam {
            name: "checkpointId".to_owned(),
            kind: ParamType::FixedBytes(32),
            indexed: false,
        }],
        anonymous: false,
    }];

    let kvs = CoreDbMemoryImpl::open("kvs");
    let db = EventDbImpl::from(kvs);
    let mut watcher = EventWatcher::new("http://localhost:8545", address, abi, db);

    let logs = watcher.poll().ok().unwrap();
    println!("{:?}", logs);
    /*
    tokio::run(future::lazy(|| {
        tokio::spawn(watcher);
        Ok(())
    }));
    */
}
