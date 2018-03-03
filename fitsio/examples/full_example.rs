extern crate fitsio;
extern crate tempdir;

/* This example docuents the following things:
 *
 * TODO: make sure this list is up to date with the example below
 *
 * creating a new file
 * writing header keys
 * writing an image into the primary hdu
 * creating a table
 * closing the file
 * re-opening it and reading the data out
 */

use std::error::Error;
use tempdir::TempDir;
use fitsio::FitsFile;
// TODO: move Image related structs/types into a more sensible place
use fitsio::fitsfile::ImageDescription;
// TODO: remove the import of imagetype from main package
use fitsio::types::ImageType;
// TODO: tidy up column description import
use fitsio::columndescription::{ColumnDataType, ColumnDescription};

fn run() -> Result<(), Box<Error>> {
    /* Create a temporary directory to work from */
    let tmp_dir = TempDir::new("fitsio")?;
    let file_path = tmp_dir.path().join("example.fits");

    // creating a new file with 512 rows and 1024 columns
    let primary_hdu_description = ImageDescription {
        // TODO: use a rust enum not C enum
        data_type: ImageType::DOUBLE_IMG,
        dimensions: &[512, 1024],
    };
    // TODO: handle this error properly and/or handle this file path properly

    {
        let mut fitsfile = FitsFile::create(file_path.to_str().unwrap())
            .with_custom_primary(&primary_hdu_description)
            .open()?;

        /* We will now add some dummy header keys. We add:
         * - the name of the project (String)
         * - the exposure time (f32)
         * - the image id (i64)
         */

        /* First we get the primary HDU */
        // TODO: add primary hdu method => let hdu = fitsfile.primary_hdu()?;
        let hdu = fitsfile.hdu(0)?;
        /* Now we add the header keys */
        // TODO: implement WritesKey for &str and String (AsRef<str>?)
        hdu.write_key(
            &mut fitsfile,
            "PROJECT",
            "My First Astronomy Project".to_string(),
        )?;

        /* Now the exposure time */
        hdu.write_key(&mut fitsfile, "EXPTIME", 15.2f32)?;

        /* And finally the image id */
        // TODO: implement write_key for u64?
        hdu.write_key(&mut fitsfile, "IMAGE_ID", 20180101010005i64)?;

        /* Now we write some dummy data to the primary HDU. We write the full image in this case.
         * */

        let dummy_data: Vec<f32> = (0..(1024 * 512))
            .map(|val| (val as f32) * 12.5 + 102.5)
            .collect();

        // TODO: remove documentation from trait methods altogether
        // TODO: document `write_image` in main lib documentation
        hdu.write_image(&mut fitsfile, &dummy_data)?;

        // TODO: Adding a new image HDU?

        /* Now we add a new table HDU called "DATA"
         *
         * We add three columns:
         *
         * 1. Object id (i32)
         * 2. Object name (String, up to 10 characters)
         * 3. Object magnitude (f32)
         * */
        let col1 = ColumnDescription::new("OBJ_ID")
            .with_type(ColumnDataType::Int)
            .create()?;
        let col2 = ColumnDescription::new("NAME")
            .with_type(ColumnDataType::String)
            .that_repeats(10)
            .create()?;
        let col3 = ColumnDescription::new("MAG")
            .with_type(ColumnDataType::Float)
            .create()?;
        let columns = &[col1, col2, col3];
        // TODO: create table should take &str or String (AsRef<str>?)
        let table_hdu = fitsfile.create_table("DATA".to_string(), columns)?;

        /* Add some data to the columns */
        // TODO: add row methods
        let n_rows = 10;
        let obj_id_data: Vec<i32> = (0..n_rows).collect();
        let name_data: Vec<String> = (0..n_rows).map(|idx| format!("N{}", idx)).collect();
        let mag_data: Vec<f32> = (0..n_rows).map(|idx| -0.2 * idx as f32).collect();

        table_hdu.write_col(&mut fitsfile, "OBJ_ID", &obj_id_data)?;
        table_hdu.write_col(&mut fitsfile, "NAME", &name_data)?;
        table_hdu.write_col(&mut fitsfile, "MAG", &mag_data)?;

        // TODO: pretty print table structure?

        /* `fitsfile` is dropped here at the end of the scope, which closes fhe file on disk */
    }

    /* Here we re-open the file. We want to adjust things, so we use the `edit` method */
    let mut fitsfile = FitsFile::edit(file_path.to_str().unwrap())?;

    /* Get the primary HDU and read a section of the image data */

    let phdu = fitsfile.hdu(0)?;
    // TODO: let phdu = fitsfile.primary_hdu()?;

    // TODO: if we drop `fitsfile`, can we still use a FitsHdu object? What is the error that
    // occurs?!

    /* Let's say we have a region around a star that we want to extract. The star is at (25, 25,
     * 1-indexed) and we want to extract a 5x5 box around it. This means we want to read rows 19 to
     * 19, and columns 19 to 29 (0-indexed). The range arguments are exclusive of the upper bound,
     * so we must use 19..29 for each axis.
     *
     * Note: this is not performant, and if this is the desired use of `fitsio`, I would not
     * suggest this approach. */
    let image_data: Vec<f32> = phdu.read_region(&mut fitsfile, &[&(19..29), &(19..29)])?;
    assert_eq!(image_data.len(), 100);

    /* We can now get the column data. Let's assume we want all of the magnitudes for objects near
     * this one (by index). This object is at index 4 (0-indexed) and we want to read one value
     * before and after it. */
    let table_hdu = fitsfile.hdu("DATA")?;
    let magnitudes: Vec<f32> = table_hdu.read_col_range(&mut fitsfile, "MAG", &(3..6))?;
    assert_eq!(magnitudes.len(), 3);

    /* The file is closed when it is dropped here */

    Ok(())
}

fn main() {
    run().unwrap();
}
