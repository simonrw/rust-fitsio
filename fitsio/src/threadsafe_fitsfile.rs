/*! Thread-safe FitsFile struct */

#![warn(missing_docs)]

use errors::Result;
use fitsfile::FitsFile;
use hdu::{DescribesHdu, FitsHdu};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ThreadsafeFitsfile(Arc<Mutex<FitsFile>>);

unsafe impl Send for ThreadsafeFitsfile {}

impl FitsFile {
    pub fn threadsafe(self) -> ThreadsafeFitsfile {
        ThreadsafeFitsfile(Arc::new(Mutex::new(self)))
    }
}

impl ThreadsafeFitsfile {
    pub fn hdu<T: DescribesHdu>(&mut self, hdu_description: T) -> Result<FitsHdu> {
        FitsHdu::new(&mut *self.0.lock()?, hdu_description)
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
        for _ in 0..10_000 {
            let mut f1 = f.clone();
            thread::spawn(move || {
                let hdu = f1.hdu(0).unwrap();
                let mut ff = f1.0.lock().unwrap();
                let image: Vec<i32> = hdu.read_image(&mut ff).unwrap();
                assert_eq!(image.len(), 10_000);
            });
        }
    }
}
