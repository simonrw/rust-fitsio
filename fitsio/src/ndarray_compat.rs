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
    fn read_section(
        _fits_file: &mut FitsFile,
        _hdu: &FitsHdu,
        _range: Range<usize>,
    ) -> Result<Self> {
        unimplemented!()
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

    #[test]
    fn test_read_image() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: ArrayD<u32> = hdu.read_image(&mut f).unwrap();
        let dim = data.dim();
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
        assert_eq!(dim[0], 2);
        assert_eq!(dim[1], 100);
        assert_eq!(data[[1, 52]], 184);
    }

    #[test]
    fn test_read_row() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: ArrayD<u32> = hdu.read_row(&mut f, 49).unwrap();
        assert_eq!(data[20], 156);
    }

    #[test]
    fn test_read_region() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: ArrayD<u32> = hdu.read_region(&mut f, &[&(70..80), &(20..50)]).unwrap();
        let dim = data.dim();
        assert_eq!(dim[0], 10);
        assert_eq!(dim[1], 30);
        assert_eq!(data[[5, 10]], 160);
    }
}
