//! Fits HDU related code

use fitsio_sys::{FLEN_COMMENT, FLEN_KEYWORD, FLEN_VALUE};

use crate::errors::Result;
use std::ffi::CStr;

/// Wraps a single header card
#[derive(Debug)]
pub struct Card {
    pub(crate) name:       [i8;FLEN_KEYWORD as usize],
    pub(crate) value:      [i8;FLEN_VALUE as usize],
    pub(crate) comment:    [i8;FLEN_COMMENT as usize],
}

impl Card {
    /// Create a new empty card.
    pub fn new() -> Card {
        Card {
            name:       [0i8;FLEN_KEYWORD as usize],
            value:      [0i8;FLEN_VALUE as usize],
            comment:    [0i8;FLEN_COMMENT as usize],
        }
    }

    /// Header keyword.
    pub fn name(&self) -> Result<&str> {
        Ok(unsafe { CStr::from_ptr(self.name.as_ptr()) }.to_str()?)
    }

    /// Header comment.
    pub fn comment(&self) -> Result<&str> {
        Ok(unsafe { CStr::from_ptr(self.comment.as_ptr()) }.to_str()?)
    }

    /// Header value as a &str.
    pub fn str_value(&self) -> Result<&str> {
        Ok(unsafe { CStr::from_ptr(self.value.as_ptr()) }.to_str()?)
    }

    pub(crate) fn set_comment(&mut self, comment: String) {
        self.comment.fill(0); // clear the buffer before using it, ensure null termination
        let mut i = 0;
        for b in comment.into_bytes() {
            self.comment[i] = b as i8;
            i += 1;
            if i >= self.comment.len() - 1 { // C string must be null terminated
                break
            }
        }
    }
}
