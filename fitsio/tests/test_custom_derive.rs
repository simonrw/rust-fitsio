/* Custom derives
*/
extern crate fitsio;
#[macro_use]
extern crate fitsio_derive;

use fitsio::FitsFile;
use fitsio::fitsfile::FitsRow;

#[derive(Default, FitsRow)]
struct Row {
    intcol: i32,
    floatcol: f32,
    strcol: String,
}

#[test]
fn read_row_as_struct() {
    let filename = "../testdata/full_example.fits[TESTEXT]";
    let mut f = FitsFile::open(filename).unwrap();
    let tbl_hdu = f.hdu("TESTEXT").unwrap();

    // let result: Row = tbl_hdu.read_row(&mut f, 4).unwrap();
    let result: Row = tbl_hdu.read_row_into_struct(&mut f, 4).unwrap();
    assert_eq!(result.intcol, 16);
}
