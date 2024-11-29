use actix_web::{web, App, HttpResponse, HttpServer};
use tracing::instrument;
use xrf1::configs::load_config;
use xrf1::telemetry::tracing_setup;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = load_config().expect("Failed to load configurations");
    let _guard = tracing_setup(&config.app.name, config.log);

    let address = format!("{}:{}", &config.app.host, &config.app.port);

    HttpServer::new(|| App::new().route("/health", web::get().to(get_app_health)))
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
