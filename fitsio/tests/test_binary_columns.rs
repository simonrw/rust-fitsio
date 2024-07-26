use fitsio::FitsFile;

// read from astropy
static EXPECTED: &[bool] = &[
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, false, false, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, false, false, true, true,
    true, true, true, true, false, false, false, false, false, false, false, false, true, true,
    true, true, true, true, true, true, true, true, false, false, false, false, false, false, true,
    true, false, false, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, false,
    false, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, false, false, false, false, false,
    false, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, false, false, true, true, false, false, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, false, false,
    true, true, false, false, false, false, true, true, true, true, true, true,
];

#[test]
fn reading_binary_columns() -> Result<(), Box<dyn std::error::Error>> {
    let mut fitsfile = FitsFile::open("../testdata/binary_columns.fits")?;
    fitsfile.pretty_print().expect("printing fits file");
    let ant_hdu = fitsfile.hdu(1)?;
    let col = ant_hdu.read_col::<bool>(&mut fitsfile, "Whitening_Filter")?;
    assert_eq!(col, EXPECTED);
    Ok(())
}
