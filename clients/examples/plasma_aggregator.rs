use bincode::serialize;
use ethereum_types::Address;
use ovm::types::StateUpdateList;
use plasma_clients::plasma::PlasmaAggregator;
use plasma_core::data_structure::abi::Decodable;
use plasma_core::data_structure::abi::Encodable;
use plasma_core::data_structure::Transaction;
use plasma_db::impls::kvs::CoreDbMemoryImpl;
use pubsub_messaging::{spawn_server, Message, Sender, ServerHandler, WsMessage};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Handle {
    plasma_aggregator: Arc<Mutex<PlasmaAggregator<CoreDbMemoryImpl>>>,
}

impl ServerHandler for Handle {
    fn handle_message(&mut self, msg: Message, sender: Sender) {
        let mut agg = self.plasma_aggregator.lock().unwrap();
        let ingest_result = agg.ingest_transaction(Transaction::from_abi(&msg.message).unwrap());
        println!("{:?}", ingest_result);

        let state_updates = StateUpdateList::new(agg.get_all_state_updates());
        println!("STATE_UPDATES: {:?}", state_updates);

        let message = Message::new("BROADCAST".to_owned(), state_updates.to_abi().to_vec());

        let msg = WsMessage::Binary(serialize(&message).unwrap());
        let _ = sender.broadcast(msg);
    }
}

fn main() {
    let mut aggregator = PlasmaAggregator::new(
        Address::zero(),
        Address::zero(),
        Address::zero(),
        "c87509a1c067bbde78beb793e6fa76530b6382a4c0241e5e4a9ec0a0f44dc0d3",
    );
    aggregator.insert_test_ranges();

    let handle = Handle {
        plasma_aggregator: Arc::new(Mutex::new(aggregator)),
    };

    handle
        .plasma_aggregator
        .lock()
        .unwrap()
        .show_queued_state_updates();
    if let Ok(server) = spawn_server("127.0.0.1:8080".to_owned(), handle) {
        let _ = server.handle.join();
    }
}
