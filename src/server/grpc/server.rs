use crate::configs::GrpcServerConfig;
use crate::core::{create_new_asset, Asset, DomainError};
use crate::server::grpc::server::asset::asset_service_server::{AssetService, AssetServiceServer};
use crate::server::grpc::server::asset::{CreateRequest, CreateResponse};
use anyhow::Context;
use sqlx::PgPool;
use tonic::transport::{Error, Server};
use tonic::{Request, Response, Status};
use tracing::info;

pub mod asset {
    tonic::include_proto!("asset_rpc");
}

#[derive(Debug)]
pub struct AssetServiceManager {
    pg_pool: PgPool,
}

impl AssetServiceManager {
    pub fn new(pg_pool: PgPool) -> Self {
        AssetServiceManager { pg_pool }
    }
}

#[tonic::async_trait]
impl AssetService for AssetServiceManager {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>, Status> {
        let req = request.into_inner();
        info!("creating new asset :: (name={} -> symbol={})", &req.name, &req.symbol);
        let asset = Asset::new(req.name, req.symbol, req.description, req.organization)
            .map_err(|e| match e {
                DomainError::DatabaseError(err) => Status::internal(err.to_string()),
                DomainError::NotFoundError(err) => Status::not_found(err.to_string()),
                DomainError::DuplicateError(err) => Status::already_exists(err.to_string()),
                DomainError::InvalidArgument(err) => Status::invalid_argument(err.to_string()),
                DomainError::ValidationError(err) => Status::invalid_argument(err.to_string()),
            })?;
        let asset_create_resp = create_new_asset(&asset, &self.pg_pool).await;
        if let Err(err) = asset_create_resp {
            return Err(Status::internal(err.to_string()));
        }
        let response = CreateResponse { asset_id: asset.id };
        Ok(Response::new(response))
    }
}

pub struct GrpcServer {
    addr: core::net::SocketAddr,
    asset_service: AssetServiceManager,
}

impl GrpcServer {
    pub fn new(pg_pool: PgPool, config: GrpcServerConfig) -> Result<Self, anyhow::Error> {
        let addr = format!("[::]:{}", config.port)
            .parse()
            .context("Failed to parse grpc server address")?;
        let asset_service = AssetServiceManager::new(pg_pool);

        Ok(Self {
            addr,
            asset_service,
        })
    }

    pub async fn run_until_stopped(self) -> Result<(), Error> {
        info!("starting gRPC server :: port {}", &self.addr.port());
        Server::builder()
            .add_service(AssetServiceServer::new(self.asset_service))
            .serve(self.addr)
            .await
    }
}
