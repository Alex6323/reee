//! Errors

use crossbeam_channel::SendError;
use std::io;

/// An error.
#[derive(Debug)]
pub enum Error {
    /// A general application error.
    App(&'static str),
    /// A channel send erro.
    Send(SendError<String>),
    /// An I/O error.
    Io(io::Error),
}

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Self {
        Error::App(msg)
    }
}

impl From<SendError<String>> for Error {
    fn from(e: SendError<String>) -> Self {
        Error::Send(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
