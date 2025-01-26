use crate::configs::{Configurations, DatabaseConfig, HttpServerConfig};
use crate::server::http::server::create_http_server;
use crate::server::GrpcServer;
use actix_web::dev::Server;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;

pub struct HttpServer {
    pub server: Server,
}

impl HttpServer {
    pub async fn new(config: &HttpServerConfig) -> Result<Self, std::io::Error> {
        info!("starting HTTP server :: port {}", config.port);
        let http_server = create_http_server(&config).await?;
        Ok(HttpServer { server: http_server })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct Application {
    pub http_server: HttpServer,
    pub grpc_server: GrpcServer
}

impl Application {
    pub async fn build(config: Configurations) -> Result<Self, anyhow::Error> {
        let http_server = HttpServer::new(&config.server.http).await?;

        let connection_pool = get_connection_pool(&config.database);
        info!("connected to database successfully :: {}", &config.database.postgres.name);
        let grpc_server = GrpcServer::new(connection_pool, config.server.grpc)?;

        Ok(Self { http_server, grpc_server })
    }
}

pub fn get_connection_pool(configuration: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new()
        .connect_lazy_with(configuration.postgres.connect_to_database(&configuration.postgres.name))
}
