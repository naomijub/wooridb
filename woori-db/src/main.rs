use actix_web::{App, HttpServer, web, HttpResponse};
use std::sync::{Arc, Mutex};

mod http;
mod controllers;
mod repository;

use http::{ping, readiness, routes};
use repository::local::LocalContext;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace");
    env_logger::init();
    
    let context = Arc::new(Mutex::new(LocalContext::new()));
    HttpServer::new(move || {
        App::new()
            .service(ping)
            .data(context.clone())
            .service(readiness)
            .configure(routes)
            .route("", web::get().to(|| HttpResponse::NotFound()))
    })
    .bind("0.0.0.0:1438")?
    .workers(num_cpus::get_physical() + 2)
    .run()
    .await
}