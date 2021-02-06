use crate::http::routes;
use actix_http::body::ResponseBody;
use actix_web::{body::Body, test, App};

#[ignore]
#[actix_rt::test]
async fn test_select_all_id_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!("Select * FROM test_select_all_id ID {}", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    assert!(body.contains("\"a\": Integer(123)"));
    assert!(body.contains("\"b\": Float(12.3)"));

    let payload = format!(
        "UPDATE test_select_all_id SET {{a: 12, c: Nil,}} INTO {}",
        uuid
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!("Select * FROM test_select_all_id ID {}", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    assert!(body.contains("\"a\": Integer(12)"));
    assert!(body.contains("\"b\": Float(12.3)"));
    assert!(body.contains("\"c\": Nil"));
}

#[ignore]
#[actix_rt::test]
async fn test_select_args_id_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3, c: 'd', e_f: \"hello\"} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!("Select #{{a, b, e_f,}} FROM test_select_all_id ID {}", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();

    assert!(body.contains("\"a\": Integer(123)"));
    assert!(body.contains("\"b\": Float(12.3)"));
    assert!(body.contains("\"e_f\": String(\"hello\")"));
}

#[actix_rt::test]
async fn test_create_on_query_endpoint() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_select_all_id")
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_client_error());
    let body = resp.take_body().as_str().to_string();
    assert_eq!(
        body,
        "Non-SELECT statements are handled by `/wql/tx` endpoint"
    );
}

#[ignore]
#[actix_rt::test]
async fn test_select_all_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 4365, b: 76.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 7654, b: 98.4, c: \"hello\",} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("Select * FROM test_select_all_id")
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    assert!(body.contains("Integer(7654)"));
    assert!(body.contains("Float(98.4)"));
    assert!(body.contains("String(\"hello\")"));
    assert!(body.contains("Float(76.3)"));
    assert!(body.contains("Integer(4365)"));
    assert!(body.contains("Float(12.3)"));
    assert!(body.contains("Integer(123)"));
}

#[ignore]
#[actix_rt::test]
async fn test_select_keys_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 4365, b: 76.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 7654, b: 98.4, c: \"hello\",} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("Select #{b,c,} FROM test_select_all_id")
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    assert!(!body.contains("Integer(7654)"));
    assert!(body.contains("Float(98.4)"));
    assert!(body.contains("String(\"hello\")"));
    assert!(body.contains("Float(76.3)"));
    assert!(!body.contains("Integer(4365)"));
    assert!(body.contains("Float(12.3)"));
    assert!(!body.contains("Integer(123)"));
}

#[ignore]
#[actix_rt::test]
async fn test_select_all_ids_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid1 = &body[(body.len() - 36)..];

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 456, b: 52.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid2 = &body[(body.len() - 36)..];

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 543, b: 32.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid3 = &body[(body.len() - 36)..];

    let payload = format!(
        "Select * FROM test_select_all_id IDS IN #{{ {}, {}, {}, }}",
        uuid1, uuid2, uuid3
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    assert!(body.contains("\"a\": Integer(123)"));
    assert!(body.contains("\"b\": Float(52.3)"));
    assert!(body.contains("\"b\": Float(32.3)"));
    assert!(body.contains(uuid1));
    assert!(body.contains(uuid2));
    assert!(body.contains(uuid3));
}

#[ignore]
#[actix_rt::test]
async fn test_select_keys_ids_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid1 = &body[(body.len() - 36)..];

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 456, b: 52.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid2 = &body[(body.len() - 36)..];

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 543, b: 32.3,} INTO test_select_all_id")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid3 = &body[(body.len() - 36)..];

    let payload = format!(
        "Select #{{a,}} FROM test_select_all_id IDS IN #{{ {}, {}, {}, }}",
        uuid1, uuid2, uuid3
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    assert!(body.contains("\"a\": Integer(123)"));
    assert!(!body.contains("\"b\": Float(52.3)"));
    assert!(!body.contains("\"b\": Float(32.3)"));
    assert!(body.contains(uuid1));
    assert!(body.contains(uuid2));
    assert!(body.contains(uuid3));
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
