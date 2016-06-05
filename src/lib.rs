#![allow(dead_code)]

pub mod raw;

extern crate libc;

use raw::*;
use libc::{c_int, c_char};
use std::ptr;
use std::ffi;

fn status_to_string(status: c_int) -> Option<String> {
    match status {
        0 => None,
        status => {
            let mut buffer: Vec<c_char> = vec![0; 31];
            unsafe {
                ffgerr(status, buffer.as_mut_ptr());
            }
            let result_str = String::from_utf8(buffer.iter()
                    .map(|&x| x as u8)
                    .filter(|&x| x != 0)
                    .collect())
                .unwrap();
            Some(result_str)
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum HduType {
    ImageHDU,
    AsciiTableHDU,
    BinTableHDU,
}

pub struct FitsFile {
    fptr: *mut fitsfile,
    status: c_int,
    pub filename: String,
}

impl FitsFile {
    pub fn open(filename: &str) -> FitsFile {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
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
                    filename: filename.to_string(),
                }
            }
            status => {
                panic!("Invalid status code: {}, msg: {}",
                       status,
                       status_to_string(status).unwrap())
            }
        };

    }

    pub fn check(&self) {
        match self.status {
            0 => {}
            status => {
                panic!("Status code {} encountered, msg: {}",
                       status,
                       status_to_string(status).unwrap())
            }
        }
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
        let mut hdu_num = 0;
        unsafe {
            ffghdn(self.fptr, &mut hdu_num);
        }
        self.check();
        assert!(hdu_num >= 1);
        (hdu_num - 1) as u32
    }

    pub fn change_hdu(&mut self, hdu_num: u32) {
        let mut _hdu_type = 0;
        unsafe {
            ffmahd(self.fptr,
                   (hdu_num + 1) as i32,
                   &mut _hdu_type,
                   &mut self.status);
        }
        self.check();
    }

    pub fn get_hdu_type(&mut self) -> HduType {
        let mut hdu_type = 3;
        unsafe {
            ffghdt(self.fptr, &mut hdu_type, &mut self.status);
        }
        self.check();

        match hdu_type {
            0 => HduType::ImageHDU,
            1 => HduType::AsciiTableHDU,
            2 => HduType::BinTableHDU,
            _ => panic!("Unknown hdu type: {}", hdu_type),
        }
    }

    pub fn get_hdu(&mut self, index: usize) -> FitsHDU {
        self.change_hdu(index as u32);
        let hdu_type = self.get_hdu_type();
        FitsHDU {
            fitsfile: self,
            hdu_type: hdu_type,
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

pub struct FitsHDU<'a> {
    fitsfile: &'a FitsFile,
    hdu_type: HduType,
}

impl<'a> FitsHDU<'a> {
    pub fn get_key(&mut self, key: &str) -> String {
        let fptr = &self.fitsfile.fptr;
        let mut value: Vec<c_char> = vec![0; MAX_VALUE_LENGTH];
        let keyname = ffi::CString::new(key).unwrap();
        let mut status = 0;

        unsafe {
            ffgkys(*fptr,
                   keyname.as_ptr(),
                   value.as_mut_ptr(),
                   ptr::null_mut(),
                   &mut status);
        }

        match status {
            0 => {
                let value: Vec<u8> = value.iter()
                    .map(|&x| x as u8)
                    .filter(|&x| x != 0)
                    .collect();
                return String::from_utf8(value).unwrap();
            }
            _ => {
                panic!("Invalid status code: {}, msg: {}",
                       status,
                       status_to_string(status).unwrap())
            }
        }
    }
}


mod test {
    #[test]
    fn returning_error_messages() {
        use super::status_to_string;

        assert_eq!(status_to_string(105).unwrap(),
                   "couldn't create the named file");
    }

    #[test]
    fn opening_an_existing_file() {
        use super::FitsFile;

        let f = FitsFile::open("testdata/full_example.fits");
        assert_eq!(f.status, 0);
    }

    #[test]
    fn filename_is_stored() {
        use super::FitsFile;

        let f = FitsFile::open("testdata/full_example.fits");
        assert_eq!(f.filename, "testdata/full_example.fits");
    }

    #[test]
    fn change_hdu() {
        use super::FitsFile;

        let mut f = FitsFile::open("testdata/full_example.fits");
        f.change_hdu(1);
        assert_eq!(f.current_hdu_number(), 1u32);
    }

    #[test]
    fn getting_current_hdu_number() {
        use super::FitsFile;

        let f = FitsFile::open("testdata/full_example.fits");
        assert_eq!(f.current_hdu_number(), 0u32);
    }

    #[test]
    fn getting_hdu_object() {
        use super::{FitsFile, HduType};

        let mut f = FitsFile::open("testdata/full_example.fits");

        {
            let primary_hdu = f.get_hdu(0);
            assert_eq!(primary_hdu.hdu_type, HduType::ImageHDU);
        }

        {
            let table_hdu = f.get_hdu(1);
            assert_eq!(table_hdu.hdu_type, HduType::BinTableHDU);
        }
    }
}
