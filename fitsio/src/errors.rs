/*!
Errors and error handling

This mostly concerns converting to and from the main error type defined
in this crate: [`Error`](enum.Error.html)
*/

use std::ffi::{IntoStringError, NulError};
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use std::ops::Range;
use std::io;
use stringutils::status_to_string;

/// Enumeration of all error types
#[derive(Debug)]
pub enum Error {
    /// Internal Fits errors
    Fits(FitsError),

    /// Invalid index error
    Index(IndexError),

    /// Generic errors from simple strings
    Message(String),

    /// String conversion errors
    Null(NulError),

    /// UTF-8 conversion errors
    Utf8(Utf8Error),

    /// IO errors
    Io(io::Error),

    /// String conversion errors
    IntoString(IntoStringError),

    /// File path already exists
    ExistingFile(String),
}

/// Error raised when the user requests invalid indexes for data
#[derive(Debug, PartialEq, Eq)]
pub struct IndexError {
    /// Error message
    pub message: String,

    /// The range requested by the user
    pub given: Range<usize>,
}

/// Handy error type for use internally
pub type Result<T> = ::std::result::Result<T, Error>;

impl ::std::convert::From<FitsError> for Error {
    fn from(error: FitsError) -> Self {
        Error::Fits(error)
    }
}

impl ::std::convert::From<IndexError> for Error {
    fn from(error: IndexError) -> Self {
        Error::Index(error)
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

impl ::std::convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl ::std::convert::From<IntoStringError> for Error {
    fn from(e: IntoStringError) -> Self {
        Error::IntoString(e)
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            Error::Fits(ref e) => write!(f, "Fits error: {:?}", e),
            Error::Message(ref s) => write!(f, "Error: {}", s),
            Error::Null(ref e) => e.fmt(f),
            Error::Utf8(ref e) => e.fmt(f),
            Error::Index(ref e) => write!(f, "Error: {:?}", e),
            Error::Io(ref e) => e.fmt(f),
            Error::IntoString(ref e) => e.fmt(f),
            Error::ExistingFile(ref filename) => write!(f, "File {} already exists", filename),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        "fitsio error"
    }
}

/**
Error type

`cfitsio` passes errors through integer status codes. This struct wraps this and its associated
error message.
*/
#[derive(Debug, PartialEq, Eq)]
pub struct FitsError {
    /// `cfitsio` error code
    pub status: i32,
    /// `cfitsio` message for error code
    pub message: String,
}

/// Function for chaining result types
pub fn check_status(status: i32) -> Result<()> {
    match status {
        0 => Ok(()),
        _ => Err(Error::Fits(FitsError {
            status,
            message: status_to_string(status)?.expect("guaranteed to be Some"),
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_status_ok() {
        assert!(check_status(0).is_ok());
    }

    #[test]
    fn test_check_status_ok_with_value() {
        assert_eq!(check_status(0).map(|_| 10i32).unwrap(), 10i32);
    }

    #[test]
    fn test_check_status_with_err() {
        assert!(check_status(105).map(|_| 10i32).is_err());
    }
}
