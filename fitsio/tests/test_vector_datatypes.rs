use fitsio::tables::{ColumnDataType, ColumnDescription};
use fitsio::FitsFile;
use tempfile::Builder;

const NV: usize = 128; // vector size
const NR: usize = 3; // number of rows

macro_rules! make_test {
    ($func:ident, $t:ty, $dt:expr, $val:expr) => {
        #[test]
        fn $func() {
            let tmp_dir = Builder::new().prefix("fitsio-").tempdir().unwrap();
            let file_path = tmp_dir.path().join("example.fits");

            let orig_data: Vec<$t> = vec![$val].repeat(NV * NR);
            {
                let mut fitsfile = FitsFile::create(&file_path).open().unwrap();

                let col = ColumnDescription::new("TEST")
                    .with_type($dt)
                    .that_repeats(NV)
                    .create()
                    .unwrap();
                let columns = &[col];
                let table_hdu = fitsfile.create_table("DATA", columns).unwrap();
                table_hdu
                    .write_col(&mut fitsfile, "TEST", &orig_data)
                    .unwrap();
            }

            let mut f = FitsFile::open(file_path).unwrap();

            let table_hdu = f.hdu("DATA").unwrap();
            let data: Vec<$t> = table_hdu.read_col(&mut f, "TEST").unwrap();
            assert_eq!(orig_data.len(), data.len());
            assert!(orig_data.iter().zip(&data).all(|(a, b)| a == b));
        }
    };
}

make_test!(
    test_read_write_vector_data_type_u8,
    u8,
    ColumnDataType::Byte,
    0xa5
);

make_test!(
    test_read_write_vector_data_type_i8,
    i8,
    ColumnDataType::SignedByte,
    0x5a
);

make_test!(
    test_read_write_vector_data_type_u16,
    u16,
    ColumnDataType::UnsignedShort,
    0xa5a5
);

make_test!(
    test_read_write_vector_data_type_i16,
    i16,
    ColumnDataType::Short,
    0x5a5a
);

make_test!(
    test_read_write_vector_data_type_i32,
    i32,
    ColumnDataType::Int,
    0x5a5a5a5a
);

make_test!(
    test_read_write_vector_data_type_u32,
    u32,
    ColumnDataType::UnsignedLong,
    0xa5a5a5a5
);

// Fails on target armv7-unknown-linux-gnueabihf with link error for ffgcvujj(). Ignore,
// as unsigned types are out-of-spec extensions from CFITSIO anyway.
#[cfg(not(target_pointer_width = "32"))]
make_test!(
    test_read_write_vector_data_type_u64,
    u64,
    ColumnDataType::UnsignedLongLong,
    0xa5a5a5a5_a5a5a5a5
);

make_test!(
    test_read_write_vector_data_type_i64,
    i64,
    ColumnDataType::LongLong,
    0x5a5a5a5a_5a5a5a5a
);

make_test!(
    test_read_write_vector_data_type_f32,
    f32,
    ColumnDataType::Float,
    3.1415926535879323
);

make_test!(
    test_read_write_vector_data_type_f64,
    f64,
    ColumnDataType::Double,
    3.1415926535879323
);
