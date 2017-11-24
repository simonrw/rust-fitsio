//! Errors and error handling
//!
//! This mostly concerns converting to and from the main error type defined
//! in this crate: [`Error`](enum.Error.html)

use std::ffi::NulError;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use fitserror::FitsError;

/// Enumeration of all error types
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Internal Fits errors
    Fits(FitsError),

    /// Generic errors from simple strings
    Message(String),

    /// String conversion errors
    Null(NulError),

    /// UTF-8 conversion errors
    Utf8(Utf8Error),
}

/// Handy error type for use internally
pub type Result<T> = ::std::result::Result<T, Error>;

impl ::std::convert::From<FitsError> for Error {
    fn from(error: FitsError) -> Self {
        Error::Fits(error)
    }
}

impl<'a> ::std::convert::From<&'a str> for Error {
    fn from(error: &'a str) -> Self {
        Error::Message(error.to_string())
    }
}

impl ::std::convert::From<NulError> for Error {
    fn from(error: NulError) -> Self {
        Error::Null(error)
    }
}

impl ::std::convert::From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::Utf8(error.utf8_error())
    }
}

impl ::std::convert::From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::Utf8(error)
    }
}

impl ::std::convert::From<Box<::std::error::Error>> for Error {
    fn from(error: Box<::std::error::Error>) -> Self {
        let description = error.description();
        let message = match error.cause() {
            Some(msg) => format!("Error: {} caused by {}", description, msg),
            None => format!("Error: {}", description),
        };
        Error::Message(message)
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            Error::Fits(ref e) => write!(f, "Fits error: {:?}", e),
            Error::Message(ref s) => write!(f, "Error: {}", s),
            Error::Null(ref e) => write!(f, "Error: {}", e),
            Error::Utf8(ref e) => write!(f, "Error: {}", e),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        "fitsio error"
    }
}
