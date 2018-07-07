#[macro_use]
extern crate criterion;

use criterion::Criterion;

fn fib(n: u64) -> u64 {
    let mut a = 0u64;
    let mut b = 1u64;
    let mut c = 0u64;

    if n == 0 {
        return 0
    }

    for _ in 0..(n+1) {
        c = a + b;
        a = b;
        b = c;
    }

    b
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fib(20)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
