use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};

mod controllers;
mod http;
mod repository;

use http::{ping, readiness, routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
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
