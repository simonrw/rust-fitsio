extern crate tempdir;

/// Function to allow access to a temporary file
pub(crate) fn with_temp_file<F>(callback: F)
where
    F: for<'a> Fn(&'a str),
{
    let tdir = tempdir::TempDir::new("fitsio-").unwrap();
    let tdir_path = tdir.path();
    let filename = tdir_path.join("test.fits");

    let filename_str = filename.to_str().expect("cannot create string filename");
    callback(filename_str);
}

/// Function to create a temporary file and copy the example file
pub(crate) fn duplicate_test_file<F>(callback: F)
where
    F: for<'a> Fn(&'a str),
{
    use std::fs;
    with_temp_file(|filename| {
        fs::copy("../testdata/full_example.fits", &filename).expect("Could not copy test file");
        callback(filename);
    });
}
