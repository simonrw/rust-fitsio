use fitsio::tables::{ColumnDataType, ColumnDescription};
use fitsio::FitsFile;
use std::path::Path;
use tempfile::Builder;

#[test]
fn test_reading_bit_data_type() {
    let source_file = Path::new("tests/fixtures/1065880128_01.mwaf");
    let mut f = FitsFile::open(source_file).unwrap();

    let table_hdu = f.hdu(1).unwrap();
    let flags: Vec<u32> = table_hdu.read_col(&mut f, "FLAGS").unwrap();
    assert_eq!(flags.len(), 1_849_344);
}

#[test]
fn test_writing_bit_data_type() {
    /* Create a temporary directory to work from */
    let tmp_dir = Builder::new().prefix("fitsio-").tempdir().unwrap();
    let file_path = tmp_dir.path().join("example.fits");

    let data: Vec<u32> = (0..64).collect();
    {
        let mut fitsfile = FitsFile::create(&file_path).open().unwrap();

        let col = ColumnDescription::new("BITMASK")
            .with_type(ColumnDataType::Bit)
            .create()
            .unwrap();
        let columns = &[col];
        let table_hdu = fitsfile.create_table("DATA", columns).unwrap();
        table_hdu
            .write_col(&mut fitsfile, "BITMASK", &data)
            .unwrap();
    }

    let mut f = FitsFile::open(file_path).unwrap();

    let table_hdu = f.hdu("DATA").unwrap();
    let flags: Vec<u32> = table_hdu.read_col(&mut f, "BITMASK").unwrap();
    assert_eq!(flags.len(), 64);
    assert!(data.iter().zip(&flags).all(|(a, b)| a == b));
}
