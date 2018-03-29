//! `ndarray` compability

use std::ops::Range;
use super::{FitsFile, FitsHdu};
use super::fitsfile::ReadImage;
use super::errors::Result;
use super::types::HduInfo;
use ndarray::{Array, ArrayD};

impl<T> ReadImage for ArrayD<T>
where
    T: Clone,
    Vec<T>: ReadImage,
{
    fn read_section(fits_file: &mut FitsFile, hdu: &FitsHdu, range: Range<usize>) -> Result<Self> {
        match hdu.info {
            HduInfo::ImageInfo { ref shape, .. } => {
                if shape.len() != 2 {
                    return Err("Only 2D images supported for now".into());
                }

                let width = shape[1];

                if range.start % width != 0 {
                    return Err("range must start on row boundary".into());
                }
                let start_pixel = range.start / width;

                let n_pixels_requested = range.end - range.start;
                if n_pixels_requested % width != 0 {
                    return Err(
                        "must request number of pixels exactly divisible by image width".into(),
                    );
                }

                let n_rows = n_pixels_requested / width;
                ReadImage::read_rows(fits_file, hdu, start_pixel, n_rows)
            }
            HduInfo::TableInfo { .. } => {
                return Err("Cannot read image data from a FITS table".into())
            }
            _ => unreachable!(),
        }
    }

    fn read_rows(
        fits_file: &mut FitsFile,
        hdu: &FitsHdu,
        start_row: usize,
        num_rows: usize,
    ) -> Result<Self> {
        let data: Vec<T> = ReadImage::read_rows(fits_file, hdu, start_row, num_rows)?;
        let arr = Array::from_vec(data);
        let row_length = arr.len() / num_rows;
        Ok(arr.into_shape(vec![num_rows, row_length]).unwrap())
    }

    fn read_row(fits_file: &mut FitsFile, hdu: &FitsHdu, row: usize) -> Result<Self> {
        let data: Vec<T> = ReadImage::read_row(fits_file, hdu, row)?;
        let shape = vec![data.len()];
        Ok(Array::from_shape_vec(shape, data).unwrap())
    }

    fn read_region(
        fits_file: &mut FitsFile,
        hdu: &FitsHdu,
        ranges: &[&Range<usize>],
    ) -> Result<Self> {
        let data: Vec<T> = ReadImage::read_region(fits_file, hdu, ranges)?;
        let shape: Vec<usize> = (0..ranges.len())
            .map(|i| ranges[i].end - ranges[i].start)
            .collect();
        let arr = Array::from_shape_vec(shape, data).unwrap();
        Ok(arr)
    }

    fn read_image(fits_file: &mut FitsFile, hdu: &FitsHdu) -> Result<Self> {
        match hdu.info {
            HduInfo::ImageInfo { ref shape, .. } => {
                let data: Vec<T> = ReadImage::read_image(fits_file, hdu)?;
                let shape: Vec<usize> = (0..2).map(|i| shape[i]).collect();
                let arr = Array::from_shape_vec(shape, data).unwrap();
                Ok(arr)
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::errors::Error;

    #[test]
    fn test_read_image() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: ArrayD<u32> = hdu.read_image(&mut f).unwrap();
        let dim = data.dim();
        assert_eq!(data.ndim(), 2);
        assert_eq!(dim[0], 100);
        assert_eq!(dim[1], 100);
        assert_eq!(data[[20, 5]], 152);
    }

    #[test]
    fn test_read_rows() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: ArrayD<u32> = hdu.read_rows(&mut f, 0, 2).unwrap();
        let dim = data.dim();
        assert_eq!(data.ndim(), 2);
        assert_eq!(dim[0], 2);
        assert_eq!(dim[1], 100);
        assert_eq!(data[[1, 52]], 184);
    }

    #[test]
    fn test_read_row() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: ArrayD<u32> = hdu.read_row(&mut f, 49).unwrap();
        assert_eq!(data.ndim(), 1);
        assert_eq!(data[20], 156);
    }

    #[test]
    fn test_read_region() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: ArrayD<u32> = hdu.read_region(&mut f, &[&(70..80), &(20..50)]).unwrap();
        let dim = data.dim();
        assert_eq!(data.ndim(), 2);
        assert_eq!(dim[0], 10);
        assert_eq!(dim[1], 30);
        assert_eq!(data[[5, 10]], 160);
    }

    #[test]
    fn test_read_section() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        match hdu.read_section::<ArrayD<u32>>(&mut f, 0, 250) {
            Err(Error::Message(msg)) => {
                assert_eq!(
                    msg,
                    "must request number of pixels exactly divisible by image width".to_string()
                );
            }
            _ => panic!("invalid result"),
        }

        let data: ArrayD<u32> = hdu.read_section(&mut f, 0, 200).unwrap();
        let dim = data.dim();
        assert_eq!(data.ndim(), 2);
        assert_eq!(dim[0], 2);
        assert_eq!(dim[1], 100);
        assert_eq!(data[[0, 10]], 160);
    }
}
