use std::collections::BTreeMap;

use crate::http::routes;
use actix_http::{body::ResponseBody, Request};
use actix_web::{body::Body, test, App};
use std::collections::HashMap;
use uuid::Uuid;
use wql::Types;

#[ignore]
#[actix_rt::test]
async fn test_select_all_limit_offset_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;

    for req in inserts("limit_offset") {
        let _ = test::call_service(&mut app, req).await;
    }

    let payload = format!("Select * FROM limit_offset LIMIT 3 OFFSET 2",);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    let _: Result<BTreeMap<Uuid, HashMap<String, Types>>, String> = match ron::de::from_str(&body) {
        Ok(s) => {
            let s: BTreeMap<Uuid, HashMap<String, Types>> = s;
            assert_eq!(s.len(), 3);
            Ok(s)
        }
        Err(e) => {
            println!("{:?}", e);
            assert!(false);
            Err(String::new())
        }
    };
}

#[ignore]
#[actix_rt::test]
async fn test_select_all_dedup_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;

    for req in inserts("dedup_test") {
        let _ = test::call_service(&mut app, req).await;
    }

    let payload = format!("Select * FROM dedup_test DEDUP a",);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    let _: Result<BTreeMap<Uuid, HashMap<String, Types>>, String> = match ron::de::from_str(&body) {
        Ok(s) => {
            let s: BTreeMap<Uuid, HashMap<String, Types>> = s;
            assert_eq!(s.len(), 5);
            Ok(s)
        }
        Err(e) => {
            println!("{:?}", e);
            assert!(false);
            Err(String::new())
        }
    };
}

#[ignore]
#[actix_rt::test]
async fn test_select_all_group_by_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;

    for req in inserts("group_by_test") {
        let _ = test::call_service(&mut app, req).await;
    }

    let payload = format!("Select * FROM group_by_test GROUP BY c",);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    let _: Result<HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>>, String> =
        match ron::de::from_str(&body) {
            Ok(s) => {
                let s: HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>> = s;
                let keys = s.keys().map(|k| k.to_owned()).collect::<Vec<String>>();
                assert!(keys.contains(&String::from("Char(\'r\')")));
                assert!(keys.contains(&String::from("Char(\'d\')")));
                assert!(keys.contains(&String::from("Char(\'c\')")));
                assert!(keys.contains(&String::from("Nil")));
                Ok(s)
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
                Err(String::new())
            }
        };
}

#[ignore]
#[actix_rt::test]
async fn test_select_where_group_by_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;

    for req in inserts("where_group_by_test") {
        let _ = test::call_service(&mut app, req).await;
    }

    let payload = format!("Select * FROM where_group_by_test WHERE {{
        ?* where_group_by_test:c ?c,
        (in ?c 'c' 'd'),
    }} GROUP BY c",);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    let _: Result<HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>>, String> =
        match ron::de::from_str(&body) {
            Ok(s) => {
                let s: HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>> = s;
                let keys = s.keys().map(|k| k.to_owned()).collect::<Vec<String>>();
                assert!(!keys.contains(&String::from("Char(\'r\')")));
                assert!(keys.contains(&String::from("Char(\'d\')")));
                assert!(keys.contains(&String::from("Char(\'c\')")));
                assert!(!keys.contains(&String::from("Nil")));
                Ok(s)
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
                Err(String::new())
            }
        };
}

#[ignore]
#[actix_rt::test]
async fn test_select_all_order_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;

    for req in inserts("order_by_test") {
        let _ = test::call_service(&mut app, req).await;
    }

    let payload = format!("Select * FROM order_by_test ORDER BY a :asc",);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    let _: Result<Vec<(Uuid, HashMap<String, Types>)>, String> = match ron::de::from_str(&body) {
        Ok(s) => {
            let s: Vec<(Uuid, HashMap<String, Types>)> = s;
            assert_eq!(s.first().unwrap().1.get("a").unwrap(), &Types::Integer(25));
            assert_eq!(s.last().unwrap().1.get("a").unwrap(), &Types::Integer(475));
            Ok(s)
        }
        Err(e) => {
            println!("{:?}", e);
            assert!(false);
            Err(String::new())
        }
    };
}

fn inserts(entity_name: &str) -> Vec<Request> {
    vec![
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!("CREATE ENTITY {}", entity_name))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!("INSERT {{a: 123, b: 12.3,}} INTO {}", entity_name))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 235, b: 12.3, c: 'c',}} INTO {}",
                entity_name
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 235, b: 12.3, c: 'd',}} INTO {}",
                entity_name
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 25, b: 12.3, c: 'c',}} INTO {}",
                entity_name
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 475, b: 12.3, c: 'd',}} INTO {}",
                entity_name
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 295, b: 12.3, c: 'r',}} INTO {}",
                entity_name
            ))
            .uri("/wql/tx")
            .to_request(),
    ]
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
