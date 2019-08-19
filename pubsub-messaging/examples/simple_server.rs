use pubsub_messaging::message::Message;
use pubsub_messaging::server::{run_server, Handler, Server};
use ws::{Message as WsMessage, Sender};

#[derive(Clone)]
struct Handle();

impl Handler for Handle {
    fn handle_message(&self, msg: Message, sender: Sender) {
        println!("{:?}", msg);
        let msg = WsMessage::Text("Hello, Alice".to_string());
        sender.broadcast(msg);
    }
}

fn main() {
    let handler = Handle();
    run_server("127.0.0.1:8080", handler);
}
