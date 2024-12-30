use fitsio_sys::cfitsio_version;

fn main() {
    println!("cfitsio version: {}", cfitsio_version());
}
