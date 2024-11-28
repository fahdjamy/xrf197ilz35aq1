use actix_web::{web, App, HttpResponse, HttpServer};
use tracing::instrument;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_setup();

    let port = 8080;
    let host = "127.0.0.1";
    let address = format!("{}:{}", host, port);

    HttpServer::new(|| {
        App::new().route("/health", web::get().to(get_app_health))
    })
        .bind(address)?
        .run()
        .await
}

#[instrument]
async fn get_app_health() -> HttpResponse {
    tracing::info!("GET /health");
    HttpResponse::Ok()
        .content_type("application/json")
        .body("healthy")
}

fn tracing_setup() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}
