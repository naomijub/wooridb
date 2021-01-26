use crate::repository::local::LocalContext;
use actix_web::{web, HttpResponse, Responder};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

pub async fn wql_handler(
    body: String,
    data: web::Data<Arc<Mutex<LocalContext>>>,
) -> impl Responder {
    let query = body;
    let response = match true {
        _ if query.starts_with("CREATE ENTITY ") => {
            create_controller(query, data.into_inner()).await
        }
        _ => Err(format!("Query \n ```{}``` \n has illegal arguments", query)),
    };

    match response {
        Err(e) => HttpResponse::BadRequest().body(e),
        Ok(resp) => HttpResponse::Ok().body(resp),
    }
}

pub async fn create_controller(
    query: String,
    data: Arc<Arc<Mutex<LocalContext>>>,
) -> Result<String, String> {
    let entity = query[14..]
        .chars()
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>();
    let mut data = data.lock().unwrap();
    data.insert(entity.trim().to_string(), BTreeMap::new());

    Ok(format!("Entity {} created", entity))
}

#[cfg(test)]
mod test {
    use crate::http::routes;
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
        assert_eq!(&Body::from("Entity test_ok created"), body)
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
            &Body::from("Query \n ```DO SOMETHIG weird``` \n has illegal arguments"),
            body
        )
    }
}
