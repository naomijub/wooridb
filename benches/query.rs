use criterion::{criterion_group, criterion_main, Criterion};
use std::process::Command;


fn criterion_benchmark(c: &mut Criterion) {
    let ent_str = "bench_entity_name";
    curl_create(ent_str);
    curl_insert(ent_str);
    c.bench_function("select_all_1_entity", |b| {
        b.iter(|| {
            curl_select(ent_str)
        })
    });

    curl_insert(ent_str);curl_insert(ent_str);
    curl_insert(ent_str);curl_insert(ent_str);
    curl_insert(ent_str);curl_insert(ent_str);
    curl_insert(ent_str);curl_insert(ent_str);
    curl_insert(ent_str);
    c.bench_function("select_all_10_entity", |b| {
        b.iter(|| {
            curl_select(ent_str)
        })
    });

    curl_insert(ent_str);curl_insert(ent_str);
    curl_insert(ent_str);curl_insert(ent_str);
    curl_insert(ent_str);curl_insert(ent_str);
    curl_insert(ent_str);curl_insert(ent_str);
    curl_insert(ent_str);curl_insert(ent_str);
    c.bench_function("select_all_20_entity", |b| {
        b.iter(|| {
            curl_select(ent_str)
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
    match String::from_utf8(val) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
}

fn curl_insert(entity: &str) {
    let action = format!("INSERT {{a: 123,}} INTO {}", entity);
    let val = Command::new("curl")
        .args(&["-X", "POST"])
        .args(&["-H", "Content-Type: application/wql"])
        .arg("localhost:1438/wql/tx")
        .args(&["-d", &action])
        .output()
        .expect("failed to execute process")
        .stdout;
    match String::from_utf8(val) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
}

fn curl_select(entity: &str) {
    let action = format!("SELECT * FROM {}", entity);
    let val = Command::new("curl")
        .args(&["-X", "POST"])
        .args(&["-H", "Content-Type: application/wql"])
        .arg("localhost:1438/wql/query")
        .args(&["-d", &action])
        .output()
        .expect("failed to execute process")
        .stdout;
    match String::from_utf8(val) {
        Ok(_) => print!("OK,"),
        Err(e) => panic!("{:?}", e),
    };
}
