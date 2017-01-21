use std::ptr;
use std::ffi;
use super::{stringutils, sys, libc};

use super::fitserror::{FitsError, Result, status_to_error};
use super::fitshdu::{FitsHdu, DescribesHdu};
use super::columndescription::ColumnDescription;
use super::types::{FileOpenMode, HduType, ImageType};



/// Description of the current HDU
///
/// If the current HDU is an image, then
/// [`fetch_hdu_info`](struct.FitsFile.html#method.fetch_hdu_info) returns `HduInfo::ImageInfo`.
/// Otherwise the variant is `HduInfo::TableInfo`.
#[derive(Debug)]
pub enum HduInfo {
    ImageInfo {
        dimensions: usize,
        shape: Vec<usize>,
    },
    TableInfo {
        column_descriptions: Vec<ColumnDescription>,
        num_rows: usize,
    },
}

pub struct ImageDescription {
    pub data_type: ImageType,
    pub dimensions: Vec<usize>,
}

/// Main entry point to the FITS file format
///
///
pub struct FitsFile {
    pub fptr: *mut sys::fitsfile,
    pub filename: String,
}

impl Clone for FitsFile {
    fn clone(&self) -> Self {
        FitsFile::open(&self.filename).unwrap()
    }
}

unsafe fn fetch_hdu_info(fptr: *mut sys::fitsfile) -> Result<HduInfo> {
    let mut status = 0;
    let mut hdu_type = 0;

    sys::ffghdt(fptr, &mut hdu_type, &mut status);
    let hdu_type = match hdu_type {
        0 => {
            let mut dimensions = 0;
            sys::ffgidm(fptr, &mut dimensions, &mut status);

            let mut shape = vec![0; dimensions as usize];
            sys::ffgisz(fptr, dimensions, shape.as_mut_ptr(), &mut status);

            HduInfo::ImageInfo {
                dimensions: dimensions as usize,
                shape: shape.iter().map(|v| *v as usize).collect(),
            }
        }
        1 | 2 => {
            let mut num_rows = 0;
            sys::ffgnrw(fptr, &mut num_rows, &mut status);

            let mut num_cols = 0;
            sys::ffgncl(fptr, &mut num_cols, &mut status);
            let mut column_descriptions = Vec::with_capacity(num_cols as usize);

            for i in 0..num_cols {
                let mut name_buffer: Vec<libc::c_char> = vec![0; 71];
                let mut type_buffer: Vec<libc::c_char> = vec![0; 71];
                sys::ffgbcl(fptr,
                            (i + 1) as i32,
                            name_buffer.as_mut_ptr(),
                            ptr::null_mut(),
                            type_buffer.as_mut_ptr(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            &mut status);

                column_descriptions.push(ColumnDescription {
                    name: stringutils::buf_to_string(&name_buffer).unwrap(),
                    data_type: stringutils::buf_to_string(&type_buffer).unwrap(),
                });
            }

            HduInfo::TableInfo {
                column_descriptions: column_descriptions,
                num_rows: num_rows as usize,
            }
        }
        _ => panic!("Invalid hdu type found"),
    };

    fits_try!(status, hdu_type)
}

impl FitsFile {
    /// Open a fits file from disk
    ///
    /// # Examples
    ///
    /// ```
    /// use fitsio::FitsFile;
    ///
    /// let f = FitsFile::open("../testdata/full_example.fits").unwrap();
    ///
    /// // Continue to use `f` afterwards
    /// ```
    pub fn open(filename: &str) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let c_filename = ffi::CString::new(filename).unwrap();

        unsafe {
            sys::ffopen(&mut fptr as *mut *mut sys::fitsfile,
                        c_filename.as_ptr(),
                        FileOpenMode::READONLY as libc::c_int,
                        &mut status);
        }

        fits_try!(status,
                  FitsFile {
                      fptr: fptr,
                      filename: filename.to_string(),
                  })
    }

    /// Create a new fits file on disk
    pub fn create(path: &str) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let c_filename = ffi::CString::new(path).unwrap();

        unsafe {
            sys::ffinit(&mut fptr as *mut *mut sys::fitsfile,
                        c_filename.as_ptr(),
                        &mut status);
        }

        fits_try!(status, {
            let f = FitsFile {
                fptr: fptr,
                filename: path.to_string(),
            };
            try!(f.add_empty_primary());
            f
        })
    }

    fn add_empty_primary(&self) -> Result<()> {
        let mut status = 0;
        unsafe {
            sys::ffphps(self.fptr, 8, 0, ptr::null_mut(), &mut status);
        }

        fits_try!(status, ())
    }

    /// Change the current HDU
    pub fn change_hdu<T: DescribesHdu>(&self, hdu_description: T) -> Result<()> {
        hdu_description.change_hdu(self)
    }

    /// Return a new HDU object
    pub fn hdu<T: DescribesHdu>(&self, hdu_description: T) -> Result<FitsHdu> {
        FitsHdu::new(self, hdu_description)
    }

    pub fn hdu_number(&self) -> usize {
        let mut hdu_num = 0;
        unsafe {
            sys::ffghdn(self.fptr, &mut hdu_num);
        }
        (hdu_num - 1) as usize
    }

    /// Get the current hdu as an HDU object
    pub fn current_hdu(&self) -> Result<FitsHdu> {
        let current_hdu_number = self.hdu_number();
        self.hdu(current_hdu_number)
    }

    /// Get the current hdu info
    pub fn fetch_hdu_info(&self) -> Result<HduInfo> {
        unsafe { fetch_hdu_info(self.fptr) }
    }

    pub fn create_table(&self,
                        extname: String,
                        table_description: &[ColumnDescription])
                        -> Result<()> {
        let tfields = {
            let stringlist = table_description.iter()
                .map(|desc| desc.name.clone())
                .collect();
            stringutils::StringList::from_vec(stringlist)
        };

        let ttype = {
            let stringlist = table_description.iter()
                .map(|desc| desc.data_type.clone())
                .collect();
            stringutils::StringList::from_vec(stringlist)
        };

        let c_extname = ffi::CString::new(extname).unwrap();


        let mut status: libc::c_int = 0;
        unsafe {
            sys::ffcrtb(self.fptr,
                        HduType::BINARY_TBL.into(),
                        0,
                        tfields.len as libc::c_int,
                        tfields.list,
                        ttype.list,
                        ptr::null_mut(),
                        c_extname.into_raw(),
                        &mut status);
        }

        fits_try!(status, ())
    }

    pub fn create_image(&self,
                        extname: String,
                        image_description: &ImageDescription)
                        -> Result<()> {
        let naxis = image_description.dimensions.len();
        let mut status = 0;
        unsafe {
            sys::ffcrim(self.fptr,
                        image_description.data_type.into(),
                        naxis as i32,
                        image_description.dimensions.as_ptr() as *mut i64,
                        &mut status);
        }

        match status {
            0 => {}
            _ => return status_to_error(status),
        }

        // Current HDU should be at the new HDU
        let current_hdu = try!(self.current_hdu());
        try!(current_hdu.write_key("EXTNAME".into(), extname));

        Ok(())
    }
}

impl Drop for FitsFile {
    fn drop(&mut self) {
        let mut status = 0;
        unsafe {
            sys::ffclos(self.fptr, &mut status);
        }
    }
}

#[cfg(test)]
mod test {
    extern crate tempdir;

    use super::*;
    use ::conversions::typechar_to_data_type;
    use ::fitserror::FitsError;
    use ::types::*;

    #[test]
    fn typechar_conversions() {
        let input = vec!["X", "B", "L", "A", "I", "J", "E", "D", "C", "M"];
        let expected = vec![DataType::TBIT,
                            DataType::TBYTE,
                            DataType::TLOGICAL,
                            DataType::TSTRING,
                            DataType::TSHORT,
                            DataType::TLONG,
                            DataType::TFLOAT,
                            DataType::TDOUBLE,
                            DataType::TCOMPLEX,
                            DataType::TDBLCOMPLEX];

        for (i, e) in input.iter().zip(expected) {
            assert_eq!(typechar_to_data_type(i), e);
        }
    }

    #[test]
    fn opening_an_existing_file() {
        match FitsFile::open("../testdata/full_example.fits") {
            Ok(_) => {}
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn creating_a_new_file() {
        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");
        assert!(!filename.exists());

        FitsFile::create(filename.to_str().unwrap())
            .map(|f| {
                assert!(filename.exists());

                // Ensure the empty primary has been written
                let naxis: i64 = f.hdu(0)
                    .unwrap()
                    .read_key("NAXIS")
                    .unwrap();
                assert_eq!(naxis, 0);
            })
            .unwrap();
    }

    #[test]
    fn fetching_a_hdu() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        for i in 0..2 {
            f.change_hdu(i).unwrap();
            assert_eq!(f.hdu_number(), i);
        }

        match f.change_hdu(2) {
            Err(e) => assert_eq!(e.status, 107),
            _ => panic!("Error checking for failure"),
        }

        f.change_hdu("TESTEXT").unwrap();
        assert_eq!(f.hdu_number(), 1);
    }

    #[test]
    fn fetching_hdu_info() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::ImageInfo { dimensions, shape }) => {
                assert_eq!(dimensions, 2);
                assert_eq!(shape, vec![100, 100]);
            }
            Err(e) => panic!("Error fetching hdu info {:?}", e),
            _ => panic!("Unknown error"),
        }

        f.change_hdu(1).unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { column_descriptions, num_rows }) => {
                assert_eq!(num_rows, 50);
                assert_eq!(column_descriptions.iter()
                               .map(|desc| desc.name.clone())
                               .collect::<Vec<String>>(),
                           vec!["intcol".to_string(),
                                "floatcol".to_string(),
                                "doublecol".to_string()]);
                assert_eq!(column_descriptions.iter()
                               .map(|ref desc| typechar_to_data_type(desc.data_type.clone()))
                               .collect::<Vec<DataType>>(),
                           vec![DataType::TLONG, DataType::TFLOAT, DataType::TDOUBLE]);
            }
            Err(e) => panic!("Error fetching hdu info {:?}", e),
            _ => panic!("Unknown error"),
        }
    }

    #[test]
    fn cloning() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let f2 = f.clone();

        assert!(f.fptr != f2.fptr);

        f.change_hdu(1).unwrap();
        assert!(f.hdu_number() != f2.hdu_number());
    }

    #[test]
    fn test_fits_try() {
        use stringutils;

        let status = 0;
        assert_eq!(fits_try!(status, 10), Ok(10));

        let status = 105;
        assert_eq!(fits_try!(status, 10),
                   Err(FitsError {
                       status: status,
                       message: stringutils::status_to_string(status).unwrap(),
                   }));
    }

    #[test]
    fn adding_new_table() {
        use super::super::columndescription::ColumnDescription;

        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");

        {
            let f = FitsFile::create(filename.to_str().unwrap()).unwrap();
            let table_description = vec![ColumnDescription {
                                             name: "bar".to_string(),
                                             data_type: "1J".to_string(),
                                         }];
            f.create_table("foo".to_string(), &table_description).unwrap();
        }

        FitsFile::open(filename.to_str().unwrap())
            .map(|f| {
                f.change_hdu("foo").unwrap();
                match f.fetch_hdu_info() {
                    Ok(HduInfo::TableInfo { column_descriptions, .. }) => {
                        let column_names = column_descriptions.iter()
                            .map(|desc| desc.name.clone())
                            .collect::<Vec<String>>();
                        let column_types = column_descriptions.iter()
                            .map(|desc| typechar_to_data_type(desc.data_type.clone()))
                            .collect::<Vec<DataType>>();
                        assert_eq!(column_names, vec!["bar".to_string()]);
                        assert_eq!(column_types, vec![DataType::TLONG]);
                    }
                    thing => panic!("{:?}", thing),
                }
            })
            .unwrap();
    }

    #[test]
    fn adding_new_image() {
        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");

        {
            let f = FitsFile::create(filename.to_str().unwrap()).unwrap();
            let image_description = ImageDescription {
                data_type: ImageType::LONG_IMG,
                dimensions: vec![100, 20],
            };
            f.create_image("foo".to_string(), &image_description).unwrap();
        }

        FitsFile::open(filename.to_str().unwrap())
            .map(|f| {
                f.change_hdu("foo").unwrap();
                match f.fetch_hdu_info() {
                    Ok(HduInfo::ImageInfo { shape, .. }) => {
                        assert_eq!(shape, vec![100, 20]);
                    }
                    thing => panic!("{:?}", thing),
                }
            })
            .unwrap();

    }

    #[test]
    fn fetching_hdu_object_hdu_info() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let testext = f.hdu("TESTEXT").unwrap();
        match testext.info {
            HduInfo::TableInfo { num_rows, .. } => {
                assert_eq!(num_rows, 50);
            }
            _ => panic!("Incorrect HDU type found"),
        }
    }

    #[test]
    fn fetch_current_hdu() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        f.change_hdu("TESTEXT").unwrap();
        let hdu = f.current_hdu().unwrap();

        assert_eq!(hdu.read_key::<String>("EXTNAME").unwrap(),
                   "TESTEXT".to_string());
    }
}
