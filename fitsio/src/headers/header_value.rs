//! Header values (values + comemnts)
//!

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
///
/// ```rust
/// # use fitsio::headers::HeaderValue;
/// let mut hv = HeaderValue {
///   value: 1u16,
///   comment: None,
/// };
/// let hv2 = hv.clone();
///
/// hv.value = 10;
/// assert_eq!(hv2.value, 1);
/// ```
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

/// Default value of `HeaderValue<T>`
///
/// ```rust
/// # use fitsio::headers::HeaderValue;
/// let hv = HeaderValue::<i32>::default();
/// assert_eq!(hv.value, 0);
/// ```
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
    /// ```rust
    /// # use fitsio::headers::HeaderValue;
    /// let hv = HeaderValue { value: 1, comment: None };
    /// let hv2 = hv.map(|value| value * 2);
    /// assert_eq!(hv2.value, 2);
    /// ```
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
    /// ```rust
    /// # use fitsio::headers::HeaderValue;
    /// let hv = HeaderValue { value: 1, comment: None };
    /// let hv2 = hv.and_then(|value| HeaderValue {
    ///     value: value * 2,
    ///     comment: Some("ok".to_string()),
    /// });
    /// assert_eq!(hv2.value, 2);
    /// assert_eq!(hv2.comment, Some("ok".to_string()));
    /// ```
    pub fn and_then<U, F>(self, f: F) -> HeaderValue<U>
    where
        F: FnOnce(T) -> HeaderValue<U>,
    {
        f(self.value)
    }
}
