use actix_web::{web, App, HttpResponse, HttpServer};

#[tokio::main]
async fn main() -> std::io::Result<()> {
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

async fn get_app_health() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("healthy")
}
