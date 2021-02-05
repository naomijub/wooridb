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
    assert_eq!(body, "{\n \"a\": Integer(123),\n \"b\": Float(12.3),\n}");
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
    assert_eq!(
        body,
        "{\n \"a\": Integer(123),\n \"b\": Float(12.3),\n \"e_f\": String(\"hello\"),\n}"
    );
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
