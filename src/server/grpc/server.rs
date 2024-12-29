use crate::configs::GrpcServerConfig;
use crate::core::{create_new_asset, find_asset_by_id, get_all_assets, Asset, DatabaseError, DomainError};
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
use tracing::{info, log};

pub mod asset {
    tonic::include_proto!("asset_rpc");
}

const MAX_LIMIT: i16 = 100;

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

    async fn fetch_assets(&self, start: i16, limit: i16) -> Result<Vec<Asset>, Status> {
        get_all_assets(&self.pg_pool, start, limit).await.map_err(|e| {
            match e {
                DatabaseError::NotFound => Status::not_found("No assets found"),
                e => {
                    log::error!("Error fetching assets: {:?}", e);
                    return Status::unknown("server error");
                }
            }
        })
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

    async fn get_paginated_assets(&self, request: Request<GetPaginatedAssetsRequest>) -> Result<Response<GetPaginatedAssetsResponse>, Status> {
        let req = request.into_inner();
        if req.start < 0 || req.limit < 0 || (req.start == 0 && req.limit == 0) {
            return Err(Status::invalid_argument("start and limit must be positive"));
        }
        if req.limit < 1 || req.limit > 3000 {
            return Err(Status::invalid_argument("limit must be between 1 and 3000"));
        }
        if req.start > req.limit {
            return Err(Status::invalid_argument("start must be less than limit"));
        }
        let start = req.start as i16;
        let limit = req.limit as i16;

        info!("fetching paginated assets :: start={} limit={}", &start, &limit);
        let assets = self.fetch_assets(start, limit).await?;

        let response = GetPaginatedAssetsResponse {
            start: req.start,
            total: assets.len() as i32,
            assets: assets.into_iter()
                .map(|a| a.into())
                .collect(),
        };
        Ok(Response::new(response))
    }

    type GetStreamedAssetsStream = Pin<
        Box<
            dyn Stream<Item=Result<GetStreamedAssetsResponse, Status>> + Send + 'static>>;

    async fn get_streamed_assets(&self, request: Request<GetStreamedAssetsRequest>) -> Result<Response<Self::GetStreamedAssetsStream>, Status> {
        let req = request.into_inner();
        validate_request_parameters(req.start, req.limit)?;
        info!("streaming assets :: start={} limit={}", &req.start, &req.limit);

        let limit = req.limit as i16;
        let mut start = req.start as i16;

        // 1. Fetch a larger batch of assets initially
        let batch_size = req.limit as i16 * 10; // Fetch 10 times the requested limit for efficiency
        let mut batch_assets = self.fetch_assets(start, batch_size).await?;

        let stream = async_stream::stream! {
            loop {
                // 2. Take the user's limit from the fetched batch
                let limited_assets: Vec<_> = batch_assets.drain(..limit as usize).collect();
                
                // 3. Yield the response
                if !limited_assets.is_empty() { // Check if NOT empty
                    let assets_response: Vec<_> = limited_assets
                        .into_iter()
                        .map(|a| {
                            let asset = a.into();
                            asset
                        })
                        .collect(); 
                    let total_assets = assets_response.len() as i32; 
                    yield Ok(GetStreamedAssetsResponse {
                        start: start as i32,
                        total: total_assets,
                        assets: assets_response,
                    })
                }

                // 4. If the batch assets is empty, fetch the next batch
                if batch_assets.is_empty() {
                    start += batch_size;
                    batch_assets = self.fetch_assets(start, batch_size).await?;

                    // 5. Break the loop if there are no more assets
                    if batch_assets.is_empty() {
                        break;
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }
}

fn validate_request_parameters(start: i32, limit: i32) -> Result<(), Status> {
    if start < 0 || limit < 0 || (start == 0 && limit == 0) {
        return Err(Status::invalid_argument("start and limit must be positive"));
    }
    if limit < 1 || limit > MAX_LIMIT.into() {
        return Err(Status::invalid_argument("limit must be between 1 and 100"));
    }
    if start > limit {
        return Err(Status::invalid_argument("start must be less than limit"));
    }
    Ok(())
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
