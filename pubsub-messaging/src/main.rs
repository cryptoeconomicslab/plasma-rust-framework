use bincode::{ serialize, deserialize };
use pubsub_messaging::{connect, spawn_server, ServerHandler, ClientHandler, Message, Sender, WsMessage};
use std::{thread, time};

#[derive(Clone)]
struct Handle();

impl ServerHandler for Handle {
    fn handle_message(&self, msg: Message, sender: Sender) {
        let msg = WsMessage::Binary(serialize(&msg).unwrap());
        let _ = sender.broadcast(msg);
    }
}

#[derive(Clone)]
struct ClientHandle();

impl ClientHandler for ClientHandle {
    fn handle_message(&self, msg: Message, sender: Sender) {
        println!("ClientHandler handle_message: {:?}", msg);
    }
} 

fn main() {
    let handler = Handle();
    if let Ok((server, handle)) = spawn_server("127.0.0.1:8080".to_string(), handler) {
        println!("server listening on port 8080!");

        let c1 = thread::spawn(|| {
            let handler = ClientHandle();
            let mut client= connect("127.0.0.1:8080".to_string(), handler).unwrap();

            let t = time::Duration::from_millis(3000);
            thread::sleep(t);
            let msg = Message::new("client".to_string(), b"aa".to_vec());
            client.send(msg);
        });

        let t = time::Duration::from_millis(5000);
        thread::sleep(t);
        let msg = Message::new("Server".to_string(), b"Hi, broadcast from server".to_vec());
        let ws_msg = WsMessage::Binary(serialize(&msg).unwrap());
        let _ = server.send(ws_msg);
        let _ = handle.join();
    }
}
