extern crate fitsio;

use fitsio::FitsFile;

#[test]
fn test_square_array() {
    /* This file contains a square array of 5x5 pixels:
     *
     * [ 0,  1,  2,  3,  4,
     *   5,  6,  7,  8,  9,
     *  10, 11, 12, 13, 14,
     *  15, 16, 17, 18, 19,
     *  20, 21, 22, 23, 24]
     *
     *  We check that the x-values (1..3) (exclusive of the top end), and y-values (2..4)
     *  (exclusive of the top end) return what we expect.
     */
    let filename = "../testdata/square_array.fits";
    let mut f = FitsFile::open(filename).unwrap();
    let phdu = f.primary_hdu().unwrap();

    let ranges = vec![&(1..3), &(2..4)];
    let data: Vec<u32> = phdu.read_region(&mut f, &ranges).unwrap();
    assert_eq!(data, vec![11, 12, 16, 17]);
}
