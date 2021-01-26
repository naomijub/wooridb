use actix_web::{ HttpResponse, Responder, web};
use futures::{StreamExt};
use std::{collections::BTreeMap, sync::{Arc, Mutex}};
use crate::repository::local::LocalContext;


pub async fn wql_handler(mut query: web::Payload, data: web::Data<Arc<Mutex<LocalContext>>>) -> impl Responder {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = query.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }

    let query = format!("{:?}!", bytes);
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
    let entity = &query[13..].chars()
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>();
    let mut data = data.lock().unwrap();
    data.insert(entity.to_string(), BTreeMap::new());
    println!("{:?}", data);
    Ok(format!("Entity {} created", entity))
}
