//! [`FitsFile`](struct.FitsFile.html) and [`FitsHdu`](struct.FitsHdu.html)

/* Depending on the architecture, different functions have to be called. For example arm systems
 * define `int` as 4 bytes, and `long` as 4 bytes, unlike x86_64 systems which define `long` types
 * as 8 bytes.
 *
 * In this case, we have to use `_longlong` cfitsio functions on arm architectures (and other
 * similar architectures).
 */

use longnam::*;
use fitsio_sys::fitsfile;
use stringutils::{self, status_to_string};
use errors::{Error, IndexError, Result};
use fitserror::{check_status, FitsError};
use columndescription::*;
use libc;
use types::{CaseSensitivity, DataType, FileOpenMode, HduInfo, ImageType};
use std::ffi;
use std::io::{self, Write};
use std::ptr;
use std::path::Path;
use std::ops::Range;

static MAX_VALUE_LENGTH: usize = 71;

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
#[derive(Clone)]
pub struct ImageDescription<'a> {
    /// Data type of the new image
    pub data_type: ImageType,

    /// Shape of the image
    ///
    /// Unlike cfitsio, the order of the dimensions follows the C convention, i.e. [row-major
    /// order](https://en.wikipedia.org/wiki/Row-_and_column-major_order).
    pub dimensions: &'a [usize],
}

/// Main entry point to the FITS file format
///
//
pub struct FitsFile {
    /// Name of the file
    pub filename: String,
    open_mode: FileOpenMode,
    fptr: *const fitsfile,
}

impl FitsFile {
    /// Open a fits file from disk
    ///
    /// ## Example
    ///
    /// ```rust
    /// use fitsio::FitsFile;
    /// # use std::error::Error;
    ///
    /// # fn run() -> Result<(), Box<Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// // let filename = ...;
    /// let fptr = FitsFile::open(filename)?;
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # run().unwrap();
    /// # }
    /// ```
    pub fn open<T: AsRef<Path>>(filename: T) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let filename = filename.as_ref().to_str().expect("converting filename");
        let c_filename = ffi::CString::new(filename)?;

        unsafe {
            fits_open_file(
                &mut fptr as *mut *mut fitsfile,
                c_filename.as_ptr(),
                FileOpenMode::READONLY as libc::c_int,
                &mut status,
            );
        }

        check_status(status).map(|_| FitsFile {
            fptr,
            open_mode: FileOpenMode::READONLY,
            filename: filename.to_string(),
        })
    }

    /// Open a fits file in read/write mode
    ///
    /// ## Example
    ///
    /// ```rust
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// use fitsio::FitsFile;
    ///
    /// // let filename = ...;
    /// let fptr = FitsFile::edit(filename)?;
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    pub fn edit<T: AsRef<Path>>(filename: T) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let filename = filename.as_ref().to_str().expect("converting filename");
        let c_filename = ffi::CString::new(filename)?;

        unsafe {
            fits_open_file(
                &mut fptr as *mut *mut _,
                c_filename.as_ptr(),
                FileOpenMode::READWRITE as libc::c_int,
                &mut status,
            );
        }

        check_status(status).map(|_| FitsFile {
            fptr,
            open_mode: FileOpenMode::READWRITE,
            filename: filename.to_string(),
        })
    }

    /// Create a new fits file on disk
    ///
    /// The [`create`] method returns a [`NewFitsFile`], which is an
    /// internal representation of a temporary fits file on disk, before the file is fully created.
    ///
    /// This representation has two methods: [`open`] and [`with_custom_primary`]. The [`open`]
    /// method actually creates the file on disk, but before calling this method, the
    /// [`with_custom_primary`] method can be used to add a custom primary HDU. This is mostly
    /// useful for images. Otherwise, a default primary HDU is created.  An example of not adding a
    /// custom primary HDU is shown above. Below we see an example of [`with_custom_primary`]:
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate tempdir;
    /// # extern crate fitsio;
    /// # use fitsio::FitsFile;
    /// # use fitsio::types::ImageType;
    /// # use fitsio::fitsfile::ImageDescription;
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
    /// # let tdir_path = tdir.path();
    /// # let filename = tdir_path.join("test.fits");
    /// use fitsio::FitsFile;
    ///
    /// // let filename = ...;
    /// let description = ImageDescription {
    ///     data_type: ImageType::Double,
    ///     dimensions: &[52, 103],
    /// };
    ///
    /// let fptr = FitsFile::create(filename)
    ///     .with_custom_primary(&description)
    ///     .open()?;
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    ///
    /// [`create`]: #method.create
    /// [`NewFitsFile`]: struct.NewFitsFile.html
    /// [`open`]: struct.NewFitsFile.html#method.open
    /// [`with_custom_primary`]: struct.NewFitsFile.html#method.with_custom_primary
    pub fn create<'a, T: AsRef<Path>>(path: T) -> NewFitsFile<'a, T> {
        NewFitsFile {
            path,
            image_description: None,
        }
    }

    /// Method to extract what open mode the file is in
    fn open_mode(&self) -> Result<FileOpenMode> {
        let mut status = 0;
        let mut iomode = 0;
        unsafe {
            fits_file_mode(self.fptr as *mut _, &mut iomode, &mut status);
        }

        check_status(status).map(|_| match iomode {
            0 => FileOpenMode::READONLY,
            1 => FileOpenMode::READWRITE,
            _ => unreachable!(),
        })
    }

    fn add_empty_primary(&self) -> Result<()> {
        let mut status = 0;
        unsafe {
            fits_write_imghdr(
                self.fptr as *mut _,
                ImageType::UnsignedByte.into(),
                0,
                ptr::null_mut(),
                &mut status,
            );
        }

        check_status(status)
    }

    /// Change the current HDU
    fn change_hdu<T: DescribesHdu>(&mut self, hdu_description: T) -> Result<()> {
        hdu_description.change_hdu(self)
    }

    /// Return a new HDU object
    ///
    /// HDU information belongs to the [`FitsHdu`] object. HDUs can be fetched by `String`/`str` or
    /// integer (0-indexed).  The `HduInfo` object contains information about the current HDU:
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate fitsio;
    /// # #[cfg(feature = "default")]
    /// # extern crate fitsio_sys as sys;
    /// # #[cfg(feature = "bindgen")]
    /// # extern crate fitsio_sys_bindgen as sys;
    /// # use fitsio::{FitsFile, HduInfo};
    /// #
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// # let mut fptr = FitsFile::open(filename)?;
    /// let hdu = fptr.hdu(0)?;
    /// // image HDU
    /// if let HduInfo::ImageInfo { shape, .. } = hdu.info {
    ///    println!("Image is {}-dimensional", shape.len());
    ///    println!("Found image with shape {:?}", shape);
    /// }
    /// # let hdu = fptr.hdu("TESTEXT")?;
    ///
    /// // tables
    /// if let HduInfo::TableInfo { column_descriptions, num_rows, .. } = hdu.info {
    ///     println!("Table contains {} rows", num_rows);
    ///     println!("Table has {} columns", column_descriptions.len());
    /// }
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    ///
    /// [`FitsHdu`]: struct.FitsHdu.html
    pub fn hdu<T: DescribesHdu>(&mut self, hdu_description: T) -> Result<FitsHdu> {
        FitsHdu::new(self, hdu_description)
    }

    /// Return the primary hdu (HDU 0)
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate fitsio;
    /// # #[cfg(feature = "default")]
    /// # extern crate fitsio_sys as sys;
    /// # #[cfg(feature = "bindgen")]
    /// # extern crate fitsio_sys_bindgen as sys;
    /// # use fitsio::{FitsFile, HduInfo};
    /// #
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// # let mut fptr = FitsFile::open(filename)?;
    /// let hdu = fptr.hdu(0)?;
    /// let phdu = fptr.primary_hdu()?;
    /// assert_eq!(hdu, phdu);
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    pub fn primary_hdu(&mut self) -> Result<FitsHdu> {
        self.hdu(0)
    }

    /// Return the number of HDU objects in the file
    fn num_hdus(&mut self) -> Result<usize> {
        let mut status = 0;
        let mut num_hdus = 0;
        unsafe {
            fits_get_num_hdus(self.fptr as *mut _, &mut num_hdus, &mut status);
        }

        check_status(status).map(|_| num_hdus as _)
    }

    /// Return the list of HDU names
    fn hdu_names(&mut self) -> Result<Vec<String>> {
        let num_hdus = self.num_hdus()?;
        let mut result = Vec::with_capacity(num_hdus);
        for i in 0..num_hdus {
            let hdu = self.hdu(i)?;
            let name = hdu.name(self)?;
            result.push(name);
        }
        Ok(result)
    }

    fn make_current(&mut self, hdu: &FitsHdu) -> Result<()> {
        self.change_hdu(hdu.hdu_num)
    }

    fn hdu_number(&self) -> usize {
        let mut hdu_num = 0;
        unsafe {
            fits_get_hdu_num(self.fptr as *mut _, &mut hdu_num);
        }
        (hdu_num - 1) as usize
    }

    /// Get the current hdu as an HDU object
    fn current_hdu(&mut self) -> Result<FitsHdu> {
        let current_hdu_number = self.hdu_number();
        self.hdu(current_hdu_number)
    }

    /// Get the current hdu info
    fn fetch_hdu_info(&self) -> Result<HduInfo> {
        let mut status = 0;
        let mut hdu_type = 0;

        unsafe {
            fits_get_hdu_type(self.fptr as *mut _, &mut hdu_type, &mut status);
        }

        let hdu_type = match hdu_type {
            0 => {
                let mut dimensions = 0;
                unsafe {
                    fits_get_img_dim(self.fptr as *mut _, &mut dimensions, &mut status);
                }

                let mut shape = vec![0; dimensions as usize];
                unsafe {
                    fits_get_img_size(
                        self.fptr as *mut _,
                        dimensions,
                        shape.as_mut_ptr(),
                        &mut status,
                    );
                }

                /* Reverse the image dimensions to be more like the C convention */
                shape.reverse();

                let mut bitpix = 0;
                unsafe {
                    /* Use equiv type as this is more useful
                     *
                     * See description here:
                     * https://heasarc.gsfc.nasa.gov/docs/software/fitsio/c/c_user/node40.html
                     */
                    fits_get_img_equivtype(self.fptr as *mut _, &mut bitpix, &mut status);
                }

                let image_type = match bitpix {
                    8 => ImageType::UnsignedByte,
                    10 => ImageType::Byte,
                    16 => ImageType::Short,
                    20 => ImageType::UnsignedShort,
                    32 => ImageType::Long,
                    40 => ImageType::UnsignedLong,
                    64 => ImageType::LongLong,
                    -32 => ImageType::Float,
                    -64 => ImageType::Double,
                    _ => unreachable!(&format!("Unhandled image bitpix type: {}", bitpix)),
                };

                HduInfo::ImageInfo {
                    shape: shape.iter().map(|v| *v as usize).collect(),
                    image_type,
                }
            }
            1 | 2 => {
                let mut num_rows = 0;
                unsafe {
                    fits_get_num_rows(self.fptr as *mut _, &mut num_rows, &mut status);
                }

                let mut num_cols = 0;
                unsafe {
                    fits_get_num_cols(self.fptr as *mut _, &mut num_cols, &mut status);
                }
                let mut column_descriptions = Vec::with_capacity(num_cols as usize);

                for i in 0..num_cols {
                    let mut name_buffer: Vec<libc::c_char> = vec![0; 71];
                    let mut type_buffer: Vec<libc::c_char> = vec![0; 71];
                    unsafe {
                        fits_get_bcolparms(
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
                    column_descriptions,
                    num_rows: num_rows as usize,
                }
            }
            _ => panic!("Invalid hdu type found"),
        };

        check_status(status).map(|_| hdu_type)
    }

    /// Create a new fits table
    ///
    /// Create a new fits table, with columns as detailed in the [`ColumnDescription`] object.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate tempdir;
    /// # extern crate fitsio;
    /// # use fitsio::columndescription::*;
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let tdir = tempdir::TempDir::new("fitsio-")?;
    /// # let tdir_path = tdir.path();
    /// # let filename = tdir_path.join("test.fits");
    /// # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    /// let first_description = ColumnDescription::new("A")
    ///     .with_type(ColumnDataType::Int)
    ///     .create()?;
    /// let second_description = ColumnDescription::new("B")
    ///     .with_type(ColumnDataType::Long)
    ///     .create()?;
    /// let descriptions = &[first_description, second_description];
    /// let hdu = fptr.create_table("EXTNAME".to_string(), descriptions)?;
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    ///
    /// [`ColumnDescription`]: ../columndescription/struct.ColumnDescription.html
    pub fn create_table<T>(
        &mut self,
        extname: T,
        table_description: &[ConcreteColumnDescription],
    ) -> Result<FitsHdu>
    where
        T: Into<String>,
    {
        fits_check_readwrite!(self);

        let tfields = {
            let stringlist: Vec<_> = table_description
                .iter()
                .map(|desc| desc.name.clone())
                .collect();
            stringutils::StringList::from_slice(stringlist.as_slice())?
        };

        let ttype = {
            let stringlist: Vec<_> = table_description
                .iter()
                .map(|desc| String::from(desc.clone().data_type))
                .collect();
            stringutils::StringList::from_slice(stringlist.as_slice())?
        };

        let c_extname = ffi::CString::new(extname.into())?;

        let hdu_info = HduInfo::TableInfo {
            column_descriptions: table_description.to_vec(),
            num_rows: 0,
        };

        let mut status: libc::c_int = 0;
        unsafe {
            fits_create_tbl(
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

        check_status(status).and_then(|_| self.current_hdu())
    }

    /// Create a new fits image, and return the [`FitsHdu`](struct.FitsHdu.html) object.
    ///
    /// This method takes an [`ImageDescription`] struct which defines the desired layout of the
    /// image HDU.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate tempdir;
    /// # extern crate fitsio;
    /// # use fitsio::fitsfile::ImageDescription;
    /// # use fitsio::types::ImageType;
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let tdir = tempdir::TempDir::new("fitsio-")?;
    /// # let tdir_path = tdir.path();
    /// # let filename = tdir_path.join("test.fits");
    /// # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    /// let image_description = ImageDescription {
    ///     data_type: ImageType::Float,
    ///     dimensions: &[100, 100],
    /// };
    /// let hdu = fptr.create_image("EXTNAME".to_string(), &image_description)?;
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    ///
    /// [`ImageDescription`]: struct.ImageDescription.html
    pub fn create_image<T>(
        &mut self,
        extname: T,
        image_description: &ImageDescription,
    ) -> Result<FitsHdu>
    where
        T: Into<String>,
    {
        fits_check_readwrite!(self);

        let naxis = image_description.dimensions.len();
        let mut status = 0;

        if status != 0 {
            return Err(FitsError {
                status,
                // unwrap guaranteed to succesed as status > 0
                message: status_to_string(status)?.unwrap(),
            }.into());
        }

        let mut dimensions: Vec<_> = image_description.dimensions.clone().to_vec();
        dimensions.reverse();

        unsafe {
            fits_create_img(
                self.fptr as *mut _,
                image_description.data_type.into(),
                naxis as i32,
                dimensions.as_ptr() as *mut libc::c_long,
                &mut status,
            );
        }

        if status != 0 {
            return Err(FitsError {
                status,
                // unwrap guaranteed to succesed as status > 0
                message: status_to_string(status)?.unwrap(),
            }.into());
        }

        // Current HDU should be at the new HDU
        let current_hdu = try!(self.current_hdu());
        current_hdu.write_key(self, "EXTNAME", extname.into())?;

        check_status(status).and_then(|_| self.current_hdu())
    }

    /// Iterate over the HDUs in the file
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate fitsio;
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// #     let mut fptr = fitsio::FitsFile::open("../testdata/full_example.fits")?;
    /// for hdu in fptr.iter() {
    ///     // Do something with hdu
    /// }
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    pub fn iter(&mut self) -> FitsHduIterator {
        FitsHduIterator {
            current: 0,
            max: self.num_hdus().unwrap(),
            fits_file: self,
        }
    }

    /// Pretty-print file to stdout
    ///
    /// Fits files can be pretty-printed with [`pretty_print`], or its more powerful
    /// cousin [`pretty_write`].
    ///
    /// ## Example
    ///
    /// ```rust
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # use fitsio::FitsFile;
    /// # let filename = "../testdata/full_example.fits";
    /// # use std::io;
    /// let mut fptr = FitsFile::open(filename)?;
    /// fptr.pretty_print()?;
    /// // or
    /// fptr.pretty_write(&mut io::stdout())?;
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    ///
    /// [`pretty_print`]: #method.pretty_print
    /// [`pretty_write`]: #method.pretty_write
    pub fn pretty_print(&mut self) -> Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        self.pretty_write(&mut handle)
    }

    /// Pretty-print the fits file structure to any `Write` implementor
    ///
    /// Fits files can be pretty-printed with [`pretty_print`], or its more powerful
    /// cousin [`pretty_write`].
    ///
    /// ## Example
    ///
    /// ```rust
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # use fitsio::FitsFile;
    /// # let filename = "../testdata/full_example.fits";
    /// # use std::io;
    /// let mut fptr = FitsFile::open(filename)?;
    /// fptr.pretty_print()?;
    /// // or
    /// fptr.pretty_write(&mut io::stdout())?;
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    ///
    /// [`pretty_print`]: #method.pretty_print
    /// [`pretty_write`]: #method.pretty_write
    pub fn pretty_write<W>(&mut self, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        writeln!(w, "\n  file: {}", self.filename)?;
        match self.open_mode {
            FileOpenMode::READONLY => writeln!(w, "  mode: READONLY")?,
            FileOpenMode::READWRITE => writeln!(w, "  mode: READWRITE")?,
        };

        /* Header line for HDUs */
        writeln!(w, "  extnum hdutype      hduname    details")?;

        let hdu_names = self.hdu_names().expect("fetching hdu names");

        for (i, hdu) in self.iter().enumerate() {
            let hdu_name = &hdu_names[i];

            match hdu.info {
                HduInfo::ImageInfo { shape, image_type } => {
                    let hdu_type = "IMAGE_HDU";
                    writeln!(
                        w,
                        "  {extnum:<6} {hdu_type:12} {hdu_name:10} dimensions: {dimensions:?}, type: {image_type:?}",
                        extnum = i,
                        hdu_type = hdu_type,
                        hdu_name = hdu_name,
                        dimensions = shape,
                        image_type = image_type,
                    )?;
                }
                HduInfo::TableInfo {
                    column_descriptions,
                    num_rows,
                } => {
                    let hdu_type = "BINARY_TBL";
                    writeln!(
                        w,
                        "  {extnum:<6} {hdu_type:12} {hdu_name:10} num_cols: {num_cols}, num_rows: {num_rows}",
                        extnum = i,
                        hdu_type = hdu_type,
                        hdu_name = hdu_name,
                        num_cols = column_descriptions.len(),
                        num_rows = num_rows,
                    )?;
                }
                HduInfo::AnyInfo => unreachable!(),
            }
        }

        Ok(())
    }

    /// Return a pointer to the underlying C `fitsfile` object representing the current file.
    ///
    /// This is marked as `unsafe` as it is definitely something that is not required by most
    /// users, and hence the unsafe-ness marks it as an advanced feature. I have also not
    /// considered possible concurrency or data race issues as yet.
    ///
    /// Any changes to the underlying fits file will not be updated in existing [`FitsHdu`]
    /// objects, so these must be recreated.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate fitsio;
    /// # #[cfg(not(feature="bindgen"))]
    /// extern crate fitsio_sys;
    /// # #[cfg(feature="bindgen")]
    /// # extern crate fitsio_sys_bindgen as fitsio_sys;
    ///
    /// # use fitsio::FitsFile;
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// let fptr = FitsFile::open(filename)?;
    ///
    /// /* Find out the number of HDUs in the file */
    /// let mut num_hdus = 0;
    /// let mut status = 0;
    ///
    /// unsafe {
    ///     let fitsfile = fptr.as_raw();
    ///
    ///     /* Use the unsafe fitsio-sys low level library to call a function that is possibly not
    ///     implemented in this crate */
    ///     fitsio_sys::ffthdu(fitsfile, &mut num_hdus, &mut status);
    /// }
    /// assert_eq!(num_hdus, 2);
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    ///
    /// [`FitsHdu`]: struct.FitsHdu.html
    pub unsafe fn as_raw(&self) -> *mut fitsfile {
        self.fptr as *mut _
    }
}

impl Drop for FitsFile {
    /// Executes the destructor for this type. [Read
    /// more](https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html#tymethod.drop)
    ///
    /// Dropping a [`FitsFile`] closes the file on disk, flushing existing buffers.
    ///
    /// [`FitsFile`]: struct.FitsFile.html
    fn drop(&mut self) {
        let mut status = 0;
        unsafe {
            fits_close_file(self.fptr as *mut _, &mut status);
        }
        self.fptr = ptr::null_mut();
    }
}

/// New fits file representation
///
/// This is a temporary struct, which describes how the primary HDU of a new file should be
/// created. It uses the builder pattern.
///
/// The [`with_custom_primary`][new-fits-file-with-custom-primary] method allows for creation of a
/// custom primary HDU.
///
/// ## Example
///
/// ```rust
/// # extern crate tempdir;
/// # extern crate fitsio;
/// # use fitsio::FitsFile;
/// # use fitsio::types::ImageType;
/// # use fitsio::fitsfile::ImageDescription;
/// # fn main() {
/// # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
/// # let tdir_path = tdir.path();
/// # let _filename = tdir_path.join("test.fits");
/// # let filename = _filename.to_str().unwrap();
/// use fitsio::FitsFile;
///
/// // let filename = ...;
/// let description = ImageDescription {
///     data_type: ImageType::Double,
///     dimensions: &[52, 103],
/// };
/// let fptr = FitsFile::create(filename)
///     .with_custom_primary(&description)
///     .open()
///     .unwrap();
/// # }
/// ```
///
/// The [`open`][new-fits-file-open] method actually creates a `Result<FitsFile>` from this
/// temporary representation.
///
/// ## Example
///
/// ```rust
/// # extern crate tempdir;
/// # extern crate fitsio;
/// # use fitsio::FitsFile;
/// # fn main() {
/// # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
/// # let tdir_path = tdir.path();
/// # let _filename = tdir_path.join("test.fits");
/// # let filename = _filename.to_str().unwrap();
/// use fitsio::FitsFile;
///
/// // let filename = ...;
/// let fptr = FitsFile::create(filename).open().unwrap();
/// # }
/// ```
/// [new-fits-file]: struct.NewFitsFile.html
/// [new-fits-file-open]: struct.NewFitsFile.html#method.open
/// [new-fits-file-with-custom-primary]: struct.NewFitsFile.html#method.with_custom_primary
pub struct NewFitsFile<'a, T>
where
    T: AsRef<Path>,
{
    path: T,
    image_description: Option<ImageDescription<'a>>,
}

impl<'a, T> NewFitsFile<'a, T>
where
    T: AsRef<Path>,
{
    /// Create a `Result<FitsFile>` from a temporary [`NewFitsFile`][new-fits-file] representation.
    ///
    /// [new-fits-file]: struct.NewFitsFile.html
    pub fn open(self) -> Result<FitsFile> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let path = self.path.as_ref().to_str().expect("converting filename");
        let c_filename = ffi::CString::new(path)?;

        unsafe {
            fits_create_file(
                &mut fptr as *mut *mut fitsfile,
                c_filename.as_ptr(),
                &mut status,
            );
        }

        check_status(status).and_then(|_| {
            let mut f = FitsFile {
                fptr,
                open_mode: FileOpenMode::READWRITE,
                filename: path.to_string(),
            };

            match self.image_description {
                Some(ref description) => {
                    let _ = f.create_image("_PRIMARY".to_string(), description)?;
                }
                None => f.add_empty_primary()?,
            }
            Ok(f)
        })
    }

    /// When creating a new file, add a custom primary HDU description before creating the
    /// [`FitsFile`] object.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate tempdir;
    /// # extern crate fitsio;
    /// # use fitsio::FitsFile;
    /// # use fitsio::types::ImageType;
    /// # use fitsio::fitsfile::ImageDescription;
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let tdir = tempdir::TempDir::new("fitsio-")?;
    /// # let tdir_path = tdir.path();
    /// # let filename = tdir_path.join("test.fits");
    /// use fitsio::FitsFile;
    ///
    /// // let filename = ...;
    /// let description = ImageDescription {
    ///     data_type: ImageType::Double,
    ///     dimensions: &[52, 103],
    /// };
    ///
    /// let fptr = FitsFile::create(filename)
    ///     .with_custom_primary(&description)
    ///     .open()?;
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    ///
    /// [`FitsFile`]: struct.FitsFile.html
    pub fn with_custom_primary(mut self, description: &ImageDescription<'a>) -> Self {
        self.image_description = Some(description.clone());
        self
    }
}

/// Iterator over fits HDUs
pub struct FitsHduIterator<'a> {
    current: usize,
    max: usize,
    fits_file: &'a mut FitsFile,
}

impl<'a> Iterator for FitsHduIterator<'a> {
    type Item = FitsHdu;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.max {
            return None;
        }

        let hdu = self.fits_file.hdu(self.current).unwrap();
        self.current += 1;
        Some(hdu)
    }
}

/// Hdu description type
///
/// Any way of describing a HDU - number or string which either
/// changes the hdu by absolute number, or by name.
pub trait DescribesHdu {
    /// Method by which the current HDU of a file can be changed
    fn change_hdu(&self, fptr: &mut FitsFile) -> Result<()>;
}

impl DescribesHdu for usize {
    fn change_hdu(&self, f: &mut FitsFile) -> Result<()> {
        let mut hdu_type = 0;
        let mut status = 0;
        unsafe {
            fits_movabs_hdu(
                f.fptr as *mut _,
                (*self + 1) as i32,
                &mut hdu_type,
                &mut status,
            );
        }

        check_status(status)
    }
}

impl<'a> DescribesHdu for &'a str {
    fn change_hdu(&self, f: &mut FitsFile) -> Result<()> {
        let mut status = 0;
        let c_hdu_name = ffi::CString::new(*self)?;

        unsafe {
            fits_movnam_hdu(
                f.fptr as *mut _,
                HduInfo::AnyInfo.into(),
                c_hdu_name.into_raw(),
                0,
                &mut status,
            );
        }

        check_status(status)
    }
}

/// Way of describing a column location
pub trait DescribesColumnLocation {
    /// Method by which the column number can be computed
    fn get_column_no(&self, hdu: &FitsHdu, fptr: &mut FitsFile) -> Result<i32>;
}

impl DescribesColumnLocation for usize {
    fn get_column_no(&self, _: &FitsHdu, _: &mut FitsFile) -> Result<i32> {
        Ok(*self as i32)
    }
}

impl<'a> DescribesColumnLocation for &'a str {
    fn get_column_no(&self, hdu: &FitsHdu, fits_file: &mut FitsFile) -> Result<i32> {
        match hdu.get_column_no(fits_file, *self) {
            Ok(value) => Ok(value as _),
            Err(e) => Err(e),
        }
    }
}

/// Trait for reading a fits column
pub trait ReadsCol {
    #[doc(hidden)]
    fn read_col_range<T: Into<String>>(
        fits_file: &FitsFile,
        name: T,
        range: &Range<usize>,
    ) -> Result<Vec<Self>>
    where
        Self: Sized;

    #[doc(hidden)]
    fn read_cell_value<T>(fits_file: &FitsFile, name: T, idx: usize) -> Result<Self>
    where
        T: Into<String>,
        Self: Sized;

    #[doc(hidden)]
    fn read_col<T: Into<String>>(fits_file: &FitsFile, name: T) -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { num_rows, .. }) => {
                let range = 0..num_rows;
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
            fn read_col_range<T: Into<String>>(fits_file: &FitsFile, name: T, range: &Range<usize>)
                -> Result<Vec<Self>> {
                    match fits_file.fetch_hdu_info() {
                        Ok(HduInfo::TableInfo { column_descriptions, .. }) => {
                            let num_output_rows = range.end - range.start;
                            let mut out = vec![$nullval; num_output_rows];
                            let test_name = name.into();
                            let column_number = column_descriptions
                                .iter()
                                .position(|ref desc| { desc.name == test_name })
                                .ok_or(Error::Message(
                                        format!("Cannot find column {:?}", test_name)))?;
                            let mut status = 0;
                            unsafe {
                                $func(fits_file.fptr as *mut _,
                                           (column_number + 1) as i32,
                                           (range.start + 1) as i64,
                                           1,
                                           num_output_rows as _,
                                           $nullval,
                                           out.as_mut_ptr(),
                                           ptr::null_mut(),
                                           &mut status);

                            }

                            match status {
                                0 => Ok(out),
                                307 => Err(IndexError {
                                    message: "given indices out of range".to_string(),
                                    given: range.clone(),
                                }.into()),
                                e => Err(FitsError {
                                    status: e,
                                    message: status_to_string(e).unwrap().unwrap(),
                                }.into()),
                            }
                        },
                        Err(e) => Err(e),
                        _ => panic!("Unknown error occurred"),
                    }
                }

            #[doc(hidden)]
            fn read_cell_value<T>(fits_file: &FitsFile, name: T, idx: usize) -> Result<Self>
                where T: Into<String>,
                      Self: Sized {
                          match fits_file.fetch_hdu_info() {
                              Ok(HduInfo::TableInfo { column_descriptions, .. }) => {
                                  let mut out = $nullval;
                                  let test_name = name.into();
                                  let column_number = column_descriptions
                                      .iter()
                                      .position(|ref desc| { desc.name == test_name })
                                      .ok_or(Error::Message(
                                              format!("Cannot find column {:?}", test_name)))?;
                                  let mut status = 0;

                                  unsafe {
                                      $func(fits_file.fptr as *mut _,
                                                 (column_number + 1) as i32,
                                                 (idx + 1) as i64,
                                                 1,
                                                 1,
                                                 $nullval,
                                                 &mut out,
                                                 ptr::null_mut(),
                                                 &mut status);
                                  }

                                  check_status(status).map(|_| out)
                              }
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
        fits_get_col_display_width(
            fits_file.fptr as *mut _,
            (column_number + 1) as _,
            &mut width,
            &mut status,
        );
    }
    check_status(status).map(|_| width as usize)
}

impl ReadsCol for String {
    fn read_col_range<T: Into<String>>(
        fits_file: &FitsFile,
        name: T,
        range: &Range<usize>,
    ) -> Result<Vec<Self>> {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo {
                column_descriptions,
                ..
            }) => {
                let num_output_rows = range.end - range.start;
                let test_name = name.into();
                let column_number = column_descriptions
                    .iter()
                    .position(|desc| desc.name == test_name)
                    .ok_or_else(|| Error::Message(format!("Cannot find column {:?}", test_name)))?;

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
                    fits_read_col_str(
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

                check_status(status)?;

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

    #[doc(hidden)]
    fn read_cell_value<T>(fits_file: &FitsFile, name: T, idx: usize) -> Result<Self>
    where
        T: Into<String>,
        Self: Sized,
    {
        // XXX Ineffient but works
        Self::read_col_range(fits_file, name, &(idx..idx + 1)).map(|v| v[0].clone())
    }
}

/// Trait representing the ability to write column data
pub trait WritesCol {
    #[doc(hidden)]
    fn write_col_range<T: Into<String>>(
        fits_file: &mut FitsFile,
        hdu: &FitsHdu,
        col_name: T,
        col_data: &[Self],
        rows: &Range<usize>,
    ) -> Result<FitsHdu>
    where
        Self: Sized;

    #[doc(hidden)]
    fn write_col<T: Into<String>>(
        fits_file: &mut FitsFile,
        hdu: &FitsHdu,
        col_name: T,
        col_data: &[Self],
    ) -> Result<FitsHdu>
    where
        Self: Sized,
    {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { .. }) => {
                let row_range = 0..col_data.len();
                Self::write_col_range(fits_file, hdu, col_name, col_data, &row_range)
            }
            Ok(HduInfo::ImageInfo { .. }) => Err("Cannot write column data to FITS image".into()),
            Ok(HduInfo::AnyInfo { .. }) => {
                Err("Cannot determine HDU type, so cannot write column data".into())
            }
            Err(e) => Err(e),
        }
    }
}

macro_rules! writes_col_impl {
    ($t: ty, $data_type: expr) => (
        impl WritesCol for $t {
            fn write_col_range<T: Into<String>>(fits_file: &mut FitsFile,
                hdu: &FitsHdu,
                col_name: T,
                col_data: &[Self],
                rows: &Range<usize>)
            -> Result<FitsHdu> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::TableInfo { .. }) => {
                        let colno = hdu.get_column_no(fits_file, col_name.into())?;
                        // TODO: check that the column exists in the file
                        let mut status = 0;
                        let n_elements = rows.end - rows.start;
                        unsafe {
                            fits_write_col(
                                fits_file.fptr as *mut _,
                                $data_type.into(),
                                (colno + 1) as _,
                                (rows.start + 1) as _,
                                1,
                                n_elements as _,
                                col_data.as_ptr() as *mut _,
                                &mut status
                            );
                        }
                        check_status(status).and_then(|_| fits_file.current_hdu())
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
        fits_file: &mut FitsFile,
        hdu: &FitsHdu,
        col_name: T,
        col_data: &[Self],
        rows: &Range<usize>,
    ) -> Result<FitsHdu> {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { .. }) => {
                let colno = hdu.get_column_no(fits_file, col_name.into())?;
                let mut status = 0;

                let start = rows.start;
                let end = rows.end;
                let n_elements = end - start;
                let mut ptr_array = Vec::with_capacity(n_elements);

                let rows = rows.clone();
                for i in rows {
                    let s = ffi::CString::new(col_data[i].clone())?;
                    ptr_array.push(s.into_raw());
                }

                unsafe {
                    fits_write_col_str(
                        fits_file.fptr as *mut _,
                        (colno + 1) as _,
                        (start + 1) as _,
                        1,
                        n_elements as _,
                        ptr_array.as_mut_ptr() as _,
                        &mut status,
                    );
                }
                check_status(status).and_then(|_| fits_file.current_hdu())
            }
            Ok(HduInfo::ImageInfo { .. }) => Err("Cannot write column data to FITS image".into()),
            Ok(HduInfo::AnyInfo { .. }) => {
                Err("Cannot determine HDU type, so cannot write column data".into())
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
    #[doc(hidden)]
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
                    $func(f.fptr as *mut _,
                           c_name.into_raw(),
                           &mut value,
                           ptr::null_mut(),
                           &mut status);
                }

                check_status(status).map(|_| value)
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
            fits_read_key_str(
                f.fptr as *mut _,
                c_name.into_raw(),
                value.as_mut_ptr(),
                ptr::null_mut(),
                &mut status,
            );
        }

        check_status(status).and_then(|_| {
            let value: Vec<u8> = value.iter().map(|&x| x as u8).filter(|&x| x != 0).collect();
            Ok(String::from_utf8(value)?)
        })
    }
}

/// Writing a fits keyword
pub trait WritesKey {
    #[doc(hidden)]
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()>;
}

macro_rules! writes_key_impl_flt {
    ($t:ty, $func:ident) => (
        impl WritesKey for $t {
            fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
                let c_name = ffi::CString::new(name)?;
                let mut status = 0;

                unsafe {
                    $func(f.fptr as *mut _,
                                c_name.into_raw(),
                                value,
                                9,
                                ptr::null_mut(),
                                &mut status);
                }
                check_status(status)
            }
        }
    )
}

impl WritesKey for i64 {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
        let c_name = ffi::CString::new(name)?;
        let mut status = 0;

        unsafe {
            fits_write_key_lng(
                f.fptr as *mut _,
                c_name.into_raw(),
                value,
                ptr::null_mut(),
                &mut status,
            );
        }
        check_status(status)
    }
}

writes_key_impl_flt!(f32, ffpkye);
writes_key_impl_flt!(f64, ffpkyd);

impl WritesKey for String {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
        WritesKey::write_key(f, name, value.as_str())
    }
}

impl<'a> WritesKey for &'a str {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
        let c_name = ffi::CString::new(name)?;
        let mut status = 0;

        unsafe {
            fits_write_key_str(
                f.fptr as *mut _,
                c_name.into_raw(),
                ffi::CString::new(value)?.into_raw(),
                ptr::null_mut(),
                &mut status,
            );
        }

        check_status(status)
    }
}

/// Reading fits images
pub trait ReadWriteImage: Sized {
    #[doc(hidden)]
    fn read_section(fits_file: &mut FitsFile, range: Range<usize>) -> Result<Vec<Self>>;

    #[doc(hidden)]
    fn read_rows(fits_file: &mut FitsFile, start_row: usize, num_rows: usize) -> Result<Vec<Self>>;

    #[doc(hidden)]
    fn read_row(fits_file: &mut FitsFile, row: usize) -> Result<Vec<Self>>;

    #[doc(hidden)]
    fn read_region(fits_file: &mut FitsFile, ranges: &[&Range<usize>]) -> Result<Vec<Self>>;

    #[doc(hidden)]
    fn read_image(fits_file: &mut FitsFile) -> Result<Vec<Self>> {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::ImageInfo { shape, .. }) => {
                let mut npixels = 1;
                for dimension in &shape {
                    npixels *= *dimension;
                }
                Self::read_section(fits_file, 0..npixels)
            }
            Ok(HduInfo::TableInfo { .. }) => Err("cannot read image data from a table hdu".into()),
            Ok(HduInfo::AnyInfo) => unreachable!(),
            Err(e) => Err(e),
        }
    }

    #[doc(hidden)]
    fn write_section(fits_file: &mut FitsFile, range: Range<usize>, data: &[Self]) -> Result<()>;

    #[doc(hidden)]
    fn write_region(
        fits_file: &mut FitsFile,
        ranges: &[&Range<usize>],
        data: &[Self],
    ) -> Result<()>;

    #[doc(hidden)]
    fn write_image(fits_file: &mut FitsFile, data: &[Self]) -> Result<()> {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::ImageInfo { shape, .. }) => {
                let image_npixels = shape.iter().product();
                if data.len() > image_npixels {
                    return Err(format!(
                        "cannot write more data ({} elements) to the current image (shape: {:?})",
                        data.len(),
                        shape
                    ).as_str()
                        .into());
                }

                Self::write_section(fits_file, 0..data.len(), data)
            }
            Ok(HduInfo::TableInfo { .. }) => Err("cannot write image data to a table hdu".into()),
            Ok(HduInfo::AnyInfo) => unreachable!(),
            Err(e) => Err(e),
        }
    }
}

macro_rules! read_write_image_impl {
    ($t: ty, $default_value: expr, $data_type: expr) => (
        impl ReadWriteImage for $t {
            fn read_section(
                fits_file: &mut FitsFile,
                range: Range<usize>) -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::ImageInfo { shape: _shape, .. }) => {
                        let nelements = range.end - range.start;
                        let mut out = vec![$default_value; nelements];
                        let mut status = 0;

                        unsafe {
                            fits_read_img(fits_file.fptr as *mut _,
                                       $data_type.into(),
                                       (range.start + 1) as i64,
                                       nelements as i64,
                                       ptr::null_mut(),
                                       out.as_mut_ptr() as *mut _,
                                       ptr::null_mut(),
                                       &mut status);
                        }

                        check_status(status).map(|_| out)
                    },
                    Ok(HduInfo::TableInfo { .. }) =>
                        Err("cannot read image data from a table hdu".into()),
                    Ok(HduInfo::AnyInfo) => unreachable!(),
                    Err(e) => Err(e),
                }
            }

            fn read_rows(fits_file: &mut FitsFile, start_row: usize, num_rows: usize)
                -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::ImageInfo { shape, .. }) => {
                        if shape.len() != 2 {
                            unimplemented!();
                        }

                        let num_cols = shape[1];
                        let start = start_row * num_cols;
                        let end = (start_row + num_rows) * num_cols;

                        Self::read_section(fits_file, start..end)
                    },
                    Ok(HduInfo::TableInfo { .. }) =>
                        Err("cannot read image data from a table hdu".into()),
                    Ok(HduInfo::AnyInfo) => unreachable!(),
                    Err(e) => Err(e),
                }
            }

            fn read_row(fits_file: &mut FitsFile, row: usize) -> Result<Vec<Self>> {
                Self::read_rows(fits_file, row, 1)
            }

            fn read_region(fits_file: &mut FitsFile, ranges: &[&Range<usize>])
                -> Result<Vec<Self>> {
                    match fits_file.fetch_hdu_info() {
                        Ok(HduInfo::ImageInfo { .. }) => {
                            let n_ranges = ranges.len();

                            let mut fpixel = Vec::with_capacity(n_ranges);
                            let mut lpixel = Vec::with_capacity(n_ranges);

                            let mut nelements = 1;
                            for range in ranges {
                                let start = range.start + 1;
                                let end = range.end + 1;
                                fpixel.push(start as _);
                                lpixel.push(end as _);

                                nelements *= end - start;
                            }

                            let mut inc: Vec<_> = (0..n_ranges).map(|_| 1).collect();
                            let mut out = vec![$default_value; nelements as usize];
                            let mut status = 0;

                            unsafe {
                                fits_read_subset(
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

                            check_status(status).map(|_| out)
                        }
                        Ok(HduInfo::TableInfo { .. }) =>
                            Err("cannot read image data from a table hdu".into()),
                        Ok(HduInfo::AnyInfo) => unreachable!(),
                        Err(e) => Err(e),
                    }
                }

            fn write_section(
                fits_file: &mut FitsFile,
                range: Range<usize>,
                data: &[Self])
                -> Result<()> {
                    match fits_file.fetch_hdu_info() {
                        Ok(HduInfo::ImageInfo { .. }) => {
                            let nelements = range.end - range.start;
                            assert!(data.len() >= nelements);
                            let mut status = 0;
                            unsafe {
                                fits_write_img(fits_file.fptr as *mut _,
                                           $data_type.into(),
                                           (range.start + 1) as i64,
                                           nelements as i64,
                                           data.as_ptr() as *mut _,
                                           &mut status);
                            }

                            check_status(status)
                        },
                        Ok(HduInfo::TableInfo { .. }) =>
                            Err("cannot write image data to a table hdu".into()),
                        Ok(HduInfo::AnyInfo) => unreachable!(),
                        Err(e) => Err(e),
                    }
                }

            fn write_region(
                fits_file: &mut FitsFile,
                ranges: &[&Range<usize>],
                data: &[Self])
                -> Result<()> {
                    match fits_file.fetch_hdu_info() {
                        Ok(HduInfo::ImageInfo { .. }) => {
                            let n_ranges = ranges.len();

                            let mut fpixel = Vec::with_capacity(n_ranges);
                            let mut lpixel = Vec::with_capacity(n_ranges);

                            for range in ranges {
                                let start = range.start + 1;
                                let end = range.end + 1;
                                fpixel.push(start as _);
                                lpixel.push(end as _);
                            }

                            let mut status = 0;

                            unsafe {
                                fits_write_subset(
                                    fits_file.fptr as *mut _,
                                    $data_type.into(),
                                    fpixel.as_mut_ptr(),
                                    lpixel.as_mut_ptr(),
                                    data.as_ptr() as *mut _,
                                    &mut status);
                            }

                            check_status(status)
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

read_write_image_impl!(i8, i8::default(), DataType::TSHORT);
read_write_image_impl!(i32, i32::default(), DataType::TINT);
#[cfg(target_pointer_width = "64")]
read_write_image_impl!(i64, i64::default(), DataType::TLONG);
#[cfg(target_pointer_width = "32")]
read_write_image_impl!(i64, i64::default() DataType::TLONGLONG);
read_write_image_impl!(u8, u8::default(), DataType::TUSHORT);
read_write_image_impl!(u32, u32::default(), DataType::TUINT);
#[cfg(target_pointer_width = "64")]
read_write_image_impl!(u64, u64::default(), DataType::TULONG);
read_write_image_impl!(f32, f32::default(), DataType::TFLOAT);
read_write_image_impl!(f64, f64::default(), DataType::TDOUBLE);

/// Columns of different types
#[allow(missing_docs)]
pub enum Column {
    Int32 { name: String, data: Vec<i32> },
    Int64 { name: String, data: Vec<i64> },
    Float { name: String, data: Vec<f32> },
    Double { name: String, data: Vec<f64> },
    String { name: String, data: Vec<String> },
}

/// Iterator type for columns
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
            }) => ColumnIterator {
                current: 0,
                column_descriptions,
                fits_file,
            },
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
                ColumnDataType::Int => i32::read_col(self.fits_file, current_name)
                    .map(|data| Column::Int32 {
                        name: current_name.to_string(),
                        data,
                    })
                    .ok(),
                ColumnDataType::Long => i64::read_col(self.fits_file, current_name)
                    .map(|data| Column::Int64 {
                        name: current_name.to_string(),
                        data,
                    })
                    .ok(),
                ColumnDataType::Float => f32::read_col(self.fits_file, current_name)
                    .map(|data| Column::Float {
                        name: current_name.to_string(),
                        data,
                    })
                    .ok(),
                ColumnDataType::Double => f64::read_col(self.fits_file, current_name)
                    .map(|data| Column::Double {
                        name: current_name.to_string(),
                        data,
                    })
                    .ok(),
                ColumnDataType::String => String::read_col(self.fits_file, current_name)
                    .map(|data| Column::String {
                        name: current_name.to_string(),
                        data,
                    })
                    .ok(),
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
#[derive(Debug, PartialEq)]
pub struct FitsHdu {
    /// Information about the current HDU
    pub info: HduInfo,
    hdu_num: usize,
}

impl FitsHdu {
    fn new<T: DescribesHdu>(fits_file: &mut FitsFile, hdu_description: T) -> Result<Self> {
        fits_file.change_hdu(hdu_description)?;
        match fits_file.fetch_hdu_info() {
            Ok(hdu_info) => Ok(FitsHdu {
                info: hdu_info,
                hdu_num: fits_file.hdu_number(),
            }),
            Err(e) => Err(e),
        }
    }

    /// Read the HDU name
    pub fn name(&self, fits_file: &mut FitsFile) -> Result<String> {
        let extname = self.read_key(fits_file, "EXTNAME")
            .unwrap_or_else(|_| "".to_string());
        Ok(extname)
    }

    /// Read header key
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate fitsio;
    /// #
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// # let mut fptr = fitsio::FitsFile::open(filename)?;
    /// # let hdu = fptr.primary_hdu()?;
    /// # {
    /// let int_value: i64 = hdu.read_key(&mut fptr, "INTTEST")?;
    /// # }
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    pub fn read_key<T: ReadsKey>(&self, fits_file: &mut FitsFile, name: &str) -> Result<T> {
        fits_file.make_current(self)?;
        T::read_key(fits_file, name)
    }

    /// Write a fits key to the current header
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate tempdir;
    /// # extern crate fitsio;
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let tdir = tempdir::TempDir::new("fitsio-")?;
    /// # let tdir_path = tdir.path();
    /// # let filename = tdir_path.join("test.fits");
    /// # {
    /// # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    /// fptr.primary_hdu()?.write_key(&mut fptr, "foo", 1i64)?;
    /// assert_eq!(fptr.hdu(0)?.read_key::<i64>(&mut fptr, "foo")?, 1i64);
    /// # Ok(())
    /// # }
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    pub fn write_key<T: WritesKey>(
        &self,
        fits_file: &mut FitsFile,
        name: &str,
        value: T,
    ) -> Result<()> {
        fits_file.make_current(self)?;
        fits_check_readwrite!(fits_file);
        T::write_key(fits_file, name, value)
    }

    /// Read pixels from an image between a start index and end index
    ///
    /// The range is exclusive of the upper value
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate fitsio;
    /// #
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// # let mut fptr = fitsio::FitsFile::open(filename)?;
    /// # let hdu = fptr.hdu(0)?;
    /// // Read the first 100 pixels
    /// let first_row: Vec<i32> = hdu.read_section(&mut fptr, 0, 100)?;
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    ///
    pub fn read_section<T: ReadWriteImage>(
        &self,
        fits_file: &mut FitsFile,
        start: usize,
        end: usize,
    ) -> Result<Vec<T>> {
        fits_file.make_current(self)?;
        T::read_section(fits_file, start..end)
    }

    /// Read multiple rows from a fits image
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate fitsio;
    /// #
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// # let mut fptr = fitsio::FitsFile::open(filename)?;
    /// # let hdu = fptr.hdu(0)?;
    /// let start_row = 0;
    /// let num_rows = 10;
    /// let first_few_rows: Vec<f32> = hdu.read_rows(&mut fptr, start_row, num_rows)?;
    ///
    /// // 10 rows of 100 columns
    /// assert_eq!(first_few_rows.len(), 1000);
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    pub fn read_rows<T: ReadWriteImage>(
        &self,
        fits_file: &mut FitsFile,
        start_row: usize,
        num_rows: usize,
    ) -> Result<Vec<T>> {
        fits_file.make_current(self)?;
        T::read_rows(fits_file, start_row, num_rows)
    }

    /// Read a single row from a fits image
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate fitsio;
    /// #
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// # let mut fptr = fitsio::FitsFile::open(filename)?;
    /// # let hdu = fptr.hdu(0)?;
    /// let chosen_row = 5;
    /// let row: Vec<f32> = hdu.read_row(&mut fptr, chosen_row)?;
    ///
    /// // Should have 100 pixel values
    /// assert_eq!(row.len(), 100);
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    pub fn read_row<T: ReadWriteImage>(
        &self,
        fits_file: &mut FitsFile,
        row: usize,
    ) -> Result<Vec<T>> {
        fits_file.make_current(self)?;
        T::read_row(fits_file, row)
    }

    /// Read a square region from the chip.
    ///
    /// Lower left indicates the starting point of the square, and the upper
    /// right defines the pixel _beyond_ the end. The range of pixels included
    /// is inclusive of the lower end, and *exclusive* of the upper end.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate fitsio;
    /// #
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// # let mut fptr = fitsio::FitsFile::open(filename)?;
    /// # let hdu = fptr.hdu(0)?;
    /// // Read a square section of the image
    /// let xcoord = 0..10;
    /// let ycoord = 0..10;
    /// let chunk: Vec<i32> = hdu.read_region(&mut fptr, &[&ycoord, &xcoord])?;
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    pub fn read_region<T: ReadWriteImage>(
        &self,
        fits_file: &mut FitsFile,
        ranges: &[&Range<usize>],
    ) -> Result<Vec<T>> {
        fits_file.make_current(self)?;
        T::read_region(fits_file, ranges)
    }

    /// Read a whole image into a new `Vec`
    ///
    /// This reads an entire image into a one-dimensional vector
    ///
    /// ## Example
    ///
    /// ```rust
    /// # extern crate fitsio;
    /// #
    /// # fn try_main() -> Result<(), Box<std::error::Error>> {
    /// # let filename = "../testdata/full_example.fits";
    /// # let mut fptr = fitsio::FitsFile::open(filename)?;
    /// # let hdu = fptr.hdu(0)?;
    /// let image_data: Vec<f32> = hdu.read_image(&mut fptr)?;
    ///
    /// // 100 rows of 100 columns
    /// assert_eq!(image_data.len(), 10_000);
    /// # Ok(())
    /// # }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    pub fn read_image<T: ReadWriteImage>(&self, fits_file: &mut FitsFile) -> Result<Vec<T>> {
        fits_file.make_current(self)?;
        T::read_image(fits_file)
    }

    /// Write raw pixel values to a FITS image
    ///
    /// If the length of the dataset exceeds the number of columns,
    /// the data wraps around to the next row.
    ///
    /// The range is exclusive of the upper value.
    pub fn write_section<T: ReadWriteImage>(
        &self,
        fits_file: &mut FitsFile,
        start: usize,
        end: usize,
        data: &[T],
    ) -> Result<()> {
        fits_file.make_current(self)?;
        fits_check_readwrite!(fits_file);
        T::write_section(fits_file, start..end, data)
    }

    /// Write a rectangular region to the fits image
    ///
    /// The ranges must have length of 2, and they represent the limits of each axis. The limits
    /// are inclusive of the lower bounds, and *exclusive* of the and upper bounds.
    ///
    /// For example, writing with ranges 0..10 and 0..10 wries an 10x10 sized image.
    pub fn write_region<T: ReadWriteImage>(
        &self,
        fits_file: &mut FitsFile,
        ranges: &[&Range<usize>],
        data: &[T],
    ) -> Result<()> {
        fits_file.make_current(self)?;
        fits_check_readwrite!(fits_file);
        T::write_region(fits_file, ranges, data)
    }

    /// Write an entire image to the HDU passed in
    ///
    /// Firstly a check is performed, making sure that the amount of data will fit in the image.
    /// After this, all of the data is written to the image.
    pub fn write_image<T: ReadWriteImage>(
        &self,
        fits_file: &mut FitsFile,
        data: &[T],
    ) -> Result<()> {
        fits_file.make_current(self)?;
        fits_check_readwrite!(fits_file);
        T::write_image(fits_file, data)
    }

    /// Resize a HDU image
    ///
    /// The `new_size` parameter defines the new size of the image. Unlike cfitsio, the order
    /// of the dimensions of `new_size` follows the C convention, i.e. [row-major
    /// order](https://en.wikipedia.org/wiki/Row-_and_column-major_order).
    pub fn resize(self, fits_file: &mut FitsFile, new_size: &[usize]) -> Result<FitsHdu> {
        fits_file.make_current(&self)?;
        fits_check_readwrite!(fits_file);

        let mut new_size = new_size.clone().to_vec();
        new_size.reverse();

        match self.info {
            HduInfo::ImageInfo { image_type, .. } => {
                let mut status = 0;
                unsafe {
                    fits_resize_img(
                        fits_file.fptr as *mut _,
                        image_type.into(),
                        new_size.len() as _,
                        new_size.as_ptr() as *mut _,
                        &mut status,
                    );
                }
                check_status(status).and_then(|_| fits_file.current_hdu())
            }
            HduInfo::TableInfo { .. } => Err("cannot resize binary table".into()),
            HduInfo::AnyInfo => unreachable!(),
        }
    }

    /// Copy an HDU to another open fits file
    pub fn copy_to(
        &self,
        src_fits_file: &mut FitsFile,
        dest_fits_file: &mut FitsFile,
    ) -> Result<()> {
        let mut status = 0;
        unsafe {
            fits_copy_hdu(
                src_fits_file.fptr as *mut _,
                dest_fits_file.fptr as *mut _,
                0,
                &mut status,
            );
        }

        check_status(status).map(|_| ())
    }

    /// Insert a column into a fits table
    ///
    /// The column location is 0-indexed. It is inserted _at_ that position, and the following
    /// columns are shifted back.
    pub fn insert_column(
        self,
        fits_file: &mut FitsFile,
        position: usize,
        description: &ConcreteColumnDescription,
    ) -> Result<FitsHdu> {
        fits_file.make_current(&self)?;
        fits_check_readwrite!(fits_file);

        let mut status = 0;

        let c_name = ffi::CString::new(description.name.clone())?;
        let c_type = ffi::CString::new(String::from(description.data_type.clone()))?;

        unsafe {
            fits_insert_col(
                fits_file.fptr as *mut _,
                (position + 1) as _,
                c_name.into_raw(),
                c_type.into_raw(),
                &mut status,
            );
        }

        check_status(status).and_then(|_| fits_file.current_hdu())
    }

    /// Add a new column to the end of the table
    pub fn append_column(
        self,
        fits_file: &mut FitsFile,
        description: &ConcreteColumnDescription,
    ) -> Result<FitsHdu> {
        fits_file.make_current(&self)?;
        fits_check_readwrite!(fits_file);

        /* We have to split up the fetching of the number of columns from the inserting of the
         * new column, as otherwise we're trying move out of self */
        let result = match self.info {
            HduInfo::TableInfo {
                ref column_descriptions,
                ..
            } => Ok(column_descriptions.len()),
            HduInfo::ImageInfo { .. } => Err("Cannot add columns to FITS image".into()),
            HduInfo::AnyInfo { .. } => {
                Err("Cannot determine HDU type, so cannot add columns".into())
            }
        };

        match result {
            Ok(colno) => self.insert_column(fits_file, colno, description),
            Err(e) => Err(e),
        }
    }

    /// Remove a column from the fits file
    ///
    /// The column can be identified by id or name.
    pub fn delete_column<T: DescribesColumnLocation>(
        self,
        fits_file: &mut FitsFile,
        col_identifier: T,
    ) -> Result<FitsHdu> {
        fits_file.make_current(&self)?;
        fits_check_readwrite!(fits_file);

        let colno = T::get_column_no(&col_identifier, &self, fits_file)?;
        let mut status = 0;

        unsafe {
            fits_delete_col(fits_file.fptr as *mut _, (colno + 1) as _, &mut status);
        }

        check_status(status).and_then(|_| fits_file.current_hdu())
    }

    /// Return the index for a given column.
    ///
    /// Internal method, not exposed.
    fn get_column_no<T: Into<String>>(
        &self,
        fits_file: &mut FitsFile,
        col_name: T,
    ) -> Result<usize> {
        fits_file.make_current(self)?;

        let mut status = 0;
        let mut colno = 0;

        let c_col_name = {
            let col_name = col_name.into();
            ffi::CString::new(col_name.as_str())?
        };

        unsafe {
            fits_get_colnum(
                fits_file.fptr as *mut _,
                CaseSensitivity::CASEINSEN as _,
                c_col_name.as_ptr() as *mut _,
                &mut colno,
                &mut status,
            );
        }
        check_status(status).map(|_| (colno - 1) as usize)
    }

    /// Read a subset of a fits column
    ///
    /// The range is exclusive of the upper value
    pub fn read_col<T: ReadsCol>(&self, fits_file: &mut FitsFile, name: &str) -> Result<Vec<T>> {
        fits_file.make_current(self)?;
        T::read_col(fits_file, name)
    }

    /// Read a subset of a fits column
    ///
    /// The range is exclusive of the upper value
    pub fn read_col_range<T: ReadsCol>(
        &self,
        fits_file: &mut FitsFile,
        name: &str,
        range: &Range<usize>,
    ) -> Result<Vec<T>> {
        fits_file.make_current(self)?;
        T::read_col_range(fits_file, name, range)
    }

    /// Write data to part of a column
    ///
    /// The range is exclusive of the upper value
    pub fn write_col_range<T: WritesCol, N: Into<String>>(
        &self,
        fits_file: &mut FitsFile,
        name: N,
        col_data: &[T],
        rows: &Range<usize>,
    ) -> Result<FitsHdu> {
        fits_file.make_current(self)?;
        fits_check_readwrite!(fits_file);
        T::write_col_range(fits_file, self, name, col_data, rows)
    }

    /// Write data to an entire column
    ///
    /// This default implementation does not check the length of the column first, but if the
    /// length of the data array is longer than the length of the table, the table will be extended
    /// with extra rows. This is as per the fitsio definition.
    pub fn write_col<T: WritesCol, N: Into<String>>(
        &self,
        fits_file: &mut FitsFile,
        name: N,
        col_data: &[T],
    ) -> Result<FitsHdu> {
        fits_file.make_current(self)?;
        fits_check_readwrite!(fits_file);
        T::write_col(fits_file, self, name, col_data)
    }

    /// Iterate over the columns in a fits file
    pub fn columns<'a>(&self, fits_file: &'a mut FitsFile) -> ColumnIterator<'a> {
        fits_file
            .make_current(self)
            .expect("Cannot make hdu current");
        ColumnIterator::new(fits_file)
    }

    /// Delete the current HDU from the fits file.
    ///
    /// Note this method takes `self` by value, and as such the hdu cannot be used after this
    /// method is called.
    pub fn delete(self, fits_file: &mut FitsFile) -> Result<()> {
        fits_file.make_current(&self)?;

        let mut status = 0;
        let mut curhdu = 0;
        unsafe {
            fits_delete_hdu(fits_file.fptr as *mut _, &mut curhdu, &mut status);
        }
        check_status(status).map(|_| ())
    }

    /// Read a single value from a fits table
    ///
    /// This will be inefficient if lots of individual values are wanted.
    pub fn read_cell_value<T>(&self, fits_file: &mut FitsFile, name: &str, idx: usize) -> Result<T>
    where
        T: ReadsCol,
    {
        fits_file.make_current(self)?;
        T::read_cell_value(fits_file, name, idx)
    }

    /// Extract a single row from the file
    ///
    /// This method uses returns a [`FitsRow`](trait.FitsRow.html), which is provided by the user,
    /// using a `derive` implementation from the [`fitsio-derive`](https://docs.rs/fitsio-derive)
    /// crate,
    ///
    /// ## Example
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate fitsio_derive;
    /// extern crate fitsio;
    /// use fitsio::fitsfile::FitsRow;
    ///
    /// #[derive(Default, FitsRow)]
    /// struct Row {
    ///     #[fitsio(colname = "intcol")]
    ///     intfoo: i32,
    ///     #[fitsio(colname = "strcol")]
    ///     foobar: String,
    /// }
    /// #
    /// # fn main() {
    /// # let filename = "../testdata/full_example.fits[TESTEXT]";
    /// # let mut f = fitsio::FitsFile::open(filename).unwrap();
    /// # let hdu = f.hdu("TESTEXT").unwrap();
    ///
    /// // Pick the 4th row
    /// let row: Row = hdu.row(&mut f, 4).unwrap();
    /// assert_eq!(row.intfoo, 16);
    /// assert_eq!(row.foobar, "value4");
    /// # }
    /// ```
    pub fn row<F>(&self, fits_file: &mut FitsFile, idx: usize) -> Result<F>
    where
        F: FitsRow,
    {
        fits_file.make_current(self)?;
        F::from_table(self, fits_file, idx)
    }
}

/// Trait derivable with custom derive
pub trait FitsRow: ::std::default::Default {
    #[doc(hidden)]
    fn from_table(tbl: &FitsHdu, fits_file: &mut FitsFile, idx: usize) -> Result<Self>
    where
        Self: Sized;
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
    use errors::{Error, IndexError, Result};
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
    fn test_opening_an_existing_file() {
        match FitsFile::open("../testdata/full_example.fits") {
            Ok(_) => {}
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_creating_a_new_file() {
        with_temp_file(|filename| {
            FitsFile::create(filename)
                .open()
                .map(|mut f| {
                    assert!(Path::new(filename).exists());

                    // Ensure the empty primary has been written
                    let hdu = f.hdu(0).unwrap();
                    let naxis: i64 = hdu.read_key(&mut f, "NAXIS").unwrap();
                    assert_eq!(naxis, 0);
                })
                .unwrap();
        });
    }

    #[test]
    fn test_create_custom_primary_hdu() {
        with_temp_file(|filename| {
            {
                let description = ImageDescription {
                    data_type: ImageType::Double,
                    dimensions: &[100, 103],
                };
                FitsFile::create(filename)
                    .with_custom_primary(&description)
                    .open()
                    .unwrap();
            }
            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu(0).unwrap();
            match hdu.info {
                HduInfo::ImageInfo { shape, image_type } => {
                    assert_eq!(shape, vec![100, 103]);
                    assert_eq!(image_type, ImageType::Double);
                }
                _ => panic!("INVALID"),
            }
        });
    }

    #[test]
    fn test_cannot_write_to_readonly_file() {
        use columndescription::*;

        duplicate_test_file(|filename| {
            let mut f = FitsFile::open(filename).unwrap();

            match f.create_image(
                "FOO".to_string(),
                &ImageDescription {
                    data_type: ImageType::Long,
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
    fn test_editing_a_current_file() {
        duplicate_test_file(|filename| {
            {
                let mut f = FitsFile::edit(filename).unwrap();
                let image_hdu = f.hdu(0).unwrap();

                let data_to_write: Vec<i64> = (0..100).map(|_| 10101).collect();
                image_hdu
                    .write_section(&mut f, 0, 100, &data_to_write)
                    .unwrap();
            }

            {
                let mut f = FitsFile::open(filename).unwrap();
                let hdu = f.hdu(0).unwrap();
                let read_data: Vec<i64> = hdu.read_section(&mut f, 0, 10).unwrap();
                assert_eq!(read_data, vec![10101; 10]);
            }
        });
    }

    #[test]
    fn test_fetching_a_hdu() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
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
    fn test_fetching_hdu_info() {
        use columndescription::*;

        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::ImageInfo { shape, image_type }) => {
                assert_eq!(shape.len(), 2);
                assert_eq!(shape, vec![100, 100]);
                assert_eq!(image_type, ImageType::Long);
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
    fn test_getting_file_open_mode() {
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
    fn test_adding_new_table() {
        use columndescription::*;

        with_temp_file(|filename| {
            {
                let mut f = FitsFile::create(filename).open().unwrap();
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
                .map(|mut f| {
                    f.change_hdu("foo").unwrap();
                    match f.fetch_hdu_info() {
                        Ok(HduInfo::TableInfo {
                            column_descriptions,
                            ..
                        }) => {
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
    fn test_adding_new_image() {
        with_temp_file(|filename| {
            {
                let mut f = FitsFile::create(filename).open().unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::Long,
                    dimensions: &[100, 20],
                };
                f.create_image("foo".to_string(), &image_description)
                    .unwrap();
            }

            FitsFile::open(filename)
                .map(|mut f| {
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
    fn test_multidimensional_images() {
        with_temp_file(|filename| {
            let dimensions = [10, 20, 15];

            {
                let mut f = FitsFile::create(filename).open().unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::Long,
                    dimensions: &dimensions,
                };
                let image_hdu = f.create_image("foo".to_string(), &image_description)
                    .unwrap();
                let data_to_write: Vec<i64> = (0..3000).collect();

                let xcoord = 0..dimensions[0] - 1;
                let ycoord = 0..dimensions[1] - 1;
                let zcoord = 0..dimensions[2] - 1;

                image_hdu
                    .write_region(&mut f, &[&xcoord, &ycoord, &zcoord], &data_to_write)
                    .unwrap();
            }

            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();

            let xcoord = 2..6;
            let ycoord = 11..17;
            let zcoord = 3..7;

            let read_data: Vec<i64> = hdu.read_region(&mut f, &vec![&xcoord, &ycoord, &zcoord])
                .unwrap();

            assert_eq!(read_data.len(), 96);
            assert_eq!(read_data[0], 712);
            assert_eq!(read_data[50], 942);
        });
    }

    #[test]
    fn test_fetching_hdu_object_hdu_info() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let testext = f.hdu("TESTEXT").unwrap();
        match testext.info {
            HduInfo::TableInfo { num_rows, .. } => {
                assert_eq!(num_rows, 50);
            }
            _ => panic!("Incorrect HDU type found"),
        }
    }

    #[test]
    fn test_fetch_current_hdu() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        f.change_hdu("TESTEXT").unwrap();
        let hdu = f.current_hdu().unwrap();

        assert_eq!(
            hdu.read_key::<String>(&mut f, "EXTNAME").unwrap(),
            "TESTEXT".to_string()
        );
    }

    #[test]
    fn test_fetch_primary_hdu() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let _hdu = f.primary_hdu().unwrap();
        assert_eq!(f.hdu_number(), 0);
    }

    #[test]
    fn test_fetch_number_of_hdus() {
        duplicate_test_file(|filename| {
            let mut f = FitsFile::open(filename).unwrap();
            let num_hdus = f.num_hdus().unwrap();
            assert_eq!(num_hdus, 2);
        });
    }

    #[test]
    fn test_fetch_hdu_names() {
        duplicate_test_file(|filename| {
            let mut f = FitsFile::open(filename).unwrap();
            let hdu_names = f.hdu_names().unwrap();
            assert_eq!(hdu_names.as_slice(), &["", "TESTEXT"]);
        });
    }

    #[test]
    fn test_creating_new_image_returns_hdu_object() {
        with_temp_file(|filename| {
            let mut f = FitsFile::create(filename).open().unwrap();
            let image_description = ImageDescription {
                data_type: ImageType::Long,
                dimensions: &[100, 20],
            };
            let hdu: FitsHdu = f.create_image("foo".to_string(), &image_description)
                .unwrap();
            assert_eq!(
                hdu.read_key::<String>(&mut f, "EXTNAME").unwrap(),
                "foo".to_string()
            );
        });
    }

    #[test]
    fn test_creating_new_table_returns_hdu_object() {
        use columndescription::*;

        with_temp_file(|filename| {
            let mut f = FitsFile::create(filename).open().unwrap();
            let table_description = vec![
                ColumnDescription::new("bar")
                    .with_type(ColumnDataType::Int)
                    .create()
                    .unwrap(),
            ];
            let hdu: FitsHdu = f.create_table("foo".to_string(), &table_description)
                .unwrap();
            assert_eq!(
                hdu.read_key::<String>(&mut f, "EXTNAME").unwrap(),
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
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = FitsHdu::new(&mut f, "TESTEXT").unwrap();
        match hdu.info {
            HduInfo::TableInfo { num_rows, .. } => {
                assert_eq!(num_rows, 50);
            }
            _ => panic!("Incorrect HDU type found"),
        }
    }

    #[test]
    fn test_reading_header_keys() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        match hdu.read_key::<i64>(&mut f, "INTTEST") {
            Ok(value) => assert_eq!(value, 42),
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match hdu.read_key::<f64>(&mut f, "DBLTEST") {
            Ok(value) => assert!(
                floats_close_f64(value, 0.09375),
                "{:?} != {:?}",
                value,
                0.09375
            ),
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match hdu.read_key::<String>(&mut f, "TEST") {
            Ok(value) => assert_eq!(value, "value"),
            Err(e) => panic!("Error reading key: {:?}", e),
        }
    }

    // Writing data
    #[test]
    fn test_writing_header_keywords() {
        with_temp_file(|filename| {
            // Scope ensures file is closed properly
            {
                let mut f = FitsFile::create(filename).open().unwrap();
                f.hdu(0).unwrap().write_key(&mut f, "FOO", 1i64).unwrap();
                f.hdu(0)
                    .unwrap()
                    .write_key(&mut f, "BAR", "baz".to_string())
                    .unwrap();
            }

            FitsFile::open(filename)
                .map(|mut f| {
                    assert_eq!(f.hdu(0).unwrap().read_key::<i64>(&mut f, "foo").unwrap(), 1);
                    assert_eq!(
                        f.hdu(0).unwrap().read_key::<String>(&mut f, "bar").unwrap(),
                        "baz".to_string()
                    );
                })
                .unwrap();
        });
    }

    #[test]
    fn test_fetching_column_width() {
        use super::column_display_width;

        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        f.hdu(1).unwrap();
        let width = column_display_width(&f, 3).unwrap();
        assert_eq!(width, 7);
    }

    #[test]
    fn test_read_columns() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let intcol_data: Vec<i32> = hdu.read_col(&mut f, "intcol").unwrap();
        assert_eq!(intcol_data[0], 18);
        assert_eq!(intcol_data[15], 10);
        assert_eq!(intcol_data[49], 12);

        let floatcol_data: Vec<f32> = hdu.read_col(&mut f, "floatcol").unwrap();
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

        let doublecol_data: Vec<f64> = hdu.read_col(&mut f, "doublecol").unwrap();
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
    fn test_read_string_col() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let strcol: Vec<String> = hdu.read_col(&mut f, "strcol").unwrap();
        assert_eq!(strcol.len(), 50);
        assert_eq!(strcol[0], "value0");
        assert_eq!(strcol[15], "value15");
        assert_eq!(strcol[49], "value49");
    }

    #[test]
    fn test_read_column_regions() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let intcol_data: Vec<i32> = hdu.read_col_range(&mut f, "intcol", &(0..2)).unwrap();
        assert_eq!(intcol_data.len(), 2);
        assert_eq!(intcol_data[0], 18);
        assert_eq!(intcol_data[1], 13);
    }

    #[test]
    fn test_read_invalid_column_range() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        match hdu.read_col_range::<i32>(&mut f, "intcol", &(0..1024)) {
            Err(Error::Index(IndexError { message, given })) => {
                assert_eq!(message, "given indices out of range".to_string());
                assert_eq!(given, (0..1024));
            }
            _ => panic!("Should be error"),
        }
    }

    #[test]
    fn test_read_string_column_regions() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let intcol_data: Vec<String> = hdu.read_col_range(&mut f, "strcol", &(0..2)).unwrap();
        assert_eq!(intcol_data.len(), 2);
        assert_eq!(intcol_data[0], "value0");
        assert_eq!(intcol_data[1], "value1");
    }

    #[test]
    fn test_read_column_region_check_ranges() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let result_data: Result<Vec<i32>> = hdu.read_col_range(&mut f, "intcol", &(0..2_000_000));
        assert!(result_data.is_err());
    }

    #[test]
    fn test_column_iterator() {
        use super::Column;

        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let column_names: Vec<String> = hdu.columns(&mut f)
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
    fn test_column_number() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu("testext").unwrap();
        assert_eq!(hdu.get_column_no(&mut f, "intcol").unwrap(), 0);
        assert_eq!(hdu.get_column_no(&mut f, "floatcol").unwrap(), 1);
        assert_eq!(hdu.get_column_no(&mut f, "doublecol").unwrap(), 2);
    }

    #[test]
    fn test_write_column_data() {
        use columndescription::*;

        with_temp_file(|filename| {
            let data_to_write: Vec<i32> = vec![10101; 10];
            {
                let mut f = FitsFile::create(filename).open().unwrap();
                let table_description = vec![
                    ColumnDescription::new("bar")
                        .with_type(ColumnDataType::Int)
                        .create()
                        .unwrap(),
                ];
                let hdu = f.create_table("foo".to_string(), &table_description)
                    .unwrap();

                hdu.write_col(&mut f, "bar", &data_to_write).unwrap();
            }

            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let data: Vec<i32> = hdu.read_col(&mut f, "bar").unwrap();
            assert_eq!(data, data_to_write);
        });
    }

    #[test]
    fn test_cannot_write_column_to_image_hdu() {
        with_temp_file(|filename| {
            let data_to_write: Vec<i32> = vec![10101; 10];

            let mut f = FitsFile::create(filename).open().unwrap();

            let image_description = ImageDescription {
                data_type: ImageType::Long,
                dimensions: &[100, 20],
            };
            let hdu = f.create_image("foo".to_string(), &image_description)
                .unwrap();

            match hdu.write_col(&mut f, "bar", &data_to_write) {
                Err(Error::Message(msg)) => {
                    assert_eq!(msg, "Cannot write column data to FITS image")
                }
                _ => panic!("Should return an error"),
            }
        });
    }

    #[test]
    fn test_write_column_subset() {
        use columndescription::*;

        with_temp_file(|filename| {
            let data_to_write: Vec<i32> = vec![10101; 10];
            {
                let mut f = FitsFile::create(filename).open().unwrap();
                let table_description = vec![
                    ColumnDescription::new("bar")
                        .with_type(ColumnDataType::Int)
                        .create()
                        .unwrap(),
                ];
                let hdu = f.create_table("foo".to_string(), &table_description)
                    .unwrap();

                hdu.write_col_range(&mut f, "bar", &data_to_write, &(0..5))
                    .unwrap();
            }

            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let data: Vec<i32> = hdu.read_col(&mut f, "bar").unwrap();
            assert_eq!(data.len(), 5);
            assert_eq!(data[..], data_to_write[0..5]);
        });
    }

    #[test]
    fn test_write_string_col() {
        use columndescription::*;

        with_temp_file(|filename| {
            let mut data_to_write: Vec<String> = Vec::new();
            for i in 0..50 {
                data_to_write.push(format!("value{}", i));
            }

            {
                let mut f = FitsFile::create(filename).open().unwrap();
                let table_description = vec![
                    ColumnDescription::new("bar")
                        .with_type(ColumnDataType::String)
                        .that_repeats(7)
                        .create()
                        .unwrap(),
                ];
                let hdu = f.create_table("foo".to_string(), &table_description)
                    .unwrap();

                hdu.write_col(&mut f, "bar", &data_to_write).unwrap();
            }

            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let data: Vec<String> = hdu.read_col(&mut f, "bar").unwrap();
            assert_eq!(data.len(), data_to_write.len());
            assert_eq!(data[0], "value0");
            assert_eq!(data[49], "value49");
        });
    }

    #[test]
    fn test_write_string_col_range() {
        use columndescription::*;

        with_temp_file(|filename| {
            let mut data_to_write: Vec<String> = Vec::new();
            for i in 0..50 {
                data_to_write.push(format!("value{}", i));
            }

            let range = 0..20;
            {
                let mut f = FitsFile::create(filename).open().unwrap();
                let table_description = vec![
                    ColumnDescription::new("bar")
                        .with_type(ColumnDataType::String)
                        .that_repeats(7)
                        .create()
                        .unwrap(),
                ];
                let hdu = f.create_table("foo".to_string(), &table_description)
                    .unwrap();

                hdu.write_col_range(&mut f, "bar", &data_to_write, &range)
                    .unwrap();
            }

            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let data: Vec<String> = hdu.read_col(&mut f, "bar").unwrap();
            assert_eq!(data.len(), range.end - range.start);
            assert_eq!(data[0], "value0");
            assert_eq!(data[19], "value19");
        });
    }

    #[test]
    fn test_read_image_data() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let first_row: Vec<i32> = hdu.read_section(&mut f, 0, 100).unwrap();
        assert_eq!(first_row.len(), 100);
        assert_eq!(first_row[0], 108);
        assert_eq!(first_row[49], 176);

        let second_row: Vec<i32> = hdu.read_section(&mut f, 100, 200).unwrap();
        assert_eq!(second_row.len(), 100);
        assert_eq!(second_row[0], 177);
        assert_eq!(second_row[49], 168);
    }

    #[test]
    fn test_read_whole_image() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let image: Vec<i32> = hdu.read_image(&mut f).unwrap();
        assert_eq!(image.len(), 10000);
    }

    #[test]
    fn test_read_image_rows() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let row: Vec<i32> = hdu.read_rows(&mut f, 0, 2).unwrap();
        let ref_row: Vec<i32> = hdu.read_section(&mut f, 0, 200).unwrap();
        assert_eq!(row, ref_row);
    }

    #[test]
    fn test_read_image_row() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let row: Vec<i32> = hdu.read_row(&mut f, 0).unwrap();
        let ref_row: Vec<i32> = hdu.read_section(&mut f, 0, 100).unwrap();
        assert_eq!(row, ref_row);
    }

    #[test]
    fn test_read_image_slice() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();

        let xcoord = 5..7;
        let ycoord = 2..3;

        let chunk: Vec<i32> = hdu.read_region(&mut f, &vec![&ycoord, &xcoord]).unwrap();
        assert_eq!(chunk.len(), 2);
        assert_eq!(chunk[0], 168);
        assert_eq!(chunk[chunk.len() - 1], 193);
    }

    #[test]
    fn test_read_image_region_from_table() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu("TESTEXT").unwrap();
        match hdu.read_region::<i32>(&mut f, &vec![&(0..10), &(0..10)]) {
            Err(Error::Message(msg)) => {
                assert!(msg.contains("cannot read image data from a table hdu"))
            }
            _ => panic!("SHOULD FAIL"),
        }
    }

    #[test]
    fn test_read_image_section_from_table() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu("TESTEXT").unwrap();
        if let Err(Error::Message(msg)) = hdu.read_section::<i32>(&mut f, 0, 100) {
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

                let mut f = FitsFile::create(filename).open().unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::Long,
                    dimensions: &[100, 20],
                };
                let hdu = f.create_image("foo".to_string(), &image_description)
                    .unwrap();
                hdu.write_section(&mut f, 0, 100, &data_to_write).unwrap();
            }

            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let first_row: Vec<i64> = hdu.read_section(&mut f, 0, 100).unwrap();
            assert_eq!(first_row, data_to_write);
        });
    }

    #[test]
    fn test_write_image_region() {
        with_temp_file(|filename| {
            // Scope ensures file is closed properly
            {
                use fitsfile::ImageDescription;

                let mut f = FitsFile::create(filename).open().unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::Long,
                    dimensions: &[100, 5],
                };
                let hdu = f.create_image("foo".to_string(), &image_description)
                    .unwrap();

                let data: Vec<i64> = (0..66).map(|v| v + 50).collect();
                hdu.write_region(&mut f, &[&(0..10), &(0..5)], &data)
                    .unwrap();
            }

            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let chunk: Vec<i64> = hdu.read_region(&mut f, &[&(0..10), &(0..5)]).unwrap();
            assert_eq!(chunk.len(), 10 * 5);
            assert_eq!(chunk[0], 50);
            assert_eq!(chunk[25], 75);
        });
    }

    #[test]
    fn test_write_image() {
        with_temp_file(|filename| {
            let data: Vec<i64> = (0..2000).collect();

            // Scope ensures file is closed properly
            {
                use fitsfile::ImageDescription;

                let mut f = FitsFile::create(filename).open().unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::Long,
                    dimensions: &[100, 20],
                };
                let hdu = f.create_image("foo".to_string(), &image_description)
                    .unwrap();

                hdu.write_image(&mut f, &data).unwrap();
            }

            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("foo").unwrap();
            let chunk: Vec<i64> = hdu.read_image(&mut f).unwrap();
            assert_eq!(chunk, data);
        });
    }

    #[test]
    fn test_resizing_images() {
        with_temp_file(|filename| {
            // Scope ensures file is closed properly
            {
                use fitsfile::ImageDescription;

                let mut f = FitsFile::create(filename).open().unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::Long,
                    dimensions: &[100, 20],
                };
                f.create_image("foo".to_string(), &image_description)
                    .unwrap();
            }

            /* Now resize the image */
            {
                let mut f = FitsFile::edit(filename).unwrap();
                let hdu = f.hdu("foo").unwrap();
                hdu.resize(&mut f, &[1024, 1024]).unwrap();
            }

            /* Images are only resized when flushed to disk, so close the file and
             * open it again */
            {
                let mut f = FitsFile::edit(filename).unwrap();
                let hdu = f.hdu("foo").unwrap();
                match hdu.info {
                    HduInfo::ImageInfo { shape, .. } => {
                        assert_eq!(shape, [1024, 1024]);
                    }
                    _ => panic!("Unexpected hdu type"),
                }
            }
        });
    }

    #[test]
    fn test_resize_3d() {
        with_temp_file(|filename| {
            // Scope ensures file is closed properly
            {
                use fitsfile::ImageDescription;

                let mut f = FitsFile::create(filename).open().unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::Long,
                    dimensions: &[100, 20],
                };
                f.create_image("foo".to_string(), &image_description)
                    .unwrap();
            }

            /* Now resize the image */
            {
                let mut f = FitsFile::edit(filename).unwrap();
                let hdu = f.hdu("foo").unwrap();
                hdu.resize(&mut f, &[1024, 1024, 5]).unwrap();
            }

            /* Images are only resized when flushed to disk, so close the file and
             * open it again */
            {
                let mut f = FitsFile::edit(filename).unwrap();
                let hdu = f.hdu("foo").unwrap();
                match hdu.info {
                    HduInfo::ImageInfo { shape, .. } => {
                        assert_eq!(shape, [1024, 1024, 5]);
                    }
                    _ => panic!("Unexpected hdu type"),
                }
            }
        });
    }

    #[test]
    fn test_write_image_section_to_table() {
        with_temp_file(|filename| {
            let data_to_write: Vec<i64> = (0..100).map(|v| v + 50).collect();

            use columndescription::*;

            let mut f = FitsFile::create(filename).open().unwrap();
            let table_description = &[
                ColumnDescription::new("bar")
                    .with_type(ColumnDataType::Int)
                    .create()
                    .unwrap(),
            ];
            let hdu = f.create_table("foo".to_string(), table_description)
                .unwrap();
            if let Err(Error::Message(msg)) = hdu.write_section(&mut f, 0, 100, &data_to_write) {
                assert_eq!(msg, "cannot write image data to a table hdu");
            } else {
                panic!("Should have thrown an error");
            }
        });
    }

    #[test]
    fn test_write_image_region_to_table() {
        use columndescription::*;

        with_temp_file(|filename| {
            let data_to_write: Vec<i64> = (0..100).map(|v| v + 50).collect();

            let mut f = FitsFile::create(filename).open().unwrap();
            let table_description = &[
                ColumnDescription::new("bar")
                    .with_type(ColumnDataType::Int)
                    .create()
                    .unwrap(),
            ];
            let hdu = f.create_table("foo".to_string(), table_description)
                .unwrap();

            let ranges = vec![&(0..10), &(0..10)];
            if let Err(Error::Message(msg)) = hdu.write_region(&mut f, &ranges, &data_to_write) {
                assert_eq!(msg, "cannot write image data to a table hdu");
            } else {
                panic!("Should have thrown an error");
            }
        });
    }

    #[test]
    fn test_multi_hdu_workflow() {
        /* Check that hdu objects change the current HDU on every file access method */

        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let primary_hdu = f.hdu(0).unwrap();
        let column_hdu = f.hdu(1).unwrap();

        let first_row: Vec<i32> = primary_hdu.read_section(&mut f, 0, 100).unwrap();
        assert_eq!(first_row.len(), 100);
        assert_eq!(first_row[0], 108);
        assert_eq!(first_row[49], 176);

        let intcol_data: Vec<i32> = column_hdu.read_col(&mut f, "intcol").unwrap();
        assert_eq!(intcol_data[0], 18);
        assert_eq!(intcol_data[49], 12);
    }

    #[test]
    fn test_access_fptr_unsafe() {
        use fitsio_sys::fitsfile;

        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let fptr: *const fitsfile = unsafe { f.as_raw() };
        assert!(!fptr.is_null());
    }

    #[test]
    fn test_extended_filename_syntax() {
        let filename = "../testdata/full_example.fits[TESTEXT]";
        let f = FitsFile::open(filename).unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { .. }) => {}
            Ok(HduInfo::ImageInfo { .. }) => panic!("Should be binary table"),
            _ => panic!("ERROR!"),
        }
    }

    #[test]
    fn test_copy_hdu() {
        duplicate_test_file(|src_filename| {
            with_temp_file(|dest_filename| {
                let mut src = FitsFile::open(src_filename).unwrap();
                let src_hdu = src.hdu("TESTEXT").unwrap();

                {
                    let mut dest = FitsFile::create(dest_filename).open().unwrap();
                    src_hdu.copy_to(&mut src, &mut dest).unwrap();
                }

                let mut dest = FitsFile::open(dest_filename).unwrap();
                let _dest_hdu = dest.hdu("TESTEXT").unwrap();

                /* If we do not error then the hdu has been copied */
            });
        });
    }

    #[test]
    fn test_changing_image_returns_new_hdu() {
        duplicate_test_file(|filename| {
            let mut f = FitsFile::edit(filename).unwrap();
            let hdu = f.hdu(0).unwrap();
            let newhdu = hdu.resize(&mut f, &vec![1024, 1024]).unwrap();

            match newhdu.info {
                HduInfo::ImageInfo { shape, .. } => {
                    assert_eq!(shape, vec![1024, 1024]);
                }
                _ => panic!("ERROR!"),
            }
        });
    }

    #[test]
    fn test_fetch_hdu_name() {
        duplicate_test_file(|filename| {
            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("TESTEXT").unwrap();
            assert_eq!(hdu.name(&mut f).unwrap(), "TESTEXT".to_string());
        });
    }

    #[test]
    fn test_inserting_columns() {
        duplicate_test_file(|filename| {
            use columndescription::{ColumnDataType, ColumnDescription};

            let mut f = FitsFile::edit(filename).unwrap();
            let hdu = f.hdu("TESTEXT").unwrap();

            let coldesc = ColumnDescription::new("abcdefg")
                .with_type(ColumnDataType::Int)
                .create()
                .unwrap();

            let newhdu = hdu.insert_column(&mut f, 0, &coldesc).unwrap();

            match newhdu.info {
                HduInfo::TableInfo {
                    column_descriptions,
                    ..
                } => {
                    assert_eq!(column_descriptions[0].name, "abcdefg");
                }
                _ => panic!("ERROR"),
            }
        });
    }

    #[test]
    fn test_appending_columns() {
        duplicate_test_file(|filename| {
            use columndescription::{ColumnDataType, ColumnDescription};

            let mut f = FitsFile::edit(filename).unwrap();
            let hdu = f.hdu("TESTEXT").unwrap();

            let coldesc = ColumnDescription::new("abcdefg")
                .with_type(ColumnDataType::Int)
                .create()
                .unwrap();

            let newhdu = hdu.append_column(&mut f, &coldesc).unwrap();

            match newhdu.info {
                HduInfo::TableInfo {
                    column_descriptions,
                    ..
                } => {
                    assert_eq!(
                        column_descriptions[column_descriptions.len() - 1].name,
                        "abcdefg"
                    );
                }
                _ => panic!("ERROR"),
            }
        });
    }

    #[test]
    fn test_deleting_columns_by_name() {
        duplicate_test_file(|filename| {
            let mut f = FitsFile::edit(filename).unwrap();
            let hdu = f.hdu("TESTEXT").unwrap();
            let newhdu = hdu.delete_column(&mut f, "intcol").unwrap();

            match newhdu.info {
                HduInfo::TableInfo {
                    column_descriptions,
                    ..
                } => for col in column_descriptions {
                    assert!(col.name != "intcol");
                },
                _ => panic!("ERROR"),
            }
        });
    }

    #[test]
    fn test_delete_hdu() {
        duplicate_test_file(|filename| {
            {
                let mut f = FitsFile::edit(filename).unwrap();
                let hdu = f.hdu("TESTEXT").unwrap();
                hdu.delete(&mut f).unwrap();
            }

            let mut f = FitsFile::open(filename).unwrap();
            let hdu_names = f.hdu_names().unwrap();
            assert!(!hdu_names.contains(&"TESTEXT".to_string()));
        });
    }

    #[test]
    fn test_deleting_columns_by_number() {
        duplicate_test_file(|filename| {
            let mut f = FitsFile::edit(filename).unwrap();
            let hdu = f.hdu("TESTEXT").unwrap();
            let newhdu = hdu.delete_column(&mut f, 0).unwrap();

            match newhdu.info {
                HduInfo::TableInfo {
                    column_descriptions,
                    ..
                } => for col in column_descriptions {
                    assert!(col.name != "intcol");
                },
                _ => panic!("ERROR"),
            }
        });
    }

    #[test]
    fn test_hdu_iterator() {
        duplicate_test_file(|filename| {
            let mut f = FitsFile::open(filename).unwrap();
            let mut counter = 0;

            for _ in f.iter() {
                counter += 1;
            }

            assert_eq!(counter, 2);
        });
    }

    #[test]
    fn test_read_single_table_value() {
        let filename = "../testdata/full_example.fits[TESTEXT]";
        let mut f = FitsFile::open(filename).unwrap();
        let tbl_hdu = f.hdu("TESTEXT").unwrap();

        let result: i64 = tbl_hdu.read_cell_value(&mut f, "intcol", 4).unwrap();
        assert_eq!(result, 16);

        let result: String = tbl_hdu.read_cell_value(&mut f, "strcol", 4).unwrap();
        assert_eq!(result, "value4".to_string());
    }
}
