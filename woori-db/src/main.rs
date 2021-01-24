use actix_web::{App, HttpServer, web, HttpResponse};

mod http;
mod controllers;

use http::{ping, readiness, routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
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