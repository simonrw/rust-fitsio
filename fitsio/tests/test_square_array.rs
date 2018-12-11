extern crate fitsio;

use fitsio::FitsFile;

#[test]
fn test_square_array() {
    let filename = "../testdata/square_array.fits";
    let mut f = FitsFile::open(filename).unwrap();
    let phdu = f.primary_hdu().unwrap();

    let ranges = vec![&(1..3), &(2..4)];
    let data: Vec<u32> = phdu.read_region(&mut f, &ranges).unwrap();
    assert_eq!(data, vec![11, 12, 16, 17]);
}
