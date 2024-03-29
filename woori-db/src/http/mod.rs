#[cfg(not(debug_assertions))]
use crate::auth::{
    controllers as auth,
    io::read_admin_info,
    middlewares::{history_validator, wql_validator},
};
#[cfg(not(debug_assertions))]
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::{
    actors::{scheduler::Scheduler, wql::Executor},
    controllers::entity_history,
    io::read::{encryption, local_data, offset, unique_data},
    repository::local::{LocalContext, SessionContext, UniquenessContext},
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
    let local_context = local_data().map_or(LocalContext::new(), |map| map);
    let encrypt_context = encryption().map_or(EncryptContext::new(), |e| e);
    let uniqueness = unique_data().map_or(UniquenessContext::new(), |u| u);
    let wql_context = Arc::new(Mutex::new(local_context));
    let unique_context = Arc::new(Mutex::new(uniqueness));
    let encrypt_context = Arc::new(Mutex::new(encrypt_context));
    let write_offset = AtomicUsize::new(offset().map_or(0_usize, |o| o));
    let actor = Executor::new().start();
    let env_cost = std::env::var("HASHING_COST").unwrap_or_else(|_| "14".to_owned());
    let cost = env_cost.parse::<u32>().expect("HASHING_COST must be a u32");

    let session_context = Arc::new(Mutex::new(SessionContext::new()));

    #[cfg(not(debug_assertions))]
    let exp_time_str =
        std::env::var("SESSION_EXPIRATION_TIME").unwrap_or_else(|_| "3600".to_owned());
    #[cfg(not(debug_assertions))]
    let exp_time = exp_time_str.parse::<i64>().unwrap_or(3600);

    #[cfg(not(debug_assertions))]
    let admin_info = read_admin_info().unwrap();

    Scheduler.start();

    #[cfg(not(debug_assertions))]
    let wql_auth = HttpAuthentication::bearer(wql_validator);
    #[cfg(not(debug_assertions))]
    let history_auth = HttpAuthentication::bearer(history_validator);

    #[cfg(not(debug_assertions))]
    config
        .data(session_context)
        .data(wql_context)
        .data(actor)
        .service(
            web::scope("/auth")
                .data(admin_info)
                .data(exp_time)
                .route("/createUser", web::post().to(auth::create_user))
                .route("/deleteUsers", web::post().to(auth::delete_users))
                .route("/putUserSession", web::put().to(auth::put_user_session)),
        )
        .service(
            web::scope("/wql")
                .guard(guard::Header("Content-Type", "application/wql"))
                .data(cost)
                .data(unique_context)
                .data(encrypt_context)
                .data(write_offset)
                .wrap(wql_auth)
                .route("/tx", web::post().to(tx::wql_handler))
                .route("/query", web::post().to(query::wql_handler)),
        )
        .service(
            web::scope("/entity-history")
                .wrap(history_auth)
                .route("", web::post().to(entity_history::history_handler)),
        )
        .route("", web::get().to(HttpResponse::NotFound));

    #[cfg(debug_assertions)]
    config
        .data(session_context)
        .data(wql_context)
        .data(actor)
        .service(
            web::scope("/wql")
                .guard(guard::Header("Content-Type", "application/wql"))
                .data(cost)
                .data(unique_context)
                .data(encrypt_context)
                .data(write_offset)
                .route("/tx", web::post().to(tx::wql_handler))
                .route("/query", web::post().to(query::wql_handler)),
        )
        .route(
            "/entity-history",
            web::post().to(entity_history::history_handler),
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
