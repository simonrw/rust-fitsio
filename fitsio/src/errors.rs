use fitserror::FitsError;

#[derive(Debug, PartialEq)]
pub enum Error {
    Fits(FitsError),
    Message(String),
}

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
