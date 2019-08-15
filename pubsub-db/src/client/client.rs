use super::message::Message;
use bincode;
use ws::{connect, CloseCode, Handler, Handshake, Message as WsMessage, Result, Sender};

pub struct Client {
    out: Sender,
    msg: Message,
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.out.send(bincode::serialize(&self.msg).unwrap())
    }

    fn on_message(&mut self, _msg: WsMessage) -> Result<()> {
        self.out.close(CloseCode::Normal)
    }
}

pub fn call(url: &str, msg: Message) {
    let url: &str = &format!("ws://{}", url);
    connect(url, |out| Client {
        out,
        msg: msg.clone(),
    })
    .unwrap()
}
