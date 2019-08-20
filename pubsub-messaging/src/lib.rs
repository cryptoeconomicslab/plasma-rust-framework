#[macro_use]
extern crate failure;
extern crate bincode;
extern crate serde;
extern crate ws;

pub mod client;
pub mod error;
pub mod message;
pub mod server;

pub use client::client::connect;
pub use client::handler::Handler as ClientHandler;
pub use error::{Error, Result};
pub use message::Message;
pub use server::{spawn_server, Handler as ServerHandler};
pub use ws::{Message as WsMessage, Sender};
