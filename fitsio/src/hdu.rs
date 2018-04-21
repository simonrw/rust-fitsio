//! Fits HDU related code

use std::ffi;
use std::ops::Range;
use fitsfile::FitsFile;
use headers::{ReadsKey, WritesKey};
use images::{ImageType, ReadImage, WriteImage};
use tables::{ColumnIterator, ConcreteColumnDescription, DescribesColumnLocation, FitsRow,
             ReadsCol, WritesCol};
use longnam::*;
use fitsfile::CaseSensitivity;
use errors::{check_status, Result};

/// Struct representing a FITS HDU
#[derive(Debug, PartialEq)]
pub struct FitsHdu {
    /// Information about the current HDU
    pub info: HduInfo,
    pub(crate) hdu_num: usize,
}

impl FitsHdu {
    pub(crate) fn new<T: DescribesHdu>(
        fits_file: &mut FitsFile,
        hdu_description: T,
    ) -> Result<Self> {
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

    /**
    Read header key

    # Example

    ```rust
    # extern crate fitsio;
    #
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let filename = "../testdata/full_example.fits";
    # let mut fptr = fitsio::FitsFile::open(filename)?;
    # let hdu = fptr.primary_hdu()?;
    # {
    let int_value: i64 = hdu.read_key(&mut fptr, "INTTEST")?;
    # }
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    */
    pub fn read_key<T: ReadsKey>(&self, fits_file: &mut FitsFile, name: &str) -> Result<T> {
        fits_file.make_current(self)?;
        T::read_key(fits_file, name)
    }

    /**
    Write a fits key to the current header

    # Example

    ```rust
    # extern crate tempdir;
    # extern crate fitsio;
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # {
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    fptr.primary_hdu()?.write_key(&mut fptr, "foo", 1i64)?;
    assert_eq!(fptr.hdu(0)?.read_key::<i64>(&mut fptr, "foo")?, 1i64);
    # Ok(())
    # }
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
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

    /**
    Read pixels from an image between a start index and end index

    The range is exclusive of the upper value

    # Example

    ```rust
    # extern crate fitsio;
    #
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let filename = "../testdata/full_example.fits";
    # let mut fptr = fitsio::FitsFile::open(filename)?;
    # let hdu = fptr.hdu(0)?;
    // Read the first 100 pixels
    let first_row: Vec<i32> = hdu.read_section(&mut fptr, 0, 100)?;
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn read_section<T: ReadImage>(
        &self,
        fits_file: &mut FitsFile,
        start: usize,
        end: usize,
    ) -> Result<T> {
        fits_file.make_current(self)?;
        T::read_section(fits_file, self, start..end)
    }

    /**
    # Example

    ```rust
    # extern crate fitsio;
    #
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let filename = "../testdata/full_example.fits";
    # let mut fptr = fitsio::FitsFile::open(filename)?;
    # let hdu = fptr.hdu(0)?;
    let start_row = 0;
    let num_rows = 10;
    let first_few_rows: Vec<f32> = hdu.read_rows(&mut fptr, start_row, num_rows)?;

    // 10 rows of 100 columns
    assert_eq!(first_few_rows.len(), 1000);
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn read_rows<T: ReadImage>(
        &self,
        fits_file: &mut FitsFile,
        start_row: usize,
        num_rows: usize,
    ) -> Result<T> {
        fits_file.make_current(self)?;
        T::read_rows(fits_file, self, start_row, num_rows)
    }

    /**
    Read a single row from a fits image

    # Example

    ```rust
    # extern crate fitsio;
    #
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let filename = "../testdata/full_example.fits";
    # let mut fptr = fitsio::FitsFile::open(filename)?;
    # let hdu = fptr.hdu(0)?;
    let chosen_row = 5;
    let row: Vec<f32> = hdu.read_row(&mut fptr, chosen_row)?;

    // Should have 100 pixel values
    assert_eq!(row.len(), 100);
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn read_row<T: ReadImage>(&self, fits_file: &mut FitsFile, row: usize) -> Result<T> {
        fits_file.make_current(self)?;
        T::read_row(fits_file, self, row)
    }

    /**
    Read a square region from the chip.

    Lower left indicates the starting point of the square, and the upper
    right defines the pixel _beyond_ the end. The range of pixels included
    is inclusive of the lower end, and *exclusive* of the upper end.

    # Example

    ```rust
    # extern crate fitsio;
    #
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let filename = "../testdata/full_example.fits";
    # let mut fptr = fitsio::FitsFile::open(filename)?;
    # let hdu = fptr.hdu(0)?;
    // Read a square section of the image
    let xcoord = 0..10;
    let ycoord = 0..10;
    let chunk: Vec<i32> = hdu.read_region(&mut fptr, &[&ycoord, &xcoord])?;
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn read_region<T: ReadImage>(
        &self,
        fits_file: &mut FitsFile,
        ranges: &[&Range<usize>],
    ) -> Result<T> {
        fits_file.make_current(self)?;
        T::read_region(fits_file, self, ranges)
    }

    /**
    Read a whole image into a new `Vec`

    This reads an entire image into a one-dimensional vector

    # Example

    ```rust
    # extern crate fitsio;
    #
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let filename = "../testdata/full_example.fits";
    # let mut fptr = fitsio::FitsFile::open(filename)?;
    # let hdu = fptr.hdu(0)?;
    let image_data: Vec<f32> = hdu.read_image(&mut fptr)?;

    // 100 rows of 100 columns
    assert_eq!(image_data.len(), 10_000);
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn read_image<T: ReadImage>(&self, fits_file: &mut FitsFile) -> Result<T> {
        fits_file.make_current(self)?;
        T::read_image(fits_file, self)
    }

    /**
    Write raw pixel values to a FITS image

    If the length of the dataset exceeds the number of columns,
    the data wraps around to the next row.

    The range is exclusive of the upper value.

    # Example

    ```rust
    # extern crate fitsio;
    # extern crate tempdir;
    # use fitsio::images::{ImageDescription, ImageType};
    #
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let desc = ImageDescription {
    #    data_type: ImageType::Float,
    #    dimensions: &[100, 100],
    # };
    # let hdu = fptr.create_image("".to_string(), &desc)?;
    let data_to_write: Vec<f64> = vec![1.0, 2.0, 3.0];
    hdu.write_section(&mut fptr, 0, data_to_write.len(), &data_to_write)?;
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn write_section<T: WriteImage>(
        &self,
        fits_file: &mut FitsFile,
        start: usize,
        end: usize,
        data: &[T],
    ) -> Result<()> {
        fits_file.make_current(self)?;
        fits_check_readwrite!(fits_file);
        T::write_section(fits_file, self, start..end, data)
    }

    /**
    Write a rectangular region to the fits image

    The ranges must have length of 2, and they represent the limits of each axis. The limits
    are inclusive of the lower bounds, and *exclusive* of the and upper bounds.

    For example, writing with ranges 0..10 and 0..10 wries an 10x10 sized image.

    # Example

    ```rust
    # extern crate fitsio;
    # extern crate tempdir;
    # use fitsio::images::{ImageDescription, ImageType};
    #
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let desc = ImageDescription {
    #    data_type: ImageType::Float,
    #    dimensions: &[100, 100],
    # };
    # let hdu = fptr.create_image("".to_string(), &desc)?;
    let data_to_write: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
    let ranges = [&(0..1), &(0..1)];
    hdu.write_region(&mut fptr, &ranges, &data_to_write)?;
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn write_region<T: WriteImage>(
        &self,
        fits_file: &mut FitsFile,
        ranges: &[&Range<usize>],
        data: &[T],
    ) -> Result<()> {
        fits_file.make_current(self)?;
        fits_check_readwrite!(fits_file);
        T::write_region(fits_file, self, ranges, data)
    }

    /**
    Write an entire image to the HDU passed in

    Firstly a check is performed, making sure that the amount of data will fit in the image.
    After this, all of the data is written to the image.

    ## Example

    ```rust
    # extern crate fitsio;
    # extern crate tempdir;
    # use fitsio::images::{ImageType, ImageDescription};
    #
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let desc = ImageDescription {
    #    data_type: ImageType::Float,
    #    dimensions: &[3, 1],
    # };
    # let hdu = fptr.create_image("".to_string(), &desc)?;
    // Image is 3x1
    assert!(hdu.write_image(&mut fptr, &[1.0, 2.0, 3.0]).is_ok());
    assert!(hdu.write_image(&mut fptr, &[1.0, 2.0, 3.0, 4.0]).is_err());
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn write_image<T: WriteImage>(&self, fits_file: &mut FitsFile, data: &[T]) -> Result<()> {
        fits_file.make_current(self)?;
        fits_check_readwrite!(fits_file);
        T::write_image(fits_file, self, data)
    }

    /**
    Resize a HDU image

    The `new_size` parameter defines the new size of the image. Unlike cfitsio, the order
    of the dimensions of `new_size` follows the C convention, i.e. [row-major
    order](https://en.wikipedia.org/wiki/Row-_and_column-major_order).

    ## Example

    ```rust
    # extern crate tempdir;
    # extern crate fitsio;
    # use std::fs::copy;
    use fitsio::hdu::HduInfo;

    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # copy("../testdata/full_example.fits", &filename)?;
    # let filename = filename.to_str().unwrap();
    # let mut fptr = fitsio::FitsFile::edit(filename)?;
    # let hdu = fptr.hdu(0)?;
    hdu.resize(&mut fptr, &[1024, 1024])?;
    #
    // Have to get the HDU again, to reflect the latest changes
    let hdu = fptr.hdu(0)?;
    match hdu.info {
        HduInfo::ImageInfo { shape, .. } => {
            assert_eq!(shape, [1024, 1024]);
        }
        _ => panic!("Unexpected hdu type"),
    }
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
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

    /**
    Copy an HDU to another open fits file

    ## Example

    ```rust
    # extern crate tempdir;
    # extern crate fitsio;
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let filename = "../testdata/full_example.fits";
    # let mut src_fptr = fitsio::FitsFile::open(filename)?;
    #
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut dest_fptr = fitsio::FitsFile::create(filename).open()?;
    #
    # let hdu = src_fptr.hdu(1)?;
    hdu.copy_to(&mut src_fptr, &mut dest_fptr)?;
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
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

    /**
    Insert a column into a fits table

    The column location is 0-indexed. It is inserted _at_ that position, and the following
    columns are shifted back.

    ## Example

    ```rust
    # extern crate fitsio;
    # extern crate tempdir;
    use fitsio::tables::{ColumnDescription, ColumnDataType};

    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let table_description = &[
    #     ColumnDescription::new("bar")
    #         .with_type(ColumnDataType::Int)
    #         .create()?,
    # ];
    # let hdu = fptr.create_table("foo".to_string(), table_description)?;
    let column_description = ColumnDescription::new("abcdefg")
        .with_type(ColumnDataType::Int)
        .create()?;
    hdu.insert_column(&mut fptr, 1, &column_description)?;
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
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

    /**
    Add a new column to the end of the table

    ## Example

    ```rust
    # extern crate fitsio;
    # extern crate tempdir;
    use fitsio::tables::{ColumnDescription, ColumnDataType};

    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let table_description = &[
    #     ColumnDescription::new("bar")
    #         .with_type(ColumnDataType::Int)
    #         .create()?,
    # ];
    # let hdu = fptr.create_table("foo".to_string(), table_description)?;
    let column_description = ColumnDescription::new("abcdefg")
        .with_type(ColumnDataType::Int)
        .create()?;
    hdu.append_column(&mut fptr, &column_description)?;
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
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

    /**
    Remove a column from the fits file

    The column can be identified by id or name.

    ## Example

    ```rust
    # extern crate fitsio;
    # extern crate tempdir;
    # use fitsio::FitsFile;
    # use fitsio::tables::{ColumnDescription, ColumnDataType};

    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = FitsFile::create(filename).open()?;
    # let table_description = &[
    #     ColumnDescription::new("bar")
    #         .with_type(ColumnDataType::Int)
    #         .create()?,
    # ];
    # let hdu = fptr.create_table("foo".to_string(), table_description)?;
    let newhdu = hdu.delete_column(&mut fptr, "bar")?;
    # }
    # {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let table_description = &[
    #     ColumnDescription::new("bar")
    #         .with_type(ColumnDataType::Int)
    #         .create()?,
    # ];
    # let hdu = fptr.create_table("foo".to_string(), table_description)?;
    // or
    let newhdu = hdu.delete_column(&mut fptr, 0)?;
    # }
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
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

    /**
    Return the index for a given column.

    Internal method, not exposed.
    */
    pub(crate) fn get_column_no<T: Into<String>>(
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

    /**
    Read a subset of a fits column

    The range is exclusive of the upper value

    ## Example

    ```rust
    # extern crate tempdir;
    # extern crate fitsio;
    # use std::fs::copy;
    # use fitsio::hdu::HduInfo;
    # use fitsio::tables::{ColumnDescription, ColumnDataType};
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let table_description = vec![
    #     ColumnDescription::new("bar")
    #         .with_type(ColumnDataType::Int)
    #         .create()?,
    # ];
    # let hdu = fptr.create_table("foo".to_string(), &table_description)?;
    let data_to_write: Vec<i32> = vec![10101; 10];
    hdu.write_col_range(&mut fptr, "bar", &data_to_write, &(0..5))?;
    let data: Vec<i32> = hdu.read_col(&mut fptr, "bar")?;
    assert_eq!(data, vec![10101, 10101, 10101, 10101, 10101]);
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn read_col<T: ReadsCol>(&self, fits_file: &mut FitsFile, name: &str) -> Result<Vec<T>> {
        fits_file.make_current(self)?;
        T::read_col(fits_file, name)
    }

    /**
    Read a subset of a fits column

    The range is exclusive of the upper value

    ## Example

    ```rust
    # extern crate tempdir;
    # extern crate fitsio;
    # use std::fs::copy;
    # use fitsio::hdu::HduInfo;
    # use fitsio::tables::{ColumnDescription, ColumnDataType};
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let table_description = vec![
    #     ColumnDescription::new("bar")
    #         .with_type(ColumnDataType::Int)
    #         .create()?,
    # ];
    # let hdu = fptr.create_table("foo".to_string(), &table_description)?;
    # let data_to_write: Vec<i32> = vec![10101; 10];
    # hdu.write_col_range(&mut fptr, "bar", &data_to_write, &(0..5))?;
    let data: Vec<i32> = hdu.read_col_range(&mut fptr, "bar", &(0..5))?;
    assert_eq!(data, vec![10101, 10101, 10101, 10101, 10101]);
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn read_col_range<T: ReadsCol>(
        &self,
        fits_file: &mut FitsFile,
        name: &str,
        range: &Range<usize>,
    ) -> Result<Vec<T>> {
        fits_file.make_current(self)?;
        T::read_col_range(fits_file, name, range)
    }

    /**
    Write data to part of a column

    The range is exclusive of the upper value

    ## Example

    ```rust
    # extern crate tempdir;
    # extern crate fitsio;
    # use std::fs::copy;
    # use fitsio::hdu::HduInfo;
    # use fitsio::tables::{ColumnDescription, ColumnDataType};
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let table_description = vec![
    #     ColumnDescription::new("bar")
    #         .with_type(ColumnDataType::Int)
    #         .create()?,
    # ];
    # let hdu = fptr.create_table("foo".to_string(), &table_description)?;
    let data_to_write: Vec<i32> = vec![10101; 10];
    hdu.write_col_range(&mut fptr, "bar", &data_to_write, &(0..5))?;
    # let data: Vec<i32> = hdu.read_col(&mut fptr, "bar")?;
    # assert_eq!(data, vec![10101, 10101, 10101, 10101, 10101]);
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
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

    /**
    Write data to an entire column

    This default implementation does not check the length of the column first, but if the
    length of the data array is longer than the length of the table, the table will be extended
    with extra rows. This is as per the fitsio definition.

    ## Example

    ```rust
    # extern crate tempdir;
    # extern crate fitsio;
    # use std::fs::copy;
    # use fitsio::hdu::HduInfo;
    # use fitsio::tables::{ColumnDescription, ColumnDataType};
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let table_description = vec![
    #     ColumnDescription::new("bar")
    #         .with_type(ColumnDataType::Int)
    #         .create()
    #         ?,
    # ];
    # let hdu = fptr.create_table("foo".to_string(), &table_description)
    #     ?;
    let data_to_write: Vec<i32> = vec![10101; 5];
    hdu.write_col(&mut fptr, "bar", &data_to_write)?;
    # let data: Vec<i32> = hdu.read_col(&mut fptr, "bar")?;
    # assert_eq!(data, vec![10101, 10101, 10101, 10101, 10101]);
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
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

    /**
    Iterate over the columns in a fits file

    ## Example

    ```rust
    # extern crate fitsio;
    #
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let filename = "../testdata/full_example.fits";
    # let mut fptr = fitsio::FitsFile::open(filename)?;
    # let hdu = fptr.hdu("TESTEXT")?;
    for column in hdu.columns(&mut fptr) {
        // Do something with column
    }
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn columns<'a>(&self, fits_file: &'a mut FitsFile) -> ColumnIterator<'a> {
        fits_file
            .make_current(self)
            .expect("Cannot make hdu current");
        ColumnIterator::new(fits_file)
    }

    /**
    Delete the current HDU from the fits file.

    Note this method takes `self` by value, and as such the hdu cannot be used after this
    method is called.

    ## Example

    ```rust
    # extern crate tempdir;
    # extern crate fitsio;
    # use fitsio::images::{ImageDescription, ImageType};
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let tdir = tempdir::TempDir::new("fitsio-")?;
    # let tdir_path = tdir.path();
    # let filename = tdir_path.join("test.fits");
    # let mut fptr = fitsio::FitsFile::create(filename).open()?;
    # let image_description = ImageDescription {
    #     data_type: ImageType::Float,
    #     dimensions: &[100, 100],
    # };
    # let hdu = fptr.create_image("EXTNAME".to_string(), &image_description)?;
    // let fptr = FitsFile::open(...)?;
    // let hdu = fptr.hdu(0)?;
    hdu.delete(&mut fptr)?;
    // Cannot use hdu after this
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn delete(self, fits_file: &mut FitsFile) -> Result<()> {
        fits_file.make_current(&self)?;

        let mut status = 0;
        let mut curhdu = 0;
        unsafe {
            fits_delete_hdu(fits_file.fptr as *mut _, &mut curhdu, &mut status);
        }
        check_status(status).map(|_| ())
    }

    /**
    Read a single value from a fits table

    This will be inefficient if lots of individual values are wanted.

    ## Example

    ```rust
    # extern crate fitsio;
    # fn try_main() -> Result<(), Box<std::error::Error>> {
    # let filename = "../testdata/full_example.fits[TESTEXT]";
    # let mut f = fitsio::FitsFile::open(filename)?;
    # let tbl_hdu = f.hdu("TESTEXT")?;
    let result: i64 = tbl_hdu.read_cell_value(&mut f, "intcol", 4)?;
    assert_eq!(result, 16);

    let result: String = tbl_hdu.read_cell_value(&mut f, "strcol", 4)?;
    assert_eq!(result, "value4".to_string());
    # Ok(())
    # }
    # fn main() { try_main().unwrap(); }
    ```
    */
    pub fn read_cell_value<T>(&self, fits_file: &mut FitsFile, name: &str, idx: usize) -> Result<T>
    where
        T: ReadsCol,
    {
        fits_file.make_current(self)?;
        T::read_cell_value(fits_file, name, idx)
    }

    /**
    Extract a single row from the file

    This method uses returns a [`FitsRow`](../tables/trait.FitsRow.html), which is provided by
    the user, using a `derive` implementation from the
    [`fitsio-derive`](https://docs.rs/fitsio-derive) crate.

    # Example

    ```rust
    #[macro_use]
    extern crate fitsio_derive;
    extern crate fitsio;
    use fitsio::tables::FitsRow;

    #[derive(Default, FitsRow)]
    struct Row {
        #[fitsio(colname = "intcol")]
        intfoo: i32,
        #[fitsio(colname = "strcol")]
        foobar: String,
    }
    #
    # fn main() {
    # let filename = "../testdata/full_example.fits[TESTEXT]";
    # let mut f = fitsio::FitsFile::open(filename).unwrap();
    # let hdu = f.hdu("TESTEXT").unwrap();

    // Pick the 4th row
    let row: Row = hdu.row(&mut f, 4).unwrap();
    assert_eq!(row.intfoo, 16);
    assert_eq!(row.foobar, "value4");
    # }
    ```
    */
    pub fn row<F>(&self, fits_file: &mut FitsFile, idx: usize) -> Result<F>
    where
        F: FitsRow,
    {
        fits_file.make_current(self)?;
        F::from_table(self, fits_file, idx)
    }
}

/// Iterator over fits HDUs
pub struct FitsHduIterator<'a> {
    pub(crate) current: usize,
    pub(crate) max: usize,
    pub(crate) fits_file: &'a mut FitsFile,
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

/**
Hdu description type

Any way of describing a HDU - number or string which either
changes the hdu by absolute number, or by name.
*/
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

/**
Description of the current HDU

If the current HDU is an image, then
[`fetch_hdu_info`][fetch-hdu-info] returns `HduInfo::ImageInfo`.
Otherwise the variant is `HduInfo::TableInfo`.

[fetch-hdu-info]: ../fitsfile/struct.FitsFile.html#method.fetch_hdu_info
*/
#[allow(missing_docs)]
#[derive(Debug, PartialEq)]
pub enum HduInfo {
    ImageInfo {
        shape: Vec<usize>,
        image_type: ImageType,
    },
    TableInfo {
        column_descriptions: Vec<ConcreteColumnDescription>,
        num_rows: usize,
    },
    AnyInfo,
}

macro_rules! hduinfo_into_impl {
    ($t: ty) => (
        impl From<HduInfo> for $t {
            fn from(original: HduInfo) -> $t {
                match original {
                    HduInfo::ImageInfo { .. } => 0,
                    HduInfo::TableInfo { .. } => 2,
                    HduInfo::AnyInfo => -1,
                }
            }
        }
    )
}

hduinfo_into_impl!(i8);
hduinfo_into_impl!(i32);
hduinfo_into_impl!(i64);

#[cfg(test)]
mod tests {
    use super::FitsFile;
    use hdu::{FitsHdu, HduInfo};
    use testhelpers::duplicate_test_file;

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
    fn test_fetch_hdu_name() {
        duplicate_test_file(|filename| {
            let mut f = FitsFile::open(filename).unwrap();
            let hdu = f.hdu("TESTEXT").unwrap();
            assert_eq!(hdu.name(&mut f).unwrap(), "TESTEXT".to_string());
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

}
