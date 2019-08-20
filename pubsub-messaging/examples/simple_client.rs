use bincode::{deserialize, serialize};
use pubsub_messaging::{connect, Client, Handler, Message, WsMessage};

#[derive(Clone)]
struct Handle();

impl Handler for Handle {
    fn handle_message(&self, msg: Message, sender: Sender) {
        let msg = WsMessage::Binary(serialize(&msg).unwrap());
        let _ = sender.send("receive message");
    }
}

fn main() {
    let handler = Handle();
    if let Ok((client, handle)) = connect("127.0.0.1:8080".to_string(), handler) {
        let msg = Message::new("Alice".to_string, "Hi, I'm Bob.".to_bytes());
        client.send(msg);
    }
}
