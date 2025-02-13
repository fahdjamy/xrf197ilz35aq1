use crate::core::domain::key::{generate_unique_key, DOMAIN_KEY_SIZE};
use crate::core::{Currency, DomainError};
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum ContractVersion {
    V1
}

impl Display for ContractVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractVersion::V1 => write!(f, "v1")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Contract {
    pub id: String,
    pub details: String,
    pub min_price: f64,
    pub summary: String,
    pub asset_id: String,
    pub update_count: i32,
    pub updated_by: String,
    pub royalty_percentage: f32,
    pub version: ContractVersion,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub anonymous_buyer_only: bool,
    pub royalty_receiver_id: String,
    pub accepted_currency: HashSet<Currency>,
}

impl Display for Contract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "contractId:{}, assetId:{}, updated_count={}", self.id, self.asset_id, self.update_count)
    }
}

impl Contract {
    pub fn new(asset_id: String,
               details: String,
               summary: String,
               user_fp: String,
               min_price: f64,
               anonymous_buyer: bool,
               royalty_percentage: f32,
               royalty_receiver_id: String,
               accepted_currency: HashSet<Currency>) -> Result<Self, DomainError> {
        if royalty_percentage < 0.0 {
            return Err(DomainError::InvalidArgument("royalty percentage can not be less than 0.0".to_string()));
        }
        if !royalty_receiver_id.is_empty() && royalty_percentage == 0.0 {
            return Err(DomainError::InvalidArgument("royalty percentage can not be less than 0.0 if royalty receiver is specified".to_string()));
        }
        if royalty_receiver_id.is_empty() && royalty_percentage > 0.0 {
            return Err(DomainError::InvalidArgument("if royalty percentage is set, royalty receiver can't be empty".to_string()));
        }
        if accepted_currency.is_empty() {
            return Err(DomainError::InvalidArgument("accepted_currency should contain at least one currency".to_string()));
        }
        if min_price <= 0.0 {
            return Err(DomainError::InvalidArgument("min_price must be greater than 0.0".to_string()));
        }

        let id = generate_unique_key(DOMAIN_KEY_SIZE);
        Ok(Self {
            id,
            details,
            summary,
            asset_id,
            min_price,
            update_count: 0,
            accepted_currency,
            royalty_percentage,
            updated_by: user_fp,
            royalty_receiver_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: ContractVersion::V1,
            anonymous_buyer_only: anonymous_buyer,
        })
    }
}
