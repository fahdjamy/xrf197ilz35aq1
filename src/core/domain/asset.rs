use crate::core::domain::error::DomainError;
use crate::core::domain::key::{generate_unique_key, DOMAIN_KEY_SIZE};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::error;

pub struct Asset {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub organization: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Asset {
    pub fn new(
        name: String,
        symbol: String,
        description: String,
        organization: String,
    ) -> Result<Self, DomainError> {
        Self::validate_name(&name)?;
        let now = Utc::now();
        let asset_id = generate_unique_key(DOMAIN_KEY_SIZE);
        Ok(Self {
            name,
            symbol,
            description,
            organization,
            id: asset_id,
            created_at: now,
            updated_at: now,
        })
    }

    fn validate_name(name: &str) -> Result<(), DomainError> {
        const MIN_LENGTH: usize = 3;
        const MAX_LENGTH: usize = 32;

        if name.is_empty() || name.len() < MIN_LENGTH || name.len() > MAX_LENGTH {
            let error_msg =
                format!("name should be between {MIN_LENGTH} and {MAX_LENGTH} characters long")
                    .to_string();
            return Err(DomainError::InvalidArgument(error_msg));
        }
        Ok(())
    }
}

#[tracing::instrument(level = "debug", skip(pg_pool, asset), name = "Create new asset")]
pub async fn create_new_asset(asset: &Asset, pg_pool: &PgPool) -> Result<bool, anyhow::Error> {
    let result = sqlx::query!(
        "
        INSERT INTO asset (
            name,
            symbol,
            description,
            organization,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
        asset.name,
        asset.symbol,
        asset.description,
        asset.organization,
        asset.created_at,
        asset.updated_at
    )
    .execute(pg_pool)
    .await
    .map_err(|e| {
        error!("Error executing SQL query: {:?}", e);
        anyhow!("something went wrong")
    })?;
    Ok(result.rows_affected() == 1)
}
