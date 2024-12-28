use crate::core::domain::error::DomainError;
use crate::core::domain::key::{generate_unique_key, DOMAIN_KEY_SIZE};
use crate::core::domain::DatabaseError;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

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
        Self::validate_symbol(&symbol)?;
        Self::validate_organization(&organization)?;
        let now = Utc::now();
        let asset_id = generate_unique_key(DOMAIN_KEY_SIZE);
        Ok(Self {
            name,
            description,
            organization,
            id: asset_id,
            created_at: now,
            updated_at: now,
            symbol: symbol.to_uppercase(),
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

    fn validate_symbol(symbol: &str) -> Result<(), DomainError> {
        const MIN_LENGTH: usize = 3;
        const MAX_LENGTH: usize = 10;
        if symbol.is_empty() || symbol.len() < MIN_LENGTH || symbol.len() > MAX_LENGTH {
            let error = format!("symbol should be between {MIN_LENGTH} and {MAX_LENGTH} characters long");
            return Err(DomainError::InvalidArgument(error));
        }
        if symbol.chars().any(|c| c.is_whitespace()) {
            let error = "symbol should not contain a whitespace".to_string();
            return Err(DomainError::InvalidArgument(error));
        }
        Ok(())
    }

    fn validate_organization(org: &str) -> Result<(), DomainError> {
        const MIN_LENGTH: usize = 32;
        if org.is_empty() || org.len() < MIN_LENGTH {
            let error = format!("orgId should at least be of length {MIN_LENGTH} characters long");
            return Err(DomainError::InvalidArgument(error));
        }
        if Uuid::parse_str(org).is_err() {
            let error = "orgId should be a valid UUID".to_string();
            return Err(DomainError::InvalidArgument(error));
        }
        Ok(())
    }
}

#[tracing::instrument(level = "debug", skip(pg_pool, asset), name = "Create new asset")]
pub async fn create_new_asset(asset: &Asset, pg_pool: &PgPool) -> Result<bool, anyhow::Error> {
    tracing::debug!("saving new asset to DB :: id={}", &asset.id);
    let result = sqlx::query!(
        "
        INSERT INTO asset (
            id,
            name,
            symbol,
            description,
            organization,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ",
        asset.id,
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

#[tracing::instrument(level = "debug", skip(pg_pool))]
pub async fn find_asset_by_id(asset_id: &str, pg_pool: &PgPool) -> Result<Asset, DatabaseError> {
    tracing::debug!("fetching asset :: id={}", asset_id);
    let result = sqlx::query_as!(
        Asset,
        r#"
        SELECT
            id, name, symbol, description, organization, created_at, updated_at
        FROM asset
        WHERE id = $1"#,
        asset_id
    ).
        fetch_one(pg_pool)
        .await?;
    Ok(result)
}

#[tracing::instrument(level = "debug", skip(pg_pool, limit, start))]
pub async fn get_all_assets(pg_pool: &PgPool, start: i16, limit: i16) -> Result<Vec<Asset>, DatabaseError> {
    tracing::debug!("fetching assets from DB :: start={} :: limit={}", &start, &limit);
    // SQLx often requires i64 for LIMIT & OFFSET to ensure compatibility w/ various DB types & potential large values.
    let limit_i64 = limit as i64;
    let start_i64 = start as i64;
    let result = sqlx::query_as!(
        Asset,
        r#"
        SELECT
            id, name, symbol, description, organization, created_at, updated_at
        FROM asset
        ORDER BY name ASC
        LIMIT $1 OFFSET $2
        "#,
        limit_i64,
        start_i64
    )
        .fetch_all(pg_pool)
        .await?;
    Ok(result)
}
