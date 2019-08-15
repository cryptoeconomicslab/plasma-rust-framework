use plasma_db::impls::kvs::CoreDbMemoryImpl;
use plasma_db::traits::db::DatabaseTrait;
use pubsub_db::db::MessageDb;
use pubsub_db::start_server;
use pubsub_db::message::Message;
use std::thread;
use bincode;

fn main() {
    let memory_db = CoreDbMemoryImpl::open("server");
    let db = MessageDb::new(memory_db);
    let child_server = thread::spawn(move || {
        start_server("127.0.0.1:8080", db);
    });

    let message = Message::new("address".to_string(), "hello, world");
    let encoded = bincode::serialize(&message).unwrap();
    println!("{:?}", encoded);
    let decoded: Message<String> = bincode::deserialize(&encoded[..]).unwrap();
    println!("{:?}", decoded);


    let _res2 = child_server.join();
}
