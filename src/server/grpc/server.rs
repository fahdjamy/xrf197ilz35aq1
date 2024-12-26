use crate::configs::GrpcServerConfig;
use crate::core::{create_new_asset, find_asset_by_id, Asset, DatabaseError, DomainError};
use crate::server::grpc::server::asset::asset_service_server::{AssetService, AssetServiceServer};
use crate::server::grpc::server::asset::{Asset as GrpcAsset, CreateRequest, CreateResponse,
                                         GetAssetByIdRequest, GetAssetByIdResponse,
                                         GetPaginatedAssetsRequest, GetPaginatedAssetsResponse,
                                         GetStreamedAssetsRequest, GetStreamedAssetsResponse};
use anyhow::Context;
use prost_types::Timestamp;
use sqlx::PgPool;
use std::pin::Pin;
use tonic::codegen::tokio_stream::Stream;
use tonic::transport::{Error, Server};
use tonic::{Request, Response, Status};
use tracing::info;

pub mod asset {
    tonic::include_proto!("asset_rpc");
}

impl From<Asset> for GrpcAsset {
    fn from(asset: Asset) -> Self {
        GrpcAsset {
            id: asset.id.into(),
            name: asset.name.into(),
            symbol: asset.symbol.into(),
            description: asset.description.into(),
            organization: asset.organization.into(),
            created_at: Some(Timestamp {
                seconds: asset.created_at.timestamp(),
                nanos: asset.created_at.timestamp_subsec_nanos() as i32,
            }),
            updated_at: Some(Timestamp {
                seconds: asset.updated_at.timestamp(),
                nanos: asset.updated_at.timestamp_subsec_nanos() as i32,
            }),
        }
    }
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

    async fn get_asset_by_id(&self, request: Request<GetAssetByIdRequest>) -> Result<Response<GetAssetByIdResponse>, Status> {
        let req = request.into_inner();
        info!("get asset by id :: id={}", &req.asset_id);
        let asset_id = req.asset_id;
        let asset = find_asset_by_id(&asset_id, &self.pg_pool)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => Status::not_found("asset not found"),
                _ => Status::unknown("server error"),
            })?;
        let response = GetAssetByIdResponse {
            asset: Some(asset.into()),
        };

        Ok(Response::new(response))
    }

    async fn get_paginated_assets(&self, _: Request<GetPaginatedAssetsRequest>) -> Result<Response<GetPaginatedAssetsResponse>, Status> {
        todo!()
    }

    type GetStreamedAssetsStream = Pin<
        Box<
            dyn Stream<Item=Result<GetStreamedAssetsResponse, Status>> + Send + 'static>>;

    async fn get_streamed_assets(&self, _: Request<GetStreamedAssetsRequest>) -> Result<Response<Self::GetStreamedAssetsStream>, Status> {
        todo!()
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
