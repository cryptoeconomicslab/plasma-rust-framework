extern crate failure;
extern crate plasma_db;
extern crate ws;

pub mod server;
pub mod client;
mod error;

pub use server::start_server;
pub use client::call;
