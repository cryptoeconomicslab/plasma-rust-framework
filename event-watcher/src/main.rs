extern crate ethabi;
extern crate futures;
extern crate tokio;

use ethabi::{Event, EventParam, ParamType};
use ethereum_types::Address;
use event_watcher::event_db::EventDbImpl;
use event_watcher::event_watcher::EventWatcher;
use futures::future;
use plasma_db::impls::kvs::memory::CoreDbMemoryImpl;
use plasma_db::traits::DatabaseTrait;

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

    let kvs = CoreDbMemoryImpl::open("kvs");
    let db = EventDbImpl::from(kvs);
    let mut watcher = EventWatcher::new("http://localhost:9545", address, abi, db);

    watcher.subscribe(Box::new(|log| {
        println!("event > {:?}", log.event_signature);
        // event > 0x90890809c654f11d6e72a28fa60149770a0d11ec6c92319d6ceb2bb0a4ea1a15

        let decoded_param = log.params.first().unwrap();
        println!(
            "param > {:?}: {:?}",
            decoded_param.event_param.name,
            decoded_param.token.clone().to_uint().unwrap()
        );
        // param > "value": 22469980537774239738630940880827529904616858526135975343779764542717423171395
    }));

    tokio::run(future::lazy(|| {
        tokio::spawn(watcher);
        Ok(())
    }));
}
