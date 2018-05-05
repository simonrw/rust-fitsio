/*! Null vector implementation

  */

#![allow(missing_docs)]

use bit_vec::BitVec;

/** Vector capable of storing NULL values
 *
 * The API should mirror the `Vec` API where possible, but the `Index` implementation returns
 * `Option` values.possible.
 */
pub struct NullVec<T> {
    data: Vec<T>,
    nullvals: BitVec,
}

// TODO: check that data and nullvals are the same length

impl<T> NullVec<T> {
    /// Create a new null vector
    pub fn new() -> Self {
        NullVec {
            data: Vec::new(),
            nullvals: BitVec::new(),
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        NullVec {
            data: Vec::with_capacity(n),
            nullvals: BitVec::with_capacity(n),
        }
    }

    pub fn len(&self) -> usize {
        assert_eq!(self.data.len(), self.nullvals.len());
        self.data.len()
    }

    pub fn capacity(&self) -> usize {
        assert_eq!(self.data.len(), self.nullvals.len());
        self.data.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let n: NullVec<i32> = NullVec::new();
        assert_eq!(n.len(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let n: NullVec<i32> = NullVec::with_capacity(10);
        assert_eq!(n.len(), 0);
        assert_eq!(n.capacity(), 10);
    }
}
