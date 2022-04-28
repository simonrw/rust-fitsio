use std::ptr;

use crate::longnam;
use crate::sys::fitsfile;

/** Convenience wrappers around longnam functions
*/

pub(crate) enum Error {
    CloseFile,
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) fn close_file(mut fptr: ptr::NonNull<fitsfile>) -> Result<()> {
    let mut status = 0;
    if unsafe { longnam::fits_close_file(fptr.as_mut() as _, &mut status) } != 0 {
        return Err(Error::CloseFile);
    }
    Ok(())
}
