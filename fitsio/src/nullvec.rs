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

impl<T> NullVec<T>
where
    // Do we really want to specify Copy here?
    T: Default + Clone + Copy,
{
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

    pub fn push(&mut self, value: Option<T>) {
        match value {
            Some(v) => {
                self.data.push(v);
                self.nullvals.push(true);
            }
            None => {
                self.data.push(Default::default());
                self.nullvals.push(false);
            }
        }
    }

    pub fn set(&mut self, idx: usize, value: Option<T>) {
        assert!(self.len() >= (idx + 1));
        match value {
            Some(v) => {
                self.data[idx] = v;
                self.nullvals.set(idx, true);
            }
            None => {
                self.data[idx] = Default::default();
                self.nullvals.set(idx, false);
            }
        }
    }

    pub fn get(&mut self, idx: usize) -> Option<T> {
        assert!(self.len() >= (idx + 1));
        if self.nullvals[idx] {
            Some(self.data[idx])
        } else {
            None
        }
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

    #[test]
    fn test_appending() {
        let mut n: NullVec<i32> = NullVec::new();
        n.push(Some(10i32));
        assert_eq!(n.len(), 1);

        n.push(None);
        assert_eq!(n.len(), 2);
    }

    #[test]
    fn test_setting_valid_value() {
        let mut n: NullVec<i32> = NullVec::new();
        n.push(Some(10i32));
        assert_eq!(n.len(), 1);

        n.set(0, Some(10i32));
        assert_eq!(n.len(), 1);
    }

    #[test]
    fn test_setting_invalid_value() {
        let mut n: NullVec<i32> = NullVec::new();
        n.push(Some(10i32));
        assert_eq!(n.len(), 1);

        n.set(0, None);
        assert_eq!(n.len(), 1);
        assert_eq!(n.get(0), None);
    }

    #[test]
    fn test_fetching_at_index() {
        let mut n: NullVec<i32> = NullVec::new();
        n.push(Some(10i32));
        assert_eq!(n.len(), 1);

        assert_eq!(n.get(0), Some(10i32));
    }
}
