//
// Created on Wed Jun 04 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

use ethabi::Error as AbiDecodeError;
use failure::{Backtrace, Context, Fail};
use hex::FromHexError;
use jsonrpc_core_client::RpcError;
use plasma_core::data_structure::error::Error as PlasmaCoreError;
use plasma_db::error::Error as PlasmaDbError;
use std::fmt;
use std::fmt::Display;
use std::io::Error as IoError;
use std::sync::PoisonError;

/// error definition for plasma core.
#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "IO error")]
    Io,
    #[fail(display = "ABI Decode error")]
    AbiDecode,
    #[fail(display = "Plasma Core")]
    PlasmaCore,
    #[fail(display = "Plasma Db error")]
    PlasmaDbError,
    #[fail(display = "Invalid transaction")]
    InvalidTransaction,
    #[fail(display = "Lock is acquired")]
    PoisonError,
    #[fail(display = "RPC error")]
    RpcError,
    #[fail(display = "Hex Error")]
    HexError,
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

impl From<AbiDecodeError> for Error {
    fn from(_error: AbiDecodeError) -> Error {
        Error {
            inner: Context::from(ErrorKind::AbiDecode),
        }
    }
}

impl From<PlasmaCoreError> for Error {
    fn from(error: PlasmaCoreError) -> Error {
        Error {
            inner: error.context(ErrorKind::PlasmaCore),
        }
    }
}

impl From<PlasmaDbError> for Error {
    fn from(error: PlasmaDbError) -> Error {
        Error {
            inner: error.context(ErrorKind::PlasmaDbError),
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_error: PoisonError<T>) -> Error {
        Error {
            inner: Context::from(ErrorKind::PoisonError),
        }
    }
}

impl From<RpcError> for Error {
    fn from(error: RpcError) -> Error {
        Error {
            inner: error.context(ErrorKind::RpcError),
        }
    }
}

impl From<FromHexError> for Error {
    fn from(error: FromHexError) -> Error {
        Error {
            inner: error.context(ErrorKind::HexError),
        }
    }
}
