use crate::actors::{
    state::State,
    wql::{Executor, InsertEntityContent, UpdateContentEntityContent, UpdateSetEntityContent},
};
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
use uuid::Uuid;
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
        Ok(Wql::UpdateContent(entity, content, uuid)) => {
            update_content_controller(
                entity,
                content,
                uuid,
                data.into_inner(),
                bytes_counter,
                actor,
            )
            .await
        }
        Ok(Wql::UpdateSet(entity, content, uuid)) => {
            update_set_controller(
                entity,
                content,
                uuid,
                data.into_inner(),
                bytes_counter,
                actor,
            )
            .await
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

    Ok(format!(
        "Entity {} inserted with Uuid {}",
        entity, content_value.1
    ))
}

pub async fn update_set_controller(
    entity: String,
    content: HashMap<String, Types>,
    id: Uuid,
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
    } else if data.contains_key(&entity) && !data.get(&entity).unwrap().contains_key(&id) {
        return Err(Error::UuidNotCreatedForEntity(entity, id));
    }

    let previous_entry = data.get(&entity).unwrap().get(&id).unwrap();
    let previous_state_str = actor.send(previous_entry.clone()).await.unwrap()?;
    let mut previous_state = actor
        .send(State(previous_state_str.clone()))
        .await
        .unwrap()?;

    content.into_iter().for_each(|(k, v)| {
        let local_state = previous_state.entry(k).or_insert(v.clone());
        *local_state = v;
    });

    let state_log = to_string_pretty(&previous_state, pretty_config())
        .map_err(|e| Error::SerializationError(e))?;

    let content_value = actor
        .send(UpdateSetEntityContent {
            name: entity.clone(),
            current_state: state_log,
            content_log,
            id,
            previous_registry: previous_entry.clone(),
        })
        .await
        .unwrap()?;

    let data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&entity) {
        if let Some(reg) = map.get_mut(&id) {
            *reg = data_register;
        }
    }

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);

    Ok(format!("Entity {} with Uuid {} updated", entity, id))
}

pub async fn update_content_controller(
    entity: String,
    content: HashMap<String, Types>,
    id: Uuid,
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
    } else if data.contains_key(&entity) && !data.get(&entity).unwrap().contains_key(&id) {
        return Err(Error::UuidNotCreatedForEntity(entity, id));
    }

    let previous_entry = data.get(&entity).unwrap().get(&id).unwrap();
    let previous_state_str = actor.send(previous_entry.clone()).await.unwrap()?;
    let mut previous_state = actor
        .send(State(previous_state_str.clone()))
        .await
        .unwrap()?;

    content.into_iter().for_each(|(k, v)| {
        let local_state = previous_state.entry(k).or_insert(v.clone());
        match v {
            Types::Char(c) => {
                *local_state = Types::Char(c);
            }
            Types::Integer(i) => {
                if let Types::Integer(local) = *local_state {
                    *local_state = Types::Integer(local + i);
                }
            }
            Types::String(s) => {
                if let Types::String(local) = local_state {
                    *local_state = Types::String(local.to_string() + &s);
                }
            }
            Types::Uuid(uuid) => {
                *local_state = Types::Uuid(uuid);
            }
            Types::Float(f) => {
                if let Types::Float(local) = *local_state {
                    *local_state = Types::Float(local + f);
                }
            }
            Types::Boolean(b) => {
                *local_state = Types::Boolean(b);
            }
            Types::Vector(mut v) => {
                if let Types::Vector(local) = local_state {
                    local.append(&mut v);
                    *local_state = Types::Vector(local.to_owned());
                }
            }
            Types::Map(m) => {
                if let Types::Map(local) = local_state {
                    m.iter().for_each(|(key, value)| {
                        let map_key = local.entry(key.to_string()).or_insert(value.to_owned());
                        *map_key = value.to_owned();
                    });
                    *local_state = Types::Map(local.to_owned());
                }
            }
            Types::Nil => {
                *local_state = Types::Nil;
            }
        }
    });

    let state_log = to_string_pretty(&previous_state, pretty_config())
        .map_err(|e| Error::SerializationError(e))?;

    let content_value = actor
        .send(UpdateContentEntityContent {
            name: entity.clone(),
            current_state: state_log,
            content_log,
            id,
            previous_registry: previous_entry.clone(),
        })
        .await
        .unwrap()?;

    let data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&entity) {
        if let Some(reg) = map.get_mut(&id) {
            *reg = data_register;
        }
    }

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);

    Ok(format!("Entity {} with Uuid {} updated", entity, id))
}

#[cfg(test)]
mod test {
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
            .uri("/wql/query")
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
        clear();
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
        clear();
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
        assert_eq!(&Body::from("\"Symbol `DO` not implemented\""), body);
        clear();
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
        read::assert_content("|test_ok|{\"a\": Integer(123),};");
        clear();
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
        clear();
    }

    #[actix_rt::test]
    async fn test_update_set_post_ok() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("CREATE ENTITY test_update")
            .uri("/wql/query")
            .to_request();

        let _ = test::call_service(&mut app, req).await;

        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("INSERT {a: 123, b: 12.3,} INTO test_update")
            .uri("/wql/query")
            .to_request();

        let mut resp_insert = test::call_service(&mut app, req).await;
        let body = resp_insert.take_body().as_str().to_string();
        let uuid = &body[(body.len() - 36)..];

        let payload = format!("UPDATE test_update SET {{a: 12, c: Nil,}} INTO {}", uuid);
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(payload)
            .uri("/wql/query")
            .to_request();

        let mut resp = test::call_service(&mut app, req).await;
        // assert_eq!("set", resp.take_body().as_str());

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

    #[actix_rt::test]
    async fn test_update_content_post_ok() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("CREATE ENTITY test_update")
            .uri("/wql/query")
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
            .uri("/wql/query")
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
            .uri("/wql/query")
            .to_request();

        let mut resp = test::call_service(&mut app, req).await;
        // assert_eq!("content", resp.take_body().as_str());
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
            .uri("/wql/query")
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
            .uri("/wql/query")
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
            .uri("/wql/query")
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
            .uri("/wql/query")
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
            .uri("/wql/query")
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
            .uri("/wql/query")
            .to_request();

        let mut resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());
        let body = resp.take_body();
        let body = body.as_str();
        assert!(body.contains("not created for entity test_update"));
        assert!(body.contains("Uuid"));
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
}
