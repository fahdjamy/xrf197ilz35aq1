use crate::core::domain::key::{generate_unique_key, DOMAIN_KEY_SIZE};
use chrono::{DateTime, Utc};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Contract {
    pub id: String,
    pub content: String,
    pub summary: String,
    pub asset_id: String,
    pub update_count: i32,
    pub updated_by: String,
    pub organization: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Display for Contract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "contractId:{}, assetId:{}, updated_count={}", self.id, self.asset_id, self.update_count)
    }
}

impl Contract {
    pub fn new(asset_id: String, content: String, summary: String, user_fp: String, organization: String) -> Self {
        let id = generate_unique_key(DOMAIN_KEY_SIZE);
        Self {
            id,
            content,
            summary,
            asset_id,
            organization,
            update_count: 0,
            updated_by: user_fp,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
