use actix_web::{
    middleware::{DefaultHeaders, Logger},
    web, App, HttpResponse, HttpServer,
};

mod controllers;
mod http;
mod repository;
mod model;
mod io;
mod core;

use http::{ping, readiness, routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace");
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
    .workers(num_cpus::get_physical() + 2)
    .run()
    .await
}
