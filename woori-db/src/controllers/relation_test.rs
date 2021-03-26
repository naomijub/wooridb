use crate::http::routes;
use actix_http::body::ResponseBody;
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
