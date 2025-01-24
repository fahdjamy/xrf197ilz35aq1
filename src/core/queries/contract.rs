use crate::core::{Contract, CurrencyList, DatabaseError};
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
    pub accepted_currency: CurrencyList, // Change to Vec<String> for database compatibility
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
            accepted_currency: CurrencyList(contract.accepted_currency
                .into_iter()
                .map(|c| c)
                .collect()), // Convert to Vec<String>
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
    let contract_asset_exists = check_if_contract_exits_for_asset(pg_pool, &contract.asset_id)
        .await
        .map_err(|err| {
            tracing::error!("failed to create contract: {:?}", err);
            return err;
        })?;
    if contract_asset_exists {
        return Err(DatabaseError::RecordExists("contract for given asset id exists".to_string()));
    }

    let db_contract: DbContract = DbContract::from(contract);
    let result = sqlx::query!(
        r#"
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
        "#,
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
        &db_contract.accepted_currency as &CurrencyList,
        db_contract.anonymous_buyer_only,
    )
        .execute(pg_pool)
        .await?;
    Ok(result.rows_affected() == 1)
}

#[tracing::instrument(skip(pg_pool))]
async fn check_if_contract_exits_for_asset(pg_pool: &PgPool, asset_id: &str) -> Result<bool, DatabaseError> {
    tracing::info!("checking for contract with assetId={}", asset_id);
    // EXISTS is a SQL operator (a keyword), not a field. It's used to test for the existence of rows in a subquery.
    // The 1 in SELECT 1 is an arbitrary placeholder value that indicates the existence of a row without needing to retrieve the actual row data.
    let result = sqlx::query!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM contract
            WHERE asset_id=$1
        ) AS "exists!"
        "#,
        asset_id
    )
        .fetch_one(pg_pool)
        .await?;

    Ok(result.exists)
}
