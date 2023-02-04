use actix_web::{get, web, App, HttpServer, Responder};

#[get("/{route}")]
async fn greet(route: web::Path<String>) -> impl Responder {
    format!("Hello {route}!")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(greet)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}