use std::result;
use super::stringutils::status_to_string;

/// Error type
///
/// `cfitsio` passes errors through integer status codes. This struct wraps this and its associated
/// error message.
#[derive(Debug, PartialEq, Eq)]
pub struct FitsError {
    pub status: i32,
    pub message: String,
}

/// Macro for returning a FITS error type
macro_rules! fits_try {
    ($status: ident, $e: expr) => {
        match $status {
            0 => Ok($e),
            _ => {
                Err(FitsError {
                    status: $status,
                    message: stringutils::status_to_string($status).unwrap(),
                })
            }
        }
    }
}

/// FITS specific result type
///
/// This is a shortcut for a result with `FitsError` as the error type
pub type Result<T> = result::Result<T, FitsError>;

pub fn status_to_error(status: i32) -> Result<()> {
    Err(FitsError {
        status: status,
        message: status_to_string(status).unwrap(),
    })
}
