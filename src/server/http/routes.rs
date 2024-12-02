use actix_web::HttpResponse;
use tracing::instrument;

#[instrument]
pub async fn get_app_health() -> HttpResponse {
    tracing::info!("GET /health");
    HttpResponse::Ok()
        .content_type("application/json")
        .body("healthy")
}
