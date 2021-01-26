use actix_web::{ HttpResponse, Responder, web};
use std::{collections::BTreeMap, sync::{Arc, Mutex}};
use crate::repository::local::LocalContext;

pub async fn wql_handler(body: String, data: web::Data<Arc<Mutex<LocalContext>>>) -> impl Responder {
    let query = body;
    let response = match true {
        _ if query.starts_with("CREATE ENTITY ") => create_controller(query, data.into_inner()).await,
        _ => Err(format!("Query \n ```{}``` \n has illegal arguments", query))
    };

    match response {
        Err(e) => HttpResponse::BadRequest().body(e),
        Ok(resp) => HttpResponse::Ok().body(resp)
    }
}

pub async fn create_controller(query: String, data: Arc<Arc<Mutex<LocalContext>>>) -> Result<String, String> {
    let entity = query[14..].chars()
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>();
    let mut data = data.lock().unwrap();
    data.insert(entity.trim().to_string(), BTreeMap::new());

    Ok(format!("Entity {} created", entity))
}
