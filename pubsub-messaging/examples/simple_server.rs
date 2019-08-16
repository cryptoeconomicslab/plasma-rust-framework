use pubsub_messaging::server::{ Server, Handler };
use pubsub_messaging::{ Message, Sender };

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
    server.spawn();

    // shutdown server
    server.close();
}

