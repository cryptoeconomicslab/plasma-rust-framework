use super::{Error, Handler, Result, Message};
use bincode::{deserialize, serialize};
use std::marker::{Send, Sync};
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as ThreadOut;
use std::thread::{spawn, JoinHandle};
use ws::{
    connect as ws_connect, Error as WsError, Handler as WsHandler, Handshake,
    Message as WsMessage, Result as WsResult, Sender,
};

pub struct Client<T: Handler> {
    handler: T,
    ws: Sender,
    tx: ThreadOut<Sender>,
}

impl<T> WsHandler for Client<T>
where
    T: Handler,
{
    fn on_open(&mut self, _: Handshake) -> WsResult<()> {
        // TODO: handle error
        let _ = self.tx.send(self.ws.clone());
        self.handler.handle_open(self.ws.clone());
        Ok(())
    }

    fn on_message(&mut self, msg: WsMessage) -> WsResult<()> {
        let res = match msg {
            WsMessage::Text(text) => deserialize(text.as_bytes()),
            WsMessage::Binary(bytes) => deserialize(&bytes),
        };

        match res {
            Ok(message) => {
                self.handler.handle_message(message, self.ws.clone());
                Ok(())
            }
            Err(e) => Err(WsError::from(e)),
        }
    }
}

pub struct ClientWrapper {
    pub sender: Sender,
    pub handle: JoinHandle<()>,
}

impl ClientWrapper {
    pub fn send(&mut self, msg: Message) {
        let ws_msg = WsMessage::Binary(serialize(&msg).unwrap());
        // TODO: error handling
        let _ = self.sender.send(ws_msg);
    }
}

pub fn connect<T: Handler + Clone + Send + Sync + 'static>(
    host: String,
    handler: T,
) -> Result<ClientWrapper> {
    let (tx, rx) = channel();
    let t = spawn(move || {
        let url: &str = &format!("ws://{}", host);
        ws_connect(url, |out| Client {
            handler: handler.clone(),
            ws: out,
            tx: tx.clone(),
        })
        .unwrap();
    });

    if let Ok(sender) = rx.recv() {
        Ok(ClientWrapper { sender, handle: t })
    } else {
        Err(Error::Thread)
    }
}
