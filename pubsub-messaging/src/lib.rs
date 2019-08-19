#[macro_use]
extern crate failure;
extern crate bincode;
extern crate serde;
extern crate ws;

pub mod client;
pub mod message;
pub mod server;

pub use client::client::call;
pub use message::Message;
pub use server::{spawn_server, Handler};
pub use ws::{Message as WsMessage, Sender};
