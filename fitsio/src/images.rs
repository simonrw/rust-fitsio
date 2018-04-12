use std::ptr;
use std::ops::Range;
use fitsfile::FitsFile;
use fitserror::check_status;
use hdu::{FitsHdu, HduInfo};
use types::DataType;
use longnam::*;
use errors::Result;

/// Reading fits images
pub trait ReadImage: Sized {
    #[doc(hidden)]
    fn read_section(fits_file: &mut FitsFile, hdu: &FitsHdu, range: Range<usize>) -> Result<Self>;

    #[doc(hidden)]
    fn read_rows(
        fits_file: &mut FitsFile,
        hdu: &FitsHdu,
        start_row: usize,
        num_rows: usize,
    ) -> Result<Self>;

    #[doc(hidden)]
    fn read_row(fits_file: &mut FitsFile, hdu: &FitsHdu, row: usize) -> Result<Self>;

    #[doc(hidden)]
    fn read_region(
        fits_file: &mut FitsFile,
        hdu: &FitsHdu,
        ranges: &[&Range<usize>],
    ) -> Result<Self>;

    #[doc(hidden)]
    fn read_image(fits_file: &mut FitsFile, hdu: &FitsHdu) -> Result<Self> {
        match hdu.info {
            HduInfo::ImageInfo { ref shape, .. } => {
                let mut npixels = 1;
                for dimension in shape {
                    npixels *= *dimension;
                }
                Self::read_section(fits_file, hdu, 0..npixels)
            }
            HduInfo::TableInfo { .. } => Err("cannot read image data from a table hdu".into()),
            HduInfo::AnyInfo => unreachable!(),
        }
    }
}

/// Reading fits images
pub trait WriteImage: Sized {
    #[doc(hidden)]
    fn write_section(
        fits_file: &mut FitsFile,
        hdu: &FitsHdu,
        range: Range<usize>,
        data: &[Self],
    ) -> Result<()>;

    #[doc(hidden)]
    fn write_region(
        fits_file: &mut FitsFile,
        hdu: &FitsHdu,
        ranges: &[&Range<usize>],
        data: &[Self],
    ) -> Result<()>;

    #[doc(hidden)]
    fn write_image(fits_file: &mut FitsFile, hdu: &FitsHdu, data: &[Self]) -> Result<()> {
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

                Self::write_section(fits_file, hdu, 0..data.len(), data)
            }
            Ok(HduInfo::TableInfo { .. }) => Err("cannot write image data to a table hdu".into()),
            Ok(HduInfo::AnyInfo) => unreachable!(),
            Err(e) => Err(e),
        }
    }
}

macro_rules! read_image_impl_vec {
    ($t: ty, $default_value: expr, $data_type: expr) => (
        impl ReadImage for Vec<$t> {

            fn read_section(
                fits_file: &mut FitsFile,
                hdu: &FitsHdu,
                range: Range<usize>) -> Result<Self> {
                match hdu.info {
                    HduInfo::ImageInfo { .. } => {
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
                    HduInfo::TableInfo { .. } =>
                        Err("cannot read image data from a table hdu".into()),
                    HduInfo::AnyInfo => unreachable!(),
                }
            }

            fn read_rows(fits_file: &mut FitsFile, hdu: &FitsHdu, start_row: usize, num_rows: usize)
                -> Result<Self> {
                    match hdu.info {
                        HduInfo::ImageInfo { ref shape, .. } => {
                            if shape.len() != 2 {
                                unimplemented!();
                            }

                            let num_cols = shape[1];
                            let start = start_row * num_cols;
                            let end = (start_row + num_rows) * num_cols;

                            Self::read_section(fits_file, hdu, start..end)
                        },
                        HduInfo::TableInfo { .. } =>
                            Err("cannot read image data from a table hdu".into()),
                        HduInfo::AnyInfo => unreachable!(),
                    }
                }

            fn read_row(fits_file: &mut FitsFile, hdu: &FitsHdu, row: usize) -> Result<Self> {
                Self::read_rows(fits_file, hdu, row, 1)
            }

            fn read_region(fits_file: &mut FitsFile, hdu: &FitsHdu, ranges: &[&Range<usize>])
                -> Result<Self> {
                    match hdu.info {
                        HduInfo::ImageInfo { .. } => {
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
                            let mut out = vec![$default_value; nelements];
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
                        HduInfo::TableInfo { .. } =>
                            Err("cannot read image data from a table hdu".into()),
                        HduInfo::AnyInfo => unreachable!(),
                    }
                }
        }
    )
}

macro_rules! write_image_impl {
    ($t: ty, $default_value: expr, $data_type: expr) => (
        impl WriteImage for $t {
            fn write_section(
                fits_file: &mut FitsFile,
                hdu: &FitsHdu,
                range: Range<usize>,
                data: &[Self])
                -> Result<()> {
                    match hdu.info {
                        HduInfo::ImageInfo { .. } => {
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
                        HduInfo::TableInfo { .. } =>
                            Err("cannot write image data to a table hdu".into()),
                        HduInfo::AnyInfo => unreachable!(),
                    }
                }

            fn write_region(
                fits_file: &mut FitsFile,
                hdu: &FitsHdu,
                ranges: &[&Range<usize>],
                data: &[Self])
                -> Result<()> {
                    match hdu.info {
                        HduInfo::ImageInfo { .. } => {
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
                        HduInfo::TableInfo { .. } =>
                            Err("cannot write image data to a table hdu".into()),
                        HduInfo::AnyInfo => unreachable!(),
                    }
                }
        }
    )
}

read_image_impl_vec!(i8, i8::default(), DataType::TSHORT);
read_image_impl_vec!(i32, i32::default(), DataType::TINT);
#[cfg(target_pointer_width = "64")]
read_image_impl_vec!(i64, i64::default(), DataType::TLONG);
#[cfg(target_pointer_width = "32")]
read_image_impl_vec!(i64, i64::default() DataType::TLONGLONG);
read_image_impl_vec!(u8, u8::default(), DataType::TUSHORT);
read_image_impl_vec!(u32, u32::default(), DataType::TUINT);
#[cfg(target_pointer_width = "64")]
read_image_impl_vec!(u64, u64::default(), DataType::TULONG);
read_image_impl_vec!(f32, f32::default(), DataType::TFLOAT);
read_image_impl_vec!(f64, f64::default(), DataType::TDOUBLE);

write_image_impl!(i8, i8::default(), DataType::TSHORT);
write_image_impl!(i32, i32::default(), DataType::TINT);
#[cfg(target_pointer_width = "64")]
write_image_impl!(i64, i64::default(), DataType::TLONG);
#[cfg(target_pointer_width = "32")]
write_image_impl!(i64, i64::default() DataType::TLONGLONG);
write_image_impl!(u8, u8::default(), DataType::TUSHORT);
write_image_impl!(u32, u32::default(), DataType::TUINT);
#[cfg(target_pointer_width = "64")]
write_image_impl!(u64, u64::default(), DataType::TULONG);
write_image_impl!(f32, f32::default(), DataType::TFLOAT);
write_image_impl!(f64, f64::default(), DataType::TDOUBLE);

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

/// Data types used for defining images
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ImageType {
    UnsignedByte,
    Byte,
    Short,
    UnsignedShort,
    Long,
    UnsignedLong,
    LongLong,
    Float,
    Double,
}

macro_rules! imagetype_into_impl {
    ($t: ty) => (
        impl From<ImageType> for $t {
            fn from(original: ImageType) -> $t {
                match original {
                    ImageType::UnsignedByte => 8,
                    ImageType::Byte => 10,
                    ImageType::Short => 16,
                    ImageType::UnsignedShort => 20,
                    ImageType::Long => 32,
                    ImageType::UnsignedLong => 40,
                    ImageType::LongLong => 64,
                    ImageType::Float => -32,
                    ImageType::Double => -64,
                }
            }
        }
    )
}

imagetype_into_impl!(i8);
imagetype_into_impl!(i32);
imagetype_into_impl!(i64);
