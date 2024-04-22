/* This example docuents the following things:
 *
 * creating a new file
 * writing header keys
 * writing an image into the primary hdu
 * creating a table
 * adding data to a table
 * closing the file
 * re-opening it
 * reading an image region
 * reading some rows from a table column
 */

use std::error::Error;

use fitsio::images::{ImageDescription, ImageType};
use fitsio::tables::{ColumnDataType, ColumnDescription, FitsRow};
use fitsio::FitsFile;
use fitsio_derive::FitsRow;
use tempfile::Builder;

fn run() -> Result<(), Box<dyn Error>> {
    /* Create a temporary directory to work from */
    let tmp_dir = Builder::new().prefix("fitsio-").tempdir()?;
    let file_path = tmp_dir.path().join("example.fits");

    // creating a new file with 512 rows and 1024 columns
    let primary_hdu_description = ImageDescription {
        data_type: ImageType::Double,
        dimensions: &[512, 1024],
    };

    {
        // .overwrite ensures that if the file already exists, the existing file is removed first.
        let mut fitsfile = FitsFile::create(&file_path)
            .with_custom_primary(&primary_hdu_description)
            .overwrite()
            .open()?;

        /* We will now add some dummy header keys. We add:
         * - the name of the project (String)
         * - the exposure time (f32)
         * - the image id (i64)
         */

        /* First we get the primary HDU */
        let hdu = fitsfile.primary_hdu()?;

        /* Now we add the header keys */
        hdu.write_key(&mut fitsfile, "PROJECT", "My First Astronomy Project")?;

        /* Now the exposure time, with a comment */
        hdu.write_key_cmt(&mut fitsfile, "EXPTIME", 15.2f32, "Exposure time [s]")?;

        /* And finally the image id */
        hdu.write_key(&mut fitsfile, "IMAGE_ID", 20180101010005i64)?;

        /* Now we write some dummy data to the primary HDU. We write the full image in this case.
         * */

        let dummy_data: Vec<f32> = (0..(1024 * 512))
            .map(|val| (val as f32) * 12.5 + 102.5)
            .collect();

        hdu.write_image(&mut fitsfile, &dummy_data)?;

        /* We can create a new image with the following */
        let image_description = ImageDescription {
            data_type: ImageType::Long,
            dimensions: &[256, 256],
        };
        fitsfile.create_image("IMG", &image_description)?;

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
        let table_hdu = fitsfile.create_table("DATA", columns)?;

        /* Add some data to the columns */
        let n_rows = 10;
        let obj_id_data: Vec<i32> = (0..n_rows).collect();
        let name_data: Vec<String> = (0..n_rows).map(|idx| format!("N{}", idx)).collect();
        let mag_data: Vec<f32> = (0..n_rows).map(|idx| -0.2 * idx as f32).collect();

        table_hdu.write_col(&mut fitsfile, "OBJ_ID", &obj_id_data)?;
        table_hdu.write_col(&mut fitsfile, "NAME", &name_data)?;
        table_hdu.write_col(&mut fitsfile, "MAG", &mag_data)?;

        /* `fitsfile` is dropped here at the end of the scope, which closes fhe file on disk */
    }

    /* Here we re-open the file. We want to adjust things, so we use the `edit` method */
    let mut fitsfile = FitsFile::edit(&file_path)?;

    /* Print the fits file contents to stdout */
    fitsfile.pretty_print().expect("printing fits file");

    /* Get the primary HDU and read a section of the image data */
    let phdu = fitsfile.primary_hdu()?;

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

    /* Read a single row from the file. Columns can be renamed to allow for more rustic attribute
     * names */
    #[derive(Default, FitsRow)]
    struct Row {
        #[fitsio(colname = "OBJ_ID")]
        obj_id: i32,
        #[fitsio(colname = "NAME")]
        name: String,
        #[fitsio(colname = "MAG")]
        mag: f32,
    }
    let row: Row = table_hdu.row(&mut fitsfile, 4)?;
    assert_eq!(row.obj_id, 4);
    assert_eq!(row.name, "N4");
    assert!(row.mag > -1.0 && row.mag < -0.5);

    /* The file is closed when it is dropped here */

    Ok(())
}

fn main() {
    run().unwrap();
}
