use actix_web::{ HttpResponse, Responder};

pub async fn wql_handler(query: String) -> impl Responder {
    let response = match true {
        _ if query.starts_with("CREATE ENTITY ") => create_controller(query).await,
        _ => Err(format!("Query \n ```{}``` \n has illegal arguments", query))
    };

    match response {
        Err(e) => HttpResponse::BadRequest().body(e),
        Ok(resp) => HttpResponse::Ok().body(resp)
    }
}

pub async fn create_controller(query: String) -> Result<String, String> {
    let entity = &query[13..].chars()
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>();
    Ok(format!("Entity {} created", entity))
}
