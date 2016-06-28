#![allow(dead_code, unused_imports)]

extern crate fitsio_sys;
extern crate libc;

use std::ptr;
use std::ffi;

mod stringutils;

/// Error type
#[derive(Debug, PartialEq, Eq)]
pub struct FitsError {
    status: i32,
    message: String,
}

pub type Result<T> = std::result::Result<T, FitsError>;

pub struct FitsFile {
    fptr: *mut fitsio_sys::fitsfile,
    pub filename: String,
}

impl FitsFile {
    fn create(path: &str) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let c_filename = ffi::CString::new(path).unwrap();

        unsafe {
            fitsio_sys::ffinit(&mut fptr as *mut *mut fitsio_sys::fitsfile,
                   c_filename.as_ptr(),
                   &mut status);
        }

        match status {
            0 => {
                Ok(FitsFile {
                    fptr: fptr,
                    filename: path.to_string(),
                })
            }
            status => {
                Err(FitsError {
                    status: status,
                    message: stringutils::status_to_string(status).unwrap(),
                })
            }
        }
    }
}

impl Drop for FitsFile {
    fn drop(&mut self) {
        let mut status = 0;
        unsafe {
            fitsio_sys::ffclos(self.fptr, &mut status);
        }
    }
}

#[cfg(test)]
mod test {
    extern crate tempdir;
    use super::FitsFile;

    #[test]
    fn creating_a_new_file() {
        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");
        assert!(!filename.exists());

        match FitsFile::create(filename.to_str().unwrap()) {
            Ok(_) => assert!(filename.exists()),
            Err(e) => panic!("Error: {:?}", e),
        }
    }

}
