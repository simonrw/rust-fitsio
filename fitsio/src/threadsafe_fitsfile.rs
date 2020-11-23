/*! Thread-safe FitsFile struct */

use crate::errors::Result;
use crate::fitsfile::FitsFile;
use std::sync::{Arc, Mutex, MutexGuard};

/** Thread-safe [`FitsFile`][fits-file] representation.

This struct wraps an `Arc<Mutex<FitsFile>>` and implements `Send`.

To get a [`ThreadsafeFitsfile`][threadsafe-fitsfile] from a [`FitsFile`][fits-file], call the
[`threadsafe`][fits-file-threadsafe] method.

[fits-file]: ../fitsfile/struct.FitsFile.html
[threadsafe-fitsfile]: struct.ThreadsafeFitsFile.html
[fits-file-threadsafe]: ../fitsfile/struct.FitsFile.html#method.threadsafe
*/
#[derive(Clone)]
pub struct ThreadsafeFitsFile(Arc<Mutex<FitsFile>>);

/* Ensure that the new struct is safe to send to other threads */
unsafe impl Send for ThreadsafeFitsFile {}

impl FitsFile {
    /**
    Create a threadsafe [`ThreadsafeFitsFile`][threadsafe-fitsfile] copy of the current
    [`FitsFile`][fits-file].

    [threadsafe-fitsfile]: struct.ThreadsafeFitsFile.html
    [fits-file]: ../fitsfile/struct.FitsFile.html
     */
    pub fn threadsafe(self) -> ThreadsafeFitsFile {
        ThreadsafeFitsFile(Arc::new(Mutex::new(self)))
    }
}

impl ThreadsafeFitsFile {
    /**
    Lock the underlying mutex to return exclusive access to the FitsFile.
    */
    pub fn lock(&self) -> Result<MutexGuard<'_, FitsFile>> {
        self.0.lock().map_err(From::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_using_other_threads() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let f = f.threadsafe();

        /* Spawn loads of threads... */
        let mut handles = Vec::new();
        for i in 0..10_000 {
            let f1 = f.clone();
            let handle = thread::spawn(move || {
                /* Get the underlyng fits file back */
                let mut t = f1.lock().unwrap();

                /* Fetch a different HDU per thread */
                let hdu_num = i % 2;
                let _hdu = t.hdu(hdu_num).unwrap();
            });
            handles.push(handle);
        }

        /* Wait for all of the threads to finish */
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
