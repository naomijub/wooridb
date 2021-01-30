use crate::actors::wql::Executor;
use crate::model::error::Error;
use crate::repository::local::LocalContext;

use actix::Addr;
use actix_web::{web, HttpResponse, Responder};
use std::{collections::BTreeMap, str::FromStr, sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    }};
use wql::Wql;

pub async fn wql_handler(
    body: String,
    data: web::Data<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> impl Responder {
    let query = wql::Wql::from_str(&body);
    let response = match query {
        Ok(Wql::CreateEntity(entity)) => create_controller(entity, data.into_inner(), bytes_counter, actor).await,
        Ok(_) =>  Err(Error::QueryFormat(format!(
            "Query \n ```{}``` \n has illegal arguments",
            body
        ))),
        Err(e) => Err(Error::QueryFormat(e))
    };

    match response {
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        Ok(resp) => HttpResponse::Ok().body(resp),
    }
}
use crate::actors::wql::CreateEntity;
pub async fn create_controller(
    entity: String,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let mut data = data.lock().unwrap();
    if !data.contains_key(&entity) {
        data.insert(entity.clone(), BTreeMap::new());
    } else {
        return Err(Error::EntityAlreadyCreated(entity));
    }

    let offset = actor
        .send(CreateEntity {
            name: entity.clone(),
        })
        .await
        .unwrap()?;

    bytes_counter.fetch_add(offset, Ordering::SeqCst);

    Ok(format!("Entity {} created", entity))
}

#[cfg(test)]
mod test {
    use crate::http::routes;
    use crate::io::read;
    use actix_web::{body::Body, test, App};

    #[actix_rt::test]
    async fn test_create_post_ok() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("CREATE ENTITY test_ok")
            .uri("/wql/query")
            .to_request();

        let mut resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
        let body = resp.take_body();
        let body = body.as_ref().unwrap();
        assert_eq!(&Body::from("Entity test_ok created"), body);
        read::assert_content("CREATE_ENTITY|test_ok");
    }

    #[actix_rt::test]
    async fn test_create_post_duplicated_err() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("CREATE ENTITY test_ok")
            .uri("/wql/query")
            .to_request();

        let _ = test::call_service(&mut app, req).await;

        let duplicated_req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("CREATE ENTITY test_ok")
            .uri("/wql/query")
            .to_request();
        let mut resp = test::call_service(&mut app, duplicated_req).await;

        assert!(resp.status().is_client_error());
        let body = resp.take_body();
        let body = body.as_ref().unwrap();
        assert_eq!(&Body::from("Entity `test_ok` already created"), body);
    }

    #[actix_rt::test]
    async fn test_create_post_bad_request() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .header("Content-Type", "text/plain")
            .set_payload("CREATE ENTITY test_ok")
            .uri("/wql/query")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_unkwon_wql_post() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("DO SOMETHIG weird")
            .uri("/wql/query")
            .to_request();

        let mut resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());
        let body = resp.take_body();
        let body = body.as_ref().unwrap();
        assert_eq!(
            &Body::from("\"Symbol `DO` not implemented\""),
            body
        )
    }
}
