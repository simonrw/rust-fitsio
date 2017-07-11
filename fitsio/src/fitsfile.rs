/* Depending on the architecture, different functions have to be called. For example arm systems
 * define `int` as 4 bytes, and `long` as 4 bytes, unlike x86_64 systems which define `long` types
 * as 8 bytes.
 *
 * In this case, we have to use `_longlong` cfitsio functions on arm architectures (and other
 * similar architectures).
 */

use sys;
use stringutils::{self, status_to_string};
use errors::{Error, Result};
use fitserror::FitsError;
use columndescription::*;
use libc;
use types::{DataType, CaseSensitivity, HduInfo, FileOpenMode, ImageType};
use std::ffi;
use std::ptr;
use std::ops::Range;

pub static MAX_VALUE_LENGTH: usize = 71;

/// Macro to return a fits error if the fits file is not open in readwrite mode
macro_rules! fits_check_readwrite {
    ($fitsfile: expr) => (
        if let Ok(FileOpenMode::READONLY) = $fitsfile.open_mode() {
            return Err(FitsError {
                status: 602,
                message: "cannot alter readonly file".to_string(),
            }.into());
        }
    )
}

/// Description of a new image
pub struct ImageDescription<'a> {
    pub data_type: ImageType,
    pub dimensions: &'a [usize],
}

/// Main entry point to the FITS file format
///
///
pub struct FitsFile {
    pub filename: String,
    fptr: *const sys::fitsfile,
}

impl FitsFile {
    /// Open a fits file from disk
    ///
    /// # Examples
    ///
    /// ```
    /// use fitsio::FitsFile;
    ///
    /// let f = FitsFile::open("../testdata/full_example.fits").unwrap();
    ///
    /// // Continue to use `f` afterwards
    /// ```
    pub fn open<T: Into<String>>(filename: T) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let filename = filename.into();
        let c_filename = ffi::CString::new(filename.as_str())?;

        unsafe {
            sys::ffopen(
                &mut fptr as *mut *mut sys::fitsfile,
                c_filename.as_ptr(),
                FileOpenMode::READONLY as libc::c_int,
                &mut status,
            );
        }

        fits_try!(
            status,
            FitsFile {
                fptr: fptr,
                filename: filename.clone(),
            }
        )
    }

    /// Open a fits file in read/write mode
    pub fn edit<T: Into<String>>(filename: T) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let filename = filename.into();
        let c_filename = ffi::CString::new(filename.as_str())?;

        unsafe {
            sys::ffopen(
                &mut fptr as *mut *mut _,
                c_filename.as_ptr(),
                FileOpenMode::READWRITE as libc::c_int,
                &mut status,
            );
        }

        fits_try!(
            status,
            FitsFile {
                fptr: fptr,
                filename: filename.clone(),
            }
        )
    }

    /// Create a new fits file on disk
    pub fn create<T: Into<String>>(path: T) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let path = path.into();
        let c_filename = ffi::CString::new(path.as_str())?;

        unsafe {
            sys::ffinit(
                &mut fptr as *mut *mut sys::fitsfile,
                c_filename.as_ptr(),
                &mut status,
            );
        }

        fits_try!(status, {
            let f = FitsFile {
                fptr: fptr,
                filename: path.clone(),
            };
            try!(f.add_empty_primary());
            f
        })
    }

    /// Method to extract what open mode the file is in
    fn open_mode(&self) -> Result<FileOpenMode> {
        let mut status = 0;
        let mut iomode = 0;
        unsafe {
            sys::ffflmd(self.fptr as *mut _, &mut iomode, &mut status);
        }

        fits_try!(
            status,
            match iomode {
                0 => FileOpenMode::READONLY,
                1 => FileOpenMode::READWRITE,
                _ => unreachable!(),
            }
        )
    }

    fn add_empty_primary(&self) -> Result<()> {
        let mut status = 0;
        unsafe {
            sys::ffphps(self.fptr as *mut _, 8, 0, ptr::null_mut(), &mut status);
        }

        fits_try!(status, ())
    }

    /// Change the current HDU
    fn change_hdu<T: DescribesHdu>(&self, hdu_description: T) -> Result<()> {
        hdu_description.change_hdu(self)
    }

    /// Return a new HDU object
    pub fn hdu<T: DescribesHdu>(&self, hdu_description: T) -> Result<FitsHdu> {
        FitsHdu::new(self, hdu_description)
    }

    fn hdu_number(&self) -> usize {
        let mut hdu_num = 0;
        unsafe {
            sys::ffghdn(self.fptr as *mut _, &mut hdu_num);
        }
        (hdu_num - 1) as usize
    }

    /// Get the current hdu as an HDU object
    pub fn current_hdu(&self) -> Result<FitsHdu> {
        let current_hdu_number = self.hdu_number();
        self.hdu(current_hdu_number)
    }

    /// Get the current hdu info
    fn fetch_hdu_info(&self) -> Result<HduInfo> {
        let mut status = 0;
        let mut hdu_type = 0;

        unsafe {
            sys::ffghdt(self.fptr as *mut _, &mut hdu_type, &mut status);
        }

        let hdu_type = match hdu_type {
            0 => {
                let mut dimensions = 0;
                unsafe {
                    sys::ffgidm(self.fptr as *mut _, &mut dimensions, &mut status);
                }

                let mut shape = vec![0; dimensions as usize];
                unsafe {
                    sys::ffgisz(
                        self.fptr as *mut _,
                        dimensions,
                        shape.as_mut_ptr(),
                        &mut status,
                    );
                }

                let mut bitpix = 0;
                unsafe {
                    /* Use equiv type as this is more useful
                     *
                     * See description here:
                     * https://heasarc.gsfc.nasa.gov/docs/software/fitsio/c/c_user/node40.html
                     */
                    sys::ffgiet(self.fptr as *mut _, &mut bitpix, &mut status);
                }

                let image_type = match bitpix {
                    8 => ImageType::BYTE_IMG,
                    16 => ImageType::SHORT_IMG,
                    20 => ImageType::USHORT_IMG,
                    32 => ImageType::LONG_IMG,
                    40 => ImageType::ULONG_IMG,
                    64 => ImageType::LONGLONG_IMG,
                    -32 => ImageType::FLOAT_IMG,
                    -64 => ImageType::DOUBLE_IMG,
                    _ => unreachable!(&format!("Unhandled image bitpix type: {}", bitpix)),
                };

                HduInfo::ImageInfo {
                    shape: shape.iter().map(|v| *v as usize).collect(),
                    image_type: image_type,
                }
            }
            1 | 2 => {
                let mut num_rows = 0;
                unsafe {
                    sys::ffgnrw(self.fptr as *mut _, &mut num_rows, &mut status);
                }

                let mut num_cols = 0;
                unsafe {
                    sys::ffgncl(self.fptr as *mut _, &mut num_cols, &mut status);
                }
                let mut column_descriptions = Vec::with_capacity(num_cols as usize);

                for i in 0..num_cols {
                    let mut name_buffer: Vec<libc::c_char> = vec![0; 71];
                    let mut type_buffer: Vec<libc::c_char> = vec![0; 71];
                    unsafe {
                        sys::ffgbcl(
                            self.fptr as *mut _,
                            (i + 1) as i32,
                            name_buffer.as_mut_ptr(),
                            ptr::null_mut(),
                            type_buffer.as_mut_ptr(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            &mut status,
                        );
                    }

                    column_descriptions.push(ConcreteColumnDescription {
                        name: stringutils::buf_to_string(&name_buffer)?,
                        data_type: stringutils::buf_to_string(&type_buffer)?
                            .parse::<ColumnDataDescription>()?,
                    });
                }

                HduInfo::TableInfo {
                    column_descriptions: column_descriptions,
                    num_rows: num_rows as usize,
                }
            }
            _ => panic!("Invalid hdu type found"),
        };

        fits_try!(status, hdu_type)
    }

    /// Create a new fits table
    ///
    /// Create a new fits table, with columns as detailed in the `ColumnDescription` object.
    ///
    ///
    ///
    ///
    ///
    pub fn create_table(
        &self,
        extname: String,
        table_description: &[ConcreteColumnDescription],
    ) -> Result<FitsHdu> {

        fits_check_readwrite!(self);

        let tfields = {
            let stringlist = table_description
                .iter()
                .map(|desc| desc.name.clone())
                .collect();
            stringutils::StringList::from_vec(stringlist)?
        };

        let ttype = {
            let stringlist = table_description
                .iter()
                .map(|desc| String::from(desc.clone().data_type))
                .collect();
            stringutils::StringList::from_vec(stringlist)?
        };

        let c_extname = ffi::CString::new(extname)?;


        let hdu_info = HduInfo::TableInfo {
            column_descriptions: table_description.to_vec(),
            num_rows: 0,
        };

        let mut status: libc::c_int = 0;
        unsafe {
            sys::ffcrtb(
                self.fptr as *mut _,
                hdu_info.into(),
                0,
                tfields.len as libc::c_int,
                tfields.as_ptr(),
                ttype.as_ptr(),
                ptr::null_mut(),
                c_extname.into_raw(),
                &mut status,
            );
        }

        if status != 0 {
            Err(
                FitsError {
                    status: status,
                    // unwrap guaranteed to succesed as status > 0
                    message: status_to_string(status)?.unwrap(),
                }.into(),
            )
        } else {
            self.current_hdu()
        }

    }

    /// Create a new fits image, and return the [`FitsHdu`](struct.FitsHdu.html) object
    pub fn create_image(
        &self,
        extname: String,
        image_description: &ImageDescription,
    ) -> Result<FitsHdu> {

        fits_check_readwrite!(self);

        let naxis = image_description.dimensions.len();
        let mut status = 0;

        if status != 0 {
            return Err(
                FitsError {
                    status: status,
                    // unwrap guaranteed to succesed as status > 0
                    message: status_to_string(status)?.unwrap(),
                }.into(),
            );
        }

        unsafe {
            sys::ffcrim(
                self.fptr as *mut _,
                image_description.data_type.into(),
                naxis as i32,
                image_description.dimensions.as_ptr() as *mut libc::c_long,
                &mut status,
            );
        }

        if status != 0 {
            return Err(
                FitsError {
                    status: status,
                    // unwrap guaranteed to succesed as status > 0
                    message: status_to_string(status)?.unwrap(),
                }.into(),
            );
        }

        // Current HDU should be at the new HDU
        let mut current_hdu = try!(self.current_hdu());
        try!(current_hdu.write_key("EXTNAME".into(), extname));

        if status != 0 {
            Err(
                FitsError {
                    status: status,
                    // unwrap guaranteed to succesed as status > 0
                    message: status_to_string(status)?.unwrap(),
                }.into(),
            )
        } else {
            self.current_hdu()
        }
    }

    /// Return a pointer to the underlying C `fitsfile` object representing the current file.
    ///
    /// This is marked as `unsafe` as it is definitely something that is not required by most
    /// users, and hence the unsafe-ness marks it as an advanced feature. I have also not
    /// considered possible concurrency or data race issues as yet.
    // XXX This may have to be wrapped in some form of access control structure, such as an
    // `std::rc::Rc`.
    pub unsafe fn as_raw(&self) -> *mut sys::fitsfile {
        self.fptr as *mut _
    }
}

impl Drop for FitsFile {
    fn drop(&mut self) {
        let mut status = 0;
        unsafe {
            sys::ffclos(self.fptr as *mut _, &mut status);
        }
    }
}

/// Hdu description type
///
/// Any way of describing a HDU - number or string which either
/// changes the hdu by absolute number, or by name.
pub trait DescribesHdu {
    fn change_hdu(&self, fptr: &FitsFile) -> Result<()>;
}

impl DescribesHdu for usize {
    fn change_hdu(&self, f: &FitsFile) -> Result<()> {
        let mut _hdu_type = 0;
        let mut status = 0;
        unsafe {
            sys::ffmahd(
                f.fptr as *mut _,
                (*self + 1) as i32,
                &mut _hdu_type,
                &mut status,
            );
        }

        fits_try!(status, ())
    }
}

impl<'a> DescribesHdu for &'a str {
    fn change_hdu(&self, f: &FitsFile) -> Result<()> {
        let mut _hdu_type = 0;
        let mut status = 0;
        let c_hdu_name = ffi::CString::new(*self)?;

        unsafe {
            sys::ffmnhd(
                f.fptr as *mut _,
                HduInfo::AnyInfo.into(),
                c_hdu_name.into_raw(),
                0,
                &mut status,
            );
        }

        fits_try!(status, ())
    }
}

/// Trait for reading a fits column
pub trait ReadsCol {
    fn read_col_range<T: Into<String>>(
        fits_file: &FitsFile,
        name: T,
        range: &Range<usize>,
    ) -> Result<Vec<Self>>
    where
        Self: Sized;

    /// Default implementation which uses `read_col_range`
    fn read_col<T: Into<String>>(fits_file: &FitsFile, name: T) -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { num_rows, .. }) => {
                let range = 0..num_rows - 1;
                Self::read_col_range(fits_file, name, &range)
            }
            Err(e) => Err(e),
            _ => panic!("Unknown error occurred"),
        }
    }
}

macro_rules! reads_col_impl {
    ($t: ty, $func: ident, $nullval: expr) => (
        impl ReadsCol for $t {
            // TODO: should we check the bounds? cfitsio will raise an error, but we
            // could be more friendly and raise our own?
            fn read_col_range<T: Into<String>>(fits_file: &FitsFile, name: T, range: &Range<usize>)
                -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::TableInfo { column_descriptions, .. }) => {
                        let num_output_rows = range.end - range.start + 1;
                        let mut out = vec![$nullval; num_output_rows];
                        let test_name = name.into();
                        let column_number = column_descriptions
                            .iter()
                            .position(|ref desc| { desc.name == test_name })
                            .ok_or(Error::Message(format!("Cannot find column {:?}", test_name)))?;
                        let mut status = 0;
                        unsafe {
                            sys::$func(fits_file.fptr as *mut _,
                                       (column_number + 1) as i32,
                                       (range.start + 1) as i64,
                                       1,
                                       num_output_rows as _,
                                       $nullval,
                                       out.as_mut_ptr(),
                                       ptr::null_mut(),
                                       &mut status);

                        }
                        fits_try!(status, out)
                    },
                    Err(e) => Err(e),
                    _ => panic!("Unknown error occurred"),
                }
            }
        }
    )
}

reads_col_impl!(i32, ffgcvk, 0);
reads_col_impl!(u32, ffgcvuk, 0);
reads_col_impl!(f32, ffgcve, 0.0);
reads_col_impl!(f64, ffgcvd, 0.0);
#[cfg(target_pointer_width = "64")]
reads_col_impl!(i64, ffgcvj, 0);
#[cfg(target_pointer_width = "32")]
reads_col_impl!(i64, ffgcvjj, 0);
#[cfg(target_pointer_width = "64")]
reads_col_impl!(u64, ffgcvuj, 0);

/// Helper function to get the display width of a column
fn column_display_width(fits_file: &FitsFile, column_number: usize) -> Result<usize> {
    let mut status = 0;
    let mut width = 0;
    unsafe {
        sys::ffgcdw(
            fits_file.fptr as *mut _,
            (column_number + 1) as _,
            &mut width,
            &mut status,
        );
    }
    fits_try!(status, width as usize)
}

impl ReadsCol for String {
    fn read_col_range<T: Into<String>>(
        fits_file: &FitsFile,
        name: T,
        range: &Range<usize>,
    ) -> Result<Vec<Self>> {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { column_descriptions, .. }) => {
                let num_output_rows = range.end - range.start + 1;
                let test_name = name.into();
                let column_number = column_descriptions
                    .iter()
                    .position(|desc| desc.name == test_name)
                    .ok_or_else(|| {
                        Error::Message(format!("Cannot find column {:?}", test_name))
                    })?;

                /* Set up the storage arrays for the column string values */
                let mut raw_char_data: Vec<*mut libc::c_char> =
                    Vec::with_capacity(num_output_rows as usize);

                let mut status = 0;
                let width = column_display_width(fits_file, column_number)?;

                let mut vecs: Vec<Vec<libc::c_char>> = Vec::with_capacity(num_output_rows as usize);
                for _ in 0..num_output_rows {
                    let mut data: Vec<libc::c_char> = vec![0; width as _];
                    let data_p = data.as_mut_ptr();
                    vecs.push(data);
                    raw_char_data.push(data_p);
                }

                unsafe {
                    sys::ffgcvs(
                        fits_file.fptr as *mut _,
                        (column_number + 1) as _,
                        (range.start + 1) as _,
                        1,
                        raw_char_data.len() as _,
                        ptr::null_mut(),
                        raw_char_data.as_ptr() as *mut *mut _,
                        ptr::null_mut(),
                        &mut status,
                    );
                }
                // TODO: check the status code
                assert_eq!(status, 0, "Status code is not 0: {}", status);

                let mut out = Vec::with_capacity(num_output_rows);
                for val in &vecs {
                    let bytes: Vec<u8> = val.into_iter()
                        .filter(|v| **v != 0)
                        .map(|v| *v as u8)
                        .collect();
                    let cstr = String::from_utf8(bytes)?;
                    out.push(cstr);
                }
                Ok(out)
            }
            Err(e) => Err(e),
            _ => panic!("Unknown error occurred"),
        }
    }
}

pub trait WritesCol {
    fn write_col_range<T: Into<String>>(
        fits_file: &FitsFile,
        hdu: &FitsHdu,
        col_name: T,
        col_data: &[Self],
        rows: &Range<usize>,
    ) -> Result<()>
    where
        Self: Sized;

    fn write_col<T: Into<String>>(
        fits_file: &FitsFile,
        hdu: &FitsHdu,
        col_name: T,
        col_data: &[Self],
    ) -> Result<()>
    where
        Self: Sized,
    {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { .. }) => {
                let row_range = 0..col_data.len() - 1;
                Self::write_col_range(fits_file, hdu, col_name, col_data, &row_range)
            }
            Ok(HduInfo::ImageInfo { .. }) => Err("Cannot write column data to FITS image".into()),
            Ok(HduInfo::AnyInfo { .. }) => {
                Err(
                    "Cannot determine HDU type, so cannot write column data".into(),
                )
            }
            Err(e) => Err(e),
        }
    }
}

macro_rules! writes_col_impl {
    ($t: ty, $data_type: expr) => (
        impl WritesCol for $t {
            fn write_col_range<T: Into<String>>(fits_file: &FitsFile,
                hdu: &FitsHdu,
                col_name: T,
                col_data: &[Self],
                rows: &Range<usize>)
            -> Result<()> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::TableInfo { .. }) => {
                        let colno = hdu.get_column_no(col_name.into())?;
                        let mut status = 0;
                        unsafe {
                            sys::ffpcl(
                                fits_file.fptr as *mut _,
                                $data_type.into(),
                                (colno + 1) as _,
                                (rows.start + 1) as _,
                                1,
                                (rows.end + 1) as _,
                                col_data.as_ptr() as *mut _,
                                &mut status
                            );
                        }
                        fits_try!(status, ())
                    },
                    Ok(HduInfo::ImageInfo { .. }) =>
                        Err("Cannot write column data to FITS image".into()),
                    Ok(HduInfo::AnyInfo { .. }) =>
                        Err("Cannot determine HDU type, so cannot write column data".into()),
                    Err(e) => Err(e),
                }
            }
        }
    )
}

writes_col_impl!(u32, DataType::TUINT);
#[cfg(target_pointer_width = "64")]
writes_col_impl!(u64, DataType::TULONG);
writes_col_impl!(i32, DataType::TINT);
#[cfg(target_pointer_width = "64")]
writes_col_impl!(i64, DataType::TLONG);
#[cfg(target_pointer_width = "32")]
writes_col_impl!(i64, DataType::TLONGLONG);
writes_col_impl!(f32, DataType::TFLOAT);
writes_col_impl!(f64, DataType::TDOUBLE);

impl WritesCol for String {
    fn write_col_range<T: Into<String>>(
        fits_file: &FitsFile,
        hdu: &FitsHdu,
        col_name: T,
        col_data: &[Self],
        rows: &Range<usize>,
    ) -> Result<()> {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { .. }) => {
                let colno = hdu.get_column_no(col_name.into())?;
                let width = column_display_width(fits_file, colno)?;
                let mut status = 0;

                // TODO: try to find a way to not dupliicate every string somehow!
                let padded_strings: Vec<String> = col_data
                    .iter()
                    .map(|s| format!("{0:>1$}", s, width))
                    .collect();

                let mut ptr_array: Vec<*mut i8> = Vec::with_capacity(rows.end - rows.start);
                for s in padded_strings {
                    let ptr = s.as_bytes().as_ptr();
                    ptr_array.push(ptr as *mut _);
                }

                unsafe {
                    sys::ffpcls(
                        fits_file.fptr as *mut _,
                        (colno + 1) as _,
                        1,
                        1,
                        col_data.len() as _,
                        ptr_array.as_mut_ptr() as _,
                        &mut status,
                    );
                }
                fits_try!(status, ())
            }
            Ok(HduInfo::ImageInfo { .. }) => Err("Cannot write column data to FITS image".into()),
            Ok(HduInfo::AnyInfo { .. }) => {
                Err(
                    "Cannot determine HDU type, so cannot write column data".into(),
                )
            }
            Err(e) => Err(e),
        }
    }
}

/// Trait applied to types which can be read from a FITS header
///
/// This is currently:
///
/// * i32
/// * i64
/// * f32
/// * f64
/// * String
pub trait ReadsKey {
    fn read_key(f: &FitsFile, name: &str) -> Result<Self>
    where
        Self: Sized;
}

macro_rules! reads_key_impl {
    ($t:ty, $func:ident) => (
        impl ReadsKey for $t {
            fn read_key(f: &FitsFile, name: &str) -> Result<Self> {
                let c_name = ffi::CString::new(name)?;
                let mut status = 0;
                let mut value: Self = Self::default();

                unsafe {
                    sys::$func(f.fptr as *mut _,
                           c_name.into_raw(),
                           &mut value,
                           ptr::null_mut(),
                           &mut status);
                }

                fits_try!(status, value)
            }
        }
    )
}

reads_key_impl!(i32, ffgkyl);
#[cfg(target_pointer_width = "64")]
reads_key_impl!(i64, ffgkyj);
#[cfg(target_pointer_width = "32")]
reads_key_impl!(i64, ffgkyjj);
reads_key_impl!(f32, ffgkye);
reads_key_impl!(f64, ffgkyd);

impl ReadsKey for String {
    fn read_key(f: &FitsFile, name: &str) -> Result<Self> {
        let c_name = ffi::CString::new(name)?;
        let mut status = 0;
        let mut value: Vec<libc::c_char> = vec![0; MAX_VALUE_LENGTH];

        unsafe {
            sys::ffgkys(
                f.fptr as *mut _,
                c_name.into_raw(),
                value.as_mut_ptr(),
                ptr::null_mut(),
                &mut status,
            );
        }

        fits_try!(status, {
            let value: Vec<u8> = value.iter().map(|&x| x as u8).filter(|&x| x != 0).collect();
            String::from_utf8(value)?
        })
    }
}

/// Writing a fits keyword
pub trait WritesKey {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()>;
}

macro_rules! writes_key_impl_flt {
    ($t:ty, $func:ident) => (
        impl WritesKey for $t {
            fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
                let c_name = ffi::CString::new(name)?;
                let mut status = 0;

                unsafe {
                    sys::$func(f.fptr as *mut _,
                                c_name.into_raw(),
                                value,
                                9,
                                ptr::null_mut(),
                                &mut status);
                }
                fits_try!(status, ())
            }
        }
    )
}

impl WritesKey for i64 {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
        let c_name = ffi::CString::new(name)?;
        let mut status = 0;

        unsafe {
            sys::ffpkyj(
                f.fptr as *mut _,
                c_name.into_raw(),
                value,
                ptr::null_mut(),
                &mut status,
            );
        }
        fits_try!(status, ())
    }
}

writes_key_impl_flt!(f32, ffpkye);
writes_key_impl_flt!(f64, ffpkyd);

impl WritesKey for String {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
        let c_name = ffi::CString::new(name)?;
        let mut status = 0;

        unsafe {
            sys::ffpkys(
                f.fptr as *mut _,
                c_name.into_raw(),
                ffi::CString::new(value)?.into_raw(),
                ptr::null_mut(),
                &mut status,
            );
        }

        fits_try!(status, ())
    }
}

/// Reading fits images
pub trait ReadWriteImage: Sized {
    /// Read pixels from an image between a start index and end index
    ///
    /// Start and end are read inclusively, so start = 0, end = 10 will read 11 pixels
    /// in a row.
    fn read_section(fits_file: &FitsFile, start: usize, end: usize) -> Result<Vec<Self>>;

    /// Read a row of pixels from a fits image
    fn read_rows(fits_file: &FitsFile, start_row: usize, num_rows: usize) -> Result<Vec<Self>>;

    /// Read a single row from the image HDU
    fn read_row(fits_file: &FitsFile, row: usize) -> Result<Vec<Self>>;

    /// Read a square region from the chip.
    ///
    /// Lower left indicates the starting point of the square, and the upper
    /// right defines the pixel _beyond_ the end. The range of pixels included
    /// is inclusive of the lower end, and *exclusive* of the upper end.
    fn read_region(fits_file: &FitsFile, ranges: &[&Range<usize>]) -> Result<Vec<Self>>;

    /// Read a whole image into a new `Vec`
    ///
    /// This reads an entire image into a one-dimensional vector
    fn read_image(fits_file: &FitsFile) -> Result<Vec<Self>> {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::ImageInfo { shape, .. }) => {
                let mut npixels = 1;
                for dimension in &shape {
                    npixels *= *dimension;
                }
                Self::read_section(fits_file, 0, npixels)
            }
            Ok(HduInfo::TableInfo { .. }) => Err("cannot read image data from a table hdu".into()),
            Ok(HduInfo::AnyInfo) => unreachable!(),
            Err(e) => Err(e),
        }
    }

    fn write_section(fits_file: &FitsFile, start: usize, end: usize, data: &[Self]) -> Result<()>;

    fn write_region(fits_file: &FitsFile, ranges: &[&Range<usize>], data: &[Self]) -> Result<()>;
}

macro_rules! read_write_image_impl {
    ($t: ty, $data_type: expr) => (
        impl ReadWriteImage for $t {
            fn read_section(
                fits_file: &FitsFile,
                start: usize,
                end: usize) -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::ImageInfo { shape: _shape, .. }) => {
                        let nelements = end - start;
                        let mut out = vec![0 as $t; nelements];
                        let mut status = 0;

                        unsafe {
                            sys::ffgpv(fits_file.fptr as *mut _,
                                       $data_type.into(),
                                       (start + 1) as i64,
                                       nelements as i64,
                                       ptr::null_mut(),
                                       out.as_mut_ptr() as *mut _,
                                       ptr::null_mut(),
                                       &mut status);
                        }

                        fits_try!(status, out)

                    },
                    Ok(HduInfo::TableInfo { .. }) =>
                        Err("cannot read image data from a table hdu".into()),
                    Ok(HduInfo::AnyInfo) => unreachable!(),
                    Err(e) => Err(e),
                }
            }

            fn read_rows(fits_file: &FitsFile, start_row: usize, num_rows: usize)
                -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::ImageInfo { shape, .. }) => {
                        if shape.len() != 2 {
                            unimplemented!();
                        }

                        let num_cols = shape[1];
                        let start = start_row * num_cols;
                        let end = (start_row + num_rows) * num_cols;

                        Self::read_section(fits_file, start, end)
                    },
                    Ok(HduInfo::TableInfo { .. }) =>
                        Err("cannot read image data from a table hdu".into()),
                    Ok(HduInfo::AnyInfo) => unreachable!(),
                    Err(e) => Err(e),
                }
            }

            fn read_row(fits_file: &FitsFile, row: usize) -> Result<Vec<Self>> {
                Self::read_rows(fits_file, row, 1)
            }

            fn read_region(fits_file: &FitsFile, ranges: &[&Range<usize>])
                -> Result<Vec<Self>> {
                    match fits_file.fetch_hdu_info() {
                        Ok(HduInfo::ImageInfo { shape, .. }) => {
                            if shape.len() != 2 {
                                unimplemented!();
                            }

                            if ranges.len() != 2 {
                                unimplemented!();
                            }

                            // These have to be mutable because of the C-api
                            let mut fpixel = [
                                (ranges[0].start + 1) as _,
                                (ranges[1].start + 1) as _
                            ];
                            let mut lpixel = [
                                (ranges[0].end + 1) as _,
                                (ranges[1].end + 1) as _
                            ];

                            let mut inc = [ 1, 1 ];
                            let nelements =
                                ((lpixel[0] - fpixel[0]) + 1) * ((lpixel[1] - fpixel[1]) + 1);
                            let mut out = vec![0 as $t; nelements as usize];
                            let mut status = 0;

                            unsafe {
                                sys::ffgsv(
                                    fits_file.fptr as *mut _,
                                    $data_type.into(),
                                    fpixel.as_mut_ptr(),
                                    lpixel.as_mut_ptr(),
                                    inc.as_mut_ptr(),
                                    ptr::null_mut(),
                                    out.as_mut_ptr() as *mut _,
                                    ptr::null_mut(),
                                    &mut status);

                            }

                            fits_try!(status, out)
                        }
                        Ok(HduInfo::TableInfo { .. }) =>
                            Err("cannot read image data from a table hdu".into()),
                        Ok(HduInfo::AnyInfo) => unreachable!(),
                        Err(e) => Err(e),
                    }
                }

            fn write_section(
                fits_file: &FitsFile,
                start: usize,
                end: usize,
                data: &[Self])
                -> Result<()> {
                    match fits_file.fetch_hdu_info() {
                        Ok(HduInfo::ImageInfo { .. }) => {
                            let nelements = end - start;
                            assert!(data.len() >= nelements);
                            let mut status = 0;
                            unsafe {
                                sys::ffppr(fits_file.fptr as *mut _,
                                           $data_type.into(),
                                           (start + 1) as i64,
                                           nelements as i64,
                                           data.as_ptr() as *mut _,
                                           &mut status);
                            }

                            fits_try!(status, ())
                        },
                        Ok(HduInfo::TableInfo { .. }) =>
                            Err("cannot write image data to a table hdu".into()),
                        Ok(HduInfo::AnyInfo) => unreachable!(),
                        Err(e) => Err(e),
                    }
                }

            fn write_region(
                fits_file: &FitsFile,
                ranges: &[&Range<usize>],
                data: &[Self])
                -> Result<()> {
                    match fits_file.fetch_hdu_info() {
                        Ok(HduInfo::ImageInfo { .. }) => {
                            let mut fpixel = [
                                (ranges[0].start + 1) as _,
                                (ranges[1].start + 1) as _
                            ];
                            let mut lpixel = [
                                (ranges[1].end + 1) as _,
                                (ranges[1].end + 1) as _
                            ];
                            let mut status = 0;

                            unsafe {
                                sys::ffpss(
                                    fits_file.fptr as *mut _,
                                    $data_type.into(),
                                    fpixel.as_mut_ptr(),
                                    lpixel.as_mut_ptr(),
                                    data.as_ptr() as *mut _,
                                    &mut status);
                            }

                            fits_try!(status, ())
                        },
                        Ok(HduInfo::TableInfo { .. }) =>
                            Err("cannot write image data to a table hdu".into()),
                        Ok(HduInfo::AnyInfo) => unreachable!(),
                        Err(e) => Err(e),
                    }
                }
        }
    )
}


read_write_image_impl!(i8, DataType::TSHORT);
read_write_image_impl!(i32, DataType::TINT);
#[cfg(target_pointer_width = "64")]
read_write_image_impl!(i64, DataType::TLONG);
#[cfg(target_pointer_width = "32")]
read_write_image_impl!(i64, DataType::TLONGLONG);
read_write_image_impl!(u8, DataType::TUSHORT);
read_write_image_impl!(u32, DataType::TUINT);
#[cfg(target_pointer_width = "64")]
read_write_image_impl!(u64, DataType::TULONG);
read_write_image_impl!(f32, DataType::TFLOAT);
read_write_image_impl!(f64, DataType::TDOUBLE);

pub enum Column {
    Int32 { name: String, data: Vec<i32> },
    Int64 { name: String, data: Vec<i64> },
    Float { name: String, data: Vec<f32> },
    Double { name: String, data: Vec<f64> },
    String { name: String, data: Vec<String> },
}

pub struct ColumnIterator<'a> {
    current: usize,
    column_descriptions: Vec<ConcreteColumnDescription>,
    fits_file: &'a FitsFile,
}

impl<'a> ColumnIterator<'a> {
    fn new(fits_file: &'a FitsFile) -> Self {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo {
                   column_descriptions,
                   num_rows: _num_rows,
               }) => {
                ColumnIterator {
                    current: 0,
                    column_descriptions: column_descriptions,
                    fits_file: fits_file,
                }
            }
            Err(e) => panic!("{:?}", e),
            _ => panic!("Unknown error occurred"),
        }
    }
}

impl<'a> Iterator for ColumnIterator<'a> {
    type Item = Column;

    fn next(&mut self) -> Option<Self::Item> {
        let ncols = self.column_descriptions.len();

        if self.current < ncols {
            let description = &self.column_descriptions[self.current];
            let current_name = description.name.as_str();
            // let current_type = typechar_to_data_type(description.data_type.as_str());
            let current_type = description.data_type.typ;

            let retval = match current_type {
                ColumnDataType::Int => {
                    i32::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Column::Int32 {
                                name: current_name.to_string(),
                                data: data,
                            }
                        })
                        .ok()
                }
                ColumnDataType::Long => {
                    i64::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Column::Int64 {
                                name: current_name.to_string(),
                                data: data,
                            }
                        })
                        .ok()
                }
                ColumnDataType::Float => {
                    f32::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Column::Float {
                                name: current_name.to_string(),
                                data: data,
                            }
                        })
                        .ok()
                }
                ColumnDataType::Double => {
                    f64::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Column::Double {
                                name: current_name.to_string(),
                                data: data,
                            }
                        })
                        .ok()
                }
                ColumnDataType::String => {
                    String::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Column::String {
                                name: current_name.to_string(),
                                data: data,
                            }
                        })
                        .ok()
                }
                _ => unimplemented!(),
            };

            self.current += 1;

            retval

        } else {
            None
        }
    }
}

/// Struct representing a FITS HDU
///
///
pub struct FitsHdu<'open> {
    fits_file: &'open FitsFile,

    /// Information about the current HDU
    pub info: HduInfo,
    hdu_num: usize,
}

impl<'open> FitsHdu<'open> {
    pub fn new<T: DescribesHdu>(fits_file: &'open FitsFile, hdu_description: T) -> Result<Self> {
        try!(fits_file.change_hdu(hdu_description));
        match fits_file.fetch_hdu_info() {
            Ok(hdu_info) => {
                Ok(FitsHdu {
                    fits_file: fits_file,
                    info: hdu_info,
                    hdu_num: fits_file.hdu_number(),
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Function to make the HDU the current hdu
    fn make_current(&self) -> Result<()> {
        self.fits_file.change_hdu(self.hdu_num)
    }

    /// Read header key
    pub fn read_key<T: ReadsKey>(&self, name: &str) -> Result<T> {
        self.make_current()?;
        T::read_key(self.fits_file, name)
    }

    /// Write header key
    pub fn write_key<T: WritesKey>(&mut self, name: &str, value: T) -> Result<()> {
        self.make_current()?;
        fits_check_readwrite!(self.fits_file);
        T::write_key(self.fits_file, name, value)
    }

    /// Read an image between pixel a and pixel b into a `Vec`
    pub fn read_section<T: ReadWriteImage>(&self, start: usize, end: usize) -> Result<Vec<T>> {
        self.make_current()?;
        T::read_section(self.fits_file, start, end)
    }

    /// Read multiple rows from a fits image
    pub fn read_rows<T: ReadWriteImage>(
        &self,
        start_row: usize,
        num_rows: usize,
    ) -> Result<Vec<T>> {
        self.make_current()?;
        T::read_rows(self.fits_file, start_row, num_rows)
    }

    /// Read a single row from a fits image
    pub fn read_row<T: ReadWriteImage>(&self, row: usize) -> Result<Vec<T>> {
        self.make_current()?;
        T::read_row(self.fits_file, row)
    }

    /// Read a whole fits image into a vector
    pub fn read_image<T: ReadWriteImage>(&self) -> Result<Vec<T>> {
        self.make_current()?;
        T::read_image(self.fits_file)
    }

    /// Write contiguous data to a fits image
    pub fn write_section<T: ReadWriteImage>(
        &mut self,
        start: usize,
        end: usize,
        data: &[T],
    ) -> Result<()> {
        self.make_current()?;
        fits_check_readwrite!(self.fits_file);
        T::write_section(self.fits_file, start, end, data)
    }

    /// Write a rectangular region to a fits image
    pub fn write_region<T: ReadWriteImage>(
        &mut self,
        ranges: &[&Range<usize>],
        data: &[T],
    ) -> Result<()> {
        self.make_current()?;
        fits_check_readwrite!(self.fits_file);
        T::write_region(self.fits_file, ranges, data)
    }

    /// Read a square region into a `Vec`
    pub fn read_region<T: ReadWriteImage>(&self, ranges: &[&Range<usize>]) -> Result<Vec<T>> {
        self.make_current()?;
        T::read_region(self.fits_file, ranges)
    }

    pub fn resize(&mut self, new_size: &[usize]) -> Result<()> {
        self.make_current()?;
        fits_check_readwrite!(self.fits_file);

        match self.info {
            HduInfo::ImageInfo { image_type, .. } => {
                let mut status = 0;
                unsafe {
                    sys::ffrsim(
                        self.fits_file.fptr as *mut _,
                        image_type.into(),
                        2,
                        new_size.as_ptr() as *mut _,
                        &mut status,
                    );
                }
                fits_try!(status, ())
            }
            HduInfo::TableInfo { .. } => Err("cannot resize binary table".into()),
            HduInfo::AnyInfo => unreachable!(),
        }

    }

    pub fn get_column_no<T: Into<String>>(&self, col_name: T) -> Result<usize> {
        self.make_current()?;

        let mut status = 0;
        let mut colno = 0;

        let c_col_name = {
            let col_name = col_name.into();
            ffi::CString::new(col_name.as_str())?
        };

        unsafe {
            sys::ffgcno(
                self.fits_file.fptr as *mut _,
                CaseSensitivity::CASEINSEN as _,
                c_col_name.as_ptr() as *mut _,
                &mut colno,
                &mut status,
            );
        }
        fits_try!(status, (colno - 1) as usize)
    }

    /// Read a binary table column
    pub fn read_col<T: ReadsCol>(&self, name: &str) -> Result<Vec<T>> {
        self.make_current()?;
        T::read_col(self.fits_file, name)
    }

    pub fn read_col_range<T: ReadsCol>(&self, name: &str, range: &Range<usize>) -> Result<Vec<T>> {
        self.make_current()?;
        T::read_col_range(self.fits_file, name, range)
    }

    pub fn write_col<T: WritesCol, N: Into<String>>(
        &mut self,
        name: N,
        col_data: &[T],
    ) -> Result<()> {
        self.make_current()?;
        fits_check_readwrite!(self.fits_file);
        T::write_col(self.fits_file, self, name, col_data)
    }

    pub fn write_col_range<T: WritesCol, N: Into<String>>(
        &mut self,
        name: N,
        col_data: &[T],
        rows: &Range<usize>,
    ) -> Result<()> {
        self.make_current()?;
        fits_check_readwrite!(self.fits_file);
        T::write_col_range(self.fits_file, self, name, col_data, rows)
    }

    pub fn columns(&self) -> ColumnIterator {
        self.make_current().expect("Could not change current hdu");
        ColumnIterator::new(self.fits_file)
    }
}


#[cfg(test)]
mod test {
    #[cfg(feature = "default")]
    extern crate fitsio_sys as sys;
    #[cfg(feature = "bindgen")]
    extern crate fitsio_sys_bindgen as sys;

    extern crate tempdir;

    use FitsHdu;
    use fitsfile::FitsFile;
    use types::*;
    use fitsfile::ImageDescription;
    use errors::{Result, Error};
    use std::path::Path;
    use std::{f32, f64};

    /// Function to allow access to a temporary file
    fn with_temp_file<F>(callback: F)
    where
        F: for<'a> Fn(&'a str),
    {
        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");

        let filename_str = filename.to_str().expect("cannot create string filename");
        callback(filename_str);
    }

    /// Function to create a temporary file and copy the example file
    fn duplicate_test_file<F>(callback: F)
    where
        F: for<'a> Fn(&'a str),
    {
        use std::fs;
        with_temp_file(|filename| {
            fs::copy("../testdata/full_example.fits", &filename).expect("Could not copy test file");
            callback(filename);
        });
    }

    #[test]
    fn opening_an_existing_file() {
        match FitsFile::open("../testdata/full_example.fits") {
            Ok(_) => {}
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn creating_a_new_file() {
        with_temp_file(|filename| {
            FitsFile::create(filename)
                .map(|f| {
                    assert!(Path::new(filename).exists());

                    // Ensure the empty primary has been written
                    let naxis: i64 = f.hdu(0).unwrap().read_key("NAXIS").unwrap();
                    assert_eq!(naxis, 0);
                })
                .unwrap();
        });
    }

    #[test]
    fn cannot_write_to_readonly_file() {
        use columndescription::*;

        duplicate_test_file(|filename| {
            let f = FitsFile::open(filename).unwrap();

            match f.create_image(
                "FOO".to_string(),
                &ImageDescription {
                    data_type: ImageType::LONG_IMG,
                    dimensions: &[100, 100],
                },
            ) {
                Err(Error::Fits(e)) => {
                    assert_eq!(e.status, 602);
                }
                _ => panic!("Should fail"),
            }

            let bar_column_description = ColumnDescription::new("bar")
                .with_type(ColumnDataType::Int)
                .create()
                .unwrap();
            match f.create_table("FOO".to_string(), &vec![bar_column_description]) {
                Err(Error::Fits(e)) => {
                    assert_eq!(e.status, 602);
                }
                _ => panic!("Should fail"),
            }
        });
    }

    #[test]
    fn editing_a_current_file() {
        duplicate_test_file(|filename| {
            {
                let f = FitsFile::edit(filename).unwrap();
                let mut image_hdu = f.hdu(0).unwrap();

                let data_to_write: Vec<i64> = (0..100).map(|_| 10101).collect();
                image_hdu.write_section(0, 100, &data_to_write).unwrap();
            }

            {
                let f = FitsFile::open(filename).unwrap();
                let hdu = f.hdu(0).unwrap();
                let read_data: Vec<i64> = hdu.read_section(0, 10).unwrap();
                assert_eq!(read_data, vec![10101; 10]);
            }
        });
    }

    #[test]
    fn fetching_a_hdu() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        for i in 0..2 {
            f.change_hdu(i).unwrap();
            assert_eq!(f.hdu_number(), i);
        }

        match f.change_hdu(2) {
            Err(Error::Fits(e)) => assert_eq!(e.status, 107),
            _ => panic!("Error checking for failure"),
        }

        f.change_hdu("TESTEXT").unwrap();
        assert_eq!(f.hdu_number(), 1);
    }

    #[test]
    fn fetching_hdu_info() {
        use columndescription::*;

        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::ImageInfo { shape, image_type }) => {
                assert_eq!(shape.len(), 2);
                assert_eq!(shape, vec![100, 100]);
                assert_eq!(image_type, ImageType::LONG_IMG);
            }
            Err(e) => panic!("Error fetching hdu info {:?}", e),
            _ => panic!("Unknown error"),
        }

        f.change_hdu(1).unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::TableInfo {
                   column_descriptions,
                   num_rows,
               }) => {
                assert_eq!(num_rows, 50);
                assert_eq!(
                    column_descriptions
                        .iter()
                        .map(|desc| desc.name.clone())
                        .collect::<Vec<String>>(),
                    vec![
                        "intcol".to_string(),
                        "floatcol".to_string(),
                        "doublecol".to_string(),
                        "strcol".to_string(),
                    ]
                );
                assert_eq!(
                    column_descriptions
                        .iter()
                        .map(|ref desc| desc.data_type.typ.clone())
                        .collect::<Vec<ColumnDataType>>(),
                    vec![
                        ColumnDataType::Int,
                        ColumnDataType::Float,
                        ColumnDataType::Double,
                        ColumnDataType::String,
                    ]
                );
            }
            Err(e) => panic!("Error fetching hdu info {:?}", e),
            _ => panic!("Unknown error"),
        }
    }

    #[test]
    fn test_fits_try() {
        use stringutils;
        use errors::Error;
        use fitserror::FitsError;

        let status = 0;
        assert_eq!(fits_try!(status, 10), Ok(10));

        let status = 105;
        assert_eq!(
            fits_try!(status, 10),
            Err(Error::Fits(FitsError {
                status: status,
                message: stringutils::status_to_string(status).unwrap().unwrap(),
            }))
        );
    }

    #[test]
    fn getting_file_open_mode() {
        duplicate_test_file(|filename| {
            let f = FitsFile::open(filename).unwrap();
            assert_eq!(f.open_mode().unwrap(), FileOpenMode::READONLY);
        });

        duplicate_test_file(|filename| {
            let f = FitsFile::edit(filename).unwrap();
            assert_eq!(f.open_mode().unwrap(), FileOpenMode::READWRITE);
        });
    }

    #[test]
    fn adding_new_table() {
        use columndescription::*;

        with_temp_file(|filename| {

            {
                let f = FitsFile::create(filename).unwrap();
                let table_description = vec![
                    ColumnDescription::new("bar")
                        .with_type(ColumnDataType::Int)
                        .create()
                        .unwrap(),
                ];
                f.create_table("foo".to_string(), &table_description)
                    .unwrap();
            }

            FitsFile::open(filename)
                .map(|f| {
                    f.change_hdu("foo").unwrap();
                    match f.fetch_hdu_info() {
                        Ok(HduInfo::TableInfo { column_descriptions, .. }) => {
                            let column_names = column_descriptions
                                .iter()
                                .map(|desc| desc.name.clone())
                                .collect::<Vec<String>>();
                            let column_types = column_descriptions
                                .iter()
                                .map(|desc| desc.data_type.typ.clone())
                                .collect::<Vec<_>>();
                            assert_eq!(column_names, vec!["bar".to_string()]);
                            assert_eq!(column_types, vec![ColumnDataType::Int]);
                        }
                        thing => panic!("{:?}", thing),
                    }
                })
                .unwrap();
        });
    }

    #[test]
    fn adding_new_image() {
        with_temp_file(|filename| {
            {
                let f = FitsFile::create(filename).unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::LONG_IMG,
                    dimensions: &[100, 20],
                };
                f.create_image("foo".to_string(), &image_description)
                    .unwrap();
            }

            FitsFile::open(filename)
                .map(|f| {
                    f.change_hdu("foo").unwrap();
                    match f.fetch_hdu_info() {
                        Ok(HduInfo::ImageInfo { shape, .. }) => {
                            assert_eq!(shape, vec![100, 20]);
                        }
                        thing => panic!("{:?}", thing),
                    }
                })
                .unwrap();

        });
    }

    #[test]
    fn fetching_hdu_object_hdu_info() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let testext = f.hdu("TESTEXT").unwrap();
        match testext.info {
            HduInfo::TableInfo { num_rows, .. } => {
                assert_eq!(num_rows, 50);
            }
            _ => panic!("Incorrect HDU type found"),
        }
    }

    #[test]
    fn fetch_current_hdu() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        f.change_hdu("TESTEXT").unwrap();
        let hdu = f.current_hdu().unwrap();

        assert_eq!(
            hdu.read_key::<String>("EXTNAME").unwrap(),
            "TESTEXT".to_string()
        );
    }

    #[test]
    fn creating_new_image_returns_hdu_object() {
        with_temp_file(|filename| {
            let f = FitsFile::create(filename).unwrap();
            let image_description = ImageDescription {
                data_type: ImageType::LONG_IMG,
                dimensions: &[100, 20],
            };
            let hdu: FitsHdu = f.create_image("foo".to_string(), &image_description)
                .unwrap();
            assert_eq!(
                hdu.read_key::<String>("EXTNAME").unwrap(),
                "foo".to_string()
            );
        });
    }

    #[test]
    fn creating_new_table_returns_hdu_object() {
        use columndescription::*;

        with_temp_file(|filename| {
            let f = FitsFile::create(filename).unwrap();
            let table_description = vec![
                ColumnDescription::new("bar")
                    .with_type(ColumnDataType::Int)
                    .create()
                    .unwrap(),
            ];
            let hdu: FitsHdu = f.create_table("foo".to_string(), &table_description)
                .unwrap();
            assert_eq!(
                hdu.read_key::<String>("EXTNAME").unwrap(),
                "foo".to_string()
            );
        });
    }

    // FitsHdu tests

    /// Helper function for float comparisons
    fn floats_close_f32(a: f32, b: f32) -> bool {
        (a - b).abs() < f32::EPSILON
    }

    fn floats_close_f64(a: f64, b: f64) -> bool {
        (a - b).abs() < f64::EPSILON
    }

    #[test]
    fn test_manually_creating_a_fits_hdu() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = FitsHdu::new(&f, "TESTEXT").unwrap();
        match hdu.info {
            HduInfo::TableInfo { num_rows, .. } => {
                assert_eq!(num_rows, 50);
            }
            _ => panic!("Incorrect HDU type found"),
        }
    }

    #[test]
    fn reading_header_keys() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        match hdu.read_key::<i64>("INTTEST") {
            Ok(value) => assert_eq!(value, 42),
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match hdu.read_key::<f64>("DBLTEST") {
            Ok(value) => {
                assert!(
                    floats_close_f64(value, 0.09375),
                    "{:?} != {:?}",
                    value,
                    0.09375
                )
            }
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match hdu.read_key::<String>("TEST") {
            Ok(value) => assert_eq!(value, "value"),
            Err(e) => panic!("Error reading key: {:?}", e),
        }
    }

    // Writing data
    #[test]
    fn writing_header_keywords() {
        with_temp_file(|filename| {
            // Scope ensures file is closed properly
            {
                let f = FitsFile::create(filename).unwrap();
                f.hdu(0).unwrap().write_key("FOO", 1i64).unwrap();
                f.hdu(0)
                    .unwrap()
                    .write_key("BAR", "baz".to_string())
                    .unwrap();
            }

            FitsFile::open(filename)
                .map(|f| {
                    assert_eq!(f.hdu(0).unwrap().read_key::<i64>("foo").unwrap(), 1);
                    assert_eq!(
                        f.hdu(0).unwrap().read_key::<String>("bar").unwrap(),
                        "baz".to_string()
                    );
                })
                .unwrap();
        });
    }

    #[test]
    fn fetching_column_width() {
        use super::column_display_width;

        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        f.hdu(1).unwrap();
        let width = column_display_width(&f, 3).unwrap();
        assert_eq!(width, 7);
    }

    #[test]
    fn read_columns() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let intcol_data: Vec<i32> = hdu.read_col("intcol").unwrap();
        assert_eq!(intcol_data[0], 18);
        assert_eq!(intcol_data[15], 10);
        assert_eq!(intcol_data[49], 12);

        let floatcol_data: Vec<f32> = hdu.read_col("floatcol").unwrap();
        assert!(
            floats_close_f32(floatcol_data[0], 17.496801),
            "{:?} != {:?}",
            floatcol_data[0],
            17.496801
        );
        assert!(
            floats_close_f32(floatcol_data[15], 19.570272),
            "{:?} != {:?}",
            floatcol_data[15],
            19.570272
        );
        assert!(
            floats_close_f32(floatcol_data[49], 10.217053),
            "{:?} != {:?}",
            floatcol_data[49],
            10.217053
        );

        let doublecol_data: Vec<f64> = hdu.read_col("doublecol").unwrap();
        assert!(
            floats_close_f64(doublecol_data[0], 16.959972808730814),
            "{:?} != {:?}",
            doublecol_data[0],
            16.959972808730814
        );
        assert!(
            floats_close_f64(doublecol_data[15], 19.013522579233065),
            "{:?} != {:?}",
            doublecol_data[15],
            19.013522579233065
        );
        assert!(
            floats_close_f64(doublecol_data[49], 16.61153656123406),
            "{:?} != {:?}",
            doublecol_data[49],
            16.61153656123406
        );
    }

    #[test]
    fn read_string_col() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let strcol: Vec<String> = hdu.read_col("strcol").unwrap();
        assert_eq!(strcol.len(), 50);
        assert_eq!(strcol[0], "value0");
        assert_eq!(strcol[15], "value15");
        assert_eq!(strcol[49], "value49");
    }

    #[test]
    fn read_column_regions() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let intcol_data: Vec<i32> = hdu.read_col_range("intcol", &(0..2)).unwrap();
        assert_eq!(intcol_data.len(), 3);
        assert_eq!(intcol_data[0], 18);
        assert_eq!(intcol_data[1], 13);
    }

    #[test]
    fn read_string_column_regions() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let intcol_data: Vec<String> = hdu.read_col_range("strcol", &(0..2)).unwrap();
        assert_eq!(intcol_data.len(), 3);
        assert_eq!(intcol_data[0], "value0");
        assert_eq!(intcol_data[1], "value1");
    }

    #[test]
    fn read_column_region_check_ranges() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let result_data: Result<Vec<i32>> = hdu.read_col_range("intcol", &(0..2_000_000));
        assert!(result_data.is_err());
    }

    #[test]
    fn column_iterator() {
        use super::Column;

        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let column_names: Vec<String> = hdu.columns()
            .map(|col| match col {
                Column::Int32 { name, .. } => name,
                Column::Int64 { name, .. } => name,
                Column::Float { name, .. } => name,
                Column::Double { name, .. } => name,
                Column::String { name, .. } => name,
            })
            .collect();

        assert_eq!(
            column_names,
            vec![
                "intcol".to_string(),
                "floatcol".to_string(),
                "doublecol".to_string(),
                "strcol".to_string(),
            ]
        );
    }

    #[test]
    fn column_number() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu("testext").unwrap();
        assert_eq!(hdu.get_column_no("intcol").unwrap(), 0);
        assert_eq!(hdu.get_column_no("floatcol").unwrap(), 1);
        assert_eq!(hdu.get_column_no("doublecol").unwrap(), 2);
    }

    #[test]
    fn write_column_data() {
        use columndescription::*;

        with_temp_file(|filename| {
            let data_to_write: Vec<i32> = vec![10101; 10];
            {
                let f = FitsFile::create(filename).unwrap();
                let table_description = vec![
                    ColumnDescription::new("bar")
                        .with_type(ColumnDataType::Int)
                        .create()
                        .unwrap(),
                ];
                let mut hdu = f.create_table("foo".to_string(), &table_description)
                    .unwrap();

                hdu.write_col("bar", &data_to_write).unwrap();
            }

            let f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let data: Vec<i32> = hdu.read_col("bar").unwrap();
            assert_eq!(data, data_to_write);
        });
    }

    #[test]
    fn cannot_write_column_to_image_hdu() {
        with_temp_file(|filename| {
            let data_to_write: Vec<i32> = vec![10101; 10];

            let f = FitsFile::create(filename).unwrap();

            let image_description = ImageDescription {
                data_type: ImageType::LONG_IMG,
                dimensions: &[100, 20],
            };
            let mut hdu = f.create_image("foo".to_string(), &image_description)
                .unwrap();

            match hdu.write_col("bar", &data_to_write) {
                Err(Error::Message(msg)) => {
                    assert_eq!(msg, "Cannot write column data to FITS image")
                }
                _ => panic!("Should return an error"),
            }
        });
    }

    #[test]
    fn write_column_subset() {
        use columndescription::*;

        with_temp_file(|filename| {
            let data_to_write: Vec<i32> = vec![10101; 10];
            {
                let f = FitsFile::create(filename).unwrap();
                let table_description = vec![
                    ColumnDescription::new("bar")
                        .with_type(ColumnDataType::Int)
                        .create()
                        .unwrap(),
                ];
                let mut hdu = f.create_table("foo".to_string(), &table_description)
                    .unwrap();

                hdu.write_col_range("bar", &data_to_write, &(0..5)).unwrap();
            }

            let f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let data: Vec<i32> = hdu.read_col("bar").unwrap();
            assert_eq!(data.len(), 6);
            assert_eq!(data[..], data_to_write[0..6]);
        });
    }

    #[test]
    #[ignore]
    fn write_string_col() {
        use columndescription::*;

        with_temp_file(|filename| {
            let mut data_to_write: Vec<String> = Vec::new();
            for i in 0..50 {
                data_to_write.push(format!("value{}", i));
            }

            {
                let f = FitsFile::create(filename).unwrap();
                let table_description = vec![
                    ColumnDescription::new("bar")
                        .with_type(ColumnDataType::String)
                        .that_repeats(7)
                        .create()
                        .unwrap(),
                ];
                let mut hdu = f.create_table("foo".to_string(), &table_description)
                    .unwrap();

                hdu.write_col("bar", &data_to_write).unwrap();
            }

            let f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let data: Vec<String> = hdu.read_col("bar").unwrap();
            assert_eq!(data.len(), data_to_write.len());
            assert_eq!(data[0], " value0");
            assert_eq!(data[49], "value49");
        });
    }

    #[test]
    fn read_image_data() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let first_row: Vec<i32> = hdu.read_section(0, 100).unwrap();
        assert_eq!(first_row.len(), 100);
        assert_eq!(first_row[0], 108);
        assert_eq!(first_row[49], 176);

        let second_row: Vec<i32> = hdu.read_section(100, 200).unwrap();
        assert_eq!(second_row.len(), 100);
        assert_eq!(second_row[0], 177);
        assert_eq!(second_row[49], 168);
    }

    #[test]
    fn read_whole_image() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let image: Vec<i32> = hdu.read_image().unwrap();
        assert_eq!(image.len(), 10000);
    }

    #[test]
    fn read_image_rows() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let row: Vec<i32> = hdu.read_rows(0, 2).unwrap();
        let ref_row: Vec<i32> = hdu.read_section(0, 200).unwrap();
        assert_eq!(row, ref_row);
    }

    #[test]
    fn read_image_row() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let row: Vec<i32> = hdu.read_row(0).unwrap();
        let ref_row: Vec<i32> = hdu.read_section(0, 100).unwrap();
        assert_eq!(row, ref_row);
    }

    #[test]
    fn read_image_slice() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();

        let xcoord = 5..7;
        let ycoord = 2..3;

        let chunk: Vec<i32> = hdu.read_region(&vec![&ycoord, &xcoord]).unwrap();
        assert_eq!(chunk.len(), 2 * 3);
        assert_eq!(chunk[0], 168);
        assert_eq!(chunk[chunk.len() - 1], 132);
    }

    #[test]
    fn read_image_region_from_table() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu("TESTEXT").unwrap();
        match hdu.read_region::<i32>(&vec![&(0..10), &(0..10)]) {
            Err(Error::Message(msg)) => {
                assert!(msg.contains("cannot read image data from a table hdu"))
            }
            _ => panic!("SHOULD FAIL"),
        }
    }

    #[test]
    fn read_image_section_from_table() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu("TESTEXT").unwrap();
        if let Err(Error::Message(msg)) = hdu.read_section::<i32>(0, 100) {
            assert!(msg.contains("cannot read image data from a table hdu"));
        } else {
            panic!("Should have been an error");
        }
    }

    #[test]
    fn test_write_image_section() {
        with_temp_file(|filename| {
            let data_to_write: Vec<i64> = (0..100).map(|v| v + 50).collect();

            // Scope ensures file is closed properly
            {
                use fitsfile::ImageDescription;

                let f = FitsFile::create(filename).unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::LONG_IMG,
                    dimensions: &[100, 20],
                };
                let mut hdu = f.create_image("foo".to_string(), &image_description)
                    .unwrap();
                hdu.write_section(0, 100, &data_to_write).unwrap();
            }

            let f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let first_row: Vec<i64> = hdu.read_section(0, 100).unwrap();
            assert_eq!(first_row, data_to_write);

        });
    }

    #[test]
    fn test_write_image_region() {
        with_temp_file(|filename| {
            // Scope ensures file is closed properly
            {
                use fitsfile::ImageDescription;

                let f = FitsFile::create(filename).unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::LONG_IMG,
                    dimensions: &[100, 20],
                };
                let mut hdu = f.create_image("foo".to_string(), &image_description)
                    .unwrap();

                let data: Vec<i64> = (0..121).map(|v| v + 50).collect();
                hdu.write_region(&[&(0..10), &(0..10)], &data).unwrap();
            }

            let f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let chunk: Vec<i64> = hdu.read_region(&[&(0..10), &(0..10)]).unwrap();
            assert_eq!(chunk.len(), 11 * 11);
            assert_eq!(chunk[0], 50);
            assert_eq!(chunk[25], 75);
        });
    }

    #[test]
    fn resizing_images() {
        with_temp_file(|filename| {

            // Scope ensures file is closed properly
            {
                use fitsfile::ImageDescription;

                let f = FitsFile::create(filename).unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::LONG_IMG,
                    dimensions: &[100, 20],
                };
                f.create_image("foo".to_string(), &image_description)
                    .unwrap();
            }

            /* Now resize the image */
            {
                let f = FitsFile::edit(filename).unwrap();
                let mut hdu = f.hdu("foo").unwrap();
                hdu.resize(&vec![1024, 1024]).unwrap();
            }

            /* Images are only resized when flushed to disk, so close the file and
             * open it again */
            {
                let f = FitsFile::edit(filename).unwrap();
                let hdu = f.hdu("foo").unwrap();
                match hdu.info {
                    HduInfo::ImageInfo { shape, .. } => {
                        assert_eq!(shape, vec![1024, 1024]);
                    }
                    _ => panic!("Unexpected hdu type"),
                }
            }
        });
    }

    #[test]
    fn write_image_section_to_table() {
        with_temp_file(|filename| {
            let data_to_write: Vec<i64> = (0..100).map(|v| v + 50).collect();

            use columndescription::*;

            let f = FitsFile::create(filename).unwrap();
            let table_description = &[
                ColumnDescription::new("bar")
                    .with_type(ColumnDataType::Int)
                    .create()
                    .unwrap(),
            ];
            let mut hdu = f.create_table("foo".to_string(), table_description)
                .unwrap();
            if let Err(Error::Message(msg)) = hdu.write_section(0, 100, &data_to_write) {
                assert_eq!(msg, "cannot write image data to a table hdu");
            } else {
                panic!("Should have thrown an error");
            }
        });
    }

    #[test]
    fn write_image_region_to_table() {
        use columndescription::*;

        with_temp_file(|filename| {
            let data_to_write: Vec<i64> = (0..100).map(|v| v + 50).collect();

            let f = FitsFile::create(filename).unwrap();
            let table_description = &[
                ColumnDescription::new("bar")
                    .with_type(ColumnDataType::Int)
                    .create()
                    .unwrap(),
            ];
            let mut hdu = f.create_table("foo".to_string(), table_description)
                .unwrap();

            if let Err(Error::Message(msg)) =
                hdu.write_region(&vec![&(0..10), &(0..10)], &data_to_write)
            {
                assert_eq!(msg, "cannot write image data to a table hdu");
            } else {
                panic!("Should have thrown an error");
            }
        });
    }

    #[test]
    fn multi_hdu_workflow() {
        /* Check that hdu objects change the current HDU on every file access method */

        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let primary_hdu = f.hdu(0).unwrap();
        let column_hdu = f.hdu(1).unwrap();

        let first_row: Vec<i32> = primary_hdu.read_section(0, 100).unwrap();
        assert_eq!(first_row.len(), 100);
        assert_eq!(first_row[0], 108);
        assert_eq!(first_row[49], 176);

        let intcol_data: Vec<i32> = column_hdu.read_col("intcol").unwrap();
        assert_eq!(intcol_data[0], 18);
        assert_eq!(intcol_data[49], 12);
    }

    #[test]
    fn access_fptr_unsafe() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let fptr: *const sys::fitsfile = unsafe { f.as_raw() };
        assert!(!fptr.is_null());
    }

    #[test]
    fn extended_filename_syntax() {
        let filename = "../testdata/full_example.fits[TESTEXT]";
        let f = FitsFile::open(filename).unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { .. }) => {}
            Ok(HduInfo::ImageInfo { .. }) => panic!("Should be binary table"),
            _ => panic!("ERROR!"),
        }
    }
}
