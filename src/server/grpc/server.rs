use crate::server::grpc::server::asset::asset_service_server::AssetService;
use crate::server::grpc::server::asset::{CreateRequest, CreateResponse};
use tonic::{Request, Response, Status};

pub mod asset {
    tonic::include_proto!("asset");
}

#[derive(Debug)]
pub struct AsserService {}

#[tonic::async_trait]
impl AssetService for AsserService {
    async fn create(&self, _: Request<CreateRequest>) -> Result<Response<CreateResponse>, Status> {
        todo!()
    }
}
