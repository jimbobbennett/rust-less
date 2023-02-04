use actix_web::{get, App, HttpServer, Responder};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// The port to start up
   #[arg(short, long)]
   port: u16,
}

/// This route is used as a test to ensure the server is running. It will return "Hello!"
#[get("/hello")]
async fn greet() -> impl Responder {
    format!("Hello from the example function app!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Create and start the server
    HttpServer::new(|| {
        App::new().service(greet)
    })
    .bind(("0.0.0.0", args.port))?
    .run()
    .await
}