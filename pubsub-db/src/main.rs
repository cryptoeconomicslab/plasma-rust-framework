use pubsub_db::{call, start_server};
use std::thread;

fn main() {
    let child_server = thread::spawn(move || {
        start_server("127.0.0.1:8080");
    });

//    let child_client = thread::spawn(move || {
//        call("127.0.0.1:8080");
//    });

//    let _res = child_client.join();
    let _res2 = child_server.join();
}
