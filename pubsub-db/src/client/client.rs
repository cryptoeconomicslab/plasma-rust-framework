use ws::{connect, CloseCode, Handler, Handshake, Message, Result, Sender};

pub struct Client {
    out: Sender,
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.out.send("Connection opened")
    }

    fn on_message(&mut self, _msg: Message) -> Result<()> {
        self.out.close(CloseCode::Normal)
    }
}

pub fn call(url: &str) {
    let url: &str = &format!("ws://{}", url);
    connect(url, |out| Client { out }).unwrap()
}
