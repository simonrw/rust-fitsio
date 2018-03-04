extern crate fitsio;

use std::env;
use std::process;

fn main() {
    let mut nfiles = 0;
    env::args().skip(1).for_each(|arg| {
        let mut f = fitsio::FitsFile::open(arg).expect("opening file");
        f.pretty_print().expect("printing summary");
        nfiles += 1;
    });

    if nfiles == 0 {
        eprintln!("No files supplied");
        process::exit(1);
    }
}
