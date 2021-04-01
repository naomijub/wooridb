use std::collections::{BTreeMap, HashMap};

use crate::http::routes;
use actix_http::body::ResponseBody;
use actix_web::{body::Body, test, App};
use uuid::Uuid;
use wql::Types;

use super::tx_test::clear;

#[ignore]
#[actix_rt::test]
async fn simple_where_clause_eq() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_simple_where_eq")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 3, b: \"hello world\", c: 45.6,} INTO test_simple_where_eq")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 43, b: \"Julia Naomi\", c: 57.6,} INTO test_simple_where_eq")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 948, b: \"Otavio Pace\", c: 5.6,} INTO test_simple_where_eq")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: \"hello johnny\", c: 4345.6,} INTO test_simple_where_eq")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(
            "Select * From test_simple_where_eq WHERE {
            ?* test_simple_where_eq:a 123,
        }",
        )
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();
    let result: BTreeMap<Uuid, HashMap<String, Types>> = ron::de::from_str(&body).unwrap();
    assert!(result.iter().count() == 1);
    if let Some((_, map)) = result.iter().last() {
        assert_eq!(map["a"], Types::Integer(123));
    } else {
        assert!(false);
    }

    clear();
}

#[ignore]
#[actix_rt::test]
async fn clause_between() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_where_between")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 3, b: \"hello world\", c: 45.6,} INTO test_where_between")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 43, b: \"Julia Naomi\", c: 57.6,} INTO test_where_between")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 948, b: \"Otavio Pace\", c: 5.6,} INTO test_where_between")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: \"hello johnny\", c: 4345.6,} INTO test_where_between")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(
            "Select * From test_where_between WHERE {
            ?* test_where_between:a ?a,
            (between ?a 0 100),
        }",
        )
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();
    let result: BTreeMap<Uuid, HashMap<String, Types>> = ron::de::from_str(&body).unwrap();
    assert!(result.iter().count() == 2);
    if let Some((_, map)) = result.iter().last() {
        assert!(map["a"] <= Types::Integer(100) || map["a"] >= Types::Integer(0));
    } else {
        assert!(false);
    }

    clear();
}

#[ignore]
#[actix_rt::test]
async fn clause_in() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_where_in")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 3, b: \"hello world\", c: 45.6,} INTO test_where_in")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 43, b: \"Julia Naomi\", c: 57.6,} INTO test_where_in")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 948, b: \"Otavio Pace\", c: 5.6,} INTO test_where_in")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: \"hello johnny\", c: 4345.6,} INTO test_where_in")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(
            "Select * From test_where_in WHERE {
            ?* test_where_in:c ?c,
            (in ?c 57.6 4345.6 5.6),
        }",
        )
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();
    let result: BTreeMap<Uuid, HashMap<String, Types>> = ron::de::from_str(&body).unwrap();
    assert!(result.iter().count() == 3);
    if let Some((_, map)) = result.iter().last() {
        assert!(
            map["c"] == Types::Float(57.6)
                || map["c"] == Types::Float(4345.6)
                || map["c"] == Types::Float(5.6)
        );
    } else {
        assert!(false);
    }

    clear();
}

#[ignore]
#[actix_rt::test]
async fn clause_ge_le() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_where_in")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 3, b: \"hello world\", c: 45.6,} INTO test_where_in")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 43, b: \"Julia Naomi\", c: 57.6,} INTO test_where_in")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 948, b: \"Otavio Pace\", c: 5.6,} INTO test_where_in")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: \"hello johnny\", c: 4345.6,} INTO test_where_in")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(
            "Select * From test_where_in WHERE {
            ?* test_where_in:c ?c,
            (>= ?c 10),
            (< ?c 60.0),
        }",
        )
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();
    println!("{}", body);
    let result: BTreeMap<Uuid, HashMap<String, Types>> = ron::de::from_str(&body).unwrap();
    assert!(result.iter().count() == 2);
    if let Some((_, map)) = result.iter().last() {
        assert!(map["c"] == Types::Float(57.6) || map["c"] == Types::Float(45.6));
    } else {
        assert!(false);
    }

    clear();
}

#[ignore]
#[actix_rt::test]
async fn clause_or() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_or")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 3, b: \"hello world\", c: 45.6,} INTO test_or")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: \"Julia Naomi\", c: 57.6,} INTO test_or")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: \"Otavio Pace\", c: 5.6,} INTO test_or")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: \"hello johnny\", c: 4345.6,} INTO test_or")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(
            "Select * From test_or WHERE {
            ?* test_or:b ?b,
            ?* test_or:c ?c,
            (or
                (>= ?c 4300.0)
                (< ?c 6.9)
                (like ?b \"%Naomi\")
            ),
        }",
        )
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();
    let result: BTreeMap<Uuid, HashMap<String, Types>> = ron::de::from_str(&body).unwrap();

    assert!(result.iter().count() == 3);
    if let Some((_, map)) = result.iter().last() {
        assert!(
            map["c"] == Types::Float(5.6)
                || map["c"] == Types::Float(4345.6)
                || map["c"] == Types::Float(57.6)
        );
    } else {
        assert!(false);
    }

    assert!(
        result
            .iter()
            .filter(|(_, c)| if let Some(Types::String(s)) = c.get("b") {
                s.starts_with("Julia")
            } else {
                false
            })
            .count()
            == 1
    );

    clear();
}

#[ignore]
#[actix_rt::test]
async fn clause_like() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_like")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 3, b: \"hello world\", c: 45.6,} INTO test_like")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: \"Julia Naomi\", c: 57.6,} INTO test_like")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: \"Otavio Pace\", c: 5.6,} INTO test_like")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: \"hello johnny\", c: 4345.6,} INTO test_like")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(
            "Select * From test_like WHERE {
            ?* test_like:b ?b,
            (like ?b \"hello%\"),
        }",
        )
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();
    let result: BTreeMap<Uuid, HashMap<String, Types>> = ron::de::from_str(&body).unwrap();

    assert!(result.iter().count() == 2);
    if let Some((_, map)) = result.iter().last() {
        assert!(match map["b"].clone() {
            Types::String(s) => s.starts_with("hello"),
            _ => false,
        })
    } else {
        assert!(false);
    }

    clear();
}

trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for ResponseBody<Body> {
    fn as_str(&self) -> &str {
        match self {
            ResponseBody::Body(ref b) => match b {
                Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                _ => panic!(),
            },
            ResponseBody::Other(ref b) => match b {
                Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                _ => panic!(),
            },
        }
    }
}
