use bincode::serialize;
use clients::plasma::PlasmaAggregator;
use ethereum_types::Address;
use plasma_db::impls::kvs::CoreDbMemoryImpl;
use plasma_db::impls::rangedb::RangeDbImpl;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use pubsub_messaging::{connect, spawn_server, Message, Sender, ServerHandler};
use std::sync::Arc;

#[derive(Clone)]
struct Handle {
    plasma_aggregator: Arc<PlasmaAggregator<CoreDbMemoryImpl>>,
}

impl ServerHandler for Handle {
    fn handle_message(&self, msg: Message, sender: Sender) {
        println!("{:?}", msg);
    }
}

fn main() {
    let db = CoreDbMemoryImpl::open("aggregator_db");
    let range_db = RangeDbImpl::from(db);
    let aggregator = PlasmaAggregator::new(
        Address::zero(),
        Address::zero(),
        Address::zero(),
        "c87509a1c067bbde78beb793e6fa76530b6382a4c0241e5e4a9ec0a0f44dc0d3",
        range_db,
    );

    let handle = Handle {
        plasma_aggregator: Arc::new(aggregator),
    };

    if let Ok(server) = spawn_server("127.0.0.1:8080", handle) {
        let _ = server.handle.join();
    }
}
