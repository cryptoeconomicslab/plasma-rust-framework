//use pubsub_db::message::Message;
//use pubsub_db::{call, start_server};
//use std::thread;
//
//fn main() {
//    let server_thread = thread::spawn(move || {
//        start_server("127.0.0.1:8080");
//    });
//
//    let client_thread = thread::spawn(|| {
//        let msg = Message::new("0x000000".to_string(), b"Hi, this is Alice.".to_vec());
//        call("127.0.0.1:8080", msg);
//    });
//
//    let _ = server_thread.join();
//    let _ = client_thread.join();
//}
//



use pubsub_messaging::server::{ Server, Handler, Sender };
use pubsub_messaging::message::Message;

struct Handle();

impl Handler for Handle {
    fn handle_message(&self, msg: Message, sender: Sender) {
        println!("{:?}", msg);
        db.clone()
            .bucket(msg.to)
            .put(msg.id, msg.payload);

        sender.broadcast(msg);
    }
}


fn main() {
    let db = SomeDB();
    let handler = Handle();
    let mut server = Server::new("127.0.0.1:8080", handler);

    // broadcast to connected clients
    server.broadcast();

    // spawn and run server
    server.run();
}

