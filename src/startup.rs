use crate::configs::Configurations;
use crate::server::http::server::create_http_server;
use actix_web::dev::Server;

pub struct Application {
    http_server: Server,
}

impl Application {
    pub async fn build(config: Configurations) -> Result<Self, std::io::Error> {
        let http_server = create_http_server(config.app).await?;
        Ok(Self { http_server })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.http_server.await
    }
}
