use crate::http::routes;
use crate::io::read;
use actix_http::body::ResponseBody;
use actix_web::{body::Body, test, App};

#[actix_rt::test]
async fn test_create_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_ok")
        .uri("/wql/tx")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
    let body = resp.take_body();
    let body = body.as_ref().unwrap();
    assert_eq!(&Body::from("Entity test_ok created"), body);
    read::assert_content("CREATE_ENTITY|test_ok;");
    clear();
}

#[actix_rt::test]
async fn test_select_post_err() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("SELECT * FROM test_ok")
        .uri("/wql/tx")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
    let body = resp.take_body();
    let body = body.as_ref().unwrap();
    assert_eq!(
        &Body::from("SELECT expressions are handled by `/wql/query` endpoint"),
        body
    );
    clear();
}

#[actix_rt::test]
async fn test_create_uniques_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_uniques UNIQUES name, ssn, id")
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
    read::assert_content("CREATE_ENTITY|test_uniques;");
    read::assert_uniques("test_uniques");
    read::assert_uniques("uniques: [\"name\",\"ssn\",\"id\",]");
    clear();
}

#[actix_rt::test]
async fn test_create_post_duplicated_err() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_ok")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let duplicated_req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_ok")
        .uri("/wql/tx")
        .to_request();
    let mut resp = test::call_service(&mut app, duplicated_req).await;

    assert!(resp.status().is_client_error());
    let body = resp.take_body();
    let body = body.as_ref().unwrap();
    assert_eq!(&Body::from("Entity `test_ok` already created"), body);
    clear();
}

#[actix_rt::test]
async fn test_create_post_bad_request() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "text/plain")
        .set_payload("CREATE ENTITY test_ok")
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
    clear();
}

#[actix_rt::test]
async fn test_unkwon_wql_post() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("DO SOMETHIG weird")
        .uri("/wql/tx")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
    let body = resp.take_body();
    let body = body.as_ref().unwrap();
    assert_eq!(&Body::from("\"Symbol `DO` not implemented\""), body);
    clear();
}

#[actix_rt::test]
async fn test_insert_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_ok")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123,} INTO test_ok")
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    read::assert_content("INSERT|");
    read::assert_content("UTC|");
    read::assert_content("|test_ok|{\"a\": Integer(123),};");
    clear();
}

#[actix_rt::test]
async fn test_insert_unique_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_insert_unique UNIQUES id")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {id: 123, a: \"hello\",} INTO test_insert_unique")
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {id: 123, a: \"world\",} INTO test_insert_unique")
        .uri("/wql/tx")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
    let body = resp.take_body();
    let body = body.as_ref().unwrap();
    assert_eq!(
        &Body::from(
            "key `id` in entity `test_insert_unique` already contains value `Integer(123)`"
        ),
        body
    );

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {id: 234, a: \"hello\",} INTO test_insert_unique")
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    clear();
}

#[actix_rt::test]
async fn test_insert_entity_not_created() {
    let mut app = test::init_service(App::new().configure(routes)).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123,} INTO missing")
        .uri("/wql/tx")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
    let body = resp.take_body();
    let body = body.as_ref().unwrap();
    assert_eq!(&Body::from("Entity `missing` not created"), body);
    clear();
}

#[ignore]
#[actix_rt::test]
async fn test_update_set_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_update")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_update")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!("UPDATE test_update SET {{a: 12, c: Nil,}} INTO {}", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());

    read::assert_content("UPDATE_SET|");
    read::assert_content("UTC|");
    read::assert_content(uuid);
    read::assert_content("|test_update|");
    read::assert_content("\"a\": Integer(12),");
    read::assert_content("\"b\": Float(12.3),");
    read::assert_content("\"c\": Nil,");
    clear();
}

#[ignore]
#[actix_rt::test]
async fn test_update_uniqueness_set_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_unique_set_update UNIQUES a")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_unique_set_update")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 321, b: 12.3,} INTO test_unique_set_update")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!(
        "UPDATE test_unique_set_update SET {{a: 123, c: Nil,}} INTO {}",
        uuid
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_client_error());
    let body = resp.take_body();
    let body = body.as_ref().unwrap();
    assert_eq!(
        &Body::from(
            "key `a` in entity `test_unique_set_update` already contains value `Integer(123)`"
        ),
        body
    );
    clear();
}
#[ignore]
#[actix_rt::test]
async fn test_update_content_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_update")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(
            "INSERT {
            a: 123,
            b: 12.3,
            c: 'd' ,
            d: true ,
            e: 4321,
            f: \"hello\",
            g: NiL,} 
        INTO test_update",
        )
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!(
        "UPDATE test_update CONTENT {{
        a: 12,
        b: -1.3,
        c: 'd' ,
        d: false ,
        e: 4,
        f: \"world\",
        g: true,}} INTO {}",
        uuid
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    read::assert_content("UPDATE_CONTENT|");
    read::assert_content("UTC|");
    read::assert_content(uuid);
    read::assert_content("|test_update|");
    read::assert_content("\"a\": Integer(135),");
    read::assert_content("\"b\": Float(11),");
    read::assert_content("\"c\": Char('d'),");
    read::assert_content("\"d\": Boolean(false),");
    read::assert_content("\"e\": Integer(4325)");
    read::assert_content("\"f\": String(\"helloworld\"),");
    read::assert_content("\"g\": Boolean(true),");
    clear();
}

#[actix_rt::test]
async fn test_update_wrong_entity() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_update")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(
            "INSERT {
            a: 123,
            g: NiL,} 
        INTO test_update",
        )
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!(
        "UPDATE test_anything CONTENT {{
        a: 12,
        g: true,}} INTO {}",
        uuid
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
    let body = resp.take_body();
    let body = body.as_ref().unwrap();
    assert_eq!(&Body::from("Entity `test_anything` not created"), body);
    clear();
}

#[actix_rt::test]
async fn test_update_any_uuid() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_update")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(
            "INSERT {
            a: 123,
            g: NiL,} 
        INTO test_update",
        )
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!(
        "UPDATE test_update CONTENT {{
        a: 12,
        g: true,}} INTO {}",
        uuid::Uuid::new_v4()
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
    let body = resp.take_body();
    let body = body.as_str();
    assert!(body.contains("not created for entity test_update"));
    assert!(body.contains("Uuid"));
    clear();
}

#[ignore]
#[actix_rt::test]
async fn test_delete_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_delete")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_delete")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!("UPDATE test_delete SET {{a: 12, c: Nil,}} INTO {}", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let payload = format!("Delete {} FROM test_delete", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());

    read::assert_content("DELETE");
    clear();
}

#[ignore]
#[actix_rt::test]
async fn test_delete_withput_update() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_delete")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_delete")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!("Delete {} FROM test_delete", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());

    read::assert_content("DELETE");
    read::assert_content("|{}|");
    clear();
}

#[ignore]
#[actix_rt::test]
async fn test_match_all_update_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_match_all")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_match_all")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!(
        "MATCH ALL(a > 100, b <= 20.0) UPDATE test_match_all SET {{a: 43, c: Nil,}} INTO {}",
        uuid
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    clear();
}

#[ignore]
#[actix_rt::test]
async fn test_match_any_update_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_match_all")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_match_all")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!(
        "MATCH ANY(a > 100, b <= 10.0) UPDATE test_match_all SET {{a: 43, c: Nil,}} INTO {}",
        uuid
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    clear();
}

#[ignore]
#[actix_rt::test]
async fn test_match_any_update_fail() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_match_all")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_match_all")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!(
        "MATCH ANY(a > 200, b <= 10.0) UPDATE test_match_all SET {{a: 43, c: Nil,}} INTO {}",
        uuid
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_client_error());
    let body = resp.take_body();
    let body = body.as_ref().unwrap();
    assert_eq!(&Body::from("One or more MATCH CONDITIONS failed"), body);
    clear();
}

#[ignore]
#[actix_rt::test]
async fn test_match_any_update_fake_key() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_match_all")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_match_all")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!(
        "MATCH ANY(g > 100, b <= 20.0) UPDATE test_match_all SET {{a: 43, c: Nil,}} INTO {}",
        uuid
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    clear();
}

#[ignore]
#[actix_rt::test]
async fn test_match_all_update_fake_key() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_match_all")
        .uri("/wql/tx")
        .to_request();

    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_match_all")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];

    let payload = format!(
        "MATCH ALL(g > 100, b <= 20.0) UPDATE test_match_all SET {{a: 43, c: Nil,}} INTO {}",
        uuid
    );
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_client_error());
    clear();
}

#[actix_rt::test]
async fn test_evict_entity_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_evict")
        .uri("/wql/tx")
        .to_request();
    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("Evict test_evict")
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
    read::assert_content("EVICT_ENTITY|");
    read::assert_content("|test_evict;");

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_evict")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    assert!(resp_insert.status().is_client_error());
    assert_eq!("Entity `test_evict` not created", body);
    clear();
}

#[actix_rt::test]
async fn test_evict_entity_id_post_ok() {
    let mut app = test::init_service(App::new().configure(routes)).await;
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("CREATE ENTITY test_evict_id")
        .uri("/wql/tx")
        .to_request();
    let _ = test::call_service(&mut app, req).await;

    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload("INSERT {a: 123, b: 12.3,} INTO test_evict_id")
        .uri("/wql/tx")
        .to_request();

    let mut resp_insert = test::call_service(&mut app, req).await;
    let body = resp_insert.take_body().as_str().to_string();
    let uuid = &body[(body.len() - 36)..];
    assert!(resp_insert.status().is_success());

    let evict = format!("Evict {} from test_evict_id", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(evict)
        .uri("/wql/tx")
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
    read::assert_content("EVICT_ENTITY_ID|");
    read::assert_content("|test_evict_id;");

    let payload = format!("UPDATE test_evict_id SET {{a: 12, c: Nil,}} INTO {}", uuid);
    let req = test::TestRequest::post()
        .header("Content-Type", "application/wql")
        .set_payload(payload)
        .uri("/wql/tx")
        .to_request();

    let mut resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
    let body = resp.take_body().as_str().to_string();

    assert_eq!(
        body,
        format!("Uuid {} not created for entity test_evict_id", uuid)
    );
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

fn clear() {
    std::process::Command::new("rm")
        .arg("-rf")
        .arg("*.log")
        .output()
        .expect("failed to execute process")
        .stdout;
}
