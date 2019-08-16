extern crate bincode;
extern crate serde;
extern crate ws;

pub mod client;
pub mod message;
pub mod server;

pub use client::client::call;
pub use message::Message;
pub use server::server::start_server;
