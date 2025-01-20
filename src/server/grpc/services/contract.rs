use crate::server::grpc::asset::contract_service_server::ContractService;
use crate::server::grpc::asset::{CreateContractRequest, CreateContractResponse};
use sqlx::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct ContractServiceManager {
    pg_pool: Arc<PgPool>,
}

impl ContractServiceManager {
    pub fn new(pg_pool: Arc<PgPool>) -> Self {
        ContractServiceManager { pg_pool }
    }
}

#[tonic::async_trait]
impl ContractService for ContractServiceManager {
    async fn create_contract(&self, _: Request<CreateContractRequest>) -> Result<Response<CreateContractResponse>, Status> {
        todo!()
    }
}
