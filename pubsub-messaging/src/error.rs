#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "WebSocket fail")]
    Ws,
    #[fail(display = "IO Error")]
    Io,
    #[fail(display = "Invalid Message")]
    InvalidMessage,
    #[fail(display = "Thread")]
    Thread,
}

pub type Result<T> = std::result::Result<T, Error>;
