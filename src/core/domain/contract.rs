use crate::core::domain::key::{generate_unique_key, DOMAIN_KEY_SIZE};
use crate::core::{Currency, DomainError};
use chrono::{DateTime, Utc};
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
    pub content: String,
    pub min_price: f64,
    pub summary: String,
    pub asset_id: String,
    pub update_count: i32,
    pub updated_by: String,
    pub version: ContractVersion,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub anonymous_buyer_only: bool,
    pub accepted_currency: Vec<Currency>,
}

impl Display for Contract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "contractId:{}, assetId:{}, updated_count={}", self.id, self.asset_id, self.update_count)
    }
}

impl Contract {
    pub fn new(asset_id: String,
               content: String,
               summary: String,
               user_fp: String,
               min_price: f64,
               anonymous_buyer: bool,
               accepted_currency: Vec<Currency>) -> Result<Self, DomainError> {
        if accepted_currency.is_empty() {
            return Err(DomainError::InvalidArgument("accepted_currency should contain at least one currency".to_string()));
        }
        if min_price <= 0.0 {
            return Err(DomainError::InvalidArgument("min_price greater than 0.0".to_string()));
        }

        let id = generate_unique_key(DOMAIN_KEY_SIZE);
        Ok(Self {
            id,
            content,
            summary,
            asset_id,
            min_price,
            update_count: 0,
            accepted_currency,
            updated_by: user_fp,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: ContractVersion::V1,
            anonymous_buyer_only: anonymous_buyer,
        })
    }
}
