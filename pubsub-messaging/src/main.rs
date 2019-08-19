use pubsub_messaging::{spawn_server, Handler, Message, Sender, WsMessage};
use std::{thread, time};

#[derive(Clone)]
struct Handle();

impl Handler for Handle {
    fn handle_message(&self, msg: Message, sender: Sender) {
        println!("{:?}", msg);
        let msg = WsMessage::Text("Hello, Alice".to_string());
        let _ = sender.broadcast(msg);
    }
}

fn main() {
    let handler = Handle();
    if let Ok((server, handle)) = spawn_server("127.0.0.1:8080".to_string(), handler) {
        println!("server listening on port 8080!");

        let t = time::Duration::from_millis(5000);
        thread::sleep(t);
        let msg = WsMessage::Text("Broadcast from server.".to_string());
        let _ = server.send(msg);
        let _ = handle.join();
    }
}
