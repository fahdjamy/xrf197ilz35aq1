use crate::configs::Application;
use crate::server::http::get_app_health;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};

pub async fn create_http_server(app: Application) -> Result<Server, std::io::Error> {
    let address = format!("{}:{}", &app.host, &app.port);

    let server = HttpServer::new(|| App::new().route("/health", web::get().to(get_app_health)))
        .bind(address)?
        .run();

    Ok(server)
}
