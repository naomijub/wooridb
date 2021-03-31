use crate::http::routes;
use actix_http::{body::ResponseBody, Request};
use actix_web::{body::Body, test, App};
use uuid::Uuid;

#[actix_rt::test]
async fn test_intersect_key() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok1")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok2")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let uuid1 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 234, c: true,}} INTO intersect_ok1 WITH {}",
        uuid1
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let uuid2 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 432, d: false,}} INTO intersect_ok2 WITH {}",
        uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!(
        "INTERSECT KEY Select * FROM intersect_ok1 ID {} | Select * FROM intersect_ok2 ID {}",
        uuid1, uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();

    assert!(body.contains("\"b\": Integer(234)"));
    assert!(body.contains("\"a\": Integer(123)"));
    assert!(!body.contains("\"c\":"));
    clear();
}

#[actix_rt::test]
async fn test_intersect_key_value() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok1")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok2")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let uuid1 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 234, c: true,}} INTO intersect_ok1 WITH {}",
        uuid1
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let uuid2 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 432, d: false,}} INTO intersect_ok2 WITH {}",
        uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!(
        "INTERSECT KEY-VALUE Select * FROM intersect_ok1 ID {} | Select * FROM intersect_ok2 ID {}",
        uuid1, uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();

    assert!(!body.contains("\"b\": Integer(234)"));
    assert!(body.contains("\"a\": Integer(123)"));
    assert!(!body.contains("\"c\":"));
    clear();
}

#[actix_rt::test]
async fn test_diff_key() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok1")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok2")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let uuid1 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 234, c: true,}} INTO intersect_ok1 WITH {}",
        uuid1
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let uuid2 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 432, d: false,}} INTO intersect_ok2 WITH {}",
        uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!(
        "DIFFERENCE KEY Select * FROM intersect_ok1 ID {} | Select * FROM intersect_ok2 ID {}",
        uuid1, uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();

    assert!(!body.contains("\"b\": Integer(234)"));
    assert!(!body.contains("\"a\": Integer(123)"));
    assert!(body.contains("\"c\": Boolean(true)"));
    clear();
}

#[actix_rt::test]
async fn test_diff_key_value() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok1")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok2")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let uuid1 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 234, c: true,}} INTO intersect_ok1 WITH {}",
        uuid1
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let uuid2 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 432, d: false,}} INTO intersect_ok2 WITH {}",
        uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!(
        "DIFFERENCE KEY-VALUE Select * FROM intersect_ok1 ID {} | Select * FROM intersect_ok2 ID {}",
        uuid1, uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();

    assert!(body.contains("\"b\": Integer(234)"));
    assert!(!body.contains("\"a\": Integer(123)"));
    assert!(body.contains("\"c\": Boolean(true)"));
    clear();
}

#[actix_rt::test]
async fn test_union_key() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok1")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok2")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let uuid1 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 234, c: true,}} INTO intersect_ok1 WITH {}",
        uuid1
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let uuid2 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 432, d: false,}} INTO intersect_ok2 WITH {}",
        uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!(
        "UNION KEY Select * FROM intersect_ok1 ID {} | Select * FROM intersect_ok2 ID {}",
        uuid1, uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();

    assert!(body.contains("\"b\": Integer(234)"));
    assert!(body.contains("\"a\": Integer(123)"));
    assert!(body.contains("\"c\": Boolean(true)"));
    assert!(body.contains("\"d\": Boolean(false)"));
    clear();
}

#[actix_rt::test]
async fn test_union_key_value() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok1")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY intersect_ok2")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let uuid1 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 234, c: true,}} INTO intersect_ok1 WITH {}",
        uuid1
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;
    let uuid2 = Uuid::new_v4().to_string();
    let payload = format!(
        "INSERT {{a: 123, b: 432, d: false,}} INTO intersect_ok2 WITH {}",
        uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!(
        "UNION KEY-VALUE Select * FROM intersect_ok1 ID {} | Select * FROM intersect_ok2 ID {}",
        uuid1, uuid2
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();

    assert!(body.contains("\"b\": Integer(234)"));
    assert!(body.contains("\"a\": Integer(123)"));
    assert!(body.contains("\"c\": Boolean(true)"));
    assert!(body.contains("\"d\": Boolean(false)"));
    assert!(body.contains("\"b:duplicated\": Integer(432)"));
    clear();
}

#[ignore]
#[actix_rt::test]
async fn test_join() {
    let mut app = test::init_service(App::new().configure(routes)).await;

    for req in inserts() {
        let _ = test::call_service(&mut app, req).await;
    }

    let payload =
        format!("JOIN (entity_A:c, entity_B:c) Select * FROM entity_A | Select * FROM entity_B");
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();

    assert!(body.contains("\"a:entity_B\": Integer(25),"));
    assert!(body.contains("\"b:entity_B\": Float(12.3),"));
    assert!(body.contains("\"c\": Char(\'c\'),"));
    assert!(body.contains("\"b\": Float(12.3)"));
    assert!(body.contains("\"a\": Integer(235)"));
    assert!(!body.contains("\"c:entity_B\""));
    assert!(!body.contains("\"tx_time:entity_B\""));
}

#[ignore]
#[actix_rt::test]
async fn test_join2() {
    let mut app = test::init_service(App::new().configure(routes)).await;

    for req in inserts2() {
        let _ = test::call_service(&mut app, req).await;
    }

    let payload =
        format!("JOIN (entity_AA:c, entity_BB:o) Select * FROM entity_AA order by c :asc | Select #{{g, f, o, b,}} FROM entity_BB ");
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();

    assert!(body.contains("b:entity_BB"));
    assert!(!body.contains("g:entity_BB"));
    assert!(body.contains("\"g\": Integer(475)"));
    assert_eq!(body.matches("Char(\'d\')").count(), 4);
}

fn inserts() -> Vec<Request> {
    vec![
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!("CREATE ENTITY {}", "entity_A"))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!("CREATE ENTITY {}", "entity_B"))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!("INSERT {{a: 123, b: 12.3,}} INTO {}", "entity_A"))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 235, b: 12.3, c: 'c',}} INTO {}",
                "entity_A"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 235, b: 12.3, c: 'd',}} INTO {}",
                "entity_A"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 25, b: 12.3, c: 'c',}} INTO {}",
                "entity_B"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 475, b: 12.3, c: 'd',}} INTO {}",
                "entity_B"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 295, b: 12.3, c: 'r',}} INTO {}",
                "entity_B"
            ))
            .uri("/wql/tx")
            .to_request(),
    ]
}

fn inserts2() -> Vec<Request> {
    vec![
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!("CREATE ENTITY {}", "entity_AA"))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!("CREATE ENTITY {}", "entity_BB"))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!("INSERT {{a: 123, b: 12.3,}} INTO {}", "entity_AA"))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 235, b: 17.3, c: 'c',}} INTO {}",
                "entity_AA"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 476, b: 312.3, c: 'd',}} INTO {}",
                "entity_AA"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!("INSERT {{a: 857, c: 'd',}} INTO {}", "entity_AA"))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{a: 66, b: 66.3, c: 'r',}} INTO {}",
                "entity_BB"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{g: 25, f: 12.3, a: 'c',}} INTO {}",
                "entity_BB"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{g: 475, b: 12.3, f: 'h', o: 'd',}} INTO {}",
                "entity_BB"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{g: 756, b: 142.3, f: 'h', o: 'c',}} INTO {}",
                "entity_BB"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{g: 76, b: 12.3, f: 't', o: 'd',}} INTO {}",
                "entity_BB"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{t: 295, b: 12.3, o: 'r',}} INTO {}",
                "entity_BB"
            ))
            .uri("/wql/tx")
            .to_request(),
        test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(format!(
                "INSERT {{t: 295, f: 12.3, o: Nil,}} INTO {}",
                "entity_BB"
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

pub fn clear() {
    std::process::Command::new("rm")
        .arg("-rf")
        .arg("data/*.log")
        .output()
        .expect("failed to execute process")
        .stdout;
}
