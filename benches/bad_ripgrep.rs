use std::hint::black_box;

use regex::Regex;

use criterion::{criterion_group, criterion_main, Criterion};
use bad_ripgrep::run_application;

fn bench_bad_ripgrep(c: &mut Criterion) {
    let search_re = Regex::new("ipsum").unwrap();
    c.bench_function("bad_ripgrep run", |b| {
        b.iter(|| run_application(black_box(&search_re)))
    });
}

criterion_group!(benches, bench_bad_ripgrep);
criterion_main!(benches);
