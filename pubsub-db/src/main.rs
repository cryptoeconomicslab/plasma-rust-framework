use bincode;
use plasma_db::impls::kvs::CoreDbMemoryImpl;
use plasma_db::traits::db::DatabaseTrait;
use pubsub_db::db::MessageDb;
use pubsub_db::message::Message;
use pubsub_db::{call, start_server};
use std::thread;

fn main() {
    let memory_db = CoreDbMemoryImpl::open("server");
    let db = MessageDb::new(memory_db);
    let server_thread = thread::spawn(move || {
        start_server("127.0.0.1:8080", db);
    });

    let client_thread = thread::spawn(|| {
        let msg = Message::new("0x000000".to_string(), b"Hi, this is Alice.".to_vec());
        call("127.0.0.1:8080", msg);
    });

    let _ = server_thread.join();
    let _ = client_thread.join();
}
