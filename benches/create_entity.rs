use criterion::{criterion_group, criterion_main, Criterion};
use std::process::Command;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("create_entity", |b| b.iter(|| 
        curl()
    ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn curl() {
    let val = Command::new("curl")
            .args(&["-X", "POST"])
            .args(&["-H",  "Content-Type: application/wql"])
            .arg("localhost:1438/wql/query")
            .args(&["-d",  "CREATE ENTITY fuck_yeah"])
            .output()
            .expect("failed to execute process").stdout;
    let s = String::from_utf8(val);
    println!("{:?}", s.unwrap());
            
}