use super::fitsfile::{FitsFile, HduInfo, DescribesHdu};
use super::fitserror::Result;

pub struct FitsHdu<'open> {
    fits_file: &'open FitsFile,
    pub hdu_info: HduInfo,
}

impl<'open> FitsHdu<'open> {
    pub fn new<T: DescribesHdu>(fits_file: &'open FitsFile, hdu_description: T) -> Result<Self> {
        try!(fits_file.change_hdu(hdu_description));
        match fits_file.fetch_hdu_info() {
            Ok(hdu_info) => Ok(FitsHdu {
                fits_file: fits_file,
                hdu_info: hdu_info,
            }),
            Err(e) => Err(e),
        }
    }

    fn change_hdu<T: DescribesHdu>(&self, hdu_description: T) -> Result<()> {
        hdu_description.change_hdu(self.fits_file)
    }
}


#[cfg(test)]
mod test {
    use super::FitsHdu;
    use super::super::fitsfile::{FitsFile, HduInfo};

    #[test]
    fn test_manually_creating_a_fits_hdu() { let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = FitsHdu::new(&f, "TESTEXT").unwrap();
        match hdu.hdu_info {
            HduInfo::TableInfo { num_rows, .. } => {
                assert_eq!(num_rows, 50);
            },
            _ => panic!("Incorrect HDU type found"),
        }
    }
}
