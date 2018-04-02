extern crate fitsio;
extern crate ndarray;

use std::error::Error;
use fitsio::FitsFile;
use ndarray::ArrayD;

fn try_main() -> Result<(), Box<Error>> {
let filename = "../testdata/full_example.fits";
let mut fptr = FitsFile::open(filename)?;
let phdu = fptr.primary_hdu()?;
let image: ArrayD<f32> = phdu.read_image(&mut fptr)?;
let new_image = (image * 4.0) / 5.2;
assert_eq!(new_image.ndim(), 2);

Ok(())
}

fn main() { try_main().unwrap() }
