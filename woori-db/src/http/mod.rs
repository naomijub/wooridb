use crate::{
    actors::wql::Executor,
    repository::local::{LocalContext, UniquenessContext},
};
use crate::{
    controllers::{query, tx},
    repository::local::EncryptContext,
};
use actix::Actor;
use actix_web::{get, guard, web, HttpResponse, Responder};
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

#[get("/ping")]
pub async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong!")
}

#[get("/~/ready")]
pub async fn readiness() -> impl Responder {
    let process = std::process::Command::new("sh")
        .arg("-c")
        .arg("echo hello")
        .output();

    match process {
        Ok(_) => HttpResponse::Accepted(),
        Err(_) => HttpResponse::InternalServerError(),
    }
}

pub fn routes(config: &mut web::ServiceConfig) {
    let wql_context = Arc::new(Mutex::new(LocalContext::new()));
    let unique_context = Arc::new(Mutex::new(UniquenessContext::new()));
    let encrypt_context = Arc::new(Mutex::new(EncryptContext::new()));
    let write_offset = AtomicUsize::new(0usize);
    let actor = Executor::new().start();

    // Deactivate scheduler for now
    // Scheduler.start();

    config
        .service(
            web::scope("/wql")
                .guard(guard::Header("Content-Type", "application/wql"))
                .data(wql_context)
                .data(unique_context)
                .data(encrypt_context)
                .data(write_offset)
                .data(actor)
                .route("/tx", web::post().to(tx::wql_handler))
                .route("/query", web::post().to(query::wql_handler)),
        )
        .route("", web::get().to(HttpResponse::NotFound));
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::{body::Body, test, App};

    #[actix_rt::test]
    async fn test_ping_get() {
        let mut app = test::init_service(App::new().service(ping)).await;
        let req = test::TestRequest::get().uri("/ping").to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
        let body = resp.take_body();
        let body = body.as_ref().unwrap();
        assert_eq!(&Body::from("pong!"), body)
    }

    #[actix_rt::test]
    async fn test_ready_get() {
        let mut app = test::init_service(App::new().service(readiness)).await;
        let req = test::TestRequest::get().uri("/~/ready").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
}
