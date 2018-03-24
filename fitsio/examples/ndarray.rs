extern crate fitsio;
extern crate ndarray;

use std::ops::{Deref, Range};
use fitsio::fitsfile::ReadImage;
use fitsio::types::HduInfo;
use fitsio::FitsFile;
use fitsio::errors::Result;

// Newtype so we can implement ReadImage
#[derive(Debug, Default)]
pub struct NdArray(ndarray::Array2<u32>);

impl Deref for NdArray {
    type Target = ndarray::Array2<u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ReadImage for NdArray {
    fn read_section(_fits_file: &mut FitsFile, _range: Range<usize>) -> Result<Self> {
        unimplemented!()
    }

    fn read_rows(_fits_file: &mut FitsFile, _start_row: usize, _num_rows: usize) -> Result<Self> {
        unimplemented!()
    }

    fn read_row(_fits_file: &mut FitsFile, _row: usize) -> Result<Self> {
        unimplemented!()
    }

    fn read_region(_fits_file: &mut FitsFile, _ranges: &[&Range<usize>]) -> Result<Self> {
        unimplemented!()
    }

    fn read_image(fits_file: &mut FitsFile) -> Result<Self> {
        let data: Vec<u32> = ReadImage::read_image(fits_file)?;
        let arr = ndarray::Array::from_vec(data);
        match fits_file.fetch_hdu_info()? {
            HduInfo::ImageInfo { shape, .. } => {
                Ok(NdArray(arr.into_shape((shape[0], shape[1])).unwrap()))
            }
            _ => unreachable!(),
        }
    }
}

fn run() -> std::result::Result<(), Box<std::error::Error>> {
    let mut f = FitsFile::open("../testdata/full_example.fits")?;
    let hdu = f.primary_hdu()?;
    let data: NdArray = hdu.read_image(&mut f)?;
    assert_eq!(data.dim(), (100, 100));

    Ok(())
}

fn main() {
    run().unwrap();
}
