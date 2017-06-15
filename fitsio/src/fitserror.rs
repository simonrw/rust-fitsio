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
                Err(Error::Fits(FitsError {
                    status: $status,
                    // unwrap guaranteed to work as we know $status > 0
                    message: stringutils::status_to_string($status).unwrap().unwrap(),
                }))
            }
        }
    }
}
