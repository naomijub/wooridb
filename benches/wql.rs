use std::str::FromStr;

use criterion::{criterion_group, criterion_main, Criterion};
use wql::Wql;

fn criterion_benchmark(c: &mut Criterion) {

    c.bench_function("create_entity", |b| {
        b.iter(|| {
           Wql::from_str("create entity my_entity")
        })
    });

    c.bench_function("inser_entity", |b| {
        b.iter(|| {
           Wql::from_str("insert {a: 1} into my_entity")
        })
    });

    c.bench_function("select_all", |b| {
        b.iter(|| {
           Wql::from_str("select * from my_entity")
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);