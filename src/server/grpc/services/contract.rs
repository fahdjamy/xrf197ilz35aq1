use crate::constant::REQUEST_ID_KEY;
use crate::core::{create_contract, find_asset_by_id, Contract, Currency, DatabaseError};
use crate::server::grpc::asset::contract_service_server::ContractService;
use crate::server::grpc::asset::{CreateContractRequest, CreateContractResponse};
use crate::server::grpc::interceptors::trace_request;
use rayon::prelude::*;
use sqlx::PgPool;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info, info_span};

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
    async fn create_contract(&self, request: Request<CreateContractRequest>) -> Result<Response<CreateContractResponse>, Status> {
        trace_request!(request, "create_contract");
        let req = request.into_inner();
        info!("creating new contract :: (assetId={})", &req.asset_id);

        let saved_asset = find_asset_by_id(&req.asset_id, &self.pg_pool).await
            .map_err(|err| match err {
                DatabaseError::NotFound => {
                    error!(?req.asset_id, " asset not found");
                    Status::not_found("invalid asset id")
                },
                _ => {
                    error!("Failed to fetch asset by id {} from database: {:?}", req.asset_id, err);
                    Status::internal("server error")
                },
            })?;

        let details = req.details;
        let asset_id = saved_asset.id;
        let min_price = req.min_price as f64;
        let user_fp = req.user_finger_print;
        let anonymous_buyers_only = req.anonymous_buyers;
        let royalty_percentage = req.royalty_percentage.unwrap_or(0.0);
        let royalty_receiver = match req.royalty_receiver {
            None => {
                // if royalty_receiver is not set and royalty_percentage is > 0.0, then set
                // receiver to user_fp that's creating the contract
                if royalty_percentage > 0.0 {
                    user_fp.clone().to_string()
                } else {
                    "".to_string()
                }
            }
            Some(receiver_user_id) => { receiver_user_id }
        };
        let accepted_currencies = process_accepted_currencies(req.accepted_currencies)
            .map_err(|er| Status::invalid_argument(er.to_string()))?;
        let contract = Contract::new(asset_id,
                                     details,
                                     req.summary,
                                     user_fp.clone(),
                                     min_price,
                                     anonymous_buyers_only,
                                     royalty_percentage,
                                     royalty_receiver,
                                     accepted_currencies)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;
        let contract_id = contract.id.clone();

        let contract_created = create_contract(&self.pg_pool, contract).await.map_err(|err| match err {
            DatabaseError::InvalidArgument(err) => {
                Status::invalid_argument(err.to_string())
            },
            DatabaseError::RecordExists(msg) => {
                Status::already_exists(msg)
            },
            _ => {
                error!("message=failed to create new asset contract :: err={:?}", err);
                Status::internal("server error")
            }
        })?;

        if !contract_created {
            error!(?contract_id, "contract not created");
            return Err(Status::internal("contract not created, something went wrong"));
        }
        Ok(Response::new(CreateContractResponse { contract_id }))
    }
}

fn process_accepted_currencies(accepted_currencies: Vec<String>) -> Result<HashSet<Currency>, String> {
    // Use Rayon's parallel iterators to ensure thread safety
    let (valid_currencies, invalid_currencies): (Vec<_>, Vec<_>) = accepted_currencies
        .par_iter()
        .map(|c| match Currency::from_str(c) {
            Ok(currency) => (Some(currency), None),
            Err(_) => (None, Some(c.clone())),
        })
        .unzip();

    let valid_currencies: HashSet<Currency> = valid_currencies.into_iter().filter_map(|x| x).collect();
    let invalid_currencies: Vec<String> = invalid_currencies.into_iter().filter_map(|x| x).collect();

    if !invalid_currencies.is_empty() {
        // Log all invalid currencies
        error!("Invalid currencies provided: {:?}", invalid_currencies);

        // Return a simple error message to the user
        return Err("Invalid currencies provided".to_string());
    }

    Ok(valid_currencies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_accepted_currencies() {
        let valid_currencies = vec![
            "USD".to_string(),
            "EUR".to_string(),
            "GBP".to_string(),
            "Euro".to_string(),
        ];

        let result = process_accepted_currencies(valid_currencies);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_currencies() {
        let invalid_currencies = vec![
            "Unknown".to_string(),
            "EUR ".to_string(),
            " GBP".to_string(),
        ];
        let result = process_accepted_currencies(invalid_currencies);
        assert!(result.is_err());
    }
}
