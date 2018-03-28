/* Custom derives
*/
extern crate fitsio;
#[macro_use]
extern crate fitsio_derive;

use fitsio::FitsFile;
use fitsio::fitsfile::FitsRow;

#[derive(Default, FitsRow)]
struct Row {
    #[fitsio(colname = "intcol")] intfoo: i32,
    #[fitsio(colname = "strcol")] foobar: String,
}

#[test]
fn test_read_row_as_struct() {
    let filename = "../testdata/full_example.fits";
    let mut f = FitsFile::open(filename).unwrap();
    let tbl_hdu = f.hdu("TESTEXT").unwrap();

    let result: Row = tbl_hdu.row(&mut f, 4).unwrap();
    assert_eq!(result.intfoo, 16);
    assert_eq!(result.foobar, "value4");
}
