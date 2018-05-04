/*! Possibly nullable types
 */

/** Defines a possibly nullable type */
pub trait PossiblyNullable<T> {
    /// Get an index value
    fn get(&self, idx: usize) -> Option<T>;
}

impl<T> PossiblyNullable<T> for Vec<T> {
    fn get(&self, idx: usize) -> Option<T> {
        Some(self[idx])
    }
}
