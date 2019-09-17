use super::{Error, Handler, Message, Result};
use bincode::{deserialize, serialize};
use std::marker::{Send, Sync};
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as ThreadOut;
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use ws::{
    connect as ws_connect, CloseCode, Error as WsError, Handler as WsHandler, Handshake,
    Message as WsMessage, Result as WsResult, Sender,
};

struct Inner<T: Handler> {
    handler: T,
    ws: Sender,
    tx: ThreadOut<Sender>,
}

impl<T> WsHandler for Inner<T>
where
    T: Handler,
{
    fn on_open(&mut self, _: Handshake) -> WsResult<()> {
        // TODO: handle error
        let _ = self.tx.send(self.ws.clone());
        self.handler.handle_open(self.ws.clone());
        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str) {
        self.handler.handle_close();
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

/// Client struct
/// abstract Sender struct of ws-rs.
#[derive(Clone)]
pub struct Client {
    pub sender: Sender,
    pub handle: Arc<JoinHandle<()>>,
}

impl Client {
    pub fn send(&mut self, msg: Message) {
        let ws_msg = WsMessage::Binary(serialize(&msg).unwrap());
        // TODO: error handling
        let _ = self.sender.send(ws_msg);
    }
}

/// create connection to given host returning Client.
///
/// ```
/// use pubsub_messaging::{ connect, ClientHandler, Sender, Message };
///
/// #[derive(Clone)]
/// struct Handle();
/// impl ClientHandler for Handle {
///     fn handle_message(&self, msg: Message, sender: Sender) {
///         println!("{:?}", msg)
///     }
/// }
///
/// let handle = Handle();
///
/// let client = connect("127.0.0.1:8080".to_owned(), handle);
/// ```
pub fn connect<T: Handler + Clone + Send + Sync + 'static>(
    host: String,
    handler: T,
) -> Result<Client> {
    let (tx, rx) = channel();
    let t = spawn(move || {
        let url: &str = &format!("ws://{}", host);
        ws_connect(url, |out| Inner {
            handler: handler.clone(),
            ws: out,
            tx: tx.clone(),
        })
        .unwrap();
    });

    if let Ok(sender) = rx.recv() {
        Ok(Client {
            sender,
            handle: Arc::new(t),
        })
    } else {
        Err(Error::Thread)
    }
}
