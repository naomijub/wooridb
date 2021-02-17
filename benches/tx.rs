use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
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
    let id = curl_insert_with_id(ent_str);
    c.bench_function("update_set_entity", |b| {
        b.iter(|| {
            curl_update_set(ent_str, id);
        })
    });
    c.bench_function("update_content_entity", |b| {
        b.iter(|| {
            curl_update_content(ent_str, id);
        })
    });
    // c.bench_function("delete_entity", |b| {
    //     b.iter(|| {
    //         curl_delete(ent_str, id);
    //     })
    // });
    // c.bench_function("evict_entity_id", |b| {
    //     b.iter(|| {
    //         curl_evict_id(ent_str, id);
    //     })
    // });
    // c.bench_function("evict_entity", |b| {
    //     b.iter(|| {
    //         curl_evict_entity(ent_str);
    //     })
    // });
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

fn curl_update_set(entity: &str, id: uuid::Uuid) {
    let action = format!("UPDATE {} SET {{a: 3, g: NiL, }} into {}", entity, id);
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

fn curl_update_content(entity: &str, id: uuid::Uuid) {
    let action = format!("UPDATE {} CONTENT {{a: 3, g: NiL, }} into {}", entity, id);
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

#[allow(dead_code)]
fn curl_delete(entity: &str, id: uuid::Uuid) {
    let action = format!("DELETE {} FROM {}", id, entity);
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

#[allow(dead_code)]
fn curl_evict_id(entity: &str, id: uuid::Uuid) {
    let action = format!("EVICT {} FROM {}", id, entity);
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

#[allow(dead_code)]
fn curl_evict_entity(entity: &str) {
    let action = format!("EVICT {}", entity);
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

fn curl_insert_with_id(entity: &str) -> uuid::Uuid {
    let action = format!("INSERT {{a: 123,}} INTO {}", entity);
    let val = Command::new("curl")
        .args(&["-X", "POST"])
        .args(&["-H", "Content-Type: application/wql"])
        .arg("localhost:1438/wql/tx")
        .args(&["-d", &action])
        .output()
        .expect("failed to execute process")
        .stdout;
    let entity = String::from_utf8(val).unwrap();
    let inserted: InsertEntityResponse = ron::de::from_str(&entity).unwrap();
    inserted.uuid
}

fn get_rand_value() -> String {
    let mut rng = rand::thread_rng();
    let rng: usize = rng.gen();
    let mut rstr = String::from("fuck_yeah");

    rstr.push_str(&rng.to_string());

    rstr
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertEntityResponse {
    entity: String,
    pub(crate) uuid: uuid::Uuid,
    message: String,
}
