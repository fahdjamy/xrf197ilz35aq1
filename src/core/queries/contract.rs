use crate::core::Contract;
use chrono::{DateTime, Utc};
use std::fmt::Display;

#[derive(Debug)]
struct DbContract {
    pub id: String,
    pub content: String,
    pub min_price: f64,
    pub summary: String,
    pub version: String,
    pub asset_id: String,
    pub update_count: i32,
    pub updated_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub anonymous_buyer_only: bool,
    pub accepted_currency: Vec<String>, // Change to Vec<String> for database compatibility
}

impl From<Contract> for DbContract {
    fn from(contract: Contract) -> Self {
        DbContract {
            id: contract.id,
            content: contract.content,
            summary: contract.summary,
            asset_id: contract.asset_id,
            min_price: contract.min_price,
            accepted_currency: vec![],
            updated_by: contract.updated_by,
            created_at: contract.created_at,
            updated_at: contract.updated_at,
            update_count: contract.update_count,
            version: contract.version.to_string(),
            anonymous_buyer_only: contract.anonymous_buyer_only,
        }
    }
}

impl Display for DbContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "contractId:{}, assetId:{}, updated_at={}", self.id, self.asset_id, self.updated_at)
    }
}
