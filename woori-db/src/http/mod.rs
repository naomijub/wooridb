use actix_web::{get, guard, web, HttpResponse, Responder};

use crate::controllers::wql::wql_handler;

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
    config.service(
        web::scope("/wql")
            // .guard(guard::Header("Content-Type", "application/wql"))
            .route("/query", web::post().to(wql_handler)),
            )
            .route("", web::get().to(|| HttpResponse::NotFound()),
    );
}