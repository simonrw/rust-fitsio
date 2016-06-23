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

extern crate fitsio_sys;
extern crate libc;

use fitsio_sys::*;
use libc::{c_int, c_long, c_char, c_void};
use std::ptr;
use std::ffi;
use std::result;

/// Error type
#[derive(Debug)]
pub struct FitsError {
    status: i32,
    message: String,
}

pub type Result<T> = result::Result<T, FitsError>;

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
pub enum FitsHduType {
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

/// Hdu description type
///
/// Any way of describing a HDU - number or string which either
/// changes the hdu by absolute number, or by name.
pub trait DescribesHdu {
    fn change_hdu(&self, fptr: &mut FitsFile);
}

impl DescribesHdu for usize {
    fn change_hdu(&self, f: &mut FitsFile) {
        let mut _hdu_type = 0;
        let mut status = 0;
        unsafe {
            ffmahd(f.fptr, (*self + 1) as i32, &mut _hdu_type, &mut status);
        }
    }
}

impl<'a> DescribesHdu for &'a str {
    fn change_hdu(&self, f: &mut FitsFile) {
        let mut _hdu_type = 0;
        let mut status = 0;
        let c_hdu_name = ffi::CString::new(*self).unwrap();

        unsafe {
            ffmnhd(f.fptr,
                   HduType::ANY_HDU as c_int,
                   c_hdu_name.into_raw(),
                   0,
                   &mut status);
        }
    }
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
    ///     match FitsFile::open("../testdata/full_example.fits") {
    ///         Ok(f) => { },
    ///         Err(e) => panic!("{:?}", e),
    ///     }
    /// # }
    pub fn open(filename: &str) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let c_filename = ffi::CString::new(filename).unwrap();

        unsafe {
            ffopen(&mut fptr as *mut *mut fitsfile,
                   c_filename.as_ptr(),
                   FileOpenMode::READONLY as c_int,
                   &mut status);
        }

        match status {
            0 => {
                Ok(FitsFile {
                    fptr: fptr,
                    status: status,
                    filename: filename.to_string(),
                })
            }
            status => {
                Err(FitsError {
                    status: status,
                    message: status_to_string(status).unwrap(),
                })
            }
        }

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
    /// match FitsFile::create(filename) {
    ///     Ok(f) => println!("Fits file created ok"),
    ///     Err(e) => panic!("Error: {:?}", e),
    /// }
    /// # }
    /// ```
    pub fn create(path: &str) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let c_filename = ffi::CString::new(path).unwrap();

        unsafe {
            ffinit(&mut fptr as *mut *mut fitsfile,
                   c_filename.as_ptr(),
                   &mut status);
        }

        match status {
            0 => {
                Ok(FitsFile {
                    fptr: fptr,
                    status: status,
                    filename: path.to_string(),
                })
            }
            status => {
                Err(FitsError {
                    status: status,
                    message: status_to_string(status).unwrap(),
                })
            }
        }
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
    /// let f = FitsFile::open("../testdata/full_example.fits").unwrap();
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
    /// # let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
    /// assert_eq!(f.current_hdu_number(), 0);
    /// f.change_hdu(1);
    /// assert_eq!(f.current_hdu_number(), 1);
    /// # }
    /// ```
    pub fn change_hdu<T: DescribesHdu>(&mut self, hdu_description: T) {
        hdu_description.change_hdu(self);
        self.check();
    }

    /// Get which type of HDU the current HDU is
    ///
    /// Results in one of the `FitsHduType` options.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fitsio::{FitsFile, FitsHduType};
    /// # fn main() {
    /// let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
    /// // Primary HDUs are always image hdus
    /// assert_eq!(f.get_hdu_type(), FitsHduType::ImageHDU);
    /// # }
    /// ```
    pub fn get_hdu_type(&mut self) -> FitsHduType {
        let mut hdu_type = 3;
        unsafe {
            ffghdt(self.fptr, &mut hdu_type, &mut self.status);
        }
        self.check();

        match hdu_type {
            0 => FitsHduType::ImageHDU,
            1 => FitsHduType::AsciiTableHDU,
            2 => FitsHduType::BinTableHDU,
            _ => panic!("Unknown hdu type: {}", hdu_type),
        }
    }

    /// Return a `FitsHDU` object for the specified HDU
    ///
    /// # Examples
    ///
    /// ```
    /// # use fitsio::{FitsFile, FitsHduType};
    /// # fn main() {
    /// # let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
    /// let primary_hdu = f.get_hdu(0);
    /// assert_eq!(primary_hdu.hdu_type, FitsHduType::ImageHDU);
    /// # }
    /// ```
    pub fn get_hdu(&mut self, index: usize) -> FitsHDU {
        self.change_hdu(index);
        let hdu_type = self.get_hdu_type();

        let image_shape = if hdu_type == FitsHduType::ImageHDU {
            // TODO: handle n-d images
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
    pub hdu_type: FitsHduType,
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
    /// let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
    /// let mut primary_hdu = f.get_hdu(0);
    /// // Image is 2-dimensional
    /// let naxis = primary_hdu.get_key("NAXIS").unwrap().parse::<i32>().unwrap();
    /// assert_eq!(naxis, 2);
    /// # }
    /// ```
    pub fn get_key(&mut self, key: &str) -> Result<String> {
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
                Ok(String::from_utf8(value).unwrap())
            }
            status => {
                Err(FitsError {
                    status: status,
                    message: status_to_string(status).unwrap(),
                })
            }
        }
    }

    fn read_all_i32(&mut self, buffer: &mut Vec<i32>) {
        let npix = self.image_shape.0 * self.image_shape.1;
        buffer.resize(npix, 0);
        let mut status = 0;

        unsafe {
            ffgpv(self.fitsfile.fptr,
                  DataType::TINT as c_int,
                  1,
                  npix as i64,
                  ptr::null_mut() as *mut c_void,
                  buffer.as_mut_ptr() as *mut c_void,
                  ptr::null_mut(),
                  &mut status);
        }
        match status {
            0 => {}
            status => panic!("Bad status value: {}", status),
        }
        self.fitsfile.check();
    }
}


#[cfg(test)]
mod test {
    extern crate tempdir;
    use super::*;
    use super::status_to_string;

    #[test]
    fn returning_error_messages() {
        assert_eq!(status_to_string(105).unwrap(),
                   "couldn't create the named file");
    }

    #[test]
    fn opening_an_existing_file() {
        match FitsFile::open("../testdata/full_example.fits") {
            Ok(f) => assert_eq!(f.status, 0),
            Err(e) => panic!("{:?}", e),
        }
    }

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

    #[test]
    fn filename_is_stored() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        assert_eq!(f.filename, "../testdata/full_example.fits");
    }

    #[test]
    fn change_hdu() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        f.change_hdu(1);
        assert_eq!(f.current_hdu_number(), 1);
    }

    #[test]
    fn change_hdu_with_str() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        f.change_hdu("TESTEXT");
        assert_eq!(f.current_hdu_number(), 1);
    }

    #[test]
    fn getting_current_hdu_number() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        assert_eq!(f.current_hdu_number(), 0);
    }

    #[test]
    fn getting_hdu_object() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();

        // TODO: get rid of these scopes
        //
        // They're there because `get_hdu` mutably borrows the `self` object and
        // the result is still in scope. We need to expire the `FitsHDU` objects
        // and hence the scopes.
        {
            let primary_hdu = f.get_hdu(0);
            assert_eq!(primary_hdu.hdu_type, FitsHduType::ImageHDU);
        }

        {
            let table_hdu = f.get_hdu(1);
            assert_eq!(table_hdu.hdu_type, FitsHduType::BinTableHDU);
        }
    }

    #[test]
    fn reading_in_image_data() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let mut primary_hdu = f.get_hdu(0);
        let mut data = Vec::new();
        primary_hdu.read_all_i32(&mut data);
        assert_eq!(data[0], 108);
        assert_eq!(data[1], 176);
    }

    #[test]
    fn get_image_dimensions() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let primary_hdu = f.get_hdu(0);
        assert_eq!(primary_hdu.image_shape, (100, 100));
    }

    #[test]
    fn get_key_returns_error_for_missing_key() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let mut primary_hdu = f.get_hdu(0);

        match primary_hdu.get_key("THISKEYDOESNOTEXIST") {
            Err(e) => assert_eq!(e.status, 202),
            Ok(_) => panic!("No error thrown"),
        }
    }
}
