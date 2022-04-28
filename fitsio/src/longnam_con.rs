use std::ptr;

use libc::c_int;

use crate::images::ImageType;
use crate::longnam;
use crate::stringutils::error_to_string;
use crate::sys::fitsfile;

/** Convenience wrappers around longnam functions
*/

type FitsFile = ptr::NonNull<fitsfile>;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<c_int> for Error {
    fn from(status: c_int) -> Self {
        let message = error_to_string(status).expect("unhandlable error");
        Self { message }
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) fn close_file(mut fptr: FitsFile) -> Result<()> {
    let mut status = 0;
    if unsafe { longnam::fits_close_file(fptr.as_mut() as _, &mut status) } != 0 {
        return Err(status.into());
    }
    Ok(())
}

pub(crate) fn copy_hdu(mut src: FitsFile, mut dst: FitsFile) -> Result<()> {
    let mut status = 0;
    if unsafe {
        longnam::fits_copy_hdu(
            src.as_mut() as *mut _,
            dst.as_mut() as *mut _,
            0,
            &mut status,
        )
    } != 0
    {
        return Err(status.into());
    }
    Ok(())
}

pub(crate) fn create_image(
    mut src: FitsFile,
    image_type: ImageType,
    shape: &[usize],
) -> Result<()> {
    let mut status = 0;

    // fitsio dimensions are in reverse order to c dimensions, so copy the dimensions to reverse
    // them.
    let mut dimensions: Vec<_> = shape.to_vec();
    dimensions.reverse();

    if unsafe {
        longnam::fits_create_img(
            src.as_mut() as *mut _,
            image_type.into(),
            shape.len() as i32,
            dimensions.as_ptr() as *mut _,
            &mut status,
        )
    } != 0
    {
        return Err(status.into());
    }

    Ok(())
}
