use super::db::MessageDb;
use super::message::Message;
use bincode::deserialize;
use plasma_db::traits::kvs::KeyValueStore;
use serde::de::Error as SerdeError;
use std::sync::Arc;
use ws::{listen, CloseCode, Handler, Message as WsMessage, Result as WsResult, Sender};

pub struct Server<KVS: KeyValueStore> {
    out: Sender,
    db: Arc<MessageDb<KVS>>,
}

impl<KVS: KeyValueStore> Handler for Server<KVS> {
    /// receive message and broadcast the message to subscribers.
    /// persist message in server database
    fn on_message(&mut self, message: WsMessage) -> WsResult<()> {
        println!("{:?}", message);
        if let Ok(msg) = match message.clone() {
            WsMessage::Text(payload) => deserialize(&payload.as_bytes()[..]),
            WsMessage::Binary(payload) => deserialize(&payload[..]),
            _ => panic!("msg is not deserializable"),
        } {
            let msg: Message = msg;
            println!("{:?}", msg);

            self.db.persist_message(msg.clone());
            self.out.broadcast(message)
        } else {
            println!("deserialize fail");
            Ok(())
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }
}

/// start server in given host
pub fn start_server<KVS: KeyValueStore>(host: &str, db: MessageDb<KVS>) {
    let db = Arc::new(db);

    listen(host, |out| Server {
        out,
        db: db.clone(),
    })
    .unwrap();
}
