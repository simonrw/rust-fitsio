use fitsio::FitsFile;
use std::time::Instant;

fn read_from_fits(filename: &str) -> Vec<f64> {
    let mut f = FitsFile::open(filename).unwrap();
    let phdu = f.primary_hdu().unwrap();
    phdu.read_image(&mut f).unwrap()
}

fn runit() -> usize {
    let bias = read_from_fits("bias.fits");
    let dark = read_from_fits("dark.fits");
    let flat = read_from_fits("flat.fits");
    let science = read_from_fits("science.fits");

    let result: Vec<_> = science
        .iter()
        .zip(bias.iter())
        .zip(dark.iter())
        .zip(flat.iter())
        .map(|(((s, b), d), f)| (s - b - d) / f)
        .collect();
    result.len()
}

fn timeit<F>(f: F, n: usize) -> f64
where
    F: Fn() -> usize,
{
    let mut times: Vec<f64> = Vec::with_capacity(n);

    for _ in 0..n {
        let now = Instant::now();
        f();
        times.push(now.elapsed().as_secs_f64());
    }

    times.iter().fold(0.0, |acc, val| val.min(acc))
}

fn main() {
    let min_time = timeit(runit, 64);
    println!("Time taken: {:.8} seconds", min_time);
}
