use std::result;
use std::fmt;
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

/// Display implementation for FitsError
///
/// This enables the error to be printed in a user-facing way
impl fmt::Display for FitsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let full_message = format!("Fits status {}: {}", self.status, self.message);
        full_message.fmt(f)
    }
}

/// Error implementation for FitsError
///
/// This enables the fits error type to be treated as a Box<Error>
impl ::std::error::Error for FitsError {
    fn description(&self) -> &str {
        "fits error"
    }
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

#[cfg(test)]
mod test {
    use std::error::Error;
    use super::FitsError;

    #[test]
    fn fits_error_implements_error() {
        fn foo() -> ::std::result::Result<(), Box<Error>> {
            let err = FitsError {
                status: 100,
                message: "test".into(),
            };
            Err(err.into())
        }

        if let Err(e) = foo() {
            assert_eq!(format!("{}", e), "Fits status 100: test".to_string());
            assert_eq!(e.description(), "fits error");
        }
    }
}
