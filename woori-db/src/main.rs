use actix_web::{App, HttpServer};

mod http;

use http::{ping, readiness};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(ping)
            .service(readiness)
    })
    .bind("0.0.0.0:1438")?
    .run()
    .await
}