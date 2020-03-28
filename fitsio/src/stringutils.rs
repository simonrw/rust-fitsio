use crate::errors::Result;
use fitsio_sys::ffgerr;
use libc::{c_char, c_int, size_t};
use std::ffi::{CStr, CString};

/// Helper function converting a C string pointer to Rust String
pub fn buf_to_string(buffer: &[c_char]) -> Result<String> {
    let c_str = unsafe { CStr::from_ptr(buffer.as_ptr()) };
    Ok(c_str.to_str()?.to_string())
}

#[repr(C)]
pub struct StringList {
    pub len: size_t,
    cap: size_t,
    mem: Vec<*mut c_char>,
}

impl StringList {
    pub fn from_slice(stringvec: &[String]) -> Result<Self> {
        let converted: Vec<*mut c_char> = stringvec
            .iter()
            .map(|x| CString::new(x.clone()).unwrap().into_raw())
            .collect();
        let listlen = converted.len();
        let listcap = converted.capacity();

        let stringlist = StringList {
            len: listlen,
            cap: listcap,
            mem: converted,
        };

        Ok(stringlist)
    }

    pub fn as_ptr(&self) -> *mut *mut c_char {
        self.mem.as_slice().as_ptr() as *mut _
    }
}

/// Internal function to get the fits error description from a status code
pub fn status_to_string(status: c_int) -> Result<Option<String>> {
    match status {
        0 => Ok(None),
        status => {
            let mut buffer: Vec<c_char> = vec![0; 31];
            unsafe {
                ffgerr(status, buffer.as_mut_ptr());
            }
            let result_str = buf_to_string(&buffer)?;
            Ok(Some(result_str))
        }
    }
}

#[cfg(test)]
mod test {
    use super::status_to_string;

    #[test]
    fn test_returning_error_messages() {
        assert_eq!(
            status_to_string(105).unwrap().unwrap(),
            "couldn't create the named file"
        );
    }
}
