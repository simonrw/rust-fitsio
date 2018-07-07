#[macro_use]
extern crate criterion;
extern crate fitsio;

use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    let filename = "../testdata/full_example.fits";
    c.bench_function("opening and closing files", move |b| b.iter(|| {
        {
            let _f = fitsio::FitsFile::open(filename).unwrap();
            /* Implicit drop */
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
