use ethabi::Error as AbiDecodeError;
use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::fmt::Display;

#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "ABI Decode error")]
    AbiDecode,
}

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
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

impl From<AbiDecodeError> for Error {
    fn from(_error: AbiDecodeError) -> Error {
        Error {
            inner: Context::from(ErrorKind::AbiDecode),
        }
    }
}
