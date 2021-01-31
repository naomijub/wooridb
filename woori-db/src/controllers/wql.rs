use crate::actors::wql::{Executor, InsertEntityContent};
use crate::model::{error::Error, DataRegister};
use crate::repository::local::LocalContext;

use actix::Addr;
use actix_web::{web, HttpResponse, Responder};
use ron::ser::{to_string_pretty, PrettyConfig};
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use wql::{Types, Wql};

fn pretty_config() -> PrettyConfig {
    PrettyConfig::new()
        .with_indentor("".to_string())
        .with_new_line("".to_string())
}

pub async fn wql_handler(
    body: String,
    data: web::Data<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> impl Responder {
    let query = wql::Wql::from_str(&body);
    let response = match query {
        Ok(Wql::CreateEntity(entity)) => {
            create_controller(entity, data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::Insert(entity, content)) => {
            insert_controller(entity, content, data.into_inner(), bytes_counter, actor).await
        }
        Err(e) => Err(Error::QueryFormat(e)),
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

    #[cfg(test)]
    {
        assert_eq!(22, bytes_counter.load(Ordering::SeqCst));
        let data_str = format!("{:?}", data);
        assert_eq!(data_str, "{\"test_ok\": {}}");
        println!("TEST ONLY");
    }

    Ok(format!("Entity {} created", entity))
}

pub async fn insert_controller(
    entity: String,
    content: HashMap<String, Types>,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let offset = bytes_counter.load(Ordering::SeqCst);
    let content_log =
        to_string_pretty(&content, pretty_config()).map_err(|e| Error::SerializationError(e))?;

    let mut data = data.lock().unwrap();
    if !data.contains_key(&entity) {
        return Err(Error::EntityNotCreated(entity));
    }

    let content_value = actor
        .send(InsertEntityContent {
            name: entity.clone(),
            content: content_log,
        })
        .await
        .unwrap()?;
    let data_register = DataRegister {
        offset,
        bytes_length: content_value.2,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&entity) {
        map.insert(content_value.1, data_register);
    }

    bytes_counter.fetch_add(content_value.2, Ordering::SeqCst);

    #[cfg(test)]
    {
        assert!(bytes_counter.load(Ordering::SeqCst) >= 126);
        let data_str = format!("{:?}", data);
        assert!(data_str.contains("offset: 22, bytes_length: 104"));
        println!("TEST ONLY");
    }

    Ok(format!(
        "Entity {} inserted with Uuid {}",
        entity, content_value.1
    ))
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
        read::assert_content("CREATE_ENTITY|test_ok;");
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
        assert_eq!(&Body::from("\"Symbol `DO` not implemented\""), body)
    }

    #[actix_rt::test]
    async fn test_insert_post_ok() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("CREATE ENTITY test_ok")
            .uri("/wql/query")
            .to_request();

        let _ = test::call_service(&mut app, req).await;

        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("INSERT {a: 123,} INTO test_ok")
            .uri("/wql/query")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        read::assert_content("INSERT|");
        read::assert_content("UTC|");
        read::assert_content("|test_ok|{\"a\": Integer(123),};")
    }

    #[actix_rt::test]
    async fn test_insert_entity_not_created() {
        let mut app = test::init_service(App::new().configure(routes)).await;

        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("INSERT {a: 123,} INTO missing")
            .uri("/wql/query")
            .to_request();

        let mut resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());
        let body = resp.take_body();
        let body = body.as_ref().unwrap();
        assert_eq!(&Body::from("Entity `missing` not created"), body);
    }
}
