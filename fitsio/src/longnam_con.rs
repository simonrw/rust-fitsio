use std::{ffi, ptr};

use libc::c_int;

use crate::images::ImageType;
use crate::longnam;
use crate::stringutils::error_to_string;
use crate::sys::fitsfile;
use crate::tables::{ConcreteColumnDescription};

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

pub(crate) fn create_table(
    mut src: FitsFile,
    name: impl AsRef<str>,
    description: &[ConcreteColumnDescription],
) -> Result<()> {
    let tfields = {
        let stringlist: Vec<_> = description.iter().map(|desc| desc.name.clone()).collect();
        crate::stringutils::StringList::from_slice(stringlist.as_slice()).unwrap()
    };

    let ttype = {
        let stringlist: Vec<_> = description
            .iter()
            .map(|desc| String::from(desc.clone().data_type))
            .collect();
        crate::stringutils::StringList::from_slice(stringlist.as_slice()).unwrap()
    };

    let c_extname = ffi::CString::new(name.as_ref()).expect("invalid hdu name; non utf-8");

    let mut status: libc::c_int = 0;
    if unsafe {
        longnam::fits_create_tbl(
            src.as_mut() as *mut _,
            2,
            0,
            tfields.len as libc::c_int,
            tfields.as_ptr(),
            ttype.as_ptr(),
            ptr::null_mut(),
            c_extname.as_ptr(),
            &mut status,
        )
    } != 0
    {
        return Err(status.into());
    }

    Ok(())
}

pub(crate) fn delete_column(mut src: FitsFile, column: usize) -> Result<()> {
    let mut status = 0;

    if unsafe { longnam::fits_delete_col(src.as_mut() as *mut _, (column + 1) as _, &mut status) }
        != 0
    {
        return Err(status.into());
    }

    Ok(())
}
