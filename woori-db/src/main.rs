use actix_web::{
    middleware::{DefaultHeaders, Logger},
    web, App, HttpResponse, HttpServer,
};

mod actors;
mod controllers;
mod core;
mod http;
mod io;
mod model;
mod repository;

use http::{ping, readiness, routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(DefaultHeaders::new().header("x-request-id", uuid::Uuid::new_v4().to_string()))
            .wrap(Logger::new("IP:%a DATETIME:%t REQUEST:\"%r\" STATUS: %s DURATION:%D X-REQUEST-ID:%{x-request-id}o"))
            .service(ping)
            .service(readiness)
            .configure(routes)
            .route("", web::get().to(|| HttpResponse::NotFound()))
    })
    .bind("0.0.0.0:1438")?
    .workers(1)
    .run()
    .await
}
