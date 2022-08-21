use actix_web::{
    middleware::{DefaultHeaders, Logger},
    web, App, HttpResponse, HttpServer,
};

mod actors;
#[allow(dead_code)]
mod auth;
mod controllers;
mod core;
mod http;
mod io;
mod model;
mod repository;
mod schemas;

use http::{ping, readiness, routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let env_port = std::env::var("PORT").unwrap_or_else(|_| "1438".to_owned());
    let port = env_port.parse::<u16>().expect("PORT must be a u16");
    let addr = format!("0.0.0.0:{}", port);
    let env_max_connections =
        std::env::var("MAX_CONNECTIONS").unwrap_or_else(|_| "1000".to_owned());
    let max_connections = env_max_connections
        .parse::<usize>()
        .expect("MAX_CONNECTIONS must be a usize");
    let env_client_shutdown =
        std::env::var("CLIENT_SHUTDOWN").unwrap_or_else(|_| "5000".to_owned());
    let client_shutdown = env_client_shutdown
        .parse::<u64>()
        .expect("CLIENT_SHUTDOWN must be a u64");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(DefaultHeaders::new().header("x-request-id", uuid::Uuid::new_v4().to_string()))
            .wrap(Logger::new("IP:%a DATETIME:%t REQUEST:\"%r\" STATUS: %s DURATION:%T X-REQUEST-ID:%{x-request-id}o"))
            .service(ping)
            .service(readiness)
            .configure(routes)
            .route("", web::get().to(HttpResponse::NotFound))
    })
    .max_connections(max_connections)
    .client_shutdown(client_shutdown)
    .workers(1)
    .bind(addr)?
    .run()
    .await
}
