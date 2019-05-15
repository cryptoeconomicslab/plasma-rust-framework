//
// Created on Wed May 08 2019
//
// Copyright (c) 2019 Cryptoeconomics Lab, Inc.
// This file is part of Plasma Chamber.
//

/// error definition for plasma chain.
use failure::{Backtrace, Context, Fail};
use plasma_core::data_structure::error::Error as PlasmaCoreError;
use std::fmt;
use std::fmt::Display;
use std::io::Error as IoError;
use std::net::AddrParseError;

#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "IO error")]
    Io,
    #[fail(display = "Parse error")]
    Parse,
    #[fail(display = "Plasma Core")]
    PlasmaCore,
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

impl From<AddrParseError> for Error {
    fn from(error: AddrParseError) -> Error {
        Error {
            inner: error.context(ErrorKind::Parse),
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
