use super::message::Message;
use super::Handler;
use bincode::deserialize;
use ws::{listen, Handler as WsHandler, Message as WsMessage, Result as WsResult, Sender};

// TODO: implement Custom Error
type Error = ();

pub struct Server<T: Handler> {
    handler: T,
    ws: Sender,
}

impl<T> WsHandler for Server<T>
where
    T: Handler,
{
    fn on_message(&mut self, _msg: WsMessage) -> WsResult<()> {
        println!("on_message: {:?}", _msg);
        // TODO: convert WsMessage to Message
        let msg = Message::new("Alice".to_string(), b"hey, this is Bob".to_vec());
        self.handler.handle_message(msg, self.ws.clone());
        Ok(())
    }
}

impl<T> Server<T>
where
    T: Handler,
{
    pub fn broadcast(&mut self, _msg: Message) -> Result<(), Error> {
        // TODO: convert Message to WsMessage
        let msg = WsMessage::Text("Hello, Alice".to_string());
        match self.ws.broadcast(msg) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }
}

pub fn run_server<T: Handler + Clone>(host: &str, handler: T) {
    listen(host, |out| Server {
        handler: handler.clone(),
        ws: out,
    });
}

//let server = Server::create_server(host, handler);
//server.listen();
//server.broadcast()

//
//pub fn listen<A, F, H>(addr: A, factory: F) -> Result<()>
//    where
//        A: ToSocketAddrs + fmt::Debug,
//        F: FnMut(Sender) -> H,
//        H: Handler,
//{
//    let ws = try!(WebSocket::new(factory));
//    try!(ws.listen(addr));
//    Ok(())
//}

//pub struct Inner<T: Handler> {
//    out: Sender,
//    handler: T
//}
//
//impl<T: Handler> WsHandler for Inner<T> {
//    fn on_message(&mut self, msg: WsMessage) -> WsResult<()> {
//        println!("on_message: {:?}", msg);
//        // TODO: convert WsMessage to Message
//        let msg = Message::new("Alice".to_string(), b"hey, this is Bob".to_vec());
//        self.handler.handle_message(msg, self.out.clone());
//        Ok(())
//    }
//}
//
//
//
//struct PubsubFactory<T: Handler> {
//    handler: T
//}
//
//impl<T> ws::Factory for PubsubFactory<T>
//where
//    T: Handler + Clone
//{
//    type Handler = Inner<T>;
//    fn connection_made(&mut self, sender: Sender) -> Self::Handler {
//        Inner {
//            out: sender,
//            handler: self.handler.clone(),
//        }
//    }
//}
//
//
//pub struct Server<T: Handler>
//where
//    PubsubFactory<T>: Factory
//{
//    host: String,
//    ws: WebSocket<PubsubFactory<T>>,
//}
//
//impl<T> Server<T>
//where
//    T: Handler + Clone
//{
//    pub fn new(host: &str, handler: &mut T) -> Self {
//        let f = PubsubFactory { handler: handler.clone() };
//        let ws = WebSocket::new(f).unwrap();
//
//        Self {
//            host: host.to_string(),
//            ws,
//        }
//    }
//
//    pub fn broadcast(&mut self, msg: Message) -> Result<(), Error> {
//        // TODO: convert Message to WsMessage
//        let msg = WsMessage::Text("Hello, Alice".to_string());
//        self.ws.broadcaster().broadcast(msg);
//        Ok(())
//    }
//
//    pub fn run(&mut self) -> Result<(), Error> {
//        self.ws.listen(&self.host).unwrap()
//    }
//}

//pub struct Server {
//    out: Sender,
//}
//
//impl Handler for Server {
//    /// receive message and broadcast the message to subscribers.
//    fn on_message(&mut self, message: WsMessage) -> WsResult<()> {
//        if let Ok(msg) = match message.clone() {
//            WsMessage::Text(payload) => deserialize(&payload.as_bytes()[..]),
//            WsMessage::Binary(payload) => deserialize(&payload[..]),
//        } {
//            let msg: Message = msg;
//            println!("{:?}", msg);
//            self.out.broadcast(message)
//        } else {
//            println!("deserialize fail");
//            Ok(())
//        }
//    }
//
//    fn on_close(&mut self, code: CloseCode, reason: &str) {
//        match code {
//            CloseCode::Normal => println!("The client is done with the connection."),
//            CloseCode::Away => println!("The client is leaving the site."),
//            _ => println!("The client encountered an error: {}", reason),
//        }
//    }
//}
//
///// start server in given host
//pub fn start_server(host: &str) {
//    listen(host, |out| Server { out }).unwrap();
//}
//
