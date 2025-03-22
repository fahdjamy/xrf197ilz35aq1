use crate::constant::REQUEST_ID_KEY;
use crate::core::{create_new_asset, delete_asset_by_id, find_asset_by_id, find_asset_by_id_and_org_id, find_assets_name_like, get_all_assets, orchestrate_transfer_asset, update_asset, Asset, DatabaseError, DomainError, OrchestrateError, OrderType, UpdateAssetRequest};
use crate::server::grpc::asset::asset_service_server::AssetService;
use crate::server::grpc::asset::{Asset as GrpcAsset, CreateRequest, CreateResponse,
                                 DeleteAssetRequest, DeleteAssetResponse,
                                 GetAssetByIdRequest, GetAssetByIdResponse, GetAssetsNameLikeRequest, GetAssetsNameLikeResponse,
                                 GetPaginatedAssetsRequest, GetPaginatedAssetsResponse, GetStreamedAssetsRequest,
                                 GetStreamedAssetsResponse, TransferAssetRequest, TransferAssetResponse, UpdateAssetRequest as GrpcUpdateAsset, UpdateAssetResponse};
use crate::server::grpc::interceptors::trace_request;
use crate::server::grpc::{get_xrf_user_auth_header, XRF_USER_FINGERPRINT};
use prost_types::Timestamp;
use sqlx::PgPool;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, info_span, log};

const MAX_DB_LIMIT: usize = 1000;

const MAX_LIMIT: i16 = 100;

impl From<Asset> for GrpcAsset {
    fn from(asset: Asset) -> Self {
        GrpcAsset {
            id: asset.id.into(),
            name: asset.name.into(),
            symbol: asset.symbol.into(),
            updated_by: asset.updated_by,
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
            listable: asset.listable,
            tradable: asset.tradable,
        }
    }
}

impl From<GrpcUpdateAsset> for UpdateAssetRequest {
    fn from(value: GrpcUpdateAsset) -> Self {
        UpdateAssetRequest {
            name: value.name,
            symbol: value.symbol,
            tradable: value.tradable,
            listable: value.listable,
            description: value.description,
            organization: Option::from(value.org_id),
        }
    }
}

impl From<&Asset> for GrpcAsset {
    fn from(asset: &Asset) -> Self {
        GrpcAsset {
            id: asset.id.to_owned().into(), // Clone the id String
            name: asset.name.to_owned().into(),
            symbol: asset.symbol.to_owned().into(),
            updated_by: asset.updated_by.to_owned(),
            description: asset.description.to_owned().into(),
            organization: asset.organization.to_owned().into(),
            created_at: Some(Timestamp {
                seconds: asset.created_at.timestamp(),
                nanos: asset.created_at.timestamp_subsec_nanos() as i32,
            }),
            updated_at: Some(Timestamp {
                seconds: asset.updated_at.timestamp(),
                nanos: asset.updated_at.timestamp_subsec_nanos() as i32,
            }),
            listable: asset.listable,
            tradable: asset.tradable,
        }
    }
}

#[derive(Debug)]
pub struct AssetServiceManager {
    pg_pool: Arc<PgPool>,
}

impl AssetServiceManager {
    pub fn new(pg_pool: Arc<PgPool>) -> Self {
        AssetServiceManager { pg_pool }
    }
}

#[tonic::async_trait]
impl AssetService for AssetServiceManager {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>, Status> {
        trace_request!(request, "create_asset");
        let user_fp = get_xrf_user_auth_header(&request.metadata(), XRF_USER_FINGERPRINT)?;
        let req = request.into_inner();
        info!("creating new asset :: (name={} -> symbol={})", &req.name, &req.symbol);
        let asset = Asset::new(req.name, req.symbol, user_fp.clone(), req.description, req.organization)
            .map_err(|e| match e {
                DomainError::ServerError(err) => Status::internal(err.to_string()),
                DomainError::DatabaseError(err) => Status::internal(err.to_string()),
                DomainError::NotFoundError(err) => Status::not_found(err.to_string()),
                DomainError::DuplicateError(err) => Status::already_exists(err.to_string()),
                DomainError::InvalidArgument(err) => Status::invalid_argument(err.to_string()),
                DomainError::ValidationError(err) => Status::invalid_argument(err.to_string()),
            })?;
        let asset_create_resp = create_new_asset(&asset, user_fp, &self.pg_pool).await;
        if let Err(err) = asset_create_resp {
            return Err(Status::internal(err.to_string()));
        }
        let response = CreateResponse { asset_id: asset.id };
        Ok(Response::new(response))
    }

    async fn update_asset(&self, request: Request<GrpcUpdateAsset>) -> Result<Response<UpdateAssetResponse>, Status> {
        trace_request!(request, "update_asset");
        let user_fp = get_xrf_user_auth_header(&request.metadata(), XRF_USER_FINGERPRINT)?;
        let req = request.into_inner();
        info!("updating asset :: id = {}", &req.asset_id);

        let org_id = req.org_id.clone();
        let asset_id = req.asset_id.clone();
        if asset_id.is_empty() || org_id.is_empty() {
            return Err(Status::invalid_argument("please provide a valid asset id and organization id"));
        }

        let updated_asset_req: UpdateAssetRequest = req.into();

        if updated_asset_req.name.is_none()
            && updated_asset_req.symbol.is_none()
            && updated_asset_req.listable.is_none()
            && updated_asset_req.tradable.is_none()
            && updated_asset_req.description.is_none() {
            return Err(Status::invalid_argument("At least one updatable field is required"));
        }

        let response = match update_asset(&asset_id, &user_fp, &updated_asset_req, &self.pg_pool).await {
            Ok(updated) => updated,
            Err(e) => {
                return match e {
                    DatabaseError::InvalidArgument(msg) => Err(Status::invalid_argument(msg)),
                    DatabaseError::NotFound => Err(Status::not_found("Asset not found")),
                    DatabaseError::PoolClosed => {
                        error!("Error while updating asset: connection closed");
                        Err(Status::unavailable("Database connection closed"))
                    }
                    _ => {
                        error!("Error while updating asset: {:?}", e);
                        Err(Status::internal("Internal server error while updating asset"))
                    }
                };
            }
        };

        Ok(Response::new(UpdateAssetResponse { updated: response }))
    }

    async fn delete_asset(&self, request: Request<DeleteAssetRequest>) -> Result<Response<DeleteAssetResponse>, Status> {
        trace_request!(request, "delete_asset");
        let req = request.into_inner();
        info!("deleting asset :: id = {}", &req.asset_id);
        let org_id = req.org_id;
        let asset_id = req.asset_id;

        let asset = find_asset_by_id_and_org_id(&asset_id, &org_id, &self.pg_pool)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => Status::not_found("invalid org id or asset id"),
                _ => Status::unknown("server error"),
            })?;

        let asset_deleted = delete_asset_by_id(&asset.id, &self.pg_pool)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => Status::not_found("invalid org id or asset id"),
                _ => Status::unknown("server error"),
            })
            .is_ok();

        Ok(Response::new(DeleteAssetResponse {
            deleted: asset_deleted,
        }))
    }

    async fn get_asset_by_id(&self, request: Request<GetAssetByIdRequest>) -> Result<Response<GetAssetByIdResponse>, Status> {
        trace_request!(request, "get_asset_by_id");
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

    async fn transfer_asset(&self, request: Request<TransferAssetRequest>)
                            -> Result<Response<TransferAssetResponse>, Status> {
        trace_request!(request, "transfer_asset");

        let req = request.into_inner();
        let org_id = req.org_id;
        let asset_id = req.asset_id;
        let new_owner_id = req.new_owner_fp;
        let new_org_owner = req.new_owner_org_id;
        let nfc = orchestrate_transfer_asset(&org_id, &asset_id, &new_org_owner,
                                             &new_owner_id, &self.pg_pool)
            .await
            .map_err(|e| match e {
                OrchestrateError::NotFoundError(err) => Status::not_found(err),
                OrchestrateError::ServerError(err) => Status::internal(err.to_string()),
                OrchestrateError::InvalidArgument(msg) => Status::invalid_argument(msg),
                OrchestrateError::DatabaseError(err) => Status::internal(err.to_string()),
            })?;

        Ok(Response::new(TransferAssetResponse { certificate_id: nfc.id }))
    }

    async fn get_assets_name_like(&self, request: Request<GetAssetsNameLikeRequest>) -> Result<Response<GetAssetsNameLikeResponse>, Status> {
        trace_request!(request, "get_assets_name_like");
        let req = request.into_inner();
        info!("get assets name-like :: name={}", &req.name);
        let assets = find_assets_name_like(&req.name, req.offset as i64, req.limit as usize, OrderType::Asc, &self.pg_pool)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => Status::not_found("No assets found"),
                DatabaseError::InvalidArgument(err) => Status::invalid_argument(err.to_string()),
                _ => Status::unknown("server error"),
            })?;
        let response = GetAssetsNameLikeResponse {
            offset: req.offset,
            total: assets.len() as i32,
            assets: assets.into_iter()
                .map(|a| a.into())
                .collect(),
        };
        Ok(Response::new(response))
    }

    async fn get_paginated_assets(&self, request: Request<GetPaginatedAssetsRequest>) -> Result<Response<GetPaginatedAssetsResponse>, Status> {
        trace_request!(request, "get_paginated_assets");

        let req = request.into_inner();
        if req.offset < 0 || req.limit < 0 || (req.offset == 0 && req.limit == 0) {
            return Err(Status::invalid_argument("start and limit must be positive"));
        }
        if req.limit < 1 || req.limit > 3000 {
            return Err(Status::invalid_argument("limit must be between 1 and 3000"));
        }
        let limit = req.limit as i16;
        let offset = req.offset as i64;

        info!("fetching paginated assets :: offset={} limit={}", req.offset, &limit);
        let assets = fetch_assets(&self.pg_pool, offset, limit, &req.sort_order).await?;

        let response = GetPaginatedAssetsResponse {
            offset: req.offset,
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
    async fn get_streamed_assets(&self, request: Request<GetStreamedAssetsRequest>)
                                 -> Result<Response<Self::GetStreamedAssetsStream>, Status> {
        trace_request!(request, "get_streamed_assets");
        let req = request.into_inner();
        validate_request_parameters(req.offset, req.limit)?;

        let limit: usize = req.limit
            .try_into()
            .map_err(|_| Status::invalid_argument("limit is invalid"))?; // Use usize for consistency and indexing
        let mut offset = req.offset
            .try_into()
            .map_err(|_| Status::invalid_argument("offset is invalid"))?;
        debug!("streaming assets :: startingAt={} limit={}", offset, limit);

        let pool = self.pg_pool.clone();

        let max_offset = 9999999; // only usage is to avoid infinite loops
        let stream = async_stream::stream! {
            let batch_size = (limit * 10).min(MAX_DB_LIMIT); // Fetch 10 times the requested limit for efficiency

            loop {
                if offset >= max_offset {
                    break;
                }
                // 1. Fetch a larger batch of assets
                let mut batch_assets = match fetch_assets(&pool, offset, batch_size as i16, &req.sort_order).await {
                    Ok(assets) => assets,
                    Err(e) => {
                        error!("Failed to fetch assets from database: {:?}", e);
                        yield Err(e);
                        break;
                    }
                };

                // 2. Break the loop if there are no more assets
                if batch_assets.is_empty() {
                    break; // End of data
                }

                while !batch_assets.is_empty() {
                    // 2. Take the user's limit from the fetched batch
                    let assets_to_send: Vec<_> = batch_assets.drain(..limit.min(batch_assets.len())).collect();
                    let total_assets = assets_to_send.len() as i32;

                    // Create a new Vec for the response to avoid consuming assets_to_send
                    let assets_response: Result<Vec<_>, Status> = assets_to_send.iter()
                        .map(|a| a.try_into().map_err(|_| Status::internal("Failed to convert asset")))
                        .collect();
                    // 3. Yield the response
                    match assets_response {
                        Ok(assets) => yield Ok(GetStreamedAssetsResponse {
                            offset: offset as i32,
                            total: total_assets,
                            assets,
                        }),
                        Err(e) => {
                            error!("Failed to serialize assets to be streamed: {:?}", e);
                            yield Err(e);
                            break;
                        }
                    }

                    offset += total_assets as i64; // Update start based on the actual sent assets
                }
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }
}

///// Helper methods
fn validate_request_parameters(start: i32, limit: i32) -> Result<(), Status> {
    if start < 0 || limit < 0 || (start == 0 && limit == 0) {
        return Err(Status::invalid_argument("start and limit must be positive"));
    }
    if limit < 1 || limit > MAX_LIMIT.into() {
        return Err(Status::invalid_argument("limit must be between 1 and 100"));
    }
    Ok(())
}

async fn fetch_assets(pg_pool: &PgPool, start: i64, limit: i16, sort_order: &str) -> Result<Vec<Asset>, Status> {
    let order_type = OrderType::from_str(sort_order)
        .map_err(|_| Status::invalid_argument("sort_order is invalid"))?;
    get_all_assets(pg_pool, start, limit as i64, order_type)
        .await
        .map_err(|e| {
            match e {
                DatabaseError::NotFound => Status::not_found("No assets found"),
                DatabaseError::InvalidArgument(err) => Status::invalid_argument(err.to_string()),
                e => {
                    log::error!("Error fetching assets: {:?}", e);
                    return Status::unknown("server error");
                }
            }
        })
}
