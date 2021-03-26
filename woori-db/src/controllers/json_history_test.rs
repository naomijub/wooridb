use crate::{auth::schemas::UserId, http::routes, schemas::tx::InsertEntityResponse};
use actix_http::body::ResponseBody;
use actix_web::{body::Body, test, App};
use uuid::Uuid;

#[actix_rt::test]
async fn test_history_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_history")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_history")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let response: InsertEntityResponse = serde_json::from_str(&body).unwrap();
    let uuid = response.uuid;

    let payload = format!("UPDATE test_history SET {{a: 12, c: Nil,}} INTO {}", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!("Delete {} FROM test_history", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!("UPDATE test_history SET {{a: 34, c: true,}} INTO {}", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!("UPDATE test_history SET {{a: 321, c: 'h',}} INTO {}", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!(
        "{{\"entity_key\": \"test_history\", \"entity_id\": \"{}\"}}",
        uuid
    );
    let req = test::TestRequest::post()
        .set_payload(payload)
        .uri("/entity-history")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let body = resp.take_body().as_str().to_string();
    assert!(body.contains("\"a\":{\"Integer\":123}"));
    assert!(body.contains("\"b\":{\"Float\":12.3}"));
    assert!(body.contains("\"c\":{\"Boolean\":true}"));
    assert!(body.contains("\"c\":{\"Char\":\"h\"}"));
    clear();
}

#[ignore]
#[actix_rt::test]
async fn query_and_tx_with_token() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
            .set_payload("{\"admin_id\": \"your_admin\", \"admin_password\": \"your_password\", \"user_info\": {\"user_password\": \"my_password\",\"role\": [\"User\"]}}")
            .uri("/auth/createUser")
            .to_request();
    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();
    let uuid: UserId = serde_json::from_str(&body).unwrap();

    let payload = format!(
        "{{\"id\": \"{}\", \"user_password\": \"my_password\"}}",
        uuid.user_id
    );
    let req = test::TestRequest::put()
        .set_payload(payload)
        .uri("/auth/putUserSession")
        .to_request();
    let mut resp = test::call_service(&mut app, req).await;
    let token = resp.take_body().as_str().to_string();
    let token = format!("Bearer {}", token);

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .header("Authorization", token.clone())
        .set_payload("CREATE ENTITY token_test_ok")
        .uri("/wql/tx")
        .to_request();
    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .header("Authorization", token.clone())
        .set_payload("INSERT {a: 123,} INTO token_test_ok")
        .uri("/wql/tx")
        .to_request();
    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .header("Authorization", token.clone())
        .set_payload("Select * FROM token_test_ok")
        .uri("/wql/query")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    let body = resp.take_body().as_str().to_string();
    println!("{}", body);
    assert!(resp.status().is_success());
    assert!(body.contains("\"a\": Integer(123)"))
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
