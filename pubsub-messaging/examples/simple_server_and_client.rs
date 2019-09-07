use bincode::serialize;
use pubsub_messaging::{
    connect, spawn_server, ClientHandler, Message, Sender, ServerHandler, WsMessage,
};
use std::{thread, time};

#[derive(Clone)]
struct Handle();

impl ServerHandler for Handle {
    fn handle_message(&mut self, msg: Message, sender: Sender) {
        println!("SERVER_RECEIVE_MESSAGE: {:?}", msg);
        let msg = WsMessage::Binary(serialize(&msg).unwrap());
        let _ = sender.broadcast(msg);
    }
}

#[derive(Clone)]
struct ClientHandle();

impl ClientHandler for ClientHandle {
    fn handle_message(&self, msg: Message, _sender: Sender) {
        println!("ClientHandler handle_message: {:?}", msg);
    }
}

fn main() {
    let handler = Handle();
    if let Ok(server) = spawn_server("127.0.0.1:8080", handler) {
        println!("server listening on port 8080!");

        let c1 = thread::spawn(|| {
            let handler = ClientHandle();
            let mut client = connect("127.0.0.1:8080", handler).unwrap();

            let t = time::Duration::from_millis(3000);
            thread::sleep(t);
            let msg = Message::new("SERVER".to_string(), b"aa".to_vec());
            client.send(msg);
        });

        let c2 = thread::spawn(|| {
            let handler = ClientHandle();
            let mut client = connect("127.0.0.1:8080", handler).unwrap();

            let t = time::Duration::from_millis(5000);
            thread::sleep(t);
            let msg = Message::new("SERVER".to_string(), b"ccaa".to_vec());
            client.send(msg);
        });

        let t = time::Duration::from_millis(5000);
        thread::sleep(t);
        let msg = Message::new("ALL".to_string(), b"Hi, broadcast from server".to_vec());
        server.broadcast(msg);
        let _ = server.handle.join();
        let _ = c1.join();
        let _ = c2.join();
    }
}
