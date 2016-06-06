#![allow(dead_code)]

//! A wrapper around [cfitsio].
//!
//! This wrapper attempts to add some safety to the [cfitsio] library, and allows `.fits` files to
//! be read from rust.
//!
//! The main entry point to the code is `FitsFile::open` which creates a new file, and
//! `FitsFile::create` which creates a new file.
//!
//! [cfitsio]: https://heasarc.gsfc.nasa.gov/docs/software/fitsio/fitsio.html

pub mod raw;

extern crate libc;

use raw::*;
use libc::{c_int, c_long, c_char};
use std::ptr;
use std::ffi;

/// Internal function to get the fits error description from a status code
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

/// General type defining what kind of HDU we're talking about
#[derive(Eq, PartialEq, Debug)]
pub enum HduType {
    ImageHDU,
    AsciiTableHDU,
    BinTableHDU,
}

/// Wrapper around a C `fitsfile` pointer.
///
/// This handles [opening][FitsFile::open], [creating][FitsFile::create] and automatically
/// closing (through the `Drop` trait) the file.
///
/// All subsequent file access is controled through this object.
///
/// [FitsFile::open]: #method.open
/// [FitsFile::create]: #method.create
pub struct FitsFile {
    fptr: *mut fitsfile,
    status: c_int,
    pub filename: String,
}

impl FitsFile {
    /// Open a fits file for reading
    ///
    /// * `filename` - Filename to pass to `cfitsio`. Can conform to the
    /// [Extended Filename Syntax][extended-filename-syntax].
    ///
    /// [extended-filename-syntax]:
    ///     https://heasarc.gsfc.nasa.gov/docs/software/fitsio/c/c_user/node82.html
    ///
    /// Examples
    ///
    /// ```
    /// # use fitsio::FitsFile;
    /// # fn main() {
    ///     let f = FitsFile::open("testdata/full_example.fits");
    /// # }
    pub fn open(filename: &str) -> Self {
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

    /// Create a new fits file
    ///
    /// An empty primary header is added so when the `drop` method is called, the file is valid.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate fitsio;
    /// # extern crate tempdir;
    /// # use fitsio::FitsFile;
    /// # fn main() {
    /// # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
    /// # let tdir_path = tdir.path();
    /// # let filename = tdir_path.join("test.fits");
    /// # let filename = filename.to_str().unwrap();
    /// let _ = FitsFile::create(filename);
    /// # }
    /// ```
    pub fn create(path: &str) -> Self {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let c_filename = ffi::CString::new(path).unwrap();

        unsafe {
            ffinit(&mut fptr as *mut *mut fitsfile,
                   c_filename.as_ptr(),
                   &mut status);
        }

        return match status {
            0 => {
                FitsFile {
                    fptr: fptr,
                    status: status,
                    filename: path.to_string(),
                }
            }
            status => {
                panic!("Invalid status code: {}, msg: {}",
                       status,
                       status_to_string(status).unwrap())
            }
        };
    }

    /// Function to check that the status code is ok.
    ///
    /// If the value of `self.status` is not 0 then exit the current process as an error has
    /// occurred.
    fn check(&self) {
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
    /// # use fitsio::FitsFile;
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

    /// Change the current HDU
    ///
    /// Examples
    ///
    /// ```
    /// # use fitsio::FitsFile;
    /// # fn main() {
    /// # let mut f = FitsFile::open("testdata/full_example.fits");
    /// assert_eq!(f.current_hdu_number(), 0);
    /// f.change_hdu(1);
    /// assert_eq!(f.current_hdu_number(), 1);
    /// # }
    /// ```
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

    /// Get which type of HDU the current HDU is
    ///
    /// Results in one of the `HduType` options.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fitsio::{FitsFile, HduType};
    /// # fn main() {
    /// let mut f = FitsFile::open("testdata/full_example.fits");
    /// // Primary HDUs are always image hdus
    /// assert_eq!(f.get_hdu_type(), HduType::ImageHDU);
    /// # }
    /// ```
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

    /// Return a `FitsHDU` object for the specified HDU
    ///
    /// # Examples
    ///
    /// ```
    /// # use fitsio::{FitsFile, HduType};
    /// # fn main() {
    /// # let mut f = FitsFile::open("testdata/full_example.fits");
    /// let primary_hdu = f.get_hdu(0);
    /// assert_eq!(primary_hdu.hdu_type, HduType::ImageHDU);
    /// # }
    /// ```
    pub fn get_hdu(&mut self, index: usize) -> FitsHDU {
        self.change_hdu(index as u32);
        let hdu_type = self.get_hdu_type();

        let image_shape = if hdu_type == HduType::ImageHDU {
            let mut naxis = vec![0, 0];
            unsafe {
                ffgisz(self.fptr, 2, naxis.as_mut_ptr(), &mut self.status);
            }
            println!("{:?}", naxis);
            (naxis[0] as usize, naxis[1] as usize)
        } else {
            (0, 0)
        };

        FitsHDU {
            fitsfile: self,
            hdu_type: hdu_type,
            image_shape: image_shape,
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

/// Wrapper around a FITS HDU
///
/// This struct is the main interface around reading and writing the file contents.
pub struct FitsHDU<'a> {
    fitsfile: &'a FitsFile,
    pub hdu_type: HduType,
    image_shape: (usize, usize),
}

impl<'a> FitsHDU<'a> {
    /// Read a header key as a string
    ///
    /// The user is responsible for converting the value type from a string to whatever type the
    /// header key contains.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fitsio::FitsFile;
    /// # fn main() {
    /// let mut f = FitsFile::open("testdata/full_example.fits");
    /// let mut primary_hdu = f.get_hdu(0);
    /// // Image is 2-dimensional
    /// let naxis = primary_hdu.get_key("NAXIS").parse::<i32>().unwrap();
    /// assert_eq!(naxis, 2);
    /// # }
    /// ```
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
    fn creating_a_new_file() {
        extern crate tempdir;

        use super::FitsFile;
        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");
        assert!(!filename.exists());

        let _ = FitsFile::create(filename.to_str().unwrap());
        assert!(filename.exists());
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

    #[test]
    fn get_image_dimensions() {
        use super::{FitsFile, HduType};

        let mut f = FitsFile::open("testdata/full_example.fits");
        let mut primary_hdu = f.get_hdu(0);
        assert_eq!(primary_hdu.image_shape, (100, 100));
    }
}
