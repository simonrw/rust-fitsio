//! Header values (values + comemnts)
//!

use crate::errors::Result;
use std::fmt::Debug;

use super::ReadsKey;

/// Struct representing a FITS header value
pub struct HeaderValue<T> {
    /// Value of the header card
    pub value: T,

    /// Optional comment of the header card
    pub comment: Option<String>,
}

// Allow printing of `HeaderValue`s
impl<T> Debug for HeaderValue<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeaderValue")
            .field("value", &self.value)
            .field("comment", &self.comment)
            .finish()
    }
}

// Allow comparing of `HeaderValue`'s where the `value` is equatable
// so that e.g. `HeaderValue<f64>` can be compared to `f64`
impl<T> PartialEq<T> for HeaderValue<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}

/// Allow `HeaderValue` to be clnned
impl<T> Clone for HeaderValue<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        HeaderValue {
            value: self.value.clone(),
            comment: self.comment.clone(),
        }
    }
}

impl<T> Default for HeaderValue<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            value: Default::default(),
            comment: Default::default(),
        }
    }
}

impl<T> HeaderValue<T>
where
    T: ReadsKey,
{
    /// Map the _value_ of a [`HeaderValue`] to another form
    pub fn map<U, F>(self, f: F) -> HeaderValue<U>
    where
        F: FnOnce(T) -> U,
    {
        HeaderValue {
            value: f(self.value),
            comment: self.comment,
        }
    }

    /// Monadic "bind" for [`HeaderValue`]
    pub fn and_then<U, F>(self, f: F) -> Result<HeaderValue<U>>
    where
        F: FnOnce(T) -> Result<U>,
    {
        match f(self.value) {
            Ok(value) => Ok(HeaderValue {
                value,
                comment: self.comment,
            }),
            Err(e) => Err(e),
        }
    }
}
