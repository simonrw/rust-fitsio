use fitsio::FitsFile;

#[test]
fn test_read_into_buffer() {
    let filename = "../testdata/full_example.fits";
    let mut f = FitsFile::open(filename).unwrap();
    let phdu = f.primary_hdu().unwrap();

    let mut buf = vec![0u32; 10];
    phdu.read_section_into(&mut f, 0, &mut buf).unwrap();

    assert_eq!(buf, vec![108, 176, 166, 177, 104, 110, 100, 193, 150, 197]);
}
