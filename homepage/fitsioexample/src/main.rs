extern crate fitsio;

use fitsio::FitsFile;
use std::error::Error;

fn try_main() -> Result<(), Box<dyn Error>> {
    let filename = "../../testdata/full_example.fits";
    let _fptr = FitsFile::open(filename)?;
    Ok(())
}

fn main() {
    try_main().unwrap()
}
