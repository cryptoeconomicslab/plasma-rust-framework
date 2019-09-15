#[macro_use]
extern crate futures;

use bincode::serialize;
use ethereum_types::Address;
use futures::{future, Async, Future, Poll, Stream};
use ovm::types::StateUpdateList;
use plasma_clients::plasma::PlasmaAggregator;
use plasma_core::data_structure::abi::Decodable;
use plasma_core::data_structure::abi::Encodable;
use plasma_core::data_structure::Transaction;
use plasma_db::impls::kvs::CoreDbMemoryImpl;
use pubsub_messaging::{spawn_server, Message, Sender, ServerHandler, WsMessage};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::timer::Interval;

struct Handle {
    plasma_aggregator: Arc<Mutex<PlasmaAggregator<CoreDbMemoryImpl>>>,
    interval: Interval,
    interval_sec: u64,
}

impl Handle {
    fn new(plasma_aggregator: PlasmaAggregator<CoreDbMemoryImpl>, interval_sec: u64) -> Self {
        Self {
            plasma_aggregator: Arc::new(Mutex::new(plasma_aggregator)),
            interval: Interval::new_interval(Duration::from_secs(interval_sec)),
            interval_sec,
        }
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Self {
            plasma_aggregator: self.plasma_aggregator.clone(),
            interval: Interval::new_interval(Duration::from_secs(self.interval_sec)),
            interval_sec: self.interval_sec,
        }
    }
}
impl Stream for Handle {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        try_ready!(self.interval.poll().map_err(|_| ()));
        println!("start to submit merkle root");
        let agg = self.plasma_aggregator.lock().unwrap();
        if agg.submit_next_block().is_ok() {
            Ok(Async::Ready(Some(())))
        } else {
            Ok(Async::NotReady)
        }
    }
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

    let interval_second = 5;
    let handle = Handle::new(aggregator, interval_second);

    handle
        .plasma_aggregator
        .lock()
        .unwrap()
        .show_queued_state_updates();
    let h = handle.clone();
    tokio::run(future::lazy(move || {
        tokio::spawn(Worker { handle: h });
        if let Ok(server) = spawn_server("127.0.0.1:8080".to_owned(), handle.clone()) {
            let _ = server.handle.join();
        }
        Ok(())
    }));
}

struct Worker {
    pub handle: Handle,
}
impl Future for Worker {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let _ = try_ready!(self.handle.poll());
        }
    }
}
