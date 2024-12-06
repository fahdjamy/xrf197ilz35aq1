use crate::core::{create_new_asset, Asset, DomainError};
use crate::server::grpc::server::asset::asset_service_server::AssetService;
use crate::server::grpc::server::asset::{CreateRequest, CreateResponse};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

pub mod asset {
    tonic::include_proto!("asset_rpc");
}

#[derive(Debug)]
pub struct AssetServiceManager {
    pg_pool: PgPool,
}

#[tonic::async_trait]
impl AssetService for AssetServiceManager {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>, Status> {
        let req = request.into_inner();
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
