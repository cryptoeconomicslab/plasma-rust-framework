use ethabi::Error as AbiError;
use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::fmt::Display;

use web3::contract::Error as Web3Error;

#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "Invalid Input Type")]
    InvalidInputType,
    #[fail(display = "Failed to connect")]
    FailedToConnect,
    #[fail(display = "ABI error")]
    Abi,
    #[fail(display = "Web3 error")]
    Web3,
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

impl From<AbiError> for Error {
    fn from(_error: AbiError) -> Error {
        Error {
            inner: Context::new(ErrorKind::Abi),
        }
    }
}

impl From<Web3Error> for Error {
    fn from(_error: Web3Error) -> Error {
        Error {
            inner: Context::new(ErrorKind::Web3),
        }
    }
}
