use crate::core::{Contract, DatabaseError};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
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
            content: contract.details,
            summary: contract.summary,
            asset_id: contract.asset_id,
            min_price: contract.min_price,
            updated_by: contract.updated_by,
            created_at: contract.created_at,
            updated_at: contract.updated_at,
            update_count: contract.update_count,
            version: contract.version.to_string(),
            anonymous_buyer_only: contract.anonymous_buyer_only,
            accepted_currency: contract.accepted_currency
                .into_iter()
                .map(|c| c.db_string().to_string())
                .collect(), // Convert to Vec<String>
        }
    }
}

impl Display for DbContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "contractId:{}, assetId:{}, updated_at={}", self.id, self.asset_id, self.updated_at)
    }
}

#[tracing::instrument(skip(pg_pool, contract))]
pub async fn create_contract(pg_pool: &PgPool, contract: Contract) -> Result<bool, DatabaseError> {
    tracing::info!("creating contract :: contractId={} :: assetId={}", contract.id, contract.asset_id);
    let db_contract: DbContract = DbContract::from(contract);
    let result = sqlx::query!(
        "
        INSERT INTO contract (
                      id,
                      content,
                      summary,
                      version,
                      asset_id,
                      min_price,
                      created_at,
                      updated_by,
                      updated_at,
                      update_count,
                      accepted_currency,
                      anonymous_buyer_only
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ",
        db_contract.id,
        db_contract.content,
        db_contract.summary,
        db_contract.version,
        db_contract.asset_id,
        db_contract.min_price,
        db_contract.created_at,
        db_contract.updated_by,
        db_contract.updated_at,
        db_contract.update_count,
        &db_contract.accepted_currency,
        db_contract.anonymous_buyer_only,
    )
        .execute(pg_pool)
        .await?;
    Ok(result.rows_affected() == 1)
}
