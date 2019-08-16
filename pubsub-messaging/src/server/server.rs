use super::message::Message;
use bincode::deserialize;
use ws::{listen, CloseCode, Handler, Message as WsMessage, Result as WsResult, Sender};

pub struct Server {
    out: Sender,
}

impl Handler for Server {
    /// receive message and broadcast the message to subscribers.
    fn on_message(&mut self, message: WsMessage) -> WsResult<()> {
        if let Ok(msg) = match message.clone() {
            WsMessage::Text(payload) => deserialize(&payload.as_bytes()[..]),
            WsMessage::Binary(payload) => deserialize(&payload[..]),
        } {
            let msg: Message = msg;
            println!("{:?}", msg);
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
pub fn start_server(host: &str) {
    listen(host, |out| Server { out }).unwrap();
}
