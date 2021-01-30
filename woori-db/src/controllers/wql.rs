use crate::actors::wql::Executor;
use crate::model::error::Error;
use crate::repository::local::LocalContext;

use actix::Addr;
use actix_web::{web, HttpResponse, Responder};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

pub async fn wql_handler(
    body: String,
    data: web::Data<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> impl Responder {
    let query = body;
    let response = match true {
        _ if query.starts_with("CREATE ENTITY ") => {
            create_controller(&query[14..], data.into_inner(), bytes_counter, actor).await
        }
        _ => Err(Error::QueryFormat(format!(
            "Query \n ```{}``` \n has illegal arguments",
            query
        ))),
    };

    match response {
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        Ok(resp) => HttpResponse::Ok().body(resp),
    }
}
use crate::actors::wql::CreateEntity;
pub async fn create_controller(
    query: &str,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let entity = query
        .chars()
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>()
        .trim()
        .to_string();
    let mut data = data.lock().unwrap();
    data.insert(entity.clone(), BTreeMap::new());

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
            &Body::from("\"Query \\n ```DO SOMETHIG weird``` \\n has illegal arguments\""),
            body
        )
    }
}
