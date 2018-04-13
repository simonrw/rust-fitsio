use std::ptr;
use std::str::FromStr;
use std::ffi;
use errors::{Error, IndexError, Result};
use fitserror::{check_status, FitsError};
use stringutils::status_to_string;
use fitsfile::FitsFile;
use hdu::{FitsHdu, HduInfo};
use types::DataType;
use std::ops::Range;
use longnam::*;
use libc;

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

reads_col_impl!(i32, fits_read_col_int, 0);
reads_col_impl!(u32, fits_read_col_uint, 0);
reads_col_impl!(f32, fits_read_col_flt, 0.0);
reads_col_impl!(f64, fits_read_col_dbl, 0.0);
#[cfg(target_pointer_width = "64")]
reads_col_impl!(i64, fits_read_col_lng, 0);
#[cfg(target_pointer_width = "32")]
reads_col_impl!(i64, fits_read_col_lnglng, 0);
#[cfg(target_pointer_width = "64")]
reads_col_impl!(u64, fits_read_col_ulng, 0);

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

/// Trait derivable with custom derive
pub trait FitsRow: ::std::default::Default {
    #[doc(hidden)]
    fn from_table(tbl: &FitsHdu, fits_file: &mut FitsFile, idx: usize) -> Result<Self>
    where
        Self: Sized;
}

/// Helper function to get the display width of a column
pub(crate) fn column_display_width(fits_file: &FitsFile, column_number: usize) -> Result<usize> {
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

/// Description for new columns
#[derive(Debug, Clone)]
pub struct ColumnDescription {
    /// Name of the column
    pub name: String,

    /// Type of the data, see the cfitsio documentation
    pub data_type: Option<ColumnDataDescription>,
}

/// Concrete representation of the description of a column
#[derive(Debug, Clone, PartialEq)]
pub struct ConcreteColumnDescription {
    /// Name of the column
    pub name: String,

    /// Type of the data, see the cfitsio documentation
    pub data_type: ColumnDataDescription,
}

impl ColumnDescription {
    /// Create a new [`ColumnDescription`](struct.ColumnDescription.html) from a name
    pub fn new<T: Into<String>>(name: T) -> Self {
        ColumnDescription {
            name: name.into(),
            data_type: None,
        }
    }

    /// Add a data type to the column description
    pub fn with_type(&mut self, typ: ColumnDataType) -> &mut ColumnDescription {
        self.data_type = Some(ColumnDataDescription::scalar(typ));
        self
    }

    /// Make the column repeat
    pub fn that_repeats(&mut self, repeat: usize) -> &mut ColumnDescription {
        if let Some(ref mut desc) = self.data_type {
            desc.repeat = repeat;
        }
        self
    }

    /// Define the column width
    pub fn with_width(&mut self, width: usize) -> &mut ColumnDescription {
        if let Some(ref mut desc) = self.data_type {
            desc.width = width;
        }
        self
    }

    /// Render the [`ColumnDescription`](struct.ColumnDescription.html) into a
    /// [`ConcreteColumnDescription`](struct.ConcreteColumnDescription.html)
    pub fn create(&self) -> Result<ConcreteColumnDescription> {
        match self.data_type {
            Some(ref d) => Ok(ConcreteColumnDescription {
                name: self.name.clone(),
                data_type: d.clone(),
            }),
            None => {
                Err("No data type given. Ensure the `with_type` method has been called.".into())
            }
        }
    }
}

/// Description of the column data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDataDescription {
    /// Does the column contain multiple values?
    pub repeat: usize,

    /// How wide is the column?
    pub width: usize,

    /// What data type does the column store?
    pub typ: ColumnDataType,
}

impl ColumnDataDescription {
    /// Create a new column data description
    pub fn new(typ: ColumnDataType, repeat: usize, width: usize) -> Self {
        ColumnDataDescription { repeat, width, typ }
    }

    /// Shortcut for creating a scalar column
    pub fn scalar(typ: ColumnDataType) -> Self {
        ColumnDataDescription::new(typ, 1, 1)
    }

    /// Shortcut for creating a vector column
    pub fn vector(typ: ColumnDataType, repeat: usize) -> Self {
        ColumnDataDescription::new(typ, repeat, 1)
    }
}

impl From<ColumnDataDescription> for String {
    fn from(orig: ColumnDataDescription) -> String {
        match orig.typ {
            ColumnDataType::Text => {
                if orig.width > 1 {
                    format!(
                        "{repeat}{data_type}{width}",
                        data_type = String::from(orig.typ),
                        repeat = orig.repeat,
                        width = orig.width
                    )
                } else {
                    format!(
                        "{repeat}{data_type}",
                        data_type = String::from(orig.typ),
                        repeat = orig.repeat
                    )
                }
            }
            _ => format!(
                "{repeat}{data_type}",
                data_type = String::from(orig.typ),
                repeat = orig.repeat
            ),
        }
    }
}

/// Types a column can represent
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnDataType {
    Int,
    Float,
    Text,
    Double,
    Short,
    Long,
    String,
}

impl From<ColumnDataType> for String {
    fn from(orig: ColumnDataType) -> String {
        use self::ColumnDataType::*;

        match orig {
            Int => "J",
            Float => "E",
            Text | String => "A",
            Double => "D",
            Short => "I",
            Long => "K",
        }.to_string()
    }
}

impl FromStr for ColumnDataDescription {
    type Err = Box<::std::error::Error>;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let chars: Vec<_> = s.chars().collect();

        let mut repeat_str = Vec::new();
        let mut last_position = 0;
        for c in &chars {
            if c.is_digit(10) {
                repeat_str.push(c);
                last_position += 1;
            } else {
                break;
            }
        }

        let repeat = if repeat_str.is_empty() {
            1
        } else {
            let repeat_str: String = repeat_str.into_iter().collect();
            repeat_str.parse::<usize>()?
        };

        let data_type_char = chars[last_position];
        last_position += 1;

        let mut width_str = Vec::new();
        for c in chars.iter().skip(last_position) {
            if c.is_digit(10) {
                width_str.push(c);
            } else {
                break;
            }
        }

        let width = if width_str.is_empty() {
            1
        } else {
            let width_str: String = width_str.into_iter().collect();
            width_str.parse::<usize>()?
        };

        let data_type = match data_type_char {
            'E' => ColumnDataType::Float,
            'J' => ColumnDataType::Int,
            'D' => ColumnDataType::Double,
            'I' => ColumnDataType::Short,
            'K' => ColumnDataType::Long,
            'A' => ColumnDataType::String,
            _ => panic!(
                "Have not implemented str -> ColumnDataType for {}",
                data_type_char
            ),
        };

        Ok(ColumnDataDescription {
            repeat,
            typ: data_type,
            width,
        })
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

macro_rules! datatype_into_impl {
    ($t: ty) => (
        impl From<DataType> for $t {
            fn from(original: DataType) -> $t {
                match original {
                    DataType::TBIT => 1,
                    DataType::TBYTE => 11,
                    DataType::TSBYTE => 12,
                    DataType::TLOGICAL => 14,
                    DataType::TSTRING => 16,
                    DataType::TUSHORT => 20,
                    DataType::TSHORT => 21,
                    DataType::TUINT => 30,
                    DataType::TINT => 31,
                    DataType::TULONG => 40,
                    DataType::TLONG => 41,
                    DataType::TLONGLONG => 81,
                    DataType::TFLOAT => 42,
                    DataType::TDOUBLE => 82,
                    DataType::TCOMPLEX => 83,
                    DataType::TDBLCOMPLEX => 163,
                }
            }
        }
    )
}

datatype_into_impl!(u8);
datatype_into_impl!(i32);
datatype_into_impl!(u32);
datatype_into_impl!(i64);
datatype_into_impl!(u64);

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
    pub(crate) fn new(fits_file: &'a FitsFile) -> Self {
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
