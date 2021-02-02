use crate::actors::{
    state::{PreviousRegistry, State},
    uniques::{CreateUniques, WriteUniques},
    wql::{
        DeleteId, Executor, InsertEntityContent, UpdateContentEntityContent, UpdateSetEntityContent,
    },
};
use crate::actors::{uniques::CheckForUnique, wql::CreateEntity};
use crate::model::{error::Error, DataRegister};
use crate::repository::local::{LocalContext, UniquenessContext};

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
    uniqueness: web::Data<Arc<Mutex<UniquenessContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> impl Responder {
    let query = wql::Wql::from_str(&body);
    let response = match query {
        Ok(Wql::CreateEntity(entity, uniques)) => {
            let _ = create_unique_controller(&entity, uniques, uniqueness, &actor).await;
            create_controller(entity, data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::Delete(entity, uuid)) => {
            delete_controller(entity, uuid, data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::Insert(entity, content)) => {
            insert_controller(
                entity,
                content,
                data.into_inner(),
                bytes_counter,
                uniqueness,
                actor,
            )
            .await
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

pub async fn create_unique_controller(
    entity: &str,
    uniques: Vec<String>,
    uniqueness: web::Data<Arc<Mutex<UniquenessContext>>>,
    actor: &web::Data<Addr<Executor>>,
) -> Result<(), Error> {
    if uniques.is_empty() {
        Ok(())
    } else {
        let data = uniqueness.into_inner();
        actor
            .send(WriteUniques {
                entity: entity.to_string(),
                uniques: uniques.clone(),
            })
            .await
            .unwrap()?;
        actor
            .send(CreateUniques {
                entity: entity.to_string(),
                uniques,
                data,
            })
            .await
            .unwrap()?;
        Ok(())
    }
}

pub async fn insert_controller(
    entity: String,
    content: HashMap<String, Types>,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    uniqueness: web::Data<Arc<Mutex<UniquenessContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let offset = bytes_counter.load(Ordering::SeqCst);
    let content_log =
        to_string_pretty(&content, pretty_config()).map_err(|e| Error::SerializationError(e))?;

    let mut data = data.lock().unwrap();
    if !data.contains_key(&entity) {
        return Err(Error::EntityNotCreated(entity));
    }

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUnique {
            entity: entity.clone(),
            content,
            uniqueness,
        })
        .await
        .unwrap()?;

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
            previous_registry: to_string_pretty(&previous_entry.clone(), pretty_config())
                .map_err(|e| Error::SerializationError(e))?,
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
                        local
                            .entry(key.to_string())
                            .and_modify(|v| *v = value.to_owned())
                            .or_insert(value.to_owned());
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
            previous_registry: to_string_pretty(&previous_entry.clone(), pretty_config())
                .map_err(|e| Error::SerializationError(e))?,
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

pub async fn delete_controller(
    entity: String,
    id: String,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let uuid = Uuid::from_str(&id).unwrap();
    let offset = bytes_counter.load(Ordering::SeqCst);
    // let content_log =
    //     to_string_pretty(&content, pretty_config()).map_err(|e| Error::SerializationError(e))?;

    let mut data = data.lock().unwrap();
    if !data.contains_key(&entity) {
        return Err(Error::EntityNotCreated(entity));
    } else if data.contains_key(&entity) && !data.get(&entity).unwrap().contains_key(&uuid) {
        return Err(Error::UuidNotCreatedForEntity(entity, uuid));
    }

    let previous_entry = data.get(&entity).unwrap().get(&uuid).unwrap();
    let previous_state_str = actor.send(previous_entry.clone()).await.unwrap()?;
    let two_registries_ago = actor
        .send(PreviousRegistry(previous_state_str))
        .await
        .unwrap()?;

    let state_to_be = match two_registries_ago {
        Some(reg) => {
            let state_str = actor.send(reg.clone()).await.unwrap()?;
            (
                actor.send(State(state_str.clone())).await.unwrap()?,
                reg.to_owned(),
            )
        }
        None => {
            let insert_reg = data.get(&entity).unwrap().get(&uuid).unwrap();
            // let state_str = actor.send(insert_reg.clone().to_owned()).await.unwrap()?;
            // (actor
            //     .send(State(state_str.clone()))
            //     .await
            //     .unwrap()?, insert_reg.to_owned())
            (HashMap::new(), insert_reg.to_owned())
        }
    };
    let content_log = to_string_pretty(&state_to_be.0, pretty_config())
        .map_err(|e| Error::SerializationError(e))?;

    let previous_register_log = to_string_pretty(&state_to_be.1, pretty_config())
        .map_err(|e| Error::SerializationError(e))?;

    let content_value = actor
        .send(DeleteId {
            name: entity.clone(),
            content_log,
            uuid,
            previous_registry: previous_register_log,
        })
        .await
        .unwrap()?;

    let data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&entity) {
        if let Some(reg) = map.get_mut(&uuid) {
            *reg = data_register;
        }
    }

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);

    Ok(format!("Entity {} with Uuid {} deleted", entity, id))
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
    async fn test_create_uniques_post_ok() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("CREATE ENTITY test_uniques UNIQUES name, ssn, id")
            .uri("/wql/query")
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
    async fn test_insert_unique_post_ok() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("CREATE ENTITY test_insert_unique UNIQUES id")
            .uri("/wql/query")
            .to_request();

        let _ = test::call_service(&mut app, req).await;

        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("INSERT {id: 123, a: \"hello\",} INTO test_insert_unique")
            .uri("/wql/query")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("INSERT {id: 123, a: \"world\",} INTO test_insert_unique")
            .uri("/wql/query")
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

    #[ignore]
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

    #[ignore]
    #[actix_rt::test]
    async fn test_delete_post_ok() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("CREATE ENTITY test_delete")
            .uri("/wql/query")
            .to_request();

        let _ = test::call_service(&mut app, req).await;

        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("INSERT {a: 123, b: 12.3,} INTO test_delete")
            .uri("/wql/query")
            .to_request();

        let mut resp_insert = test::call_service(&mut app, req).await;
        let body = resp_insert.take_body().as_str().to_string();
        let uuid = &body[(body.len() - 36)..];

        let payload = format!("UPDATE test_delete SET {{a: 12, c: Nil,}} INTO {}", uuid);
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(payload)
            .uri("/wql/query")
            .to_request();

        let _ = test::call_service(&mut app, req).await;

        let payload = format!("Delete {} FROM test_delete", uuid);
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(payload)
            .uri("/wql/query")
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
            .uri("/wql/query")
            .to_request();

        let _ = test::call_service(&mut app, req).await;

        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload("INSERT {a: 123, b: 12.3,} INTO test_delete")
            .uri("/wql/query")
            .to_request();

        let mut resp_insert = test::call_service(&mut app, req).await;
        let body = resp_insert.take_body().as_str().to_string();
        let uuid = &body[(body.len() - 36)..];

        let payload = format!("Delete {} FROM test_delete", uuid);
        let req = test::TestRequest::post()
            .header("Content-Type", "application/wql")
            .set_payload(payload)
            .uri("/wql/query")
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());

        read::assert_content("DELETE");
        read::assert_content("|{}|");
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
