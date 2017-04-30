use sys::ffgerr;
use libc::{c_char, c_int, size_t};
use std::string::FromUtf8Error;
use std::ffi::CString;
use std::mem;

/// Helper function converting a C string pointer to Rust String
pub fn buf_to_string(buffer: &[c_char]) -> Result<String, FromUtf8Error> {
    String::from_utf8(buffer
                          .iter()
                          .map(|&x| x as u8)
                          .filter(|&x| x != 0)
                          .collect())
}

#[repr(C)]
pub struct StringList {
    pub list: *mut *mut c_char,
    pub len: size_t,
    cap: size_t,
}

impl StringList {
    pub fn from_vec(stringvec: Vec<String>) -> Self {
        let mut converted: Vec<*mut c_char> = stringvec
            .iter()
            .map(|x| CString::new(x.clone()).unwrap().into_raw())
            .collect();
        let listlen = converted.len();
        let listcap = converted.capacity();
        let ptr = converted.as_mut_ptr();

        assert!(!ptr.is_null());

        let stringlist = StringList {
            list: ptr,
            len: listlen,
            cap: listcap,
        };
        mem::forget(converted);
        stringlist
    }
}

impl Drop for StringList {
    fn drop(&mut self) {
        unsafe {
            let v: Vec<*mut c_char> = Vec::from_raw_parts(self.list, self.len, self.cap);
            for ptr in v {
                let _ = CString::from_raw(ptr);
            }
        }
    }
}


/// Internal function to get the fits error description from a status code
pub fn status_to_string(status: c_int) -> Option<String> {
    match status {
        0 => None,
        status => {
            let mut buffer: Vec<c_char> = vec![0; 31];
            unsafe {
                ffgerr(status, buffer.as_mut_ptr());
            }
            let result_str = buf_to_string(&buffer).unwrap();
            Some(result_str)
        }
    }
}

#[cfg(test)]
mod test {
    use super::status_to_string;

    #[test]
    fn returning_error_messages() {
        assert_eq!(status_to_string(105).unwrap(),
                   "couldn't create the named file");
    }
}
