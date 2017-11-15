use errors::{Error, Result};
use stringutils::status_to_string;

/// Error type
///
/// `cfitsio` passes errors through integer status codes. This struct wraps this and its associated
/// error message.
#[derive(Debug, PartialEq, Eq)]
pub struct FitsError {
    pub status: i32,
    pub message: String,
}

/// Function for chaining result types
pub fn check_status(status: i32) -> Result<()> {
    match status {
        0 => Ok(()),
        _ => Err(Error::Fits(FitsError {
            status: status,
            message: status_to_string(status).unwrap().unwrap(),
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_status_ok() {
        assert_eq!(check_status(0), Ok(()));
    }

    #[test]
    fn test_check_status_ok_with_value() {
        assert_eq!(check_status(0).map(|_| 10i32), Ok(10i32));
    }

    #[test]
    fn test_check_status_with_err() {
        assert!(check_status(105).map(|_| 10i32).is_err());
    }
}
