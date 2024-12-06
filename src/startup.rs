use crate::configs::{Configurations, DatabaseConfig};
use crate::server::http::server::create_http_server;
use crate::server::GrpcServer;
use actix_web::dev::Server;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub struct Application {
    http_server: Server,
    grpc_server: GrpcServer
}

impl Application {
    pub async fn build(config: Configurations) -> Result<Self, std::io::Error> {
        let http_server = create_http_server(config.app).await?;
        let connection_pool = get_connection_pool(&config.database);
        let grpc_server = GrpcServer::new(connection_pool, config.server.grpc);
        Ok(Self { http_server, grpc_server })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.http_server.await?;
        self.grpc_server.start().await.expect("Failed to start grpc server");
        Ok(())
    }
}

pub fn get_connection_pool(configuration: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new()
        .connect_lazy_with(configuration.postgres.connect_to_database())
}
