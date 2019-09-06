use clients::plasma::PlasmaAggregator;
use ethereum_types::Address;
use plasma_core::data_structure::abi::Decodable;
use plasma_core::data_structure::Transaction;
use plasma_db::impls::kvs::CoreDbMemoryImpl;
use plasma_db::impls::rangedb::RangeDbImpl;
use plasma_db::traits::db::DatabaseTrait;
use pubsub_messaging::{spawn_server, Message, Sender, ServerHandler};
use std::sync::Arc;

#[derive(Clone)]
struct Handle {
    plasma_aggregator: Arc<PlasmaAggregator<CoreDbMemoryImpl>>,
}

impl ServerHandler for Handle {
    fn handle_message(&mut self, msg: Message, _sender: Sender) {
        let agg = Arc::get_mut(&mut self.plasma_aggregator).unwrap();
        let _ = agg.ingest_transaction(Transaction::from_abi(&msg.message).unwrap());
        agg.show_queued_state_updates();
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
