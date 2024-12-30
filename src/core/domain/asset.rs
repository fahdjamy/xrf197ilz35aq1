use crate::core::domain::error::DomainError;
use crate::core::domain::key::{generate_unique_key, DOMAIN_KEY_SIZE};
use crate::core::domain::DatabaseError;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::fmt::Display;
use std::str::FromStr;
use tracing::{error, warn};
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

#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    Asc,
    Desc,
}

impl Default for OrderType {
    fn default() -> Self {
        OrderType::Asc
    }
}

impl FromStr for OrderType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" | "ascending" => Ok(OrderType::Asc),
            "desc" | "descending" => Ok(OrderType::Desc),
            _ => Err(anyhow!("invalid order type: {}", s)),
        }
    }
}

impl Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Asc => write!(f, "ASC"),
            OrderType::Desc => write!(f, "DESC"),
        }
    }
}

impl From<String> for OrderType {
    fn from(s: String) -> Self {
        OrderType::from_str(&s).unwrap_or_else(|e| {
            warn!("invalid order type {} defaulting to: {}", e, OrderType::Asc);
            OrderType::Asc
        })
    }
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
pub async fn get_all_assets(pg_pool: &PgPool, start: i64, limit: i16) -> Result<Vec<Asset>, DatabaseError> {
    tracing::debug!("fetching assets from DB :: start={} :: limit={}", &start, &limit);
    // SQLx often requires i64 for LIMIT & OFFSET to ensure compatibility w/ various DB types & potential large values.
    let limit_i64 = limit as i64;
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
        start
    )
        .fetch_all(pg_pool)
        .await?;
    Ok(result)
}

#[tracing::instrument(level = "debug", skip(pg_pool, limit))]
pub async fn get_assets_by_symbol(symbol: &str, limit: i16, order_type: OrderType, pg_pool: &PgPool) -> Result<Vec<Asset>, DatabaseError> {
    // TO DO: Look into
    // 1. Full-text search: Better for complex searches w/ multiple words, phrases, & linguistic considerations
    // OR
    // 2. Trigram matching: Efficient for finding similar strings, even with typos or partial matches
    let search_term = format!("%{}%", &symbol.to_uppercase());
    tracing::debug!("fetching assets from DB :: symbol={}", &search_term);
    let result = sqlx::query_as!(
        Asset,
        r#"
        SELECT
            id, name, symbol, description, organization, created_at, updated_at
        FROM asset
        WHERE symbol LIKE $1
        ORDER BY symbol ASC
        LIMIT $2"#,
        search_term,
        limit as i64
    )
        .fetch_all(pg_pool)
        .await?;
    Ok(result)
}

#[tracing::instrument(level = "debug", skip(pg_pool, limit, order_type))]
pub async fn get_assets_by_name(name: &str, limit: i16, order_type: OrderType, pg_pool: &PgPool) -> Result<Vec<Asset>, DatabaseError> {
    let search_term = format!("%{}%", &name);
    tracing::debug!("fetching assets from DB :: name={}", &search_term);

    let result = match order_type {
        OrderType::Asc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at
                FROM asset
                WHERE symbol LIKE $1
                ORDER BY symbol ASC
                LIMIT $2"#,
                search_term,
                limit as i64
            ).fetch_all(pg_pool).await?
        }
        OrderType::Desc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at
                FROM asset
                WHERE symbol LIKE $1
                ORDER BY symbol DESC
                LIMIT $2"#,
                search_term,
                limit as i64
            ).fetch_all(pg_pool).await?
        }
    };

    Ok(result)
}
