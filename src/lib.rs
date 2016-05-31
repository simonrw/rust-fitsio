pub mod raw;

extern crate libc;

use raw::*;
use libc::c_int;
use std::ptr;
use std::ffi;

pub struct FitsFile {
    fptr: *mut fitsfile,
    status: c_int,
}

impl FitsFile {
    pub fn open(filename: &str) -> FitsFile {
        let mut fptr: *mut fitsfile = ptr::null_mut();
        let mut status: c_int = 0;
        let c_filename = ffi::CString::new(filename).unwrap();

        unsafe {
            ffopen(&mut fptr as *mut *mut fitsfile,
                   c_filename.as_ptr(),
                   0,
                   &mut status);
        }

        return match status {
            0 => {
                FitsFile {
                    fptr: fptr,
                    status: status,
                }
            }
            status => panic!("Invalid status code: {}", status),
        };

    }

    /// Returns the current HDU number, 0-indexed
    ///
    /// The FITS standard is 1-indexed (thanks Fortran), so this function
    /// converts the indexing to be 0-based so the primary HDU is at
    /// index 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use fitsio::FitsFile;
    ///
    /// let f = FitsFile::open("testdata/full_example.fits");
    /// assert_eq!(f.current_hdu_number(), 0);
    /// ```
    pub fn current_hdu_number(&self) -> u32 {
        let mut hdu_num: c_int = 0;
        unsafe {
            ffghdn(self.fptr, &mut hdu_num);
        }
        assert!(hdu_num >= 1);
        (hdu_num - 1) as u32
    }

    pub fn change_hdu(&mut self, hdu_num: u32) {
        let mut _hdu_type: c_int = 0;
        unsafe {
            ffmahd(self.fptr,
                   (hdu_num + 1) as i32,
                   &mut _hdu_type,
                   &mut self.status);
        }
    }
}

impl Drop for FitsFile {
    fn drop(&mut self) {
        unsafe {
            ffclos(self.fptr, &mut self.status);
        }
    }
}

mod test {
    use super::FitsFile;

    #[test]
    fn opening_an_existing_file() {
        let f = FitsFile::open("testdata/full_example.fits");
        assert_eq!(f.status, 0);
    }

    #[test]
    fn change_hdu() {
        let mut f = FitsFile::open("testdata/full_example.fits");
        f.change_hdu(1);
        assert_eq!(f.current_hdu_number(), 1u32);
    }

    #[test]
    fn getting_current_hdu_number() {
        let f = FitsFile::open("testdata/full_example.fits");
        assert_eq!(f.current_hdu_number(), 0u32);
    }
}
