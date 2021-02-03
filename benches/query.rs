use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use std::process::Command;

fn criterion_benchmark(c: &mut Criterion) {
    let entity = get_rand_value();
    let ent_str = entity.as_str();
    c.bench_function("create_entity", |b| {
        b.iter(|| {
            curl_create(ent_str);
        })
    });
    c.bench_function("insert_entity", |b| {
        b.iter(|| {
            curl_insert(ent_str);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn curl_create(entity: &str) {
    let action = format!("CREATE ENTITY {}", entity);
    let val = Command::new("curl")
        .args(&["-X", "POST"])
        .args(&["-H", "Content-Type: application/wql"])
        .arg("localhost:1438/wql/tx")
        .args(&["-d", &action])
        .output()
        .expect("failed to execute process")
        .stdout;
    let _ = String::from_utf8(val);
}

fn curl_insert(entity: &str) {
    let action = format!("INSERT {{a: 123}} INTO {}", entity);
    let val = Command::new("curl")
        .args(&["-X", "POST"])
        .args(&["-H", "Content-Type: application/wql"])
        .arg("localhost:1438/wql/tx")
        .args(&["-d", &action])
        .output()
        .expect("failed to execute process")
        .stdout;
    let _ = String::from_utf8(val);
}

fn get_rand_value() -> String {
    let mut rng = rand::thread_rng();
    let rng: usize = rng.gen();
    let mut rstr = String::from("fuck_yeah");

    rstr.push_str(&rng.to_string());

    rstr
}
