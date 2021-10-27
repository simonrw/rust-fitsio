//! Image related code
use crate::errors::{check_status, Result};
use crate::fitsfile::FitsFile;
use crate::hdu::{FitsHdu, HduInfo};
use crate::longnam::*;
use crate::types::DataType;
use std::ops::Range;
use std::ptr;

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
                    )
                    .as_str()
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
    ($t:ty, $default_value:expr, $data_type:expr) => {
        impl ReadImage for Vec<$t> {
            fn read_section(
                fits_file: &mut FitsFile,
                hdu: &FitsHdu,
                range: Range<usize>,
            ) -> Result<Self> {
                match hdu.info {
                    HduInfo::ImageInfo { .. } => {
                        let nelements = range.end - range.start;
                        let mut out = vec![$default_value; nelements];
                        let mut status = 0;

                        unsafe {
                            fits_read_img(
                                fits_file.fptr.as_mut() as *mut _,
                                $data_type.into(),
                                (range.start + 1) as i64,
                                nelements as i64,
                                ptr::null_mut(),
                                out.as_mut_ptr() as *mut _,
                                ptr::null_mut(),
                                &mut status,
                            );
                        }

                        check_status(status).map(|_| out)
                    }
                    HduInfo::TableInfo { .. } => {
                        Err("cannot read image data from a table hdu".into())
                    }
                    HduInfo::AnyInfo => unreachable!(),
                }
            }

            fn read_rows(
                fits_file: &mut FitsFile,
                hdu: &FitsHdu,
                start_row: usize,
                num_rows: usize,
            ) -> Result<Self> {
                match hdu.info {
                    HduInfo::ImageInfo { ref shape, .. } => {
                        if shape.len() != 2 {
                            unimplemented!();
                        }

                        let num_cols = shape[1];
                        let start = start_row * num_cols;
                        let end = (start_row + num_rows) * num_cols;

                        Self::read_section(fits_file, hdu, start..end)
                    }
                    HduInfo::TableInfo { .. } => {
                        Err("cannot read image data from a table hdu".into())
                    }
                    HduInfo::AnyInfo => unreachable!(),
                }
            }

            fn read_row(fits_file: &mut FitsFile, hdu: &FitsHdu, row: usize) -> Result<Self> {
                Self::read_rows(fits_file, hdu, row, 1)
            }

            fn read_region(
                fits_file: &mut FitsFile,
                hdu: &FitsHdu,
                ranges: &[&Range<usize>],
            ) -> Result<Self> {
                match hdu.info {
                    HduInfo::ImageInfo { .. } => {
                        let n_ranges = ranges.len();

                        let mut fpixel = Vec::with_capacity(n_ranges);
                        let mut lpixel = Vec::with_capacity(n_ranges);

                        let mut nelements = 1;
                        for range in ranges {
                            let start = range.start + 1;
                            // No +1 as the range is exclusive
                            let end = range.end;
                            fpixel.push(start as _);
                            lpixel.push(end as _);

                            nelements *= (end + 1) - start;
                        }

                        let mut inc: Vec<_> = (0..n_ranges).map(|_| 1).collect();
                        let vec_size = nelements;
                        let mut out = vec![$default_value; vec_size];
                        let mut status = 0;

                        unsafe {
                            fits_read_subset(
                                fits_file.fptr.as_mut() as *mut _, // fptr
                                $data_type.into(),                 // datatype
                                fpixel.as_mut_ptr(),               // fpixel
                                lpixel.as_mut_ptr(),               // lpixel
                                inc.as_mut_ptr(),                  // inc
                                ptr::null_mut(),                   // nulval
                                out.as_mut_ptr() as *mut _,        // array
                                ptr::null_mut(),                   // anynul
                                &mut status,                       // status
                            );
                        }

                        check_status(status).map(|_| out)
                    }
                    HduInfo::TableInfo { .. } => {
                        Err("cannot read image data from a table hdu".into())
                    }
                    HduInfo::AnyInfo => unreachable!(),
                }
            }
        }
    };
}

macro_rules! write_image_impl {
    ($t:ty, $default_value:expr, $data_type:expr) => {
        impl WriteImage for $t {
            fn write_section(
                fits_file: &mut FitsFile,
                hdu: &FitsHdu,
                range: Range<usize>,
                data: &[Self],
            ) -> Result<()> {
                match hdu.info {
                    HduInfo::ImageInfo { .. } => {
                        let nelements = range.end - range.start;
                        assert!(data.len() >= nelements);
                        let mut status = 0;
                        unsafe {
                            fits_write_img(
                                fits_file.fptr.as_mut() as *mut _,
                                $data_type.into(),
                                (range.start + 1) as i64,
                                nelements as i64,
                                data.as_ptr() as *mut _,
                                &mut status,
                            );
                        }

                        check_status(status)
                    }
                    HduInfo::TableInfo { .. } => {
                        Err("cannot write image data to a table hdu".into())
                    }
                    HduInfo::AnyInfo => unreachable!(),
                }
            }

            fn write_region(
                fits_file: &mut FitsFile,
                hdu: &FitsHdu,
                ranges: &[&Range<usize>],
                data: &[Self],
            ) -> Result<()> {
                match hdu.info {
                    HduInfo::ImageInfo { .. } => {
                        let n_ranges = ranges.len();

                        let mut fpixel = Vec::with_capacity(n_ranges);
                        let mut lpixel = Vec::with_capacity(n_ranges);

                        for range in ranges {
                            let start = range.start + 1;
                            // No +1 as the range is exclusive
                            let end = range.end;
                            fpixel.push(start as _);
                            lpixel.push(end as _);
                        }

                        let mut status = 0;

                        unsafe {
                            fits_write_subset(
                                fits_file.fptr.as_mut() as *mut _,
                                $data_type.into(),
                                fpixel.as_mut_ptr(),
                                lpixel.as_mut_ptr(),
                                data.as_ptr() as *mut _,
                                &mut status,
                            );
                        }

                        check_status(status)
                    }
                    HduInfo::TableInfo { .. } => {
                        Err("cannot write image data to a table hdu".into())
                    }
                    HduInfo::AnyInfo => unreachable!(),
                }
            }
        }
    };
}

read_image_impl_vec!(i8, i8::default(), DataType::TSBYTE);
read_image_impl_vec!(i16, i16::default(), DataType::TSHORT);
read_image_impl_vec!(i32, i32::default(), DataType::TINT);
#[cfg(target_pointer_width = "64")]
read_image_impl_vec!(i64, i64::default(), DataType::TLONG);
#[cfg(target_pointer_width = "32")]
read_image_impl_vec!(i64, i64::default(), DataType::TLONGLONG);
read_image_impl_vec!(u8, u8::default(), DataType::TBYTE);
read_image_impl_vec!(u16, u16::default(), DataType::TUSHORT);
read_image_impl_vec!(u32, u32::default(), DataType::TUINT);
#[cfg(target_pointer_width = "64")]
read_image_impl_vec!(u64, u64::default(), DataType::TULONG);
read_image_impl_vec!(f32, f32::default(), DataType::TFLOAT);
read_image_impl_vec!(f64, f64::default(), DataType::TDOUBLE);

write_image_impl!(i8, i8::default(), DataType::TSBYTE);
write_image_impl!(i16, i16::default(), DataType::TSHORT);
write_image_impl!(i32, i32::default(), DataType::TINT);
#[cfg(target_pointer_width = "64")]
write_image_impl!(i64, i64::default(), DataType::TLONG);
#[cfg(target_pointer_width = "32")]
write_image_impl!(i64, i64::default(), DataType::TLONGLONG);
write_image_impl!(u8, u8::default(), DataType::TBYTE);
write_image_impl!(u16, u16::default(), DataType::TUSHORT);
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

    /**
    Shape of the image

    Unlike cfitsio, the order of the dimensions follows the C convention, i.e. [row-major
    order](https://en.wikipedia.org/wiki/Row-_and_column-major_order).
    */
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
    ($t:ty) => {
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
    };
}

imagetype_into_impl!(i8);
imagetype_into_impl!(i16);
imagetype_into_impl!(i32);
imagetype_into_impl!(i64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fitsfile::FitsFile;
    use crate::testhelpers::with_temp_file;

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
        assert_eq!(chunk.len(), (7 - 5) * (3 - 2));
        assert_eq!(chunk[0], 168);
        assert_eq!(chunk[chunk.len() - 1], 112);
    }

    #[test]
    fn test_write_image_section() {
        with_temp_file(|filename| {
            let data_to_write: Vec<i64> = (0..100).map(|v| v + 50).collect();

            // Scope ensures file is closed properly
            {
                let mut f = FitsFile::create(filename).open().unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::Long,
                    dimensions: &[100, 20],
                };
                let hdu = f
                    .create_image("foo".to_string(), &image_description)
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
                let mut f = FitsFile::create(filename).open().unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::Long,
                    dimensions: &[100, 5],
                };
                let hdu = f
                    .create_image("foo".to_string(), &image_description)
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
            assert_eq!(chunk[25], 80);
        });
    }

    #[test]
    fn test_write_image() {
        with_temp_file(|filename| {
            let data: Vec<i64> = (0..2000).collect();

            // Scope ensures file is closed properly
            {
                let mut f = FitsFile::create(filename).open().unwrap();
                let image_description = ImageDescription {
                    data_type: ImageType::Long,
                    dimensions: &[100, 20],
                };
                let hdu = f
                    .create_image("foo".to_string(), &image_description)
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

    // helper macro to write a file with an image and table of a specific
    // type, using the low level `fitsio-sys`.
    macro_rules! example_file {
        ($type:ty, $image_type:expr, $data_type:expr, $cb:expr) => {
            use crate::stringutils::StringList;

            with_temp_file(|filename| {
                let dimensions = vec![5usize, 5];
                let image_data: Vec<$type> =
                    (0..(dimensions[0] * dimensions[1]) as $type).collect();
                let column: Vec<_> = (0..10).into_iter().map(|i| i + 200).collect();
                {
                    let mut fptr = std::ptr::null_mut();
                    let mut status = 0;
                    let c_filename = std::ffi::CString::new(filename).unwrap();

                    unsafe {
                        crate::longnam::fits_create_file(
                            &mut fptr as *mut *mut crate::sys::fitsfile,
                            c_filename.as_ptr(),
                            &mut status,
                        );
                    }
                    let _ = crate::errors::check_status(status).unwrap();

                    // write the primary u16 image
                    let naxis = dimensions.len();
                    unsafe {
                        crate::longnam::fits_create_img(
                            fptr as *mut _,
                            $image_type,
                            naxis as _,
                            dimensions.as_ptr() as *mut _,
                            &mut status,
                        );
                    }
                    let _ = crate::errors::check_status(status).unwrap();

                    unsafe {
                        crate::longnam::fits_write_img(
                            fptr as *mut _,
                            $data_type,
                            1,
                            image_data.len() as _,
                            image_data.as_ptr() as *mut _,
                            &mut status,
                        );
                    }
                    let _ = crate::errors::check_status(status).unwrap();

                    // write the table
                    let ttype = StringList::from_slice(&["example".to_string()]).unwrap();
                    // TODO(srw): this form string should be different depending on the type
                    let tform = StringList::from_slice(&["1I".to_string()]).unwrap();
                    let tunit = StringList::from_slice(&["".to_string()]).unwrap();
                    unsafe {
                        crate::longnam::fits_create_tbl(
                            fptr as *mut _,
                            crate::sys::BINARY_TBL as _,
                            0,
                            1,
                            ttype.as_ptr(),
                            tform.as_ptr(),
                            tunit.as_ptr(),
                            std::ptr::null_mut(),
                            &mut status,
                        );
                    }

                    unsafe {
                        crate::longnam::fits_write_col(
                            fptr as *mut _,
                            $data_type,
                            1,
                            1,
                            1,
                            10,
                            column.as_ptr() as *mut _,
                            &mut status,
                        );
                    }

                    unsafe {
                        crate::longnam::fits_close_file(fptr as *mut _, &mut status);
                    }
                    let _ = crate::errors::check_status(status).unwrap();
                }
                $cb(filename, image_data, column);
            });
        };
    }

    #[test]
    fn i16_image() {
        example_file!(
            i16,
            crate::images::ImageType::Short.into(),
            crate::images::DataType::TSHORT.into(),
            |filename, image_data, column| {
                // now read the file and check the output
                let mut f = FitsFile::open(filename).unwrap();
                let hdu = f.primary_hdu().unwrap();
                let data: Vec<i16> = hdu.read_image(&mut f).unwrap();
                assert_eq!(data, image_data);

                let hdu = f.hdu(1).unwrap();
                let data: Vec<i16> = hdu.read_col(&mut f, "example").unwrap();
                assert_eq!(data, column);
            }
        );
    }

    #[test]
    fn u16_image() {
        example_file!(
            u16,
            crate::images::ImageType::UnsignedShort.into(),
            crate::images::DataType::TUSHORT.into(),
            |filename, image_data, column| {
                // now read the file and check the output
                let mut f = FitsFile::open(filename).unwrap();
                let hdu = f.primary_hdu().unwrap();
                let data: Vec<u16> = hdu.read_image(&mut f).unwrap();
                assert_eq!(data, image_data);

                let hdu = f.hdu(1).unwrap();
                let data: Vec<u16> = hdu.read_col(&mut f, "example").unwrap();
                assert_eq!(data, column);
            }
        );
    }
}
