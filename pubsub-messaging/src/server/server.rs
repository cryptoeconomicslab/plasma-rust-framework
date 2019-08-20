use super::{Error, Handler, Message, Result};
use bincode::{deserialize, serialize};
use std::marker::{Send, Sync};
use std::sync::mpsc::channel;
use std::thread::{spawn, JoinHandle};
use ws::{
    CloseCode, Error as WsError, Handler as WsHandler, Handshake, Message as WsMessage,
    Result as WsResult, Sender, WebSocket,
};

#[derive(Clone)]
struct Inner<T: Handler> {
    handler: T,
    ws: Sender,
}

impl<T> WsHandler for Inner<T>
where
    T: Handler,
{
    fn on_open(&mut self, _: Handshake) -> WsResult<()> {
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

/// Server struct
/// abstract Sender struct of ws-rs.
pub struct Server {
    pub sender: Sender,
    pub handle: JoinHandle<()>,
}

impl Server {
    /// Broad message to all connections
    pub fn broadcast(&self, msg: Message) {
        let ws_msg = WsMessage::Binary(serialize(&msg).unwrap());
        // TODO: error handling
        let _ = self.sender.send(ws_msg);
    }
}

/// spawn server event loop and returns ws connection and join handle
///
/// # Example
/// ```
/// use pubsub_messaging::spawn_server;
///
/// #[derive(Clone)]
/// pub struct Handle();
/// impl Handler for Handle {
///     pub fn handle_message(&self, msg: Message, sender: Sender) {
///         println!("{:?}", msg)
///     }
/// }
///
/// if let Ok((server, handle)) = spawn_server("127.0.0.1:8080".to_string(), handler) {
///     println!("server is listening on port 8080");
/// }
/// ```
pub fn spawn_server<T: Handler + Clone + Send + Sync + 'static>(
    host: &'static str,
    handler: T,
) -> Result<Server> {
    let (tx, rx) = channel();
    let ws = WebSocket::new(move |out: Sender| Inner {
        handler: handler.clone(),
        ws: out,
    })
    .unwrap();

    let t = spawn(move || {
        // TODO: handle result
        let _ = tx.send(ws.broadcaster());
        // TODO: handle result
        let _ = ws.listen(host);
    });

    if let Ok(sender) = rx.recv() {
        Ok(Server { sender, handle: t })
    } else {
        Err(Error::Thread)
    }
}
