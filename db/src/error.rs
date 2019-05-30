/// error definition for plasma db.
use failure::{Backtrace, Context, Fail};
use leveldb::error::Error as LeveldbError;
use std::fmt;
use std::fmt::Display;
use std::io::Error as IoError;

#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "IO error")]
    Io,
    #[fail(display = "Dammy error")]
    Dammy,
    #[fail(display = "LevelDb error")]
    LevelDb,
}

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn new(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }

    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error {
            inner: error.context(ErrorKind::Io),
        }
    }
}

impl From<LeveldbError> for Error {
    fn from(error: LeveldbError) -> Error {
        Error {
            inner: error.context(ErrorKind::LevelDb),
        }
    }
}
