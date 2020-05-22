use fitsio::FitsFile;
use std::path::Path;

#[test]
fn test_reading_bit_data_type() {
    let source_file = Path::new("tests/fixtures/1065880128_01.mwaf");
    let mut f = FitsFile::open(source_file).unwrap();

    let table_hdu = f.hdu(1).unwrap();
    let flags: Vec<u32> = table_hdu.read_col(&mut f, "FLAGS").unwrap();
    assert_eq!(flags.len(), 1_849_344);
}
