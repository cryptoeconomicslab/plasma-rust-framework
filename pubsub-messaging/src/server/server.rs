use super::{Error, Handler, Result};
use bincode::deserialize;
use std::marker::{Send, Sync};
use std::sync::mpsc::channel;
use std::thread::{spawn, JoinHandle};
use ws::{
    Error as WsError, Handler as WsHandler, Message as WsMessage, Result as WsResult, Sender,
    WebSocket,
};

#[derive(Clone)]
struct Server<T: Handler> {
    handler: T,
    ws: Sender,
}

impl<T> WsHandler for Server<T>
where
    T: Handler,
{
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
    host: String,
    handler: T,
) -> Result<(Sender, JoinHandle<()>)> {
    let (tx, rx) = channel();
    let ws = WebSocket::new(move |out: Sender| Server {
        handler: handler.clone(),
        ws: out,
    })
    .unwrap();

    let t = spawn(move || {
        // TODO: handle result
        let _ = tx.send(ws.broadcaster());
        // TODO: handle result
        let _ = ws.listen(&host.clone());
    });

    if let Ok(sender) = rx.recv() {
        Ok((sender, t))
    } else {
        Err(Error::Thread)
    }
}
