extern crate fitsio;

use std::error::Error;
use fitsio::FitsFile;

fn try_main() -> Result<(), Box<Error>> {
    let filename = "../../testdata/full_example.fits";
    let _fptr = FitsFile::open(filename)?;
    Ok(())
}

fn main() { try_main().unwrap() }
