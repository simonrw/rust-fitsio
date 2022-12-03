use fitsio::FitsFile;

fn main() {
    let fname = "examples/nasa.fits";
    let mut f = FitsFile::open(fname).unwrap();
    f.pretty_print().unwrap();

    let img_hdu = f.primary_hdu().unwrap();
    let image_data: Vec<f32> = img_hdu.read_image(&mut f).unwrap();
    dbg!(&image_data[0..10]);
}
