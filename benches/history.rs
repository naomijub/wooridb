use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::process::Command;
use uuid::Uuid;

fn criterion_benchmark(c: &mut Criterion) {
    let ent_str = "test_history_bench";
    curl_create(ent_str);
    let id = curl_insert_with_id(ent_str);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);

    c.bench_function("history_10_registries_for_entity", |b| {
        b.iter(|| curl_history(ent_str, id))
    });
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    curl_update_set(ent_str, id);
    c.bench_function("history_20_registries_for_entity", |b| {
        b.iter(|| curl_history(ent_str, id))
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
    let inserted: TxResponse = ron::de::from_str(&entity).unwrap();
    inserted.uuid
}

fn curl_update_set(entity: &str, id: uuid::Uuid) {
    let val = get_rand_value();
    let action = format!(
        "UPDATE {} SET {{a: 3, b: \"{}\", }} into {}",
        entity, val, id
    );
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

fn get_rand_value() -> String {
    let mut rng = rand::thread_rng();
    let rng: usize = rng.gen();
    let mut rstr = String::from("fuck_yeah");

    rstr.push_str(&rng.to_string());

    rstr
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxType {
    Create,
    Insert,
    UpdateSet,
    UpdateContent,
    Delete,
    EvictEntity,
    EvictEntityTree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxResponse {
    tx_type: TxType,
    entity: String,
    pub uuid: Option<Uuid>,
    state: String,
    message: String,
}
fn curl_history(entity: &str, id: Uuid) {
    let action = format!("(entity_key: \"{}\", entity_id: \"{}\",)", entity, id);
    let val = Command::new("curl")
        .args(&["-X", "POST"])
        .arg("localhost:1438/entity-history")
        .args(&["-d", &action])
        .output()
        .expect("failed to execute process")
        .stdout;
    match String::from_utf8(val) {
        Ok(_) => print!("OK,"),
        Err(e) => panic!("{:?}", e),
    };
}
