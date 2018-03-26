//! `ndarray` compability

use std::ops::Range;
use super::{FitsFile, FitsHdu};
use super::fitsfile::ReadImage;
use super::errors::Result;
use super::types::HduInfo;
use ndarray::{Array, Array1, Array2};

impl<T> ReadImage for Array2<T>
where
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
        if num_rows == 0 || num_rows == 1 {
            unimplemented!("not implemented for ndarray::Array2")
        }
        let data: Vec<T> = ReadImage::read_rows(fits_file, hdu, start_row, num_rows)?;
        let arr = Array::from_vec(data);
        let row_length = arr.len() / num_rows;
        Ok(arr.into_shape((num_rows, row_length)).unwrap())
    }

    fn read_row(fits_file: &mut FitsFile, hdu: &FitsHdu, row: usize) -> Result<Self> {
        Self::read_rows(fits_file, hdu, row, 1)
    }

    fn read_region(
        fits_file: &mut FitsFile,
        hdu: &FitsHdu,
        ranges: &[&Range<usize>],
    ) -> Result<Self> {
        if ranges.len() != 2 {
            unimplemented!("only 2d regions can be read into an `ndarray::Array`");
        }

        let data: Vec<T> = ReadImage::read_region(fits_file, hdu, ranges)?;
        let arr = Array::from_vec(data);
        let shape = (
            ranges[0].end - ranges[0].start,
            ranges[1].end - ranges[1].start,
        );
        println!("{:?}", ranges);
        println!("{:?} {}", shape, arr.len());
        Ok(arr.into_shape(shape).unwrap())
    }

    fn read_image(fits_file: &mut FitsFile, hdu: &FitsHdu) -> Result<Self> {
        let data: Vec<T> = ReadImage::read_image(fits_file, hdu)?;
        let arr = Array::from_vec(data);
        match hdu.info {
            HduInfo::ImageInfo { ref shape, .. } => {
                Ok(arr.into_shape((shape[0], shape[1])).unwrap())
            }
            _ => unreachable!(),
        }
    }
}

impl<T> ReadImage for Array1<T>
where
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
        _fits_file: &mut FitsFile,
        _hdu: &FitsHdu,
        _start_row: usize,
        _num_rows: usize,
    ) -> Result<Self> {
        unimplemented!()
    }

    fn read_row(fits_file: &mut FitsFile, hdu: &FitsHdu, row: usize) -> Result<Self> {
        let data: Vec<T> = ReadImage::read_row(fits_file, hdu, row)?;
        Ok(Array::from_vec(data))
    }

    fn read_region(
        _fits_file: &mut FitsFile,
        _hdu: &FitsHdu,
        _ranges: &[&Range<usize>],
    ) -> Result<Self> {
        unimplemented!()
    }

    fn read_image(_fits_file: &mut FitsFile, _hdu: &FitsHdu) -> Result<Self> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_image() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: Array2<u32> = hdu.read_image(&mut f).unwrap();
        assert_eq!(data.dim(), (100, 100));
        assert_eq!(data[[20, 5]], 152);
    }

    #[test]
    fn test_read_rows() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: Array2<u32> = hdu.read_rows(&mut f, 0, 2).unwrap();
        assert_eq!(data.dim(), (2, 100));
        assert_eq!(data[[1, 52]], 184);
    }

    #[test]
    fn test_read_row() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: Array1<u32> = hdu.read_row(&mut f, 49).unwrap();
        assert_eq!(data[20], 156);
    }

    #[test]
    fn test_read_region() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let data: Array2<u32> = hdu.read_region(&mut f, &[&(70..80), &(20..50)]).unwrap();
        assert_eq!(data.dim(), (10, 30));
        assert_eq!(data[[5, 10]], 160);
    }
}
